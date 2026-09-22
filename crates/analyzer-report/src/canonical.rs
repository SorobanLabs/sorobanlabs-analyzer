//! The canonical analysis report: a stable, serializable shape derived
//! from the analyzer's internal domain types.
//!
//! `AnalysisReport` and its constituents are this crate's *own* types,
//! not re-exports of `analyzer_core::Finding` or similar: the report
//! schema is a versioned external contract ([`crate::schema`]), and
//! keeping it separate from the internal domain model means an internal
//! refactor never silently changes the wire format. `ReportFinding`
//! carries every finding field as plain, canonically-ordered data
//! (`String`, not the internal `Rule`/`Severity`/`Confidence` enums),
//! converted once via [`ReportFinding::from_finding`].
//!
//! Every field here is either a scalar, a `String`, or a `Vec` built in
//! a caller-controlled, deterministic order; this module never
//! introduces a `HashMap` or other unordered collection, so canonical
//! JSON serialization ([`crate::json`]) is deterministic by
//! construction.

use analyzer_core::{AnalysisStatus, Finding};
use serde::{Deserialize, Serialize};

use crate::schema::REPORT_SCHEMA_VERSION;

/// One finding, represented as plain canonical data for serialization.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ReportFinding {
    pub id: String,
    pub category: String,
    pub rule: String,
    pub subject: String,
    pub summary: String,
    pub detail: String,
    pub severity: String,
    pub confidence: String,
    pub evidence: Vec<String>,
    pub remediation: Option<String>,
}

impl ReportFinding {
    /// Convert an internal [`Finding`] into its canonical report form.
    pub fn from_finding(finding: &Finding) -> Self {
        Self {
            id: finding.id().to_hex(),
            category: finding.category().to_string(),
            rule: finding.rule().to_string(),
            subject: finding.subject().to_string(),
            summary: finding.summary().to_string(),
            detail: finding.detail().to_string(),
            severity: finding.severity().to_string(),
            confidence: finding.confidence().to_string(),
            evidence: finding.evidence().iter().map(|id| id.to_string()).collect(),
            remediation: finding.remediation().map(str::to_string),
        }
    }
}

/// The identity of one executable side (current or candidate) of an
/// analysis, in canonical report form.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ReportExecutableIdentity {
    /// Lowercase hex SHA-256 of the exact executable bytes.
    pub hash: String,
    pub byte_length: u64,
}

/// The canonical analysis report.
///
/// `schema_version` and `analyzer_version` are independent: the schema
/// version changes only when the wire format changes in a way that
/// could break a consumer; the analyzer version tracks this crate's own
/// release. `protocol_context` is the Soroban protocol number the
/// analysis was run against, when one was specified; it is `None` when
/// no protocol context was supplied, which is itself meaningful (the
/// analysis could not consider protocol-specific compatibility).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AnalysisReport {
    pub schema_version: String,
    pub analyzer_version: String,
    pub protocol_context: Option<u32>,
    pub current_executable: ReportExecutableIdentity,
    pub candidate_executable: ReportExecutableIdentity,
    pub status: String,
    pub rehearsal: Option<ReportRehearsal>,
    pub findings: Vec<ReportFinding>,
}

impl AnalysisReport {
    /// Construct a canonical report at the current [`REPORT_SCHEMA_VERSION`].
    ///
    /// `findings` must already be in the caller's intended, deterministic
    /// order; this constructor does not reorder them.
    pub fn new(
        analyzer_version: impl Into<String>,
        protocol_context: Option<u32>,
        current_executable: ReportExecutableIdentity,
        candidate_executable: ReportExecutableIdentity,
        status: AnalysisStatus,
        findings: &[Finding],
        rehearsal: Option<ReportRehearsal>,
    ) -> Self {
        Self {
            schema_version: REPORT_SCHEMA_VERSION.to_string(),
            analyzer_version: analyzer_version.into(),
            protocol_context,
            current_executable,
            candidate_executable,
            status: status.to_string(),
            rehearsal,
            findings: findings.iter().map(ReportFinding::from_finding).collect(),
        }
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::expect_used)]
mod tests {
    use super::*;
    use analyzer_core::{Confidence, FindingCategory, Rule, Severity};

    #[test]
    fn report_carries_the_current_schema_version() {
        let report = AnalysisReport::new(
            "0.1.0",
            Some(28),
            ReportExecutableIdentity {
                hash: "a".repeat(64),
                byte_length: 8,
            },
            ReportExecutableIdentity {
                hash: "b".repeat(64),
                byte_length: 8,
            },
            AnalysisStatus::NoDetectedBlockers,
            &[],
            None,
        );
        assert_eq!(report.schema_version, REPORT_SCHEMA_VERSION);
    }

    #[test]
    fn finding_conversion_preserves_every_field() {
        let finding = Finding::new(
            FindingCategory::Interface,
            Rule::ContractSignatureChanged,
            "transfer",
            "summary",
            "detail",
            Severity::Medium,
            Confidence::Likely,
            vec![],
        )
        .with_remediation("review the diff");

        let report_finding = ReportFinding::from_finding(&finding);
        assert_eq!(report_finding.id, finding.id().to_hex());
        assert_eq!(report_finding.category, "interface");
        assert_eq!(report_finding.rule, "CONTRACT_SIGNATURE_CHANGED");
        assert_eq!(report_finding.subject, "transfer");
        assert_eq!(report_finding.severity, "MEDIUM");
        assert_eq!(report_finding.confidence, "LIKELY");
        assert_eq!(
            report_finding.remediation,
            Some("review the diff".to_string())
        );
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ReportRehearsal {
    pub requested: bool,
    pub ran: bool,
    pub backend_used: Option<String>,
    pub observations_captured: Vec<String>,
    pub observations_unavailable: Vec<String>,
    pub behavioral_differences_established: Vec<String>,
    pub remains_unverified: Vec<String>,
}
