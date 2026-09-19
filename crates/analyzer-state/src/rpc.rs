//! Read-only Soroban RPC state access.
//!
//! Uses the official `getLedgerEntries` JSON-RPC method
//! (<https://developers.stellar.org/docs/data/apis/rpc/api-reference/methods/getLedgerEntries>,
//! verified against that page's documented request/response shape) to
//! fetch a contract's instance entry (its [`crate::snapshot::ExecutableForm`])
//! and, when the caller supplies specific keys it cares about, matching
//! contract data entries. This module never sends a transaction, never
//! requires signing, and never mutates any remote state: it only ever
//! issues `getLedgerEntries` reads.
//!
//! [`Transport`] is the seam that keeps this testable without a live
//! network: [`UreqTransport`] is the real, bounded HTTPS implementation
//! used in production; unit tests substitute a mock. **Live endpoint
//! behavior has not been exercised in this codebase's automated test
//! suite** (no network access in that environment); only the mocked
//! request/response parsing paths are `TESTED_LOCALLY`. Treat this
//! module as `UNVERIFIED` against a real Soroban RPC endpoint until it
//! has actually been run against one.
//!
//! `getLedgerEntries` cannot enumerate a contract's full data footprint
//! on its own; callers must know the keys they want. This source
//! therefore always resolves the contract's instance entry, and
//! additionally resolves whatever specific `(ScVal, ContractDataDurability)`
//! keys the caller supplies; it never claims to have fetched "all"
//! contract data.

use std::time::Duration;

use analyzer_core::{AnalysisError, AnalyzerError, BackendError, SerializationError};
use analyzer_evidence::EvidenceSource;
use serde_json::{json, Value};
use stellar_xdr::{
    ContractDataDurability, ContractDataEntry, LedgerEntryData, LedgerKey, LedgerKeyContractData,
    Limits, ReadXdr, ScAddress, ScVal, WriteXdr,
};

use crate::snapshot::{ContractDataEntrySnapshot, Durability, ExecutableForm, StateSnapshot};

/// Sends a JSON-RPC request body to `endpoint` and returns the raw
/// response body. Exists so tests can substitute a mock without a real
/// network call.
pub trait Transport {
    fn post_json_rpc(&self, endpoint: &str, request_body: &str) -> Result<String, AnalyzerError>;
}

/// A [`Transport`] backed by a real, bounded blocking HTTPS POST via
/// `ureq`. `UNVERIFIED`: not exercised against a live endpoint in this
/// codebase's test suite.
#[derive(Debug, Clone)]
pub struct UreqTransport {
    timeout: Duration,
}

impl UreqTransport {
    /// Construct a transport with the given request timeout.
    pub fn new(timeout: Duration) -> Self {
        Self { timeout }
    }
}

impl Default for UreqTransport {
    fn default() -> Self {
        Self::new(Duration::from_secs(20))
    }
}

impl Transport for UreqTransport {
    fn post_json_rpc(&self, endpoint: &str, request_body: &str) -> Result<String, AnalyzerError> {
        let agent = ureq::AgentBuilder::new().timeout(self.timeout).build();

        let response = agent
            .post(endpoint)
            .set("Content-Type", "application/json")
            .send_string(request_body)
            .map_err(|source| {
                BackendError::with_source(format!("RPC request to '{endpoint}' failed"), source)
            })?;

        response.into_string().map_err(|source| {
            BackendError::with_source("failed to read RPC response body", source).into()
        })
    }
}

/// One additional contract data key a caller wants resolved alongside
/// the contract instance.
#[derive(Debug, Clone)]
pub struct DataKeyRequest {
    pub key: ScVal,
    pub durability: ContractDataDurability,
}

/// A provider-neutral, read-only Soroban RPC state source.
pub struct RpcStateSource<T: Transport = UreqTransport> {
    transport: T,
    endpoint: String,
    contract_address: ScAddress,
    additional_data_keys: Vec<DataKeyRequest>,
}

impl RpcStateSource<UreqTransport> {
    /// Construct a source using the real [`UreqTransport`].
    pub fn new(endpoint: impl Into<String>, contract_address: ScAddress) -> Self {
        Self::with_transport(endpoint, contract_address, UreqTransport::default())
    }
}

impl<T: Transport> RpcStateSource<T> {
    /// Construct a source using a caller-supplied [`Transport`] (used in
    /// tests to avoid a real network call).
    pub fn with_transport(
        endpoint: impl Into<String>,
        contract_address: ScAddress,
        transport: T,
    ) -> Self {
        Self {
            transport,
            endpoint: endpoint.into(),
            contract_address,
            additional_data_keys: Vec::new(),
        }
    }

    /// Request that `key`/`durability` also be resolved when
    /// [`Self::load`] is called, in addition to the contract instance.
    pub fn with_data_key(mut self, key: ScVal, durability: ContractDataDurability) -> Self {
        self.additional_data_keys
            .push(DataKeyRequest { key, durability });
        self
    }

    /// Fetch the contract's instance entry and any requested data keys
    /// via `getLedgerEntries`, and assemble a [`StateSnapshot`].
    pub fn load(&self) -> Result<StateSnapshot, AnalyzerError> {
        let mut keys = vec![LedgerKey::ContractData(LedgerKeyContractData {
            contract: self.contract_address.clone(),
            key: ScVal::LedgerKeyContractInstance,
            durability: ContractDataDurability::Persistent,
        })];
        for request in &self.additional_data_keys {
            keys.push(LedgerKey::ContractData(LedgerKeyContractData {
                contract: self.contract_address.clone(),
                key: request.key.clone(),
                durability: request.durability,
            }));
        }

        let key_strings = keys
            .iter()
            .map(encode_xdr_base64)
            .collect::<Result<Vec<_>, _>>()?;

        let request_body = json!({
            "jsonrpc": "2.0",
            "id": 1,
            "method": "getLedgerEntries",
            "params": { "keys": key_strings },
        })
        .to_string();

        let response_body = self
            .transport
            .post_json_rpc(&self.endpoint, &request_body)?;
        let response: Value = serde_json::from_str(&response_body).map_err(|source| {
            SerializationError::with_source("failed to parse RPC response as JSON", source)
        })?;

        if let Some(error) = response.get("error") {
            return Err(BackendError::new(format!(
                "RPC endpoint returned a JSON-RPC error: {error}"
            ))
            .into());
        }

        let result = response
            .get("result")
            .ok_or_else(|| BackendError::new("RPC response has no 'result' field"))?;
        let latest_ledger = result
            .get("latestLedger")
            .and_then(Value::as_u64)
            .ok_or_else(|| BackendError::new("RPC response is missing 'result.latestLedger'"))?;
        let entries = result
            .get("entries")
            .and_then(Value::as_array)
            .ok_or_else(|| BackendError::new("RPC response is missing 'result.entries'"))?;

        let mut executable = None;
        let mut contract_data = Vec::new();

        for entry in entries {
            let xdr_base64 = entry
                .get("xdr")
                .and_then(Value::as_str)
                .ok_or_else(|| BackendError::new("RPC ledger entry is missing 'xdr'"))?;
            let live_until_ledger_seq = entry
                .get("liveUntilLedgerSeq")
                .and_then(Value::as_u64)
                .and_then(|v| u32::try_from(v).ok());

            let entry_data = decode_xdr_base64::<LedgerEntryData>(xdr_base64)?;
            let LedgerEntryData::ContractData(data) = entry_data else {
                continue;
            };

            if data.key == ScVal::LedgerKeyContractInstance {
                if let ScVal::ContractInstance(instance) = &data.val {
                    executable = Some(ExecutableForm::from_xdr(&instance.executable)?);
                }
                continue;
            }

            contract_data.push(contract_data_entry_snapshot(&data, live_until_ledger_seq)?);
        }

        let executable = executable.ok_or_else(|| {
            AnalysisError::new(
                "RPC response did not include the contract's instance entry; the contract may not exist at this address",
            )
        })?;

        let latest_ledger = u32::try_from(latest_ledger).map_err(|source| {
            AnalysisError::with_source("latestLedger did not fit in u32", source)
        })?;

        Ok(StateSnapshot {
            contract_address_xdr_hex: encode_xdr_hex(&self.contract_address)?,
            executable,
            contract_data,
            ledger_sequence: latest_ledger,
            ledger_close_time: None,
            source: EvidenceSource::RpcResponse {
                method: "getLedgerEntries".to_string(),
                endpoint: self.endpoint.clone(),
            },
        })
    }
}

impl<T: Transport> super::source::StateSource for RpcStateSource<T> {
    fn load(&self) -> Result<StateSnapshot, AnalyzerError> {
        RpcStateSource::load(self)
    }
}

fn contract_data_entry_snapshot(
    data: &ContractDataEntry,
    live_until_ledger_seq: Option<u32>,
) -> Result<ContractDataEntrySnapshot, AnalyzerError> {
    Ok(ContractDataEntrySnapshot {
        key_xdr_hex: encode_xdr_hex(&data.key)?,
        val_xdr_hex: encode_xdr_hex(&data.val)?,
        durability: match data.durability {
            ContractDataDurability::Temporary => Durability::Temporary,
            ContractDataDurability::Persistent => Durability::Persistent,
        },
        live_until_ledger_seq,
    })
}

fn encode_xdr_hex<Xdr: WriteXdr>(value: &Xdr) -> Result<String, AnalyzerError> {
    let bytes = value
        .to_xdr(Limits::none())
        .map_err(|source| AnalysisError::with_source("failed to encode XDR value", source))?;
    Ok(hex::encode(bytes))
}

fn encode_xdr_base64<Xdr: WriteXdr>(value: &Xdr) -> Result<String, AnalyzerError> {
    value.to_xdr_base64(Limits::none()).map_err(|source| {
        AnalysisError::with_source("failed to base64-encode XDR value", source).into()
    })
}

fn decode_xdr_base64<Xdr: ReadXdr>(text: &str) -> Result<Xdr, AnalyzerError> {
    Xdr::from_xdr_base64(text, Limits::none()).map_err(|source| {
        AnalysisError::with_source("failed to decode base64 XDR value", source).into()
    })
}

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::expect_used)]
mod tests {
    use super::*;
    use crate::source::StateSource;
    use stellar_xdr::{
        ContractDataEntry, ContractExecutable, ContractId, ExtensionPoint, Hash, ScContractInstance,
    };

    struct MockTransport {
        response: String,
    }

    impl Transport for MockTransport {
        fn post_json_rpc(
            &self,
            _endpoint: &str,
            _request_body: &str,
        ) -> Result<String, AnalyzerError> {
            Ok(self.response.clone())
        }
    }

    fn sample_contract_address() -> ScAddress {
        ScAddress::Contract(ContractId(Hash([3u8; 32])))
    }

    fn mock_response_with_instance() -> String {
        let instance_key_entry = ContractDataEntry {
            ext: ExtensionPoint::V0,
            contract: sample_contract_address(),
            key: ScVal::LedgerKeyContractInstance,
            durability: ContractDataDurability::Persistent,
            val: ScVal::ContractInstance(ScContractInstance {
                executable: ContractExecutable::Wasm(Hash([9u8; 32])),
                storage: None,
            }),
        };
        let xdr = encode_xdr_base64(&LedgerEntryData::ContractData(instance_key_entry)).unwrap();

        json!({
            "jsonrpc": "2.0",
            "id": 1,
            "result": {
                "latestLedger": 555,
                "entries": [
                    { "key": "unused", "xdr": xdr, "lastModifiedLedgerSeq": 500 }
                ]
            }
        })
        .to_string()
    }

    #[test]
    fn resolves_executable_form_from_instance_entry() {
        let transport = MockTransport {
            response: mock_response_with_instance(),
        };
        let source = RpcStateSource::with_transport(
            "https://example.invalid/rpc",
            sample_contract_address(),
            transport,
        );

        let snapshot = StateSource::load(&source).unwrap();
        assert_eq!(
            snapshot.executable,
            ExecutableForm::Wasm {
                hash: hex::encode([9u8; 32])
            }
        );
        assert_eq!(snapshot.ledger_sequence, 555);
        assert!(snapshot.contract_data.is_empty());
    }

    #[test]
    fn missing_instance_entry_is_an_analysis_error_not_a_panic() {
        let empty_response = json!({
            "jsonrpc": "2.0",
            "id": 1,
            "result": { "latestLedger": 1, "entries": [] }
        })
        .to_string();
        let transport = MockTransport {
            response: empty_response,
        };
        let source = RpcStateSource::with_transport(
            "https://example.invalid/rpc",
            sample_contract_address(),
            transport,
        );

        let result = StateSource::load(&source);
        assert!(matches!(result, Err(AnalyzerError::Analysis(_))));
    }

    #[test]
    fn json_rpc_error_response_is_a_structured_backend_error() {
        let error_response = json!({
            "jsonrpc": "2.0",
            "id": 1,
            "error": { "code": -32602, "message": "invalid params" }
        })
        .to_string();
        let transport = MockTransport {
            response: error_response,
        };
        let source = RpcStateSource::with_transport(
            "https://example.invalid/rpc",
            sample_contract_address(),
            transport,
        );

        let result = StateSource::load(&source);
        assert!(matches!(result, Err(AnalyzerError::Backend(_))));
    }

    #[test]
    fn malformed_response_body_is_a_structured_serialization_error() {
        let transport = MockTransport {
            response: "not json".to_string(),
        };
        let source = RpcStateSource::with_transport(
            "https://example.invalid/rpc",
            sample_contract_address(),
            transport,
        );

        let result = StateSource::load(&source);
        assert!(matches!(result, Err(AnalyzerError::Serialization(_))));
    }
}
