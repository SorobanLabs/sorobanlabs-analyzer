//! Library half of the SorobanLabs Analyzer CLI: argument definitions,
//! command implementations, and pipeline orchestration.
//!
//! The binary (`src/main.rs`) stays a thin entry point that parses
//! [`cli::Cli`], dispatches to [`commands`], and exits with the
//! returned process code. See [`orchestration`] for why the actual
//! pipeline sequencing lives in `analyzer-cli` rather than
//! `analyzer-core`.

pub mod cli;
pub mod commands;
pub mod orchestration;
