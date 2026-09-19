//! What kind of thing a piece of evidence was examined from.
//!
//! Each variant carries enough to identify *which* underlying artifact,
//! response, or computation the evidence came from, without embedding
//! the full content: a byte hash, a section or key name, an endpoint
//! and method, or a short description. Full content belongs to the
//! producing subsystem (the artifact loader, the state snapshot, the
//! RPC client), not to the evidence record itself.

use serde::{Deserialize, Serialize};

/// Where a piece of evidence originated.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum EvidenceSource {
    /// A local WASM artifact file, identified by its canonical byte
    /// hash (see `analyzer_executable::ArtifactHash`).
    LocalArtifact { artifact_hash: String },
    /// A named custom WASM section extracted from an artifact.
    CustomWasmSection {
        artifact_hash: String,
        section_name: String,
    },
    /// A value decoded from XDR bytes belonging to some other evidence.
    XdrDecodedValue {
        type_name: String,
        artifact_hash: String,
    },
    /// A response from a Soroban RPC endpoint.
    RpcResponse { method: String, endpoint: String },
    /// A ledger entry read from network or local state.
    LedgerEntry { key_summary: String },
    /// An observation captured during controlled rehearsal execution.
    ExecutionTrace { invocation: String },
    /// The output of evaluating a specific analyzer rule.
    RuleEvaluation { rule_id: String },
    /// An author-supplied migration manifest.
    MigrationManifest { manifest_hash: String },
    /// A comparison computed from two other pieces of evidence
    /// (identified by their own evidence ids).
    DerivedComparison { inputs: Vec<String> },
}

impl EvidenceSource {
    /// A short, stable label for the kind of source, used in
    /// human-readable output and as part of the canonical serialization
    /// evidence identity is derived from.
    pub fn kind(&self) -> &'static str {
        match self {
            Self::LocalArtifact { .. } => "local_artifact",
            Self::CustomWasmSection { .. } => "custom_wasm_section",
            Self::XdrDecodedValue { .. } => "xdr_decoded_value",
            Self::RpcResponse { .. } => "rpc_response",
            Self::LedgerEntry { .. } => "ledger_entry",
            Self::ExecutionTrace { .. } => "execution_trace",
            Self::RuleEvaluation { .. } => "rule_evaluation",
            Self::MigrationManifest { .. } => "migration_manifest",
            Self::DerivedComparison { .. } => "derived_comparison",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn kind_is_stable_per_variant() {
        assert_eq!(
            EvidenceSource::LocalArtifact {
                artifact_hash: "abc".to_string()
            }
            .kind(),
            "local_artifact"
        );
        assert_eq!(
            EvidenceSource::RpcResponse {
                method: "getLedgerEntries".to_string(),
                endpoint: "https://soroban-testnet.stellar.org".to_string(),
            }
            .kind(),
            "rpc_response"
        );
    }
}
