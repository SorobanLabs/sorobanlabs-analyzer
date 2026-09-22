//! Command-line argument definitions for the SorobanLabs Analyzer.
//!
//! This module only defines the CLI's surface (clap types); it performs
//! no file I/O and calls into no analysis crate. See [`crate::commands`]
//! for what each command actually does.

use std::path::PathBuf;

use clap::{Args, Parser, Subcommand, ValueEnum};

const LONG_ABOUT: &str = "\
SorobanLabs Analyzer determines what can be established about replacing a \
deployed Soroban contract's executable: changes to its identity, interface, \
state compatibility, and authorization surface, plus, when a rehearsal \
manifest is supplied, observed behavioral differences under controlled \
execution.

Every finding carries a confidence (DETECTED, LIKELY, POTENTIAL, or \
NOT_DETERMINABLE) and a severity, reported separately. The analyzer does \
not produce a SAFE/UNSAFE verdict, and it never performs a real contract \
upgrade: it only reads the executables and any state/manifest files it is \
given, and, for rehearsal, runs bounded local execution through the \
existing soroban-env-host backend. It does not access the network, does \
not require a wallet or private key, and does not submit transactions.

EXIT CODES
  0  the analysis pipeline ran to completion. The report's own `status` \
field (NO_DETECTED_BLOCKERS, REVIEW_REQUIRED, MIGRATION_REQUIRED, or \
INCONCLUSIVE) carries the analytical outcome; none of those statuses is \
itself treated as a CLI failure.
  2  invalid input: bad command-line arguments, an empty executable \
path, an executable whose bytes are not valid WASM, a malformed \
migration manifest, a malformed rehearsal input file, or a report \
serialization failure. This is about the CONTENT of what was supplied.
  3  a required input file (executable, migration manifest, or \
rehearsal input) could not be read: missing file, permission denied, \
or the path names a directory. This is about being unable to READ \
what was supplied, before its content is even examined.
  5  an internal analysis-stage failure (the artifact loaded but an \
analysis operation itself could not complete).";

#[derive(Debug, Parser)]
#[command(name = "sorobanlabs-analyzer", version, about, long_about = LONG_ABOUT)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Debug, Subcommand)]
pub enum Commands {
    /// Run the full upgrade analysis pipeline against a current and
    /// candidate executable and print a report.
    Analyze(AnalyzeArgs),
}

#[derive(Debug, Args)]
pub struct AnalyzeArgs {
    /// Path to the current (deployed) contract executable (.wasm).
    #[arg(long)]
    pub current: PathBuf,

    /// Path to the candidate replacement contract executable (.wasm).
    #[arg(long)]
    pub candidate: PathBuf,

    /// The Soroban protocol number to record the analysis against. Carried
    /// through to the report; omit if you don't know it or it doesn't apply.
    #[arg(long)]
    pub protocol: Option<u32>,

    /// Path to an author-supplied migration manifest (JSON). Treated as a
    /// declaration, not proof; see the report's rehearsal/state sections
    /// for what the analyzer itself established.
    #[arg(long = "migration-manifest")]
    pub migration_manifest: Option<PathBuf>,

    /// Path to a rehearsal input file (JSON, the analyzer_rehearsal
    /// RehearsalInput format) describing invocations to run against both
    /// executables under the real soroban-env-host backend. Omit to skip
    /// controlled rehearsal.
    #[arg(long)]
    pub rehearsal: Option<PathBuf>,

    /// Output format.
    #[arg(long, value_enum, default_value_t = OutputFormat::Terminal)]
    pub format: OutputFormat,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, ValueEnum)]
pub enum OutputFormat {
    /// The canonical JSON report, written to stdout.
    Json,
    /// A human-readable plain-text report, written to stdout.
    Terminal,
}
