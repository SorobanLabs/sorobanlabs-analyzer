//! Report rendering for the SorobanLabs Analyzer.
//!
//! Produces the canonical JSON report ([`canonical::AnalysisReport`],
//! [`json`]), validated against `schemas/analysis-result.schema.json`.
//! Output ordering is deterministic and canonical results exclude
//! nondeterministic values such as timestamps or random identifiers.
//!
//! Markdown and terminal rendering are added when the CLI `report`
//! command that consumes them exists.

mod canonical;
mod json;
mod schema;

pub use canonical::{AnalysisReport, ReportExecutableIdentity, ReportFinding, ReportRehearsal};
pub use json::{from_canonical_json, to_canonical_json};
pub use schema::REPORT_SCHEMA_VERSION;
