//! A point-in-time snapshot of one contract's on-chain state.
//!
//! `StateSnapshot` is provider-neutral: it is the same shape whether it
//! was built from a local JSON file ([`crate::source::LocalSnapshotSource`])
//! or read from a live network (the read-only RPC source added in a
//! later step). Every field here is either a scalar, an XDR-derived hex
//! string, or a caller-ordered `Vec`; there is no `HashMap`, so
//! serializing a snapshot is deterministic.
//!
//! Contract data keys and values are kept as their canonical XDR
//! encoding (hex of the raw `ScVal` bytes), not reinterpreted into a
//! "friendly" type: this analyzer does not assume it can reconstruct a
//! contract's storage layout, and guessing at key semantics from binary
//! patterns would be exactly the kind of fabricated evidence this
//! project must not produce.

use analyzer_core::{AnalysisError, AnalyzerError};
use analyzer_evidence::EvidenceSource;
use serde::{Deserialize, Serialize};
use stellar_xdr::{ContractExecutable, Limits, ScAddress, WriteXdr};

/// The form a contract's executable takes, per the current official XDR
/// (`ContractExecutable`): a WASM module by hash, the native Stellar
/// Asset contract, or an external reference to an executable owned by
/// another entity.
///
/// Local WASM inspection ([`analyzer_executable`](../analyzer_executable))
/// only applies to [`ExecutableForm::Wasm`]. For the other two forms,
/// this analyzer has no WASM to inspect at all; callers must treat that
/// as a `NOT_DETERMINABLE` limitation, never synthesize a WASM analysis
/// for a non-WASM executable.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "form", rename_all = "snake_case")]
pub enum ExecutableForm {
    /// A WASM module, identified by its hex-encoded hash (the same
    /// identity space as `analyzer_executable::ArtifactHash`).
    Wasm { hash: String },
    /// The native Stellar Asset Contract implementation. There is no
    /// WASM here at all; it is provided directly by the host.
    StellarAsset,
    /// A reference to an executable owned by another address, tagged
    /// for that owner's own resolution scheme. This analyzer does not
    /// resolve external references; doing so would require a source of
    /// truth this crate does not have.
    ExternalRef {
        /// The owning address, as hex of its canonical XDR encoding.
        executable_owner_xdr_hex: String,
        tag: String,
    },
}

impl ExecutableForm {
    /// Build an [`ExecutableForm`] from the official `ContractExecutable`
    /// XDR type.
    pub fn from_xdr(executable: &ContractExecutable) -> Result<Self, AnalyzerError> {
        let form = match executable {
            ContractExecutable::Wasm(hash) => Self::Wasm {
                hash: hex::encode(hash.0),
            },
            ContractExecutable::StellarAsset => Self::StellarAsset,
            ContractExecutable::ExternalRef(reference) => Self::ExternalRef {
                executable_owner_xdr_hex: encode_xdr_hex(&reference.executable_owner)?,
                tag: reference.tag.to_utf8_string().map_err(|source| {
                    AnalysisError::with_source("external ref tag is not valid UTF-8", source)
                })?,
            },
        };
        Ok(form)
    }
}

/// Whether a contract data entry is temporary (evicted after its TTL
/// unless extended) or persistent.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Durability {
    Temporary,
    Persistent,
}

/// One contract data (ledger) entry, with its key and value preserved
/// as canonical XDR hex rather than reinterpreted.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ContractDataEntrySnapshot {
    /// Hex of the entry's canonical `ScVal` key XDR encoding.
    pub key_xdr_hex: String,
    /// Hex of the entry's canonical `ScVal` value XDR encoding.
    pub val_xdr_hex: String,
    pub durability: Durability,
    /// The entry's live-until ledger sequence, when the snapshot source
    /// captured TTL information for it.
    pub live_until_ledger_seq: Option<u32>,
}

/// A point-in-time snapshot of one contract's on-chain state.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct StateSnapshot {
    /// The contract's address, as hex of its canonical `ScAddress` XDR
    /// encoding.
    pub contract_address_xdr_hex: String,
    pub executable: ExecutableForm,
    pub contract_data: Vec<ContractDataEntrySnapshot>,
    /// The ledger sequence this snapshot was captured at.
    pub ledger_sequence: u32,
    /// The ledger close time this snapshot was captured at, when known.
    pub ledger_close_time: Option<u64>,
    /// Where this snapshot came from.
    pub source: EvidenceSource,
}

impl StateSnapshot {
    /// Build the contract address field from an official `ScAddress`.
    pub fn encode_address(address: &ScAddress) -> Result<String, AnalyzerError> {
        encode_xdr_hex(address)
    }
}

fn encode_xdr_hex<T: WriteXdr>(value: &T) -> Result<String, AnalyzerError> {
    let bytes = value
        .to_xdr(Limits::none())
        .map_err(|source| AnalysisError::with_source("failed to encode XDR value", source))?;
    Ok(hex::encode(bytes))
}

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::expect_used)]
mod tests {
    use super::*;
    use stellar_xdr::{ContractExecutableExternalRef, ContractId, Hash, ScString, StringM};

    #[test]
    fn wasm_executable_form_carries_hex_hash() {
        let hash = Hash([7u8; 32]);
        let form = ExecutableForm::from_xdr(&ContractExecutable::Wasm(hash)).unwrap();
        assert_eq!(
            form,
            ExecutableForm::Wasm {
                hash: hex::encode([7u8; 32])
            }
        );
    }

    #[test]
    fn stellar_asset_executable_form_has_no_wasm() {
        let form = ExecutableForm::from_xdr(&ContractExecutable::StellarAsset).unwrap();
        assert_eq!(form, ExecutableForm::StellarAsset);
    }

    #[test]
    fn external_ref_executable_form_preserves_owner_and_tag() {
        let owner = ScAddress::Contract(ContractId(Hash([9u8; 32])));
        let executable = ContractExecutable::ExternalRef(ContractExecutableExternalRef {
            executable_owner: owner.clone(),
            tag: ScString::from(StringM::try_from("v1").unwrap()),
        });
        let form = ExecutableForm::from_xdr(&executable).unwrap();
        match form {
            ExecutableForm::ExternalRef {
                executable_owner_xdr_hex,
                tag,
            } => {
                assert_eq!(executable_owner_xdr_hex, encode_xdr_hex(&owner).unwrap());
                assert_eq!(tag, "v1");
            }
            other => panic!("expected ExternalRef, got {other:?}"),
        }
    }

    #[test]
    fn snapshot_serializes_deterministically() {
        let snapshot = StateSnapshot {
            contract_address_xdr_hex: "ab".repeat(18),
            executable: ExecutableForm::Wasm {
                hash: "cd".repeat(32),
            },
            contract_data: vec![ContractDataEntrySnapshot {
                key_xdr_hex: "01".to_string(),
                val_xdr_hex: "02".to_string(),
                durability: Durability::Persistent,
                live_until_ledger_seq: Some(1000),
            }],
            ledger_sequence: 42,
            ledger_close_time: Some(1_700_000_000),
            source: EvidenceSource::LedgerEntry {
                key_summary: "contract instance".to_string(),
            },
        };

        let first = serde_json::to_string(&snapshot).unwrap();
        let second = serde_json::to_string(&snapshot).unwrap();
        assert_eq!(first, second);

        let parsed: StateSnapshot = serde_json::from_str(&first).unwrap();
        assert_eq!(parsed, snapshot);
    }
}
