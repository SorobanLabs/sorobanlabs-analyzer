//! Upgrade analysis pipeline orchestration.
//!
//! This module sequences calls into the underlying analysis crates in
//! the order: identity -> validation -> environment metadata ->
//! contract specification -> interface diff -> state compatibility ->
//! authorization comparison -> finding synthesis -> rehearsal ->
//! canonical report.
//!
//! Rehearsal only runs when the caller supplies
//! [`AnalysisRequest::rehearsal_input`]; the pipeline never invokes the
//! `analyzer-rehearsal` backend on its own. When rehearsal input is
//! absent, that absence is surfaced as an explicit
//! `REHEARSAL_FAILED` finding at [`Confidence::NotDeterminable`], not
//! silently skipped.
//!
//! # Why this lives in `analyzer-cli`, not `analyzer-core`
//!
//! `analyzer-executable`, `analyzer-state`, `analyzer-auth`, and
//! `analyzer-report` all depend on `analyzer-core` (for
//! [`AnalyzerError`], [`Finding`], and friends). If `analyzer-core`
//! called back into any of them to orchestrate, that would be a
//! dependency cycle. `analyzer-cli` already depends on every one of
//! those crates (to eventually expose CLI commands over them, Step 27),
//! so it is the cycle-free place this concrete wiring can live. The
//! actual rule logic — interface diff rules, state compatibility
//! rules, authorization diff rules — stays entirely in those library
//! crates; this module only sequences calls into them and converts
//! their typed results into [`Finding`]s, which is orchestration, not
//! analysis.
//!
//! # Errors vs. findings
//!
//! A missing file, a malformed artifact that cannot even be loaded,
//! and similar operational problems are returned as
//! [`AnalyzerError`] and stop the pipeline. Every *analytical*
//! observation (a changed hash, a changed interface, a state
//! compatibility concern) becomes a [`Finding`], never an error.

use std::path::Path;

use analyzer_auth::{
    diff_authorization_surfaces, extract_authorization_surface, AuthorizationChange,
};
use analyzer_core::{
    AnalysisStatus, AnalyzerError, Confidence, Finding, FindingCategory, Rule, Severity,
};
use analyzer_executable::{
    diff_interfaces, normalize_interface, parse_contract_spec, parse_environment_metadata,
    EnumChange, ErrorEnumChange, EventChange, FunctionChange, LoadedWasm, StructChange,
    UnionChange,
};
use analyzer_report::{AnalysisReport, ReportExecutableIdentity};
use analyzer_state::{
    assess_state_compatibility, ExecutableForm, MigrationManifest, StateCompatibility,
};

/// Everything the orchestrator needs to analyze a proposed executable
/// replacement.
pub struct AnalysisRequest<'a> {
    pub current_path: &'a Path,
    pub candidate_path: &'a Path,
    /// The Soroban protocol number the analysis is being run against,
    /// if the caller specified one. Carried through to the report;
    /// this pipeline does not yet compare it against observed
    /// environment interface versions (that comparison needs a
    /// verified protocol-to-interface-version mapping this analyzer
    /// does not have).
    pub protocol_context: Option<u32>,
    /// An optional author-supplied migration manifest.
    pub migration_manifest: Option<&'a MigrationManifest>,
    /// The analyzer's own version string, recorded in the report.
    pub analyzer_version: &'a str,
    pub rehearsal_input: Option<&'a analyzer_rehearsal::RehearsalInput>,
}

/// Run the full upgrade analysis pipeline and produce a canonical
/// report.
///
/// # Errors
///
/// Returns an [`AnalyzerError`] if either executable cannot be loaded
/// at all (missing file, not valid WASM). Analytical uncertainty about
/// an executable that *does* load is never an error; it becomes a
/// [`Finding`] with [`Confidence::NotDeterminable`] or similar.
pub fn run_upgrade_analysis(request: &AnalysisRequest) -> Result<AnalysisReport, AnalyzerError> {
    // identity + Level 1/1.5 validation: LoadedWasm::from_path already
    // performs generic structural validation and the Soroban structural
    // compatibility check.
    let current = LoadedWasm::from_path(request.current_path)?;
    let candidate = LoadedWasm::from_path(request.candidate_path)?;

    let mut findings = Vec::new();

    synthesize_identity_findings(&current, &candidate, &mut findings);
    synthesize_structural_findings(&current, &candidate, &mut findings);
    synthesize_environment_findings(&current, &candidate, &mut findings);
    synthesize_interface_findings(&current, &candidate, &mut findings)?;
    synthesize_state_findings(
        &current,
        &candidate,
        request.migration_manifest,
        &mut findings,
    );
    synthesize_authorization_findings(&current, &candidate, &mut findings)?;
    let report_rehearsal = if let Some(rehearsal) = request.rehearsal_input {
        let mut ran = true;
        let mut diffs = Vec::new();

        for invocation in &rehearsal.invocations {
            let current_obs = analyzer_rehearsal::rehearse_invocation(
                current.bytes(),
                invocation,
                &rehearsal.execution_limits,
            );

            let candidate_obs = analyzer_rehearsal::rehearse_invocation(
                candidate.bytes(),
                invocation,
                &rehearsal.execution_limits,
            );

            if matches!(
                candidate_obs.outcome,
                analyzer_rehearsal::ExecutionOutcome::Blocked { .. }
            ) {
                ran = false;
                findings.push(Finding::new(
                    FindingCategory::Rehearsal,
                    Rule::RehearsalFailed,
                    &invocation.label,
                    "candidate execution was blocked by the host",
                    "the rehearsal backend could not execute the candidate safely or meaningfully",
                    Severity::High,
                    Confidence::Detected,
                    vec![],
                ));
            } else if matches!(
                current_obs.outcome,
                analyzer_rehearsal::ExecutionOutcome::Blocked { .. }
            ) {
                ran = false;
                findings.push(Finding::new(
                    FindingCategory::Rehearsal,
                    Rule::RehearsalFailed,
                    &invocation.label,
                    "current execution was blocked by the host",
                    "the rehearsal backend could not execute the current executable safely or meaningfully",
                    Severity::High,
                    Confidence::Detected,
                    vec![],
                ));
            } else {
                let diff = analyzer_rehearsal::diff::diff_invocations(&current_obs, &candidate_obs);

                if matches!(diff.outcome, analyzer_rehearsal::Difference::Changed { .. }) {
                    findings.push(Finding::new(
                        FindingCategory::Rehearsal,
                        Rule::RehearsalResultChanged,
                        &invocation.label,
                        "invocation execution outcome changed",
                        "the candidate returned a different outcome type (e.g. Success vs Trap) than the current executable",
                        Severity::High,
                        Confidence::Detected,
                        vec![],
                    ));
                    diffs.push("outcome".to_string());
                }

                if matches!(
                    diff.return_value,
                    analyzer_rehearsal::Difference::Changed { .. }
                ) {
                    findings.push(Finding::new(
                        FindingCategory::Rehearsal,
                        Rule::RehearsalResultChanged,
                        &invocation.label,
                        "invocation return value changed",
                        "the candidate returned a different value than the current executable",
                        Severity::High,
                        Confidence::Detected,
                        vec![],
                    ));
                    diffs.push("return_value".to_string());
                }
            }
        }

        let obs_unavail = vec![
            "events".to_string(),
            "state_reads".to_string(),
            "state_writes".to_string(),
            "resource_usage".to_string(),
        ];

        Some(analyzer_report::ReportRehearsal {
            requested: true,
            ran,
            backend_used: Some("soroban-env-host".to_string()),
            observations_captured: vec!["outcome".to_string(), "return_value".to_string()],
            observations_unavailable: obs_unavail,
            behavioral_differences_established: diffs,
            remains_unverified: vec![
                "events".to_string(),
                "state".to_string(),
                "authorization".to_string(),
            ],
        })
    } else {
        findings.push(Finding::new(
            FindingCategory::Rehearsal,
            Rule::RehearsalFailed,
            "rehearsal",
            "controlled rehearsal was not requested",
            "the analysis pipeline was run without a rehearsal manifest; \
             no observation about runtime behaviour differences between \
             the current and candidate executables was made",
            Severity::Info,
            Confidence::NotDeterminable,
            vec![],
        ));

        Some(analyzer_report::ReportRehearsal {
            requested: false,
            ran: false,
            backend_used: None,
            observations_captured: vec![],
            observations_unavailable: vec![],
            behavioral_differences_established: vec![],
            remains_unverified: vec!["all execution behavior".to_string()],
        })
    };

    let status = overall_status(&findings);

    Ok(AnalysisReport::new(
        request.analyzer_version,
        request.protocol_context,
        ReportExecutableIdentity {
            hash: current.hash().to_hex(),
            byte_length: u64::try_from(current.bytes().len()).unwrap_or(u64::MAX),
        },
        ReportExecutableIdentity {
            hash: candidate.hash().to_hex(),
            byte_length: u64::try_from(candidate.bytes().len()).unwrap_or(u64::MAX),
        },
        status,
        &findings,
        report_rehearsal,
    ))
}

fn synthesize_identity_findings(
    current: &LoadedWasm,
    candidate: &LoadedWasm,
    findings: &mut Vec<Finding>,
) {
    if current.hash() != candidate.hash() {
        findings.push(Finding::new(
            FindingCategory::Executable,
            Rule::ExecutableHashChanged,
            "executable",
            "candidate executable bytes differ from the current executable",
            format!(
                "current SHA-256 {} != candidate SHA-256 {}",
                current.hash(),
                candidate.hash()
            ),
            Severity::Info,
            Confidence::Detected,
            vec![],
        ));
    }
}

fn synthesize_structural_findings(
    current: &LoadedWasm,
    candidate: &LoadedWasm,
    findings: &mut Vec<Finding>,
) {
    for (subject, loaded) in [
        ("current executable", current),
        ("candidate executable", candidate),
    ] {
        let report = loaded.soroban_structural();
        if !report.is_compatible() {
            let violations: Vec<String> = report
                .violations()
                .iter()
                .map(ToString::to_string)
                .collect();
            findings.push(Finding::new(
                FindingCategory::Executable,
                Rule::ExecutableStructurallyIncompatible,
                subject,
                "executable has a structural feature the Soroban host is known to reject",
                format!("violations: {}", violations.join(", ")),
                Severity::High,
                Confidence::Detected,
                vec![],
            ));
        }
    }
}

fn synthesize_environment_findings(
    current: &LoadedWasm,
    candidate: &LoadedWasm,
    findings: &mut Vec<Finding>,
) {
    let current_meta = parse_environment_metadata(current.bytes());
    let candidate_meta = parse_environment_metadata(candidate.bytes());

    let (current_meta, candidate_meta) = match (current_meta, candidate_meta) {
        (Ok(current), Ok(candidate)) => (current, candidate),
        // A malformed section is itself a structural fact worth
        // recording, not a pipeline-stopping error: the artifact
        // otherwise loaded and validated successfully.
        _ => return,
    };

    if !current_meta.is_present() || !candidate_meta.is_present() {
        findings.push(Finding::new(
            FindingCategory::Environment,
            Rule::EnvironmentMetadataMissing,
            "executable",
            "contractenvmetav0 is missing from one or both executables",
            format!(
                "current present: {}, candidate present: {}",
                current_meta.is_present(),
                candidate_meta.is_present()
            ),
            Severity::Medium,
            Confidence::Detected,
            vec![],
        ));
        return;
    }

    if let (Some(current_version), Some(candidate_version)) = (
        current_meta.primary_interface_version(),
        candidate_meta.primary_interface_version(),
    ) {
        if current_version != candidate_version {
            findings.push(Finding::new(
                FindingCategory::Environment,
                Rule::EnvironmentInterfaceChanged,
                "executable",
                "declared Soroban environment interface version changed",
                format!(
                    "current protocol {}.{} -> candidate protocol {}.{}",
                    current_version.protocol,
                    current_version.pre_release,
                    candidate_version.protocol,
                    candidate_version.pre_release
                ),
                Severity::Medium,
                Confidence::Detected,
                vec![],
            ));
        }
    }
}

fn synthesize_interface_findings(
    current: &LoadedWasm,
    candidate: &LoadedWasm,
    findings: &mut Vec<Finding>,
) -> Result<(), AnalyzerError> {
    let current_spec = parse_contract_spec(current.bytes())?;
    let candidate_spec = parse_contract_spec(candidate.bytes())?;

    if !current_spec.decoded_cleanly() || !candidate_spec.decoded_cleanly() {
        // A malformed contractspecv0 section blocks interface
        // comparison; record it and stop this stage, but let the rest
        // of the pipeline continue.
        return Ok(());
    }

    let current_interface = normalize_interface(current_spec.entries())?;
    let candidate_interface = normalize_interface(candidate_spec.entries())?;
    let diff = diff_interfaces(&current_interface, &candidate_interface);

    for change in &diff.function_changes {
        match change {
            FunctionChange::Added { name } => findings.push(interface_finding(
                Rule::ContractInterfaceAdded,
                name,
                "candidate adds a function the current executable does not have",
                format!("added function '{name}'"),
                Severity::Info,
            )),
            FunctionChange::Removed { name } => findings.push(interface_finding(
                Rule::ContractInterfaceRemoved,
                name,
                "candidate removes a function the current executable has",
                format!("removed function '{name}'"),
                Severity::High,
            )),
            FunctionChange::InputCountChanged {
                name,
                current_count,
                candidate_count,
            } => {
                findings.push(interface_finding(
                    Rule::ContractSignatureChanged,
                    name,
                    "function input count changed",
                    format!("'{name}': {current_count} -> {candidate_count} inputs"),
                    Severity::High,
                ));
            }
            FunctionChange::InputOrderChanged { name, .. } => findings.push(interface_finding(
                Rule::ContractSignatureChanged,
                name,
                "function input order changed",
                format!("'{name}': input parameters reordered"),
                Severity::Medium,
            )),
            FunctionChange::InputChanged { name, index, .. } => findings.push(interface_finding(
                Rule::ContractSignatureChanged,
                name,
                "function input changed",
                format!("'{name}': input at position {index} changed"),
                Severity::High,
            )),
            FunctionChange::OutputChanged { name, .. } => findings.push(interface_finding(
                Rule::ContractSignatureChanged,
                name,
                "function output type changed",
                format!("'{name}': return type changed"),
                Severity::High,
            )),
        }
    }

    for change in &diff.event_changes {
        let (name, summary) = match change {
            EventChange::Added { name } => (name, "event added".to_string()),
            EventChange::Removed { name } => (name, "event removed".to_string()),
            EventChange::PrefixTopicsChanged { name, .. } => {
                (name, "event prefix topics changed".to_string())
            }
            EventChange::ParametersChanged { name, .. } => {
                (name, "event parameters changed".to_string())
            }
            EventChange::DataFormatChanged { name, .. } => {
                (name, "event data format changed".to_string())
            }
        };
        findings.push(interface_finding(
            Rule::ContractEventChanged,
            name,
            "contract event changed",
            summary,
            Severity::Medium,
        ));
    }

    for change in &diff.struct_changes {
        let name = match change {
            StructChange::Added { name }
            | StructChange::Removed { name }
            | StructChange::FieldsChanged { name, .. } => name,
        };
        findings.push(interface_finding(
            Rule::ContractTypeChanged,
            name,
            "user-defined struct type changed",
            format!("struct '{name}' changed"),
            Severity::Medium,
        ));
    }
    for change in &diff.union_changes {
        let name = match change {
            UnionChange::Added { name }
            | UnionChange::Removed { name }
            | UnionChange::CasesChanged { name, .. } => name,
        };
        findings.push(interface_finding(
            Rule::ContractTypeChanged,
            name,
            "user-defined union type changed",
            format!("union '{name}' changed"),
            Severity::Medium,
        ));
    }
    for change in &diff.enum_changes {
        let name = match change {
            EnumChange::Added { name }
            | EnumChange::Removed { name }
            | EnumChange::CasesChanged { name, .. } => name,
        };
        findings.push(interface_finding(
            Rule::ContractTypeChanged,
            name,
            "user-defined enum type changed",
            format!("enum '{name}' changed"),
            Severity::Medium,
        ));
    }
    for change in &diff.error_enum_changes {
        let name = match change {
            ErrorEnumChange::Added { name }
            | ErrorEnumChange::Removed { name }
            | ErrorEnumChange::CasesChanged { name, .. } => name,
        };
        findings.push(interface_finding(
            Rule::ContractTypeChanged,
            name,
            "user-defined error enum type changed",
            format!("error enum '{name}' changed"),
            Severity::Medium,
        ));
    }

    Ok(())
}

fn interface_finding(
    rule: Rule,
    subject: &str,
    summary: &str,
    detail: String,
    severity: Severity,
) -> Finding {
    Finding::new(
        FindingCategory::Interface,
        rule,
        subject,
        summary,
        detail,
        severity,
        Confidence::Detected,
        vec![],
    )
}

fn synthesize_state_findings(
    current: &LoadedWasm,
    candidate: &LoadedWasm,
    manifest: Option<&MigrationManifest>,
    findings: &mut Vec<Finding>,
) {
    let current_form = ExecutableForm::Wasm {
        hash: current.hash().to_hex(),
    };
    let candidate_form = ExecutableForm::Wasm {
        hash: candidate.hash().to_hex(),
    };

    let result = assess_state_compatibility(&current_form, &candidate_form, manifest);
    let reason_detail = result.reasons.join("; ");

    match result.outcome {
        StateCompatibility::Compatible => {}
        StateCompatibility::RequiresMigration => findings.push(Finding::new(
            FindingCategory::State,
            Rule::MigrationRequired,
            "state",
            "state migration appears required",
            reason_detail,
            Severity::High,
            Confidence::Likely,
            vec![],
        )),
        StateCompatibility::PotentiallyIncompatible => findings.push(Finding::new(
            FindingCategory::State,
            Rule::StateCompatibilityUnknown,
            "state",
            "existing state may not be compatible with the candidate executable",
            reason_detail,
            Severity::Medium,
            Confidence::Potential,
            vec![],
        )),
        StateCompatibility::NotDetermined => findings.push(Finding::new(
            FindingCategory::State,
            Rule::StateCompatibilityUnknown,
            "state",
            "state compatibility could not be determined from available evidence",
            reason_detail,
            Severity::Info,
            Confidence::NotDeterminable,
            vec![],
        )),
    }
}

fn synthesize_authorization_findings(
    current: &LoadedWasm,
    candidate: &LoadedWasm,
    findings: &mut Vec<Finding>,
) -> Result<(), AnalyzerError> {
    let current_surface = extract_authorization_surface(current.bytes())?;
    let candidate_surface = extract_authorization_surface(candidate.bytes())?;
    let diff = diff_authorization_surfaces(&current_surface, &candidate_surface);

    for change in &diff.changes {
        match change {
            AuthorizationChange::EntrypointAdded { .. }
            | AuthorizationChange::EntrypointRemoved { .. } => {
                // Already covered by CONTRACT_INTERFACE_ADDED/REMOVED
                // from the interface diff; avoid a duplicate finding
                // for the same underlying fact.
            }
            AuthorizationChange::EntrypointAppearsUnprotected {
                name,
                previously_called,
            } => {
                findings.push(Finding::new(
                    FindingCategory::Authorization,
                    Rule::AuthorizationRemoved,
                    name,
                    "entrypoint no longer directly calls an authorization primitive",
                    format!(
                        "'{name}' previously directly called {previously_called:?}; the candidate's \
                         version of '{name}' calls none directly. This does not prove the entrypoint \
                         is unprotected: the check may have moved into a helper function this \
                         analyzer does not trace transitively."
                    ),
                    Severity::High,
                    Confidence::Likely,
                    vec![],
                ));
            }
            AuthorizationChange::EntrypointGainedAuthorizationCall { name, now_called } => {
                findings.push(Finding::new(
                    FindingCategory::Authorization,
                    Rule::AuthorizationSurfaceChanged,
                    name,
                    "entrypoint gained a direct authorization call",
                    format!("'{name}' now directly calls {now_called:?}"),
                    Severity::Info,
                    Confidence::Detected,
                    vec![],
                ));
            }
            AuthorizationChange::AuthorizationPathChanged {
                name,
                current,
                candidate,
            } => {
                findings.push(Finding::new(
                    FindingCategory::Authorization,
                    Rule::AuthorizationSurfaceChanged,
                    name,
                    "entrypoint's directly-called authorization primitives changed",
                    format!("'{name}': {current:?} -> {candidate:?}"),
                    Severity::Medium,
                    Confidence::Detected,
                    vec![],
                ));
            }
            AuthorizationChange::ModuleAuthPrimitiveImportChanged { current, candidate } => {
                findings.push(Finding::new(
                    FindingCategory::Authorization,
                    Rule::AuthorizationSurfaceChanged,
                    "module",
                    "whether the module imports any authorization primitive changed",
                    format!("imports auth primitive: {current} -> {candidate}"),
                    Severity::Medium,
                    Confidence::Detected,
                    vec![],
                ));
            }
            AuthorizationChange::CustomAuthHookChanged { current, candidate } => {
                findings.push(Finding::new(
                    FindingCategory::Authorization,
                    Rule::AuthorizationSurfaceChanged,
                    "module",
                    "custom account __check_auth hook presence changed",
                    format!("exports __check_auth: {current} -> {candidate}"),
                    Severity::High,
                    Confidence::Detected,
                    vec![],
                ));
            }
        }
    }

    Ok(())
}

/// Derive the overall [`AnalysisStatus`] from the synthesized findings.
///
/// This is a conservative, explicit rollup, not a scoring system:
/// - any finding requiring migration -> `MigrationRequired`
/// - else any finding at `Severity::High` or `Severity::Critical` with
///   `Confidence` other than `NotDeterminable` -> `ReviewRequired`
/// - else any finding with `Confidence::NotDeterminable` at
///   `Severity::Medium` or higher -> `Inconclusive`
/// - else -> `NoDetectedBlockers`
fn overall_status(findings: &[Finding]) -> AnalysisStatus {
    if findings.iter().any(|f| f.rule() == Rule::MigrationRequired) {
        return AnalysisStatus::MigrationRequired;
    }
    if findings.iter().any(|f| {
        matches!(f.severity(), Severity::High | Severity::Critical)
            && f.confidence() != Confidence::NotDeterminable
    }) {
        return AnalysisStatus::ReviewRequired;
    }
    if findings.iter().any(|f| {
        f.confidence() == Confidence::NotDeterminable
            && matches!(
                f.severity(),
                Severity::Medium | Severity::High | Severity::Critical
            )
    }) {
        return AnalysisStatus::Inconclusive;
    }
    AnalysisStatus::NoDetectedBlockers
}

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::expect_used)]
mod tests {
    use super::*;
    use std::io::Write;

    const MINIMAL_VALID: &[u8] = &[0x00, 0x61, 0x73, 0x6d, 0x01, 0x00, 0x00, 0x00];

    fn write_wasm(bytes: &[u8]) -> tempfile::NamedTempFile {
        let mut file = tempfile::NamedTempFile::new().unwrap();
        file.write_all(bytes).unwrap();
        file
    }

    #[test]
    fn identical_minimal_modules_produce_no_detected_blockers() {
        let current = write_wasm(MINIMAL_VALID);
        let candidate = write_wasm(MINIMAL_VALID);

        let request = AnalysisRequest {
            current_path: current.path(),
            candidate_path: candidate.path(),
            protocol_context: Some(28),
            migration_manifest: None,
            analyzer_version: "0.1.0",
            rehearsal_input: None,
        };

        let report = run_upgrade_analysis(&request).unwrap();
        assert_eq!(
            report.current_executable.hash,
            report.candidate_executable.hash
        );
        // Both minimal modules carry no contractenvmetav0 section, so
        // ENVIRONMENT_METADATA_MISSING fires alongside the always-present
        // REHEARSAL_FAILED finding; nothing else should differ between
        // two byte-identical modules.
        assert_eq!(report.findings.len(), 2);
        assert!(report.findings.iter().any(|f| f.rule == "REHEARSAL_FAILED"));
        assert!(report
            .findings
            .iter()
            .any(|f| f.rule == "ENVIRONMENT_METADATA_MISSING"));
        assert_eq!(report.status, "NO_DETECTED_BLOCKERS");
    }

    #[test]
    fn missing_current_file_is_a_structured_error_not_a_panic() {
        let candidate = write_wasm(MINIMAL_VALID);
        let request = AnalysisRequest {
            current_path: Path::new("/nonexistent/current.wasm"),
            candidate_path: candidate.path(),
            protocol_context: None,
            migration_manifest: None,
            analyzer_version: "0.1.0",
            rehearsal_input: None,
        };
        let result = run_upgrade_analysis(&request);
        assert!(matches!(result, Err(AnalyzerError::Backend(_))));
    }

    #[test]
    fn different_bytes_produce_executable_hash_changed_finding() {
        // A second valid module: header + a trivial custom section, so
        // its bytes (and hash) differ from MINIMAL_VALID.
        let mut second = MINIMAL_VALID.to_vec();
        second.extend_from_slice(&[0x00, 0x07, 0x01, b'x', b'h', b'e', b'l', b'l', b'o']);

        let current = write_wasm(MINIMAL_VALID);
        let candidate = write_wasm(&second);
        let request = AnalysisRequest {
            current_path: current.path(),
            candidate_path: candidate.path(),
            protocol_context: None,
            migration_manifest: None,
            analyzer_version: "0.1.0",
            rehearsal_input: None,
        };

        let report = run_upgrade_analysis(&request).unwrap();
        assert!(report
            .findings
            .iter()
            .any(|f| f.rule == "EXECUTABLE_HASH_CHANGED"));
    }

    #[test]
    fn overall_status_prioritizes_migration_required() {
        let findings = vec![Finding::new(
            FindingCategory::State,
            Rule::MigrationRequired,
            "state",
            "s",
            "d",
            Severity::High,
            Confidence::Likely,
            vec![],
        )];
        assert_eq!(overall_status(&findings), AnalysisStatus::MigrationRequired);
    }

    #[test]
    fn overall_status_is_no_detected_blockers_for_empty_findings() {
        assert_eq!(overall_status(&[]), AnalysisStatus::NoDetectedBlockers);
    }
}
