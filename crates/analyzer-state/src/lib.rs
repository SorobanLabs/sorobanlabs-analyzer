//! Contract state representation and compatibility analysis for the
//! SorobanLabs Analyzer.
//!
//! Responsibilities: local state snapshot loading, ledger entry
//! normalization, and comparison of known state requirements between a
//! current and candidate executable. Compatibility outcomes must
//! distinguish COMPATIBLE, REQUIRES_MIGRATION, POTENTIALLY_INCOMPATIBLE,
//! and NOT_DETERMINED; the analyzer must never claim compatibility when
//! evidence is missing.
//!
//! This initial version establishes the crate only.
