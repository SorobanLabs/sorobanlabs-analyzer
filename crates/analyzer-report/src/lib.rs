//! Report rendering for the SorobanLabs Analyzer.
//!
//! Produces the canonical JSON report ([`canonical::AnalysisReport`],
//! [`json`]), validated against `schemas/analysis-result.schema.json`,
//! and a human-readable terminal rendering of the same data
//! ([`terminal`]). Output ordering is deterministic and canonical
//! results exclude nondeterministic values such as timestamps or random
//! identifiers.
//!
//! Markdown rendering is added if and when a concrete consumer needs it.

mod canonical;
mod json;
mod schema;
mod terminal;

pub use canonical::{
    AnalysisReport, ReportEvidence, ReportExecutableIdentity, ReportFinding, ReportRehearsal,
};
pub use json::{from_canonical_json, to_canonical_json};
pub use schema::REPORT_SCHEMA_VERSION;
pub use terminal::render_terminal;
