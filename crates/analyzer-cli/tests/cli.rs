//! End-to-end tests of the compiled `sorobanlabs-analyzer` binary.
//!
//! These spawn the real binary (via `CARGO_BIN_EXE_sorobanlabs-analyzer`,
//! set automatically by Cargo for integration tests in this package) and
//! assert on its actual stdout, stderr, and process exit code, so they
//! exercise argument parsing, file I/O, the real analysis pipeline, and
//! rendering together rather than any one layer in isolation.

#![allow(clippy::unwrap_used, clippy::expect_used)]

use std::path::PathBuf;
use std::process::{Command, Output};

use analyzer_rehearsal::{ExecutionLimits, RehearsalInput, RehearsalInvocation};
use analyzer_state::MigrationManifest;

fn bin() -> Command {
    Command::new(env!("CARGO_BIN_EXE_sorobanlabs-analyzer"))
}

fn fixture(name: &str) -> PathBuf {
    let mut p = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    p.push("../../fixtures/executable");
    p.push(name);
    p
}

fn stdout_of(output: &Output) -> String {
    String::from_utf8(output.stdout.clone()).expect("stdout must be UTF-8")
}

fn stderr_of(output: &Output) -> String {
    String::from_utf8(output.stderr.clone()).expect("stderr must be UTF-8")
}

fn i32_xdr(n: i32) -> String {
    use soroban_env_host::xdr::{Limits, ScVal, WriteXdr};
    hex::encode(ScVal::I32(n).to_xdr(Limits::none()).unwrap())
}

fn add_invocation(label: &str, a: i32, b: i32) -> RehearsalInvocation {
    RehearsalInvocation {
        label: label.to_string(),
        function_name: "add".to_string(),
        arguments_xdr_hex: vec![i32_xdr(a), i32_xdr(b)],
    }
}

fn write_rehearsal_input(dir: &std::path::Path, invocations: Vec<RehearsalInvocation>) -> PathBuf {
    write_rehearsal_input_with_embedded_bytes(dir, invocations, vec![], vec![])
}

/// Like [`write_rehearsal_input`], but lets the caller control the
/// embedded `current_executable`/`candidate_executable` bytes, so tests
/// can prove those fields are not consumed by the pipeline (it
/// rehearses the bytes loaded from `--current`/`--candidate` instead;
/// see `analyzer_rehearsal::input`'s module docs and
/// `orchestration.rs`).
fn write_rehearsal_input_with_embedded_bytes(
    dir: &std::path::Path,
    invocations: Vec<RehearsalInvocation>,
    current_executable: Vec<u8>,
    candidate_executable: Vec<u8>,
) -> PathBuf {
    let input = RehearsalInput {
        current_executable,
        candidate_executable,
        state_snapshot: None,
        invocations,
        protocol_context: Some(28),
        execution_limits: ExecutionLimits::default(),
        deterministic_seed: None,
    };
    let path = dir.join("rehearsal.json");
    std::fs::write(&path, serde_json::to_string(&input).unwrap()).unwrap();
    path
}

// ── argument validation ──────────────────────────────────────────────

#[test]
fn missing_current_argument_is_rejected_by_argument_parsing() {
    let output = bin()
        .args(["analyze", "--candidate"])
        .arg(fixture("v1.wasm"))
        .output()
        .expect("failed to run binary");

    assert!(!output.status.success());
    assert_eq!(output.status.code(), Some(2));
    assert!(stderr_of(&output).contains("--current"));
}

#[test]
fn missing_candidate_argument_is_rejected_by_argument_parsing() {
    let output = bin()
        .args(["analyze", "--current"])
        .arg(fixture("v1.wasm"))
        .output()
        .expect("failed to run binary");

    assert!(!output.status.success());
    assert_eq!(output.status.code(), Some(2));
    assert!(stderr_of(&output).contains("--candidate"));
}

// ── missing/unreadable executables ──────────────────────────────────

#[test]
fn nonexistent_current_executable_exits_with_backend_failure() {
    let output = bin()
        .args(["analyze", "--current", "/nonexistent/current.wasm"])
        .args(["--candidate"])
        .arg(fixture("v1.wasm"))
        .output()
        .expect("failed to run binary");

    assert_eq!(output.status.code(), Some(3));
    assert!(stdout_of(&output).is_empty());
    assert!(stderr_of(&output).contains("failed to read artifact"));
}

#[test]
fn nonexistent_candidate_executable_exits_with_backend_failure() {
    let output = bin()
        .args(["analyze", "--current"])
        .arg(fixture("v1.wasm"))
        .args(["--candidate", "/nonexistent/candidate.wasm"])
        .output()
        .expect("failed to run binary");

    assert_eq!(output.status.code(), Some(3));
    assert!(stderr_of(&output).contains("failed to read artifact"));
}

// ── valid analysis, both output formats ─────────────────────────────

#[test]
fn valid_analysis_produces_well_formed_json_matching_the_schema_shape() {
    let output = bin()
        .args(["analyze", "--current"])
        .arg(fixture("v1.wasm"))
        .args(["--candidate"])
        .arg(fixture("v2_identical.wasm"))
        .args(["--format", "json"])
        .output()
        .expect("failed to run binary");

    assert_eq!(output.status.code(), Some(0));
    assert!(stderr_of(&output).is_empty());

    let value: serde_json::Value = serde_json::from_str(&stdout_of(&output)).unwrap();
    assert_eq!(value["schema_version"], "1.0.0");
    assert!(value["status"].is_string());
    assert!(value["current_executable"]["hash"].is_string());
    assert!(value["candidate_executable"]["hash"].is_string());
    assert!(value["findings"].is_array());
}

#[test]
fn valid_analysis_produces_readable_terminal_output() {
    let output = bin()
        .args(["analyze", "--current"])
        .arg(fixture("v1.wasm"))
        .args(["--candidate"])
        .arg(fixture("v2_identical.wasm"))
        .args(["--format", "terminal"])
        .output()
        .expect("failed to run binary");

    assert_eq!(output.status.code(), Some(0));
    let text = stdout_of(&output);
    assert!(text.contains("status:"));
    assert!(text.contains("current executable:"));
    assert!(text.contains("candidate executable:"));
    assert!(text.contains("findings ("));
}

#[test]
fn terminal_is_the_default_format() {
    let output = bin()
        .args(["analyze", "--current"])
        .arg(fixture("v1.wasm"))
        .args(["--candidate"])
        .arg(fixture("v2_identical.wasm"))
        .output()
        .expect("failed to run binary");

    assert_eq!(output.status.code(), Some(0));
    assert!(stdout_of(&output).contains("status:"));
}

// ── protocol context ─────────────────────────────────────────────────

#[test]
fn protocol_context_is_passed_through_to_the_report() {
    let output = bin()
        .args(["analyze", "--current"])
        .arg(fixture("v1.wasm"))
        .args(["--candidate"])
        .arg(fixture("v2_identical.wasm"))
        .args(["--protocol", "28", "--format", "json"])
        .output()
        .expect("failed to run binary");

    assert_eq!(output.status.code(), Some(0));
    let value: serde_json::Value = serde_json::from_str(&stdout_of(&output)).unwrap();
    assert_eq!(value["protocol_context"], 28);
}

#[test]
fn omitted_protocol_context_is_null_in_the_report() {
    let output = bin()
        .args(["analyze", "--current"])
        .arg(fixture("v1.wasm"))
        .args(["--candidate"])
        .arg(fixture("v2_identical.wasm"))
        .args(["--format", "json"])
        .output()
        .expect("failed to run binary");

    let value: serde_json::Value = serde_json::from_str(&stdout_of(&output)).unwrap();
    assert!(value["protocol_context"].is_null());
}

// ── migration manifest ───────────────────────────────────────────────

#[test]
fn valid_migration_manifest_is_accepted() {
    let dir = tempfile::tempdir().unwrap();
    let manifest = MigrationManifest::default();
    let path = dir.path().join("manifest.json");
    std::fs::write(&path, manifest.to_canonical_json().unwrap()).unwrap();

    let output = bin()
        .args(["analyze", "--current"])
        .arg(fixture("v1.wasm"))
        .args(["--candidate"])
        .arg(fixture("v2_identical.wasm"))
        .args(["--migration-manifest"])
        .arg(&path)
        .args(["--format", "json"])
        .output()
        .expect("failed to run binary");

    assert_eq!(output.status.code(), Some(0));
    assert!(stderr_of(&output).is_empty());
}

#[test]
fn malformed_migration_manifest_is_rejected_with_invalid_input_exit_code() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("manifest.json");
    std::fs::write(&path, "not valid json").unwrap();

    let output = bin()
        .args(["analyze", "--current"])
        .arg(fixture("v1.wasm"))
        .args(["--candidate"])
        .arg(fixture("v2_identical.wasm"))
        .args(["--migration-manifest"])
        .arg(&path)
        .output()
        .expect("failed to run binary");

    assert_eq!(output.status.code(), Some(2));
    assert!(stdout_of(&output).is_empty());
    assert!(stderr_of(&output).contains("failed to parse migration manifest"));
}

// ── rehearsal input ───────────────────────────────────────────────────

#[test]
fn valid_rehearsal_input_reaches_the_real_soroban_host_backend() {
    let dir = tempfile::tempdir().unwrap();
    let rehearsal_path = write_rehearsal_input(dir.path(), vec![add_invocation("add(2,3)", 2, 3)]);

    let output = bin()
        .args(["analyze", "--current"])
        .arg(fixture("v1.wasm"))
        .args(["--candidate"])
        .arg(fixture("v2_identical.wasm"))
        .args(["--rehearsal"])
        .arg(&rehearsal_path)
        .args(["--format", "json"])
        .output()
        .expect("failed to run binary");

    assert_eq!(output.status.code(), Some(0));
    let value: serde_json::Value = serde_json::from_str(&stdout_of(&output)).unwrap();
    let rehearsal = &value["rehearsal"];
    assert_eq!(rehearsal["requested"], true);
    assert_eq!(rehearsal["ran"], true);
    assert_eq!(rehearsal["backend_used"], "soroban-env-host");
    assert!(!value["findings"]
        .as_array()
        .unwrap()
        .iter()
        .any(|f| f["rule"] == "REHEARSAL_RESULT_CHANGED"));
}

#[test]
fn embedded_rehearsal_executable_bytes_are_not_consumed_by_the_pipeline() {
    // RehearsalInput.current_executable/candidate_executable are
    // documented as not consumed by this orchestration (it rehearses
    // the --current/--candidate bytes it already loaded instead). Prove
    // that honestly: garbage bytes in those fields must not change the
    // rehearsal outcome versus leaving them empty.
    let dir = tempfile::tempdir().unwrap();
    let rehearsal_path = write_rehearsal_input_with_embedded_bytes(
        dir.path(),
        vec![add_invocation("add(2,3)", 2, 3)],
        b"not a real wasm module at all".to_vec(),
        b"also not a real wasm module".to_vec(),
    );

    let output = bin()
        .args(["analyze", "--current"])
        .arg(fixture("v1.wasm"))
        .args(["--candidate"])
        .arg(fixture("v2_identical.wasm"))
        .args(["--rehearsal"])
        .arg(&rehearsal_path)
        .args(["--format", "json"])
        .output()
        .expect("failed to run binary");

    assert_eq!(output.status.code(), Some(0));
    let value: serde_json::Value = serde_json::from_str(&stdout_of(&output)).unwrap();
    let rehearsal = &value["rehearsal"];
    // If the garbage bytes had been used, the host would have blocked
    // the invocation instead of running it successfully.
    assert_eq!(rehearsal["ran"], true);
    assert_eq!(rehearsal["backend_used"], "soroban-env-host");
    assert!(!value["findings"]
        .as_array()
        .unwrap()
        .iter()
        .any(|f| f["rule"] == "REHEARSAL_FAILED" && f["severity"] == "HIGH"));
}

#[test]
fn rehearsal_behavioral_difference_is_a_finding_not_a_cli_failure() {
    let dir = tempfile::tempdir().unwrap();
    let rehearsal_path = write_rehearsal_input(dir.path(), vec![add_invocation("add(2,3)", 2, 3)]);

    let output = bin()
        .args(["analyze", "--current"])
        .arg(fixture("v1.wasm"))
        .args(["--candidate"])
        .arg(fixture("v2_changed_return.wasm"))
        .args(["--rehearsal"])
        .arg(&rehearsal_path)
        .args(["--format", "json"])
        .output()
        .expect("failed to run binary");

    // A behavioral difference is an analytical finding, not an
    // operational failure: the process must still exit 0.
    assert_eq!(output.status.code(), Some(0));
    let value: serde_json::Value = serde_json::from_str(&stdout_of(&output)).unwrap();
    assert!(value["findings"]
        .as_array()
        .unwrap()
        .iter()
        .any(|f| f["rule"] == "REHEARSAL_RESULT_CHANGED"));
    // REHEARSAL_RESULT_CHANGED is Severity::High/Confidence::Detected,
    // so the overall status escalates to REVIEW_REQUIRED; that escalation
    // is carried in the report's own `status` field, never the exit code.
    assert_eq!(value["status"], "REVIEW_REQUIRED");
}

#[test]
fn invalid_rehearsal_input_is_rejected_with_invalid_input_exit_code() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("rehearsal.json");
    std::fs::write(&path, "not valid json").unwrap();

    let output = bin()
        .args(["analyze", "--current"])
        .arg(fixture("v1.wasm"))
        .args(["--candidate"])
        .arg(fixture("v2_identical.wasm"))
        .args(["--rehearsal"])
        .arg(&path)
        .output()
        .expect("failed to run binary");

    assert_eq!(output.status.code(), Some(2));
    assert!(stderr_of(&output).contains("failed to parse rehearsal input"));
}

// ── canonical JSON round trip ─────────────────────────────────────────

#[test]
fn json_output_round_trips_through_the_canonical_report_type() {
    let output = bin()
        .args(["analyze", "--current"])
        .arg(fixture("v1.wasm"))
        .args(["--candidate"])
        .arg(fixture("v2_identical.wasm"))
        .args(["--format", "json"])
        .output()
        .expect("failed to run binary");

    let stdout = stdout_of(&output);
    let report: analyzer_report::AnalysisReport =
        analyzer_report::from_canonical_json(&stdout).expect("must parse as AnalysisReport");
    let reserialized = analyzer_report::to_canonical_json(&report).expect("must reserialize");
    assert_eq!(stdout, reserialized);
}

// ── --version / --help ────────────────────────────────────────────────

#[test]
fn version_flag_reports_the_workspace_package_version() {
    let output = bin()
        .arg("--version")
        .output()
        .expect("failed to run binary");
    assert!(output.status.success());
    let text = stdout_of(&output);
    assert!(text.contains(env!("CARGO_PKG_VERSION")));
}

#[test]
fn help_flag_documents_analyze_and_exit_behavior() {
    let output = bin().arg("--help").output().expect("failed to run binary");
    assert!(output.status.success());
    let text = stdout_of(&output);
    assert!(text.contains("analyze"));
    assert!(text.contains("EXIT CODES"));
    assert!(text.to_lowercase().contains("does not"));
}

#[test]
fn analyze_help_documents_required_and_optional_inputs() {
    let output = bin()
        .args(["analyze", "--help"])
        .output()
        .expect("failed to run binary");
    assert!(output.status.success());
    let text = stdout_of(&output);
    assert!(text.contains("--current"));
    assert!(text.contains("--candidate"));
    assert!(text.contains("--protocol"));
    assert!(text.contains("--migration-manifest"));
    assert!(text.contains("--rehearsal"));
    assert!(text.contains("--format"));
}
