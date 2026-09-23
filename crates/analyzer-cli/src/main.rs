//! Command-line entry point for the SorobanLabs Analyzer.
//!
//! This binary only parses arguments and dispatches; see
//! `analyzer_cli::cli` for the argument surface and
//! `analyzer_cli::commands` for what each command does. The actual
//! analysis pipeline lives in `analyzer_cli::orchestration` and the
//! underlying `analyzer-*` crates, never here.

use analyzer_cli::cli::{Cli, Commands};
use analyzer_cli::commands::run_analyze;
use clap::Parser;

fn main() {
    let cli = Cli::parse();

    let code = match cli.command {
        Commands::Analyze(args) => {
            run_analyze(&args, &mut std::io::stdout(), &mut std::io::stderr())
        }
    };

    std::process::exit(code);
}
