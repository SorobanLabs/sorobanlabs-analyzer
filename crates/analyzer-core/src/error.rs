//! Common typed error model for the SorobanLabs Analyzer.
//!
//! [`AnalyzerError`] is the shared error type other workspace crates use
//! (or map into) when an operation cannot be completed. It distinguishes
//! six categories of operational failure:
//!
//! - [`InvalidInputError`]: the caller supplied invalid arguments,
//!   paths, identifiers, or analysis options.
//! - [`UnsupportedArtifactError`]: the input exists and may be valid,
//!   but this analyzer cannot currently interpret it (for example, a
//!   WASM structure or Soroban executable representation it does not
//!   yet support).
//! - [`AnalysisError`]: the artifact was loaded and understood, but an
//!   analysis operation itself failed (normalization, an internal
//!   invariant, an inconsistent intermediate representation).
//! - [`BackendError`]: an external data source failed (filesystem, RPC,
//!   or another backend the analyzer reads from).
//! - [`ConfigurationError`]: analyzer or policy configuration is invalid
//!   or could not be loaded.
//! - [`SerializationError`]: serialization or deserialization of data
//!   the analyzer produces or consumes failed.
//!
//! # Errors versus findings
//!
//! `AnalyzerError` means the analyzer could not successfully perform the
//! requested operation at all. It is distinct from a *finding*: findings
//! are the result of a successful analysis that discovered something
//! about the contract (for example, a removed function, or a state
//! structure the analyzer cannot classify, reported with confidence
//! `NOT_DETERMINABLE`). A finding is never represented as an
//! `AnalyzerError`, and an `AnalyzerError` never carries a confidence or
//! severity; those concepts belong to the analysis result model.

/// A boxed, thread-safe source error. Used by categories that commonly
/// wrap a lower-level failure (I/O, an external parser, and so on)
/// while keeping the top-level error type concrete and inspectable.
type BoxedSource = Box<dyn std::error::Error + Send + Sync + 'static>;

/// The common error type for the SorobanLabs Analyzer.
///
/// Every variant wraps a category-specific error type that carries the
/// actual context. Match on the variant to distinguish the category of
/// failure; use the category error type directly when only that
/// category is possible at a given call site.
#[derive(Debug, thiserror::Error)]
pub enum AnalyzerError {
    /// The caller supplied invalid arguments, paths, values,
    /// identifiers, or analysis options. See [`InvalidInputError`].
    #[error(transparent)]
    InvalidInput(#[from] InvalidInputError),

    /// The artifact exists but is not one this analyzer currently
    /// supports (wrong type, unsupported version, unsupported
    /// encoding, or an unsupported structural feature). See
    /// [`UnsupportedArtifactError`].
    #[error(transparent)]
    UnsupportedArtifact(#[from] UnsupportedArtifactError),

    /// The artifact was loaded and understood, but an analysis
    /// operation itself failed. See [`AnalysisError`].
    #[error(transparent)]
    Analysis(#[from] AnalysisError),

    /// An external data source (filesystem, RPC, or another backend)
    /// failed. See [`BackendError`].
    #[error(transparent)]
    Backend(#[from] BackendError),

    /// Analyzer or policy configuration is invalid or could not be
    /// loaded. See [`ConfigurationError`].
    #[error(transparent)]
    Configuration(#[from] ConfigurationError),

    /// Serialization or deserialization failed. See
    /// [`SerializationError`].
    #[error(transparent)]
    Serialization(#[from] SerializationError),
}

/// The caller supplied invalid arguments, paths, values, identifiers,
/// or analysis options.
///
/// Use this for problems the caller could have avoided by supplying
/// different input: a missing required field, a malformed identifier,
/// contradictory analysis options, or a reference to a state snapshot
/// that does not exist. Do not use this for a well-formed artifact this
/// analyzer cannot interpret; that is [`UnsupportedArtifactError`].
#[derive(Debug, thiserror::Error)]
#[error("invalid input: {message}")]
pub struct InvalidInputError {
    message: String,
}

impl InvalidInputError {
    /// Construct an invalid-input error with a human-readable, actionable message.
    pub fn new(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
        }
    }

    /// The human-readable message describing the invalid input.
    pub fn message(&self) -> &str {
        &self.message
    }
}

/// The input exists and may be valid, but this analyzer cannot
/// currently analyze it.
///
/// Use this when an artifact is present but is not a supported type,
/// uses an unsupported encoding or version, or relies on a structural
/// feature this analyzer does not yet understand. This includes
/// malformed WASM: the bytes exist, but they cannot be interpreted as a
/// supported Soroban artifact.
#[derive(Debug, thiserror::Error)]
#[error("unsupported artifact: {message}")]
pub struct UnsupportedArtifactError {
    message: String,
}

impl UnsupportedArtifactError {
    /// Construct an unsupported-artifact error with a human-readable message.
    pub fn new(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
        }
    }

    /// The human-readable message describing why the artifact is unsupported.
    pub fn message(&self) -> &str {
        &self.message
    }
}

/// The artifact was loaded and understood, but an analysis operation
/// itself failed.
///
/// Use this for normalization failures, violated internal invariants,
/// or an inconsistent intermediate representation encountered while the
/// analyzer was actively computing a result. Do not use this for
/// ordinary invalid user input; that is [`InvalidInputError`].
#[derive(Debug, thiserror::Error)]
#[error("analysis failed: {message}")]
pub struct AnalysisError {
    message: String,
    #[source]
    source: Option<BoxedSource>,
}

impl AnalysisError {
    /// Construct an analysis error with no known underlying cause.
    pub fn new(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
            source: None,
        }
    }

    /// Construct an analysis error that wraps an underlying cause.
    pub fn with_source(
        message: impl Into<String>,
        source: impl std::error::Error + Send + Sync + 'static,
    ) -> Self {
        Self {
            message: message.into(),
            source: Some(Box::new(source)),
        }
    }

    /// The human-readable message describing the analysis failure.
    pub fn message(&self) -> &str {
        &self.message
    }
}

/// An external data source (filesystem, RPC, or another backend) failed.
///
/// Use this when the analyzer could not retrieve data it needed from an
/// external source: a local file read fails, an RPC request fails, a
/// remote endpoint returns a response the analyzer cannot use, or a
/// backend times out. The message should identify the operation that
/// failed without exposing secrets (private keys, authorization tokens,
/// API credentials, or full secret configuration values).
#[derive(Debug, thiserror::Error)]
#[error("backend operation failed: {message}")]
pub struct BackendError {
    message: String,
    #[source]
    source: Option<BoxedSource>,
}

impl BackendError {
    /// Construct a backend error with no known underlying cause.
    pub fn new(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
            source: None,
        }
    }

    /// Construct a backend error that wraps an underlying cause, such as
    /// an [`std::io::Error`] from a local filesystem read.
    pub fn with_source(
        message: impl Into<String>,
        source: impl std::error::Error + Send + Sync + 'static,
    ) -> Self {
        Self {
            message: message.into(),
            source: Some(Box::new(source)),
        }
    }

    /// The human-readable message describing the backend failure.
    pub fn message(&self) -> &str {
        &self.message
    }
}

/// Analyzer or policy configuration is invalid or could not be loaded.
///
/// Use this for malformed configuration files, invalid policy settings,
/// unsupported configuration combinations, or a missing required
/// configuration value. Keep this distinct from
/// [`SerializationError`]: a configuration file that fails to parse is
/// a configuration problem, not a generic serialization problem, even
/// though a parser error may be the underlying cause.
#[derive(Debug, thiserror::Error)]
#[error("invalid configuration: {message}")]
pub struct ConfigurationError {
    message: String,
    #[source]
    source: Option<BoxedSource>,
}

impl ConfigurationError {
    /// Construct a configuration error with no known underlying cause.
    pub fn new(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
            source: None,
        }
    }

    /// Construct a configuration error that wraps an underlying cause,
    /// such as a TOML parser error.
    pub fn with_source(
        message: impl Into<String>,
        source: impl std::error::Error + Send + Sync + 'static,
    ) -> Self {
        Self {
            message: message.into(),
            source: Some(Box::new(source)),
        }
    }

    /// The human-readable message describing the configuration failure.
    pub fn message(&self) -> &str {
        &self.message
    }
}

/// Serialization or deserialization failed.
///
/// Use this for failures converting analyzer data to or from an
/// external representation: invalid JSON, invalid TOML representation,
/// canonical report serialization failure, or a schema-related
/// representation failure. Do not use this for a configuration file
/// that fails to parse as application configuration; that is
/// [`ConfigurationError`], even though the underlying cause may be the
/// same kind of parser error.
#[derive(Debug, thiserror::Error)]
#[error("serialization failed: {message}")]
pub struct SerializationError {
    message: String,
    #[source]
    source: Option<BoxedSource>,
}

impl SerializationError {
    /// Construct a serialization error with no known underlying cause.
    pub fn new(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
            source: None,
        }
    }

    /// Construct a serialization error that wraps an underlying cause,
    /// such as a JSON encoder or decoder error.
    pub fn with_source(
        message: impl Into<String>,
        source: impl std::error::Error + Send + Sync + 'static,
    ) -> Self {
        Self {
            message: message.into(),
            source: Some(Box::new(source)),
        }
    }

    /// The human-readable message describing the serialization failure.
    pub fn message(&self) -> &str {
        &self.message
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::expect_used)]
mod tests {
    use super::*;

    #[test]
    fn invalid_input_constructs_and_displays() {
        let err = InvalidInputError::new("missing required field 'contract_id'");
        assert_eq!(err.message(), "missing required field 'contract_id'");
        assert_eq!(
            err.to_string(),
            "invalid input: missing required field 'contract_id'"
        );
    }

    #[test]
    fn unsupported_artifact_constructs_and_displays() {
        let err = UnsupportedArtifactError::new("unsupported executable format: not a WASM module");
        assert_eq!(
            err.to_string(),
            "unsupported artifact: unsupported executable format: not a WASM module"
        );
    }

    #[test]
    fn analysis_constructs_and_displays() {
        let err = AnalysisError::new("normalization produced an inconsistent function table");
        assert_eq!(
            err.to_string(),
            "analysis failed: normalization produced an inconsistent function table"
        );
    }

    #[test]
    fn backend_constructs_and_displays() {
        let err = BackendError::new("failed to fetch contract state from backend: timed out");
        assert_eq!(
            err.to_string(),
            "backend operation failed: failed to fetch contract state from backend: timed out"
        );
    }

    #[test]
    fn configuration_constructs_and_displays() {
        let err = ConfigurationError::new("policy.unknown_as_blocker must be a boolean");
        assert_eq!(
            err.to_string(),
            "invalid configuration: policy.unknown_as_blocker must be a boolean"
        );
    }

    #[test]
    fn serialization_constructs_and_displays() {
        let err = SerializationError::new("report is not valid JSON");
        assert_eq!(
            err.to_string(),
            "serialization failed: report is not valid JSON"
        );
    }

    #[test]
    fn backend_error_preserves_source_context() {
        let io_err = std::io::Error::new(std::io::ErrorKind::NotFound, "snapshot.json not found");
        let err = BackendError::with_source("failed to read local state snapshot", io_err);

        let source = std::error::Error::source(&err).expect("source should be preserved");
        assert_eq!(source.to_string(), "snapshot.json not found");
    }

    #[test]
    fn analysis_error_preserves_source_context() {
        let parse_err = "unexpected token".to_string();
        #[derive(Debug, thiserror::Error)]
        #[error("{0}")]
        struct DummySource(String);

        let err =
            AnalysisError::with_source("failed to normalize contract spec", DummySource(parse_err));
        let source = std::error::Error::source(&err).expect("source should be preserved");
        assert_eq!(source.to_string(), "unexpected token");
    }

    #[test]
    fn each_category_converts_into_analyzer_error() {
        let invalid: AnalyzerError = InvalidInputError::new("bad input").into();
        let unsupported: AnalyzerError = UnsupportedArtifactError::new("bad artifact").into();
        let analysis: AnalyzerError = AnalysisError::new("bad analysis").into();
        let backend: AnalyzerError = BackendError::new("bad backend").into();
        let configuration: AnalyzerError = ConfigurationError::new("bad configuration").into();
        let serialization: AnalyzerError = SerializationError::new("bad serialization").into();

        assert!(matches!(invalid, AnalyzerError::InvalidInput(_)));
        assert!(matches!(unsupported, AnalyzerError::UnsupportedArtifact(_)));
        assert!(matches!(analysis, AnalyzerError::Analysis(_)));
        assert!(matches!(backend, AnalyzerError::Backend(_)));
        assert!(matches!(configuration, AnalyzerError::Configuration(_)));
        assert!(matches!(serialization, AnalyzerError::Serialization(_)));
    }

    #[test]
    fn analyzer_error_is_a_standard_error() {
        fn assert_is_std_error<E: std::error::Error>() {}
        assert_is_std_error::<AnalyzerError>();

        fn returns_boxed_std_error() -> Result<(), Box<dyn std::error::Error>> {
            let err: AnalyzerError = InvalidInputError::new("example").into();
            Err(Box::new(err))
        }
        assert!(returns_boxed_std_error().is_err());
    }

    #[test]
    fn questionmark_propagates_category_errors_into_analyzer_error() {
        fn fallible() -> Result<(), AnalyzerError> {
            Err(InvalidInputError::new("propagated via ?"))?;
            Ok(())
        }

        let err = fallible().unwrap_err();
        assert_eq!(err.to_string(), "invalid input: propagated via ?");
    }
}
