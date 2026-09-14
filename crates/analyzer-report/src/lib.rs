//! Report rendering for the SorobanLabs Analyzer.
//!
//! Produces the canonical JSON report (validated against
//! `schemas/analysis-result.schema.json`), Markdown reports, and
//! human-readable terminal reports. Output ordering is deterministic and
//! canonical results exclude nondeterministic values such as timestamps
//! or random identifiers.
//!
//! This initial version establishes the crate only.
