//! Domain model for the SorobanLabs Analyzer.
//!
//! This crate owns the concepts shared across the analyzer: the typed
//! error model, findings, confidence, severity, rules, the overall
//! analysis status, and the upgrade plan. It does not orchestrate the
//! other analysis crates itself: `analyzer-executable`, `analyzer-state`,
//! `analyzer-auth`, and `analyzer-report` all depend on this crate, so
//! this crate calling back into them would be a dependency cycle. The
//! concrete pipeline that sequences calls into those crates lives in
//! `analyzer-cli` (see that crate's `orchestration` module), which
//! already depends on all of them.
//!
//! The domain model is introduced incrementally. This version
//! establishes the shared typed error model ([`error`]), the finding
//! model (confidence, severity, rules, deterministic finding ids, and
//! the overall analysis status; [`findings`] and [`analysis`]), and the
//! upgrade plan ([`analysis::UpgradePlan`]). Analysis input and policy
//! types are added in later steps.

mod analysis;
mod error;
mod findings;

pub use analysis::{AnalysisStatus, UpgradePlan};
pub use error::{
    AnalysisError, AnalyzerError, BackendError, ConfigurationError, InvalidInputError,
    SerializationError, UnsupportedArtifactError,
};
pub use findings::{Confidence, Finding, FindingCategory, FindingId, Rule, Severity};
