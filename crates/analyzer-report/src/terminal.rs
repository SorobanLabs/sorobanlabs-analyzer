//! Human-readable terminal rendering of [`AnalysisReport`].
//!
//! This is a plain-text rendering of exactly the data already present in
//! the canonical report; it introduces no additional analysis, no
//! severity/confidence rounding, and no data not already in
//! [`AnalysisReport`]. Output is deterministic (fixed field order,
//! findings rendered in the order they appear in the report) and uses
//! no terminal color codes, so it is equally readable piped to a file or
//! a non-color terminal.

use std::fmt::Write as _;

use crate::canonical::{AnalysisReport, ReportFinding, ReportRehearsal};

/// Render `report` as a human-readable terminal report.
///
/// The output always ends with a single trailing newline.
pub fn render_terminal(report: &AnalysisReport) -> String {
    let mut out = String::new();

    let _ = writeln!(out, "SorobanLabs Analyzer upgrade analysis report");
    let _ = writeln!(out, "schema version:   {}", report.schema_version);
    let _ = writeln!(out, "analyzer version: {}", report.analyzer_version);
    let _ = writeln!(
        out,
        "protocol context: {}",
        report
            .protocol_context
            .map_or_else(|| "not specified".to_string(), |p| p.to_string())
    );
    let _ = writeln!(out);

    let _ = writeln!(out, "current executable:");
    let _ = writeln!(out, "  hash:        {}", report.current_executable.hash);
    let _ = writeln!(
        out,
        "  byte length: {}",
        report.current_executable.byte_length
    );
    let _ = writeln!(out);

    let _ = writeln!(out, "candidate executable:");
    let _ = writeln!(out, "  hash:        {}", report.candidate_executable.hash);
    let _ = writeln!(
        out,
        "  byte length: {}",
        report.candidate_executable.byte_length
    );
    let _ = writeln!(out);

    let _ = writeln!(out, "status: {}", report.status);
    let _ = writeln!(out);

    if let Some(rehearsal) = &report.rehearsal {
        render_rehearsal(&mut out, rehearsal);
        let _ = writeln!(out);
    }

    let _ = writeln!(out, "findings ({}):", report.findings.len());
    if report.findings.is_empty() {
        let _ = writeln!(out, "  (none)");
    } else {
        for finding in &report.findings {
            let _ = writeln!(out);
            render_finding(&mut out, finding);
        }
    }

    out
}

fn render_rehearsal(out: &mut String, rehearsal: &ReportRehearsal) {
    let _ = writeln!(out, "rehearsal:");
    let _ = writeln!(out, "  requested:    {}", rehearsal.requested);
    let _ = writeln!(out, "  ran:          {}", rehearsal.ran);
    let _ = writeln!(
        out,
        "  backend:      {}",
        rehearsal.backend_used.as_deref().unwrap_or("(none)")
    );
    let _ = writeln!(
        out,
        "  observed:     {}",
        join_or_none(&rehearsal.observations_captured)
    );
    let _ = writeln!(
        out,
        "  not observed: {}",
        join_or_none(&rehearsal.observations_unavailable)
    );
    let _ = writeln!(
        out,
        "  differences:  {}",
        join_or_none(&rehearsal.behavioral_differences_established)
    );
    let _ = writeln!(
        out,
        "  unverified:   {}",
        join_or_none(&rehearsal.remains_unverified)
    );
}

fn join_or_none(items: &[String]) -> String {
    if items.is_empty() {
        "(none)".to_string()
    } else {
        items.join(", ")
    }
}

fn render_finding(out: &mut String, finding: &ReportFinding) {
    let _ = writeln!(
        out,
        "[{}] [{}/{}] {}: {}",
        finding.rule, finding.severity, finding.confidence, finding.category, finding.subject
    );
    let _ = writeln!(out, "  summary: {}", finding.summary);
    let _ = writeln!(out, "  detail:  {}", finding.detail);
    if let Some(remediation) = &finding.remediation {
        let _ = writeln!(out, "  remediation: {remediation}");
    }
    if !finding.evidence.is_empty() {
        let _ = writeln!(out, "  evidence: {}", finding.evidence.join(", "));
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::expect_used)]
mod tests {
    use super::*;
    use crate::canonical::ReportExecutableIdentity;
    use analyzer_core::{AnalysisStatus, Confidence, Finding, FindingCategory, Rule, Severity};

    fn sample_report() -> AnalysisReport {
        let finding = Finding::new(
            FindingCategory::Interface,
            Rule::ContractInterfaceRemoved,
            "transfer",
            "candidate removes a function the current executable has",
            "removed function 'transfer'",
            Severity::High,
            Confidence::Detected,
            vec![],
        )
        .with_remediation("review callers of transfer");

        AnalysisReport::new(
            "0.1.0",
            Some(28),
            ReportExecutableIdentity {
                hash: "a".repeat(64),
                byte_length: 8,
            },
            ReportExecutableIdentity {
                hash: "b".repeat(64),
                byte_length: 16,
            },
            AnalysisStatus::ReviewRequired,
            std::slice::from_ref(&finding),
            Some(ReportRehearsal {
                requested: true,
                ran: true,
                backend_used: Some("soroban-env-host".to_string()),
                observations_captured: vec!["outcome".to_string()],
                observations_unavailable: vec!["events".to_string()],
                behavioral_differences_established: vec![],
                remains_unverified: vec!["events".to_string()],
            }),
            &[],
        )
    }

    #[test]
    fn renders_status_and_executable_identities() {
        let text = render_terminal(&sample_report());
        assert!(text.contains("status: REVIEW_REQUIRED"));
        assert!(text.contains(&"a".repeat(64)));
        assert!(text.contains(&"b".repeat(64)));
    }

    #[test]
    fn renders_every_finding_field() {
        let text = render_terminal(&sample_report());
        assert!(text.contains("CONTRACT_INTERFACE_REMOVED"));
        assert!(text.contains("HIGH/DETECTED"));
        assert!(text.contains("interface: transfer"));
        assert!(text.contains("candidate removes a function the current executable has"));
        assert!(text.contains("removed function 'transfer'"));
        assert!(text.contains("review callers of transfer"));
    }

    #[test]
    fn renders_rehearsal_summary_when_present() {
        let text = render_terminal(&sample_report());
        assert!(text.contains("rehearsal:"));
        assert!(text.contains("backend:      soroban-env-host"));
    }

    #[test]
    fn omits_rehearsal_section_when_absent() {
        let mut report = sample_report();
        report.rehearsal = None;
        let text = render_terminal(&report);
        assert!(!text.contains("rehearsal:"));
    }

    #[test]
    fn reports_no_findings_explicitly() {
        let mut report = sample_report();
        report.findings.clear();
        let text = render_terminal(&report);
        assert!(text.contains("findings (0):"));
        assert!(text.contains("(none)"));
    }

    #[test]
    fn rendering_is_deterministic() {
        let report = sample_report();
        assert_eq!(render_terminal(&report), render_terminal(&report));
    }

    #[test]
    fn output_ends_with_a_trailing_newline() {
        let text = render_terminal(&sample_report());
        assert!(text.ends_with('\n'));
    }
}
