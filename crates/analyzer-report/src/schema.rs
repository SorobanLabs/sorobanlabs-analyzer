//! Schema version identity for the canonical analysis report.
//!
//! Consumers key their parsing on this string, not on the crate's own
//! `version.workspace` package version: the report schema and the
//! analyzer's own release cadence are independent. A change that breaks
//! existing consumers (removing a field, changing a field's meaning or
//! type, changing an enum's textual values) must increment this
//! version. A purely additive, backward-compatible change (a new
//! optional field) may keep the same version, but should still be
//! recorded in `schemas/analysis-result.schema.json` and in
//! `docs/analysis-model.md`.

/// The current canonical report schema version.
pub const REPORT_SCHEMA_VERSION: &str = "1.0.0";
