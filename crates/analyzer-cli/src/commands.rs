//! CLI command implementations.
//!
//! Each function here does exactly three things: turn CLI arguments
//! into inputs the analysis crates already understand, call the
//! existing pipeline (see [`crate::orchestration`]), and render the
//! result. No analysis logic lives here; parsing a migration manifest
//! or rehearsal input file is plumbing (reading bytes and calling an
//! existing `parse`/`Deserialize` implementation), not analysis.

use std::io::Write;
use std::path::Path;

use analyzer_core::{AnalyzerError, BackendError, SerializationError};
use analyzer_rehearsal::RehearsalInput;
use analyzer_report::{render_terminal, to_canonical_json};
use analyzer_state::MigrationManifest;

use crate::cli::{AnalyzeArgs, OutputFormat};
use crate::orchestration::{run_upgrade_analysis, AnalysisRequest};

/// Process exit codes this CLI uses. See [`crate::cli`]'s `long_about`
/// for the user-facing documentation of this table.
mod exit_code {
    /// The analysis pipeline ran to completion. The report's own
    /// `status` field carries the analytical outcome.
    pub const SUCCESS: i32 = 0;
    /// Invalid input: bad arguments, an executable that is not valid
    /// WASM, a malformed migration manifest or rehearsal file, or a
    /// report serialization failure.
    pub const INVALID_INPUT: i32 = 2;
    /// A required input file could not be read.
    pub const BACKEND_FAILURE: i32 = 3;
    /// An internal analysis-stage failure.
    pub const ANALYSIS_FAILURE: i32 = 5;
}

fn exit_code_for(err: &AnalyzerError) -> i32 {
    match err {
        AnalyzerError::InvalidInput(_)
        | AnalyzerError::UnsupportedArtifact(_)
        | AnalyzerError::Configuration(_)
        | AnalyzerError::Serialization(_) => exit_code::INVALID_INPUT,
        AnalyzerError::Backend(_) => exit_code::BACKEND_FAILURE,
        AnalyzerError::Analysis(_) => exit_code::ANALYSIS_FAILURE,
    }
}

/// Run `sorobanlabs-analyzer analyze` and return the process exit code.
///
/// Writes the report (JSON or terminal, per `args.format`) to `stdout`
/// on success; writes a concise, actionable error message to `stderr`
/// and returns a non-zero code for any operational failure (missing or
/// unreadable file, malformed WASM, malformed migration manifest,
/// malformed rehearsal input, or serialization failure). A completed
/// analysis is always exit code 0 regardless of its `status`; that
/// status is carried in the report itself, not the process exit code.
pub fn run_analyze(args: &AnalyzeArgs, stdout: &mut dyn Write, stderr: &mut dyn Write) -> i32 {
    let migration_manifest = match &args.migration_manifest {
        Some(path) => match read_migration_manifest(path) {
            Ok(manifest) => Some(manifest),
            Err(err) => return report_error(stderr, &err),
        },
        None => None,
    };

    let rehearsal_input = match &args.rehearsal {
        Some(path) => match read_rehearsal_input(path) {
            Ok(input) => Some(input),
            Err(err) => return report_error(stderr, &err),
        },
        None => None,
    };

    let request = AnalysisRequest {
        current_path: &args.current,
        candidate_path: &args.candidate,
        protocol_context: args.protocol,
        migration_manifest: migration_manifest.as_ref(),
        analyzer_version: env!("CARGO_PKG_VERSION"),
        rehearsal_input: rehearsal_input.as_ref(),
    };

    let report = match run_upgrade_analysis(&request) {
        Ok(report) => report,
        Err(err) => return report_error(stderr, &err),
    };

    match args.format {
        OutputFormat::Json => match to_canonical_json(&report) {
            Ok(json) => {
                let _ = write!(stdout, "{json}");
                exit_code::SUCCESS
            }
            Err(err) => report_error(stderr, &err),
        },
        OutputFormat::Terminal => {
            let _ = write!(stdout, "{}", render_terminal(&report));
            exit_code::SUCCESS
        }
    }
}

fn report_error(stderr: &mut dyn Write, err: &AnalyzerError) -> i32 {
    let _ = writeln!(stderr, "error: {err}");
    exit_code_for(err)
}

fn read_migration_manifest(path: &Path) -> Result<MigrationManifest, AnalyzerError> {
    let text = std::fs::read_to_string(path).map_err(|source| {
        BackendError::with_source(
            format!("failed to read migration manifest at '{}'", path.display()),
            source,
        )
    })?;
    MigrationManifest::parse_json(&text).map_err(|source| {
        SerializationError::with_source(
            format!("failed to parse migration manifest at '{}'", path.display()),
            source,
        )
        .into()
    })
}

fn read_rehearsal_input(path: &Path) -> Result<RehearsalInput, AnalyzerError> {
    let text = std::fs::read_to_string(path).map_err(|source| {
        BackendError::with_source(
            format!("failed to read rehearsal input at '{}'", path.display()),
            source,
        )
    })?;
    serde_json::from_str(&text).map_err(|source| {
        SerializationError::with_source(
            format!("failed to parse rehearsal input at '{}'", path.display()),
            source,
        )
        .into()
    })
}

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::expect_used)]
mod tests {
    use super::*;
    use crate::cli::OutputFormat;
    use std::path::PathBuf;

    fn fixture(name: &str) -> PathBuf {
        let mut p = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        p.push("../../fixtures/executable");
        p.push(name);
        p
    }

    fn args(current: PathBuf, candidate: PathBuf, format: OutputFormat) -> AnalyzeArgs {
        AnalyzeArgs {
            current,
            candidate,
            protocol: None,
            migration_manifest: None,
            rehearsal: None,
            format,
        }
    }

    #[test]
    fn nonexistent_current_file_exits_with_backend_failure() {
        let mut out = Vec::new();
        let mut err = Vec::new();
        let a = args(
            PathBuf::from("/nonexistent/current.wasm"),
            fixture("v1.wasm"),
            OutputFormat::Json,
        );
        let code = run_analyze(&a, &mut out, &mut err);
        assert_eq!(code, exit_code::BACKEND_FAILURE);
        assert!(out.is_empty());
        let message = String::from_utf8(err).unwrap();
        assert!(message.starts_with("error: "));
        assert!(!message.contains("Backend(")); // no Rust debug dump
    }

    #[test]
    fn valid_analysis_json_output_is_well_formed() {
        let mut out = Vec::new();
        let mut err = Vec::new();
        let a = args(
            fixture("v1.wasm"),
            fixture("v2_identical.wasm"),
            OutputFormat::Json,
        );
        let code = run_analyze(&a, &mut out, &mut err);
        assert_eq!(code, exit_code::SUCCESS);
        assert!(err.is_empty());
        let json = String::from_utf8(out).unwrap();
        let value: serde_json::Value = serde_json::from_str(&json).unwrap();
        assert!(value.get("status").is_some());
    }

    #[test]
    fn valid_analysis_terminal_output_mentions_status() {
        let mut out = Vec::new();
        let mut err = Vec::new();
        let a = args(
            fixture("v1.wasm"),
            fixture("v2_identical.wasm"),
            OutputFormat::Terminal,
        );
        let code = run_analyze(&a, &mut out, &mut err);
        assert_eq!(code, exit_code::SUCCESS);
        let text = String::from_utf8(out).unwrap();
        assert!(text.contains("status:"));
    }

    #[test]
    fn malformed_migration_manifest_is_rejected() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("manifest.json");
        std::fs::write(&path, "not valid json").unwrap();

        let mut a = args(
            fixture("v1.wasm"),
            fixture("v2_identical.wasm"),
            OutputFormat::Json,
        );
        a.migration_manifest = Some(path);

        let mut out = Vec::new();
        let mut err = Vec::new();
        let code = run_analyze(&a, &mut out, &mut err);
        assert_eq!(code, exit_code::INVALID_INPUT);
        assert!(out.is_empty());
        let message = String::from_utf8(err).unwrap();
        assert!(message.contains("failed to parse migration manifest"));
    }

    #[test]
    fn malformed_rehearsal_input_is_rejected() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("rehearsal.json");
        std::fs::write(&path, "not valid json").unwrap();

        let mut a = args(
            fixture("v1.wasm"),
            fixture("v2_identical.wasm"),
            OutputFormat::Json,
        );
        a.rehearsal = Some(path);

        let mut out = Vec::new();
        let mut err = Vec::new();
        let code = run_analyze(&a, &mut out, &mut err);
        assert_eq!(code, exit_code::INVALID_INPUT);
        let message = String::from_utf8(err).unwrap();
        assert!(message.contains("failed to parse rehearsal input"));
    }
}
