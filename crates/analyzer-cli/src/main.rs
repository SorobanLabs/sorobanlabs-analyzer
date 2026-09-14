//! Command-line entry point for the SorobanLabs Analyzer.
//!
//! This crate is an orchestration layer only; it must not contain
//! analysis algorithms. Subcommands (inspect, diff, analyze, state,
//! auth, report) are added as the underlying analysis crates implement
//! the corresponding functionality; no subcommand is exposed before it
//! is backed by real analysis.

use clap::Parser;

/// SorobanLabs Analyzer: analyzes the impact of replacing a deployed
/// Soroban contract's executable.
#[derive(Parser, Debug)]
#[command(name = "sorobanlabs-analyzer", version, about)]
struct Cli;

fn main() {
    let _cli = Cli::parse();
    println!("sorobanlabs-analyzer: no analysis commands are implemented yet.");
}
