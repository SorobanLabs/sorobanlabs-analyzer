//! Domain model and orchestration for the SorobanLabs Analyzer.
//!
//! This crate owns the concepts shared across the analyzer: analysis
//! inputs, policy, results, findings, confidence, severity, status, and
//! upgrade plans. It orchestrates the other crates rather than performing
//! low-level Stellar/Soroban parsing itself.
//!
//! The domain model is introduced incrementally. This version
//! establishes the shared typed error model (see [`error`]); the
//! remaining domain types (analysis input, policy, findings, confidence,
//! severity, status, upgrade plans) are added in later steps.

mod error;

pub use error::{
    AnalysisError, AnalyzerError, BackendError, ConfigurationError, InvalidInputError,
    SerializationError, UnsupportedArtifactError,
};
