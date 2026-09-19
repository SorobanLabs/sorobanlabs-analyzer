//! Library half of the SorobanLabs Analyzer CLI: pipeline orchestration.
//!
//! The binary (`src/main.rs`) stays a thin CLI entry point; the actual
//! sequencing of calls into the analysis crates lives here so it is
//! unit-testable independent of argument parsing. See
//! [`orchestration`] for why this orchestration lives in
//! `analyzer-cli` rather than `analyzer-core`.

pub mod orchestration;
