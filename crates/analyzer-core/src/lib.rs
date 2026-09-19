//! Domain model and orchestration for the SorobanLabs Analyzer.
//!
//! This crate owns the concepts shared across the analyzer: analysis
//! inputs, policy, results, findings, confidence, severity, status, and
//! upgrade plans. It orchestrates the other crates rather than performing
//! low-level Stellar/Soroban parsing itself.
//!
//! The domain model is introduced incrementally. This version
//! establishes the shared typed error model (see [`error`]) and the
//! finding model (confidence, severity, rules, deterministic finding
//! ids, and the overall analysis status; see [`findings`] and
//! [`analysis`]). The remaining domain types (analysis input, policy,
//! upgrade plans) are added in later steps.

mod analysis;
mod error;
mod findings;

pub use analysis::AnalysisStatus;
pub use error::{
    AnalysisError, AnalyzerError, BackendError, ConfigurationError, InvalidInputError,
    SerializationError, UnsupportedArtifactError,
};
pub use findings::{Confidence, Finding, FindingCategory, FindingId, Rule, Severity};
