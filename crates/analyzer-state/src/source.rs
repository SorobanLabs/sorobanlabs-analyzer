//! Provider-neutral access to contract state.
//!
//! [`StateSource`] is the seam between "how a snapshot was obtained"
//! and "what the analyzer does with it": every later analysis stage
//! only ever sees a [`crate::snapshot::StateSnapshot`], never a
//! source-specific type. [`LocalSnapshotSource`] reads a snapshot
//! already captured to a local JSON file. A read-only RPC source is
//! added in a later step without changing this trait.

use std::path::{Path, PathBuf};

use analyzer_core::{AnalyzerError, BackendError, InvalidInputError, SerializationError};

use crate::snapshot::StateSnapshot;

/// A provider-neutral source of one contract's state snapshot.
pub trait StateSource {
    /// Obtain the snapshot. Implementations must not mutate any remote
    /// state as a side effect of loading.
    fn load(&self) -> Result<StateSnapshot, AnalyzerError>;
}

/// Reads a [`StateSnapshot`] already captured to a local JSON file.
///
/// This does not capture a snapshot from a live source itself; it loads
/// one that was already written out (by the read-only RPC source, by a
/// test fixture, or by a user-supplied file matching the snapshot
/// shape).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LocalSnapshotSource {
    path: PathBuf,
}

impl LocalSnapshotSource {
    /// Construct a source that will read `path` when [`Self::load`] is
    /// called.
    pub fn new(path: impl AsRef<Path>) -> Self {
        Self {
            path: path.as_ref().to_path_buf(),
        }
    }
}

impl StateSource for LocalSnapshotSource {
    fn load(&self) -> Result<StateSnapshot, AnalyzerError> {
        if self.path.as_os_str().is_empty() {
            return Err(InvalidInputError::new("state snapshot path must not be empty").into());
        }

        let text = std::fs::read_to_string(&self.path).map_err(|source| {
            BackendError::with_source(
                format!("failed to read state snapshot at '{}'", self.path.display()),
                source,
            )
        })?;

        serde_json::from_str(&text).map_err(|source| {
            SerializationError::with_source("failed to parse state snapshot", source).into()
        })
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::expect_used)]
mod tests {
    use super::*;
    use crate::snapshot::ExecutableForm;
    use analyzer_evidence::EvidenceSource;
    use std::io::Write;

    fn sample_snapshot_json() -> String {
        let snapshot = StateSnapshot {
            contract_address_xdr_hex: "ab".repeat(18),
            executable: ExecutableForm::Wasm {
                hash: "cd".repeat(32),
            },
            contract_data: vec![],
            ledger_sequence: 100,
            ledger_close_time: None,
            source: EvidenceSource::LedgerEntry {
                key_summary: "contract instance".to_string(),
            },
        };
        serde_json::to_string(&snapshot).unwrap()
    }

    #[test]
    fn loads_a_valid_local_snapshot_file() {
        let mut file = tempfile::NamedTempFile::new().unwrap();
        file.write_all(sample_snapshot_json().as_bytes()).unwrap();

        let source = LocalSnapshotSource::new(file.path());
        let snapshot = source.load().unwrap();
        assert_eq!(snapshot.ledger_sequence, 100);
    }

    #[test]
    fn missing_file_returns_structured_backend_error() {
        let source = LocalSnapshotSource::new("/nonexistent/path/snapshot.json");
        let result = source.load();
        assert!(matches!(result, Err(AnalyzerError::Backend(_))));
    }

    #[test]
    fn malformed_json_returns_structured_serialization_error() {
        let mut file = tempfile::NamedTempFile::new().unwrap();
        file.write_all(b"not valid json").unwrap();

        let source = LocalSnapshotSource::new(file.path());
        let result = source.load();
        assert!(matches!(result, Err(AnalyzerError::Serialization(_))));
    }

    #[test]
    fn empty_path_returns_invalid_input_error() {
        let source = LocalSnapshotSource::new("");
        assert!(matches!(source.load(), Err(AnalyzerError::InvalidInput(_))));
    }
}
