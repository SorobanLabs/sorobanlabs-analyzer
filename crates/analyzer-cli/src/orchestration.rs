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
use analyzer_evidence::{Evidence, EvidenceCollector, EvidenceSource};
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
    let mut evidence = EvidenceCollector::new();

    synthesize_identity_findings(&current, &candidate, &mut findings, &mut evidence);
    synthesize_structural_findings(&current, &candidate, &mut findings, &mut evidence);
    synthesize_environment_findings(&current, &candidate, &mut findings, &mut evidence);
    synthesize_interface_findings(&current, &candidate, &mut findings, &mut evidence)?;
    synthesize_state_findings(
        &current,
        &candidate,
        request.migration_manifest,
        &mut findings,
        &mut evidence,
    );
    synthesize_authorization_findings(&current, &candidate, &mut findings, &mut evidence)?;
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
                let ev = record_evidence(
                    &mut evidence,
                    EvidenceSource::ExecutionTrace {
                        invocation: invocation.label.clone(),
                    },
                    "analyzer-rehearsal::host",
                    Some(invocation.label.clone()),
                    format!("candidate execution outcome: {:?}", candidate_obs.outcome),
                );
                findings.push(Finding::new(
                    FindingCategory::Rehearsal,
                    Rule::RehearsalFailed,
                    &invocation.label,
                    "candidate execution was blocked by the host",
                    "the rehearsal backend could not execute the candidate safely or meaningfully",
                    Severity::High,
                    Confidence::Detected,
                    vec![ev],
                ));
            } else if matches!(
                current_obs.outcome,
                analyzer_rehearsal::ExecutionOutcome::Blocked { .. }
            ) {
                ran = false;
                let ev = record_evidence(
                    &mut evidence,
                    EvidenceSource::ExecutionTrace {
                        invocation: invocation.label.clone(),
                    },
                    "analyzer-rehearsal::host",
                    Some(invocation.label.clone()),
                    format!("current execution outcome: {:?}", current_obs.outcome),
                );
                findings.push(Finding::new(
                    FindingCategory::Rehearsal,
                    Rule::RehearsalFailed,
                    &invocation.label,
                    "current execution was blocked by the host",
                    "the rehearsal backend could not execute the current executable safely or meaningfully",
                    Severity::High,
                    Confidence::Detected,
                    vec![ev],
                ));
            } else {
                let diff = analyzer_rehearsal::diff::diff_invocations(&current_obs, &candidate_obs);

                if let analyzer_rehearsal::Difference::Changed { old, new } = &diff.outcome {
                    let ev = record_evidence(
                        &mut evidence,
                        EvidenceSource::ExecutionTrace {
                            invocation: invocation.label.clone(),
                        },
                        "analyzer-rehearsal::diff",
                        Some(invocation.label.clone()),
                        format!("execution outcome changed: {old:?} -> {new:?}"),
                    );
                    findings.push(Finding::new(
                        FindingCategory::Rehearsal,
                        Rule::RehearsalResultChanged,
                        &invocation.label,
                        "invocation execution outcome changed",
                        "the candidate returned a different outcome type (e.g. Success vs Trap) than the current executable",
                        Severity::High,
                        Confidence::Detected,
                        vec![ev],
                    ));
                    diffs.push("outcome".to_string());
                }

                if let analyzer_rehearsal::Difference::Changed { old, new } = &diff.return_value {
                    let ev = record_evidence(
                        &mut evidence,
                        EvidenceSource::ExecutionTrace {
                            invocation: invocation.label.clone(),
                        },
                        "analyzer-rehearsal::diff",
                        Some(invocation.label.clone()),
                        format!("return value changed: {old:?} -> {new:?}"),
                    );
                    findings.push(Finding::new(
                        FindingCategory::Rehearsal,
                        Rule::RehearsalResultChanged,
                        &invocation.label,
                        "invocation return value changed",
                        "the candidate returned a different value than the current executable",
                        Severity::High,
                        Confidence::Detected,
                        vec![ev],
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
                "resource_usage".to_string(),
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
        &evidence.into_records(),
    ))
}

/// Construct an [`Evidence`] record, record it in `collector`, and
/// return its id, ready to attach to a [`Finding`]. A thin, explicit
/// wrapper so every call site at least states what it is claiming as
/// evidence, rather than a bare `vec![]`.
fn record_evidence(
    collector: &mut EvidenceCollector,
    source: EvidenceSource,
    producer: &str,
    location: Option<String>,
    observation: impl Into<String>,
) -> analyzer_evidence::EvidenceId {
    let evidence = Evidence::new(source, producer, location, observation);
    let id = evidence.id();
    collector.record(evidence);
    id
}

fn synthesize_identity_findings(
    current: &LoadedWasm,
    candidate: &LoadedWasm,
    findings: &mut Vec<Finding>,
    evidence: &mut EvidenceCollector,
) {
    if current.hash() != candidate.hash() {
        let ev = record_evidence(
            evidence,
            EvidenceSource::DerivedComparison {
                inputs: vec![current.hash().to_hex(), candidate.hash().to_hex()],
            },
            "analyzer-executable::artifact",
            None,
            format!(
                "current SHA-256 {} != candidate SHA-256 {}",
                current.hash(),
                candidate.hash()
            ),
        );
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
            vec![ev],
        ));
    }
}

fn synthesize_structural_findings(
    current: &LoadedWasm,
    candidate: &LoadedWasm,
    findings: &mut Vec<Finding>,
    evidence: &mut EvidenceCollector,
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
            let ev = record_evidence(
                evidence,
                EvidenceSource::LocalArtifact {
                    artifact_hash: loaded.hash().to_hex(),
                },
                "analyzer-executable::validation",
                None,
                format!("violations: {}", violations.join(", ")),
            );
            findings.push(Finding::new(
                FindingCategory::Executable,
                Rule::ExecutableStructurallyIncompatible,
                subject,
                "executable has a structural feature the Soroban host is known to reject",
                format!("violations: {}", violations.join(", ")),
                Severity::High,
                Confidence::Detected,
                vec![ev],
            ));
        }
    }
}

fn synthesize_environment_findings(
    current: &LoadedWasm,
    candidate: &LoadedWasm,
    findings: &mut Vec<Finding>,
    evidence: &mut EvidenceCollector,
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
        let ev = record_evidence(
            evidence,
            EvidenceSource::DerivedComparison {
                inputs: vec![current.hash().to_hex(), candidate.hash().to_hex()],
            },
            "analyzer-executable::environment_meta",
            Some("contractenvmetav0".to_string()),
            format!(
                "current present: {}, candidate present: {}",
                current_meta.is_present(),
                candidate_meta.is_present()
            ),
        );
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
            vec![ev],
        ));
        return;
    }

    if let (Some(current_version), Some(candidate_version)) = (
        current_meta.primary_interface_version(),
        candidate_meta.primary_interface_version(),
    ) {
        if current_version != candidate_version {
            let ev = record_evidence(
                evidence,
                EvidenceSource::DerivedComparison {
                    inputs: vec![current.hash().to_hex(), candidate.hash().to_hex()],
                },
                "analyzer-executable::environment_meta",
                Some("contractenvmetav0".to_string()),
                format!(
                    "current protocol {}.{} -> candidate protocol {}.{}",
                    current_version.protocol,
                    current_version.pre_release,
                    candidate_version.protocol,
                    candidate_version.pre_release
                ),
            );
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
                vec![ev],
            ));
        }
    }
}

fn synthesize_interface_findings(
    current: &LoadedWasm,
    candidate: &LoadedWasm,
    findings: &mut Vec<Finding>,
    evidence: &mut EvidenceCollector,
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

    // Every interface finding below is derived from the two
    // executables' contractspecv0 sections. Which side (or both) the
    // evidence names must match which side the underlying fact is
    // actually about: an "Added" fact only exists in the candidate's
    // section, a "Removed" fact only in the current section, and a
    // "Changed" fact requires comparing both.
    let spec_evidence = |evidence: &mut EvidenceCollector,
                         side: InterfaceEvidenceSide,
                         name: &str,
                         observation: String| {
        let source = match side {
            InterfaceEvidenceSide::Current => EvidenceSource::CustomWasmSection {
                artifact_hash: current.hash().to_hex(),
                section_name: "contractspecv0".to_string(),
            },
            InterfaceEvidenceSide::Candidate => EvidenceSource::CustomWasmSection {
                artifact_hash: candidate.hash().to_hex(),
                section_name: "contractspecv0".to_string(),
            },
            InterfaceEvidenceSide::Both => EvidenceSource::DerivedComparison {
                inputs: vec![current.hash().to_hex(), candidate.hash().to_hex()],
            },
        };
        record_evidence(
            evidence,
            source,
            "analyzer-executable::diff",
            Some(name.to_string()),
            observation,
        )
    };

    for change in &diff.function_changes {
        match change {
            FunctionChange::Added { name } => {
                let detail = format!("added function '{name}'");
                let ev = spec_evidence(
                    evidence,
                    InterfaceEvidenceSide::Candidate,
                    name,
                    detail.clone(),
                );
                findings.push(interface_finding(
                    Rule::ContractInterfaceAdded,
                    name,
                    "candidate adds a function the current executable does not have",
                    detail,
                    Severity::Info,
                    ev,
                ));
            }
            FunctionChange::Removed { name } => {
                let detail = format!("removed function '{name}'");
                let ev = spec_evidence(
                    evidence,
                    InterfaceEvidenceSide::Current,
                    name,
                    detail.clone(),
                );
                findings.push(interface_finding(
                    Rule::ContractInterfaceRemoved,
                    name,
                    "candidate removes a function the current executable has",
                    detail,
                    Severity::High,
                    ev,
                ));
            }
            FunctionChange::InputCountChanged {
                name,
                current_count,
                candidate_count,
            } => {
                let detail = format!("'{name}': {current_count} -> {candidate_count} inputs");
                let ev = spec_evidence(evidence, InterfaceEvidenceSide::Both, name, detail.clone());
                findings.push(interface_finding(
                    Rule::ContractSignatureChanged,
                    name,
                    "function input count changed",
                    detail,
                    Severity::High,
                    ev,
                ));
            }
            FunctionChange::InputOrderChanged { name, .. } => {
                let detail = format!("'{name}': input parameters reordered");
                let ev = spec_evidence(evidence, InterfaceEvidenceSide::Both, name, detail.clone());
                findings.push(interface_finding(
                    Rule::ContractSignatureChanged,
                    name,
                    "function input order changed",
                    detail,
                    Severity::Medium,
                    ev,
                ));
            }
            FunctionChange::InputChanged { name, index, .. } => {
                let detail = format!("'{name}': input at position {index} changed");
                let ev = spec_evidence(evidence, InterfaceEvidenceSide::Both, name, detail.clone());
                findings.push(interface_finding(
                    Rule::ContractSignatureChanged,
                    name,
                    "function input changed",
                    detail,
                    Severity::High,
                    ev,
                ));
            }
            FunctionChange::OutputChanged { name, .. } => {
                let detail = format!("'{name}': return type changed");
                let ev = spec_evidence(evidence, InterfaceEvidenceSide::Both, name, detail.clone());
                findings.push(interface_finding(
                    Rule::ContractSignatureChanged,
                    name,
                    "function output type changed",
                    detail,
                    Severity::High,
                    ev,
                ));
            }
        }
    }

    for change in &diff.event_changes {
        let (name, side, summary) = match change {
            EventChange::Added { name } => (
                name,
                InterfaceEvidenceSide::Candidate,
                "event added".to_string(),
            ),
            EventChange::Removed { name } => (
                name,
                InterfaceEvidenceSide::Current,
                "event removed".to_string(),
            ),
            EventChange::PrefixTopicsChanged { name, .. } => (
                name,
                InterfaceEvidenceSide::Both,
                "event prefix topics changed".to_string(),
            ),
            EventChange::ParametersChanged { name, .. } => (
                name,
                InterfaceEvidenceSide::Both,
                "event parameters changed".to_string(),
            ),
            EventChange::DataFormatChanged { name, .. } => (
                name,
                InterfaceEvidenceSide::Both,
                "event data format changed".to_string(),
            ),
        };
        let ev = spec_evidence(evidence, side, name, summary.clone());
        findings.push(interface_finding(
            Rule::ContractEventChanged,
            name,
            "contract event changed",
            summary,
            Severity::Medium,
            ev,
        ));
    }

    for change in &diff.struct_changes {
        let (name, side) = match change {
            StructChange::Added { name } => (name, InterfaceEvidenceSide::Candidate),
            StructChange::Removed { name } => (name, InterfaceEvidenceSide::Current),
            StructChange::FieldsChanged { name, .. } => (name, InterfaceEvidenceSide::Both),
        };
        let detail = format!("struct '{name}' changed");
        let ev = spec_evidence(evidence, side, name, detail.clone());
        findings.push(interface_finding(
            Rule::ContractTypeChanged,
            name,
            "user-defined struct type changed",
            detail,
            Severity::Medium,
            ev,
        ));
    }
    for change in &diff.union_changes {
        let (name, side) = match change {
            UnionChange::Added { name } => (name, InterfaceEvidenceSide::Candidate),
            UnionChange::Removed { name } => (name, InterfaceEvidenceSide::Current),
            UnionChange::CasesChanged { name, .. } => (name, InterfaceEvidenceSide::Both),
        };
        let detail = format!("union '{name}' changed");
        let ev = spec_evidence(evidence, side, name, detail.clone());
        findings.push(interface_finding(
            Rule::ContractTypeChanged,
            name,
            "user-defined union type changed",
            detail,
            Severity::Medium,
            ev,
        ));
    }
    for change in &diff.enum_changes {
        let (name, side) = match change {
            EnumChange::Added { name } => (name, InterfaceEvidenceSide::Candidate),
            EnumChange::Removed { name } => (name, InterfaceEvidenceSide::Current),
            EnumChange::CasesChanged { name, .. } => (name, InterfaceEvidenceSide::Both),
        };
        let detail = format!("enum '{name}' changed");
        let ev = spec_evidence(evidence, side, name, detail.clone());
        findings.push(interface_finding(
            Rule::ContractTypeChanged,
            name,
            "user-defined enum type changed",
            detail,
            Severity::Medium,
            ev,
        ));
    }
    for change in &diff.error_enum_changes {
        let (name, side) = match change {
            ErrorEnumChange::Added { name } => (name, InterfaceEvidenceSide::Candidate),
            ErrorEnumChange::Removed { name } => (name, InterfaceEvidenceSide::Current),
            ErrorEnumChange::CasesChanged { name, .. } => (name, InterfaceEvidenceSide::Both),
        };
        let detail = format!("error enum '{name}' changed");
        let ev = spec_evidence(evidence, side, name, detail.clone());
        findings.push(interface_finding(
            Rule::ContractTypeChanged,
            name,
            "user-defined error enum type changed",
            detail,
            Severity::Medium,
            ev,
        ));
    }

    Ok(())
}

/// Which side of an interface comparison an evidence record's source
/// should name, matching which side the underlying fact is actually
/// about (see `synthesize_interface_findings`).
#[derive(Clone, Copy)]
enum InterfaceEvidenceSide {
    Current,
    Candidate,
    Both,
}

fn interface_finding(
    rule: Rule,
    subject: &str,
    summary: &str,
    detail: String,
    severity: Severity,
    evidence: analyzer_evidence::EvidenceId,
) -> Finding {
    Finding::new(
        FindingCategory::Interface,
        rule,
        subject,
        summary,
        detail,
        severity,
        Confidence::Detected,
        vec![evidence],
    )
}

fn synthesize_state_findings(
    current: &LoadedWasm,
    candidate: &LoadedWasm,
    manifest: Option<&MigrationManifest>,
    findings: &mut Vec<Finding>,
    evidence: &mut EvidenceCollector,
) {
    let current_form = ExecutableForm::Wasm {
        hash: current.hash().to_hex(),
    };
    let candidate_form = ExecutableForm::Wasm {
        hash: candidate.hash().to_hex(),
    };

    let result = assess_state_compatibility(&current_form, &candidate_form, manifest);
    let reason_detail = result.reasons.join("; ");

    if matches!(result.outcome, StateCompatibility::Compatible) {
        return;
    }

    // When a manifest was supplied, it is the evidence this outcome is
    // actually derived from (see assess_state_compatibility's doc
    // comment: manifest-derived signals take priority). Otherwise the
    // outcome is derived purely from comparing the two executable
    // hashes.
    let state_evidence_source = match manifest.and_then(|m| m.content_hash().ok()) {
        Some(manifest_hash) => EvidenceSource::MigrationManifest { manifest_hash },
        None => EvidenceSource::DerivedComparison {
            inputs: vec![current.hash().to_hex(), candidate.hash().to_hex()],
        },
    };
    let ev = record_evidence(
        evidence,
        state_evidence_source,
        "analyzer-state::compatibility",
        None,
        reason_detail.clone(),
    );

    match result.outcome {
        StateCompatibility::Compatible => unreachable!("handled above"),
        StateCompatibility::RequiresMigration => findings.push(Finding::new(
            FindingCategory::State,
            Rule::MigrationRequired,
            "state",
            "state migration appears required",
            reason_detail,
            Severity::High,
            Confidence::Likely,
            vec![ev],
        )),
        StateCompatibility::PotentiallyIncompatible => findings.push(Finding::new(
            FindingCategory::State,
            Rule::StateCompatibilityUnknown,
            "state",
            "existing state may not be compatible with the candidate executable",
            reason_detail,
            Severity::Medium,
            Confidence::Potential,
            vec![ev],
        )),
        StateCompatibility::NotDetermined => findings.push(Finding::new(
            FindingCategory::State,
            Rule::StateCompatibilityUnknown,
            "state",
            "state compatibility could not be determined from available evidence",
            reason_detail,
            Severity::Info,
            Confidence::NotDeterminable,
            vec![ev],
        )),
    }
}

fn synthesize_authorization_findings(
    current: &LoadedWasm,
    candidate: &LoadedWasm,
    findings: &mut Vec<Finding>,
    evidence: &mut EvidenceCollector,
) -> Result<(), AnalyzerError> {
    let current_surface = extract_authorization_surface(current.bytes())?;
    let candidate_surface = extract_authorization_surface(candidate.bytes())?;
    let diff = diff_authorization_surfaces(&current_surface, &candidate_surface);

    // Captured before the match below, which shadows `current`/
    // `candidate` with per-change field bindings of the same name.
    let comparison_inputs = vec![current.hash().to_hex(), candidate.hash().to_hex()];

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
                let detail = format!(
                    "'{name}' previously directly called {previously_called:?}; the candidate's \
                     version of '{name}' calls none directly. This does not prove the entrypoint \
                     is unprotected: the check may have moved into a helper function this \
                     analyzer does not trace transitively."
                );
                let ev = record_evidence(
                    evidence,
                    EvidenceSource::DerivedComparison {
                        inputs: comparison_inputs.clone(),
                    },
                    "analyzer-auth::diff",
                    Some(name.clone()),
                    detail.clone(),
                );
                findings.push(Finding::new(
                    FindingCategory::Authorization,
                    Rule::AuthorizationRemoved,
                    name,
                    "entrypoint no longer directly calls an authorization primitive",
                    detail,
                    Severity::High,
                    Confidence::Likely,
                    vec![ev],
                ));
            }
            AuthorizationChange::EntrypointGainedAuthorizationCall { name, now_called } => {
                let detail = format!("'{name}' now directly calls {now_called:?}");
                let ev = record_evidence(
                    evidence,
                    EvidenceSource::DerivedComparison {
                        inputs: comparison_inputs.clone(),
                    },
                    "analyzer-auth::diff",
                    Some(name.clone()),
                    detail.clone(),
                );
                findings.push(Finding::new(
                    FindingCategory::Authorization,
                    Rule::AuthorizationSurfaceChanged,
                    name,
                    "entrypoint gained a direct authorization call",
                    detail,
                    Severity::Info,
                    Confidence::Detected,
                    vec![ev],
                ));
            }
            AuthorizationChange::AuthorizationPathChanged {
                name,
                current: current_calls,
                candidate: candidate_calls,
            } => {
                let detail = format!("'{name}': {current_calls:?} -> {candidate_calls:?}");
                let ev = record_evidence(
                    evidence,
                    EvidenceSource::DerivedComparison {
                        inputs: comparison_inputs.clone(),
                    },
                    "analyzer-auth::diff",
                    Some(name.clone()),
                    detail.clone(),
                );
                findings.push(Finding::new(
                    FindingCategory::Authorization,
                    Rule::AuthorizationSurfaceChanged,
                    name,
                    "entrypoint's directly-called authorization primitives changed",
                    detail,
                    Severity::Medium,
                    Confidence::Detected,
                    vec![ev],
                ));
            }
            AuthorizationChange::ModuleAuthPrimitiveImportChanged {
                current: current_imports,
                candidate: candidate_imports,
            } => {
                let detail =
                    format!("imports auth primitive: {current_imports} -> {candidate_imports}");
                let ev = record_evidence(
                    evidence,
                    EvidenceSource::DerivedComparison {
                        inputs: comparison_inputs.clone(),
                    },
                    "analyzer-auth::diff",
                    Some("module".to_string()),
                    detail.clone(),
                );
                findings.push(Finding::new(
                    FindingCategory::Authorization,
                    Rule::AuthorizationSurfaceChanged,
                    "module",
                    "whether the module imports any authorization primitive changed",
                    detail,
                    Severity::Medium,
                    Confidence::Detected,
                    vec![ev],
                ));
            }
            AuthorizationChange::CustomAuthHookChanged {
                current: current_hook,
                candidate: candidate_hook,
            } => {
                let detail = format!("exports __check_auth: {current_hook} -> {candidate_hook}");
                let ev = record_evidence(
                    evidence,
                    EvidenceSource::DerivedComparison {
                        inputs: comparison_inputs.clone(),
                    },
                    "analyzer-auth::diff",
                    Some("module".to_string()),
                    detail.clone(),
                );
                findings.push(Finding::new(
                    FindingCategory::Authorization,
                    Rule::AuthorizationSurfaceChanged,
                    "module",
                    "custom account __check_auth hook presence changed",
                    detail,
                    Severity::High,
                    Confidence::Detected,
                    vec![ev],
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

    fn second_module() -> Vec<u8> {
        let mut second = MINIMAL_VALID.to_vec();
        second.extend_from_slice(&[0x00, 0x07, 0x01, b'x', b'h', b'e', b'l', b'l', b'o']);
        second
    }

    #[test]
    fn executable_hash_changed_finding_references_real_evidence_in_the_report() {
        let current = write_wasm(MINIMAL_VALID);
        let candidate = write_wasm(&second_module());
        let request = AnalysisRequest {
            current_path: current.path(),
            candidate_path: candidate.path(),
            protocol_context: None,
            migration_manifest: None,
            analyzer_version: "0.1.0",
            rehearsal_input: None,
        };

        let report = run_upgrade_analysis(&request).unwrap();
        let finding = report
            .findings
            .iter()
            .find(|f| f.rule == "EXECUTABLE_HASH_CHANGED")
            .unwrap();

        assert!(
            !finding.evidence.is_empty(),
            "EXECUTABLE_HASH_CHANGED must reference real evidence, not an empty vec"
        );
        // Every id the finding references must actually resolve to a
        // record present in the report's own evidence list: this is
        // the finding -> evidence id -> evidence record chain.
        for evidence_id in &finding.evidence {
            assert!(
                report.evidence.iter().any(|e| &e.id == evidence_id),
                "finding references evidence id {evidence_id} that is not in report.evidence"
            );
        }
    }

    #[test]
    fn identical_analysis_inputs_produce_identical_evidence_ids() {
        let current = write_wasm(MINIMAL_VALID);
        let candidate = write_wasm(&second_module());
        let request = AnalysisRequest {
            current_path: current.path(),
            candidate_path: candidate.path(),
            protocol_context: None,
            migration_manifest: None,
            analyzer_version: "0.1.0",
            rehearsal_input: None,
        };

        let first = run_upgrade_analysis(&request).unwrap();
        let second = run_upgrade_analysis(&request).unwrap();

        let first_ids: Vec<&str> = first.evidence.iter().map(|e| e.id.as_str()).collect();
        let second_ids: Vec<&str> = second.evidence.iter().map(|e| e.id.as_str()).collect();
        assert_eq!(
            first_ids, second_ids,
            "identical inputs must produce identical evidence ids in identical order"
        );
    }

    #[test]
    fn changing_the_candidate_changes_the_hash_comparison_evidence_id() {
        let current = write_wasm(MINIMAL_VALID);
        let candidate_a = write_wasm(&second_module());
        // A distinct valid module: same shape as second_module(), but a
        // different custom section name, so its bytes (and hash) differ
        // from candidate_a without becoming truncated/invalid WASM.
        let mut third = MINIMAL_VALID.to_vec();
        third.extend_from_slice(&[0x00, 0x07, 0x01, b'y', b'h', b'e', b'l', b'l', b'o']);
        let candidate_b = write_wasm(&third);

        let request_a = AnalysisRequest {
            current_path: current.path(),
            candidate_path: candidate_a.path(),
            protocol_context: None,
            migration_manifest: None,
            analyzer_version: "0.1.0",
            rehearsal_input: None,
        };
        let request_b = AnalysisRequest {
            candidate_path: candidate_b.path(),
            ..request_a
        };

        let report_a = run_upgrade_analysis(&request_a).unwrap();
        let report_b = run_upgrade_analysis(&request_b).unwrap();

        let hash_evidence_id = |report: &AnalysisReport| {
            let finding = report
                .findings
                .iter()
                .find(|f| f.rule == "EXECUTABLE_HASH_CHANGED")
                .unwrap();
            finding.evidence[0].clone()
        };

        assert_ne!(
            hash_evidence_id(&report_a),
            hash_evidence_id(&report_b),
            "a different candidate must change the hash-comparison evidence id"
        );
    }

    #[test]
    fn every_evidence_record_is_referenced_by_at_least_one_finding() {
        // No fabricated placeholder evidence: this analyzer never
        // records an evidence entry that nothing actually cites.
        let current = write_wasm(MINIMAL_VALID);
        let candidate = write_wasm(&second_module());
        let request = AnalysisRequest {
            current_path: current.path(),
            candidate_path: candidate.path(),
            protocol_context: None,
            migration_manifest: None,
            analyzer_version: "0.1.0",
            rehearsal_input: None,
        };

        let report = run_upgrade_analysis(&request).unwrap();
        assert!(
            !report.evidence.is_empty(),
            "test setup should produce some evidence"
        );

        let referenced: std::collections::HashSet<&str> = report
            .findings
            .iter()
            .flat_map(|f| f.evidence.iter().map(String::as_str))
            .collect();
        for record in &report.evidence {
            assert!(
                referenced.contains(record.id.as_str()),
                "evidence record {} is not referenced by any finding",
                record.id
            );
        }
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

    #[test]
    fn migration_manifest_evidence_and_finding_preserve_unverified_framing() {
        use analyzer_state::MigrationFunction;

        let current = write_wasm(MINIMAL_VALID);
        let candidate = write_wasm(&second_module());
        let manifest = MigrationManifest {
            migration_function: Some(MigrationFunction {
                name: "migrate".to_string(),
                one_time: true,
            }),
            ..Default::default()
        };
        let request = AnalysisRequest {
            current_path: current.path(),
            candidate_path: candidate.path(),
            protocol_context: None,
            migration_manifest: Some(&manifest),
            analyzer_version: "0.1.0",
            rehearsal_input: None,
        };

        let report = run_upgrade_analysis(&request).unwrap();
        let finding = report
            .findings
            .iter()
            .find(|f| f.rule == "MIGRATION_REQUIRED")
            .expect("manifest declares a migration function");
        assert_eq!(finding.confidence, "LIKELY");
        assert!(
            finding.detail.starts_with("UNVERIFIED (author-supplied)"),
            "finding detail lost the unverified-author-supplied framing: {}",
            finding.detail
        );

        let evidence_id = finding.evidence.first().unwrap();
        let evidence = report
            .evidence
            .iter()
            .find(|e| &e.id == evidence_id)
            .expect("evidence resolves from the report");
        assert!(
            evidence
                .observation
                .starts_with("UNVERIFIED (author-supplied)"),
            "evidence observation lost the unverified-author-supplied framing: {}",
            evidence.observation
        );
        assert!(matches!(
            evidence.source,
            EvidenceSource::MigrationManifest { .. }
        ));
    }

    #[test]
    fn rehearsal_ran_report_lists_resource_usage_as_remaining_unverified() {
        let current = write_wasm(MINIMAL_VALID);
        let candidate = write_wasm(MINIMAL_VALID);
        let rehearsal_input = analyzer_rehearsal::RehearsalInput {
            current_executable: MINIMAL_VALID.to_vec(),
            candidate_executable: MINIMAL_VALID.to_vec(),
            state_snapshot: None,
            invocations: vec![],
            protocol_context: Some(28),
            execution_limits: analyzer_rehearsal::ExecutionLimits::default(),
            deterministic_seed: None,
        };
        let request = AnalysisRequest {
            current_path: current.path(),
            candidate_path: candidate.path(),
            protocol_context: None,
            migration_manifest: None,
            analyzer_version: "0.1.0",
            rehearsal_input: Some(&rehearsal_input),
        };

        let report = run_upgrade_analysis(&request).unwrap();
        let rehearsal = report.rehearsal.expect("rehearsal was requested");
        assert!(rehearsal.ran);
        assert!(rehearsal
            .observations_unavailable
            .contains(&"resource_usage".to_string()));
        assert!(
            rehearsal
                .remains_unverified
                .contains(&"resource_usage".to_string()),
            "remains_unverified should list resource_usage consistently with observations_unavailable: {:?}",
            rehearsal.remains_unverified
        );
        assert!(rehearsal.remains_unverified.contains(&"events".to_string()));
        assert!(rehearsal.remains_unverified.contains(&"state".to_string()));
        assert!(rehearsal
            .remains_unverified
            .contains(&"authorization".to_string()));
    }

    mod interface_evidence_provenance {
        use super::*;
        use stellar_xdr::{
            Limits, ScSpecEntry, ScSpecFunctionInputV0, ScSpecFunctionV0, ScSpecTypeDef, ScSymbol,
            StringM, VecM, WriteXdr,
        };

        fn write_leb128(out: &mut Vec<u8>, mut value: u64) {
            loop {
                let byte = (value & 0x7f) as u8;
                value >>= 7;
                if value == 0 {
                    out.push(byte);
                    break;
                }
                out.push(byte | 0x80);
            }
        }

        fn custom_section(name: &str, data: &[u8]) -> Vec<u8> {
            let mut name_bytes = Vec::new();
            write_leb128(&mut name_bytes, name.len() as u64);
            name_bytes.extend_from_slice(name.as_bytes());

            let mut content = name_bytes;
            content.extend_from_slice(data);

            let mut section = vec![0x00];
            write_leb128(&mut section, content.len() as u64);
            section.extend_from_slice(&content);
            section
        }

        fn symbol(s: &str) -> ScSymbol {
            ScSymbol(StringM::try_from(s).unwrap())
        }

        fn simple_function(name: &str) -> ScSpecEntry {
            ScSpecEntry::FunctionV0(ScSpecFunctionV0 {
                doc: StringM::default(),
                name: symbol(name),
                inputs: VecM::try_from(vec![ScSpecFunctionInputV0 {
                    doc: StringM::default(),
                    name: StringM::try_from("amount").unwrap(),
                    type_: ScSpecTypeDef::I128,
                }])
                .unwrap(),
                outputs: VecM::default(),
            })
        }

        fn function_with_extra_input(name: &str) -> ScSpecEntry {
            ScSpecEntry::FunctionV0(ScSpecFunctionV0 {
                doc: StringM::default(),
                name: symbol(name),
                inputs: VecM::try_from(vec![
                    ScSpecFunctionInputV0 {
                        doc: StringM::default(),
                        name: StringM::try_from("amount").unwrap(),
                        type_: ScSpecTypeDef::I128,
                    },
                    ScSpecFunctionInputV0 {
                        doc: StringM::default(),
                        name: StringM::try_from("memo").unwrap(),
                        type_: ScSpecTypeDef::I128,
                    },
                ])
                .unwrap(),
                outputs: VecM::default(),
            })
        }

        fn module_with_spec(entries: &[ScSpecEntry]) -> Vec<u8> {
            let mut data = Vec::new();
            for entry in entries {
                data.extend_from_slice(&entry.to_xdr(Limits::none()).unwrap());
            }
            let mut module = MINIMAL_VALID.to_vec();
            module.extend_from_slice(&custom_section("contractspecv0", &data));
            module
        }

        fn analyze(current_bytes: &[u8], candidate_bytes: &[u8]) -> AnalysisReport {
            let current = write_wasm(current_bytes);
            let candidate = write_wasm(candidate_bytes);
            let request = AnalysisRequest {
                current_path: current.path(),
                candidate_path: candidate.path(),
                protocol_context: None,
                migration_manifest: None,
                analyzer_version: "0.1.0",
                rehearsal_input: None,
            };
            run_upgrade_analysis(&request).unwrap()
        }

        fn evidence_for(report: &AnalysisReport, rule: &str) -> analyzer_evidence::EvidenceSource {
            let finding = report
                .findings
                .iter()
                .find(|f| f.rule == rule)
                .unwrap_or_else(|| panic!("no {rule} finding in report"));
            let evidence_id = finding.evidence.first().expect("finding has evidence");
            report
                .evidence
                .iter()
                .find(|e| &e.id == evidence_id)
                .expect("evidence resolves from the report")
                .source
                .clone()
        }

        #[test]
        fn added_function_evidence_names_the_candidate_side() {
            let current = module_with_spec(&[]);
            let candidate = module_with_spec(&[simple_function("transfer")]);

            let report = analyze(&current, &candidate);
            let candidate_hash = report.candidate_executable.hash.clone();
            match evidence_for(&report, "CONTRACT_INTERFACE_ADDED") {
                EvidenceSource::CustomWasmSection { artifact_hash, .. } => {
                    assert_eq!(artifact_hash, candidate_hash);
                }
                other => panic!("expected CustomWasmSection, got {other:?}"),
            }
        }

        #[test]
        fn removed_function_evidence_names_the_current_side() {
            let current = module_with_spec(&[simple_function("transfer")]);
            let candidate = module_with_spec(&[]);

            let report = analyze(&current, &candidate);
            let current_hash = report.current_executable.hash.clone();
            match evidence_for(&report, "CONTRACT_INTERFACE_REMOVED") {
                EvidenceSource::CustomWasmSection { artifact_hash, .. } => {
                    assert_eq!(artifact_hash, current_hash);
                }
                other => panic!("expected CustomWasmSection, got {other:?}"),
            }
        }

        #[test]
        fn changed_signature_evidence_references_both_sides() {
            let current = module_with_spec(&[simple_function("transfer")]);
            let candidate = module_with_spec(&[function_with_extra_input("transfer")]);

            let report = analyze(&current, &candidate);
            let current_hash = report.current_executable.hash.clone();
            let candidate_hash = report.candidate_executable.hash.clone();
            match evidence_for(&report, "CONTRACT_SIGNATURE_CHANGED") {
                EvidenceSource::DerivedComparison { inputs } => {
                    assert_eq!(inputs, vec![current_hash, candidate_hash]);
                }
                other => panic!("expected DerivedComparison, got {other:?}"),
            }
        }

        #[test]
        fn interface_evidence_ids_are_deterministic() {
            let current = module_with_spec(&[simple_function("transfer")]);
            let candidate = module_with_spec(&[]);

            let report_a = analyze(&current, &candidate);
            let report_b = analyze(&current, &candidate);
            assert_eq!(
                evidence_for(&report_a, "CONTRACT_INTERFACE_REMOVED"),
                evidence_for(&report_b, "CONTRACT_INTERFACE_REMOVED")
            );
        }
    }
}
