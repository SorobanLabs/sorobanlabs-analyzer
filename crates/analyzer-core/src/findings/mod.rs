//! The analyzer's finding model: confidence, severity, rules, and the
//! [`Finding`] record itself.
//!
//! A finding is the result of a *successful* analysis operation that
//! observed or inferred something about a proposed executable
//! replacement. It is never used to represent an operational failure
//! (see [`crate::error::AnalyzerError`] for that); an
//! [`AnalyzerError`](crate::error::AnalyzerError) means the analyzer
//! could not complete the requested operation, while a [`Finding`]
//! means it could, and this is what it found.
//!
//! # Confidence versus severity
//!
//! These are independent axes, deliberately never collapsed into a
//! single score:
//!
//! - [`Confidence`] says *how sure* the analyzer is that the condition
//!   holds, given the evidence it had.
//! - [`Severity`] says *how much it would matter* if the condition does
//!   hold.
//!
//! A [`Confidence::NotDeterminable`] finding is not a negative result:
//! it means the analyzer could not reach a conclusion from the
//! available evidence, not that nothing is wrong.

use analyzer_evidence::EvidenceId;
use sha2::{Digest, Sha256};
use std::fmt;

/// How certain the analyzer is that an observed or inferred condition
/// actually holds, given the evidence available to it.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Confidence {
    /// The condition was directly established from available evidence.
    Detected,
    /// Available evidence strongly indicates the condition, but the
    /// analyzer cannot establish it with complete certainty.
    Likely,
    /// The change could create the condition, but the available
    /// evidence is insufficient to establish the actual impact.
    Potential,
    /// There is not enough information to reach a meaningful
    /// conclusion. This is not a negative finding.
    NotDeterminable,
}

impl Confidence {
    /// The stable, uppercase textual form used in reports and rule
    /// documentation.
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Detected => "DETECTED",
            Self::Likely => "LIKELY",
            Self::Potential => "POTENTIAL",
            Self::NotDeterminable => "NOT_DETERMINABLE",
        }
    }
}

impl fmt::Display for Confidence {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

/// How much it would matter if a finding's condition holds. Independent
/// of [`Confidence`]: a high-severity condition can still carry low
/// confidence, and must be reported as such rather than rounded up or
/// down.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Severity {
    /// Purely informational; not expected to require action.
    Info,
    Low,
    Medium,
    High,
    /// The most severe category this analyzer assigns. This is an
    /// analyzer-internal severity bucket, not an external certification
    /// or audit rating.
    Critical,
}

impl Severity {
    /// The stable, uppercase textual form used in reports.
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Info => "INFO",
            Self::Low => "LOW",
            Self::Medium => "MEDIUM",
            Self::High => "HIGH",
            Self::Critical => "CRITICAL",
        }
    }
}

impl fmt::Display for Severity {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The broad area of the analysis a finding belongs to.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum FindingCategory {
    Executable,
    Environment,
    Interface,
    State,
    Authorization,
    Rehearsal,
    Resource,
}

impl FindingCategory {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Executable => "executable",
            Self::Environment => "environment",
            Self::Interface => "interface",
            Self::State => "state",
            Self::Authorization => "authorization",
            Self::Rehearsal => "rehearsal",
            Self::Resource => "resource",
        }
    }
}

impl fmt::Display for FindingCategory {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A stable analyzer rule identifier. Every finding names the rule that
/// produced it; business logic for detecting a condition lives with the
/// subsystem that owns the rule, never as ad hoc string comparisons
/// scattered through the codebase.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Rule {
    ExecutableHashChanged,
    /// The executable is structurally valid generic WASM but fails one
    /// or more of Soroban's known structural requirements (start
    /// section, component-model sections, memory64, shared memory; see
    /// `analyzer_executable::validation`). Added when orchestration
    /// (Step 20) needed a rule for this condition and the original
    /// eighteen-rule catalog did not include one.
    ExecutableStructurallyIncompatible,
    EnvironmentInterfaceChanged,
    EnvironmentMetadataMissing,
    ContractInterfaceAdded,
    ContractInterfaceRemoved,
    ContractSignatureChanged,
    ContractEventChanged,
    ContractTypeChanged,
    StateSchemaChanged,
    MigrationRequired,
    StateCompatibilityUnknown,
    AuthorizationSurfaceChanged,
    AuthorizationRemoved,
    RehearsalResultChanged,
    RehearsalStateChanged,
    RehearsalEventChanged,
    RehearsalFailed,
    RehearsalErrorChanged,
    RehearsalAuthorizationChanged,
    RehearsalResourceChanged,
    RehearsalObservationIncomplete,
    ResourceUsageChanged,
}

impl Rule {
    /// The stable, `SCREAMING_SNAKE_CASE` rule identifier used in
    /// reports and documentation.
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::ExecutableHashChanged => "EXECUTABLE_HASH_CHANGED",
            Self::ExecutableStructurallyIncompatible => "EXECUTABLE_STRUCTURALLY_INCOMPATIBLE",
            Self::EnvironmentInterfaceChanged => "ENVIRONMENT_INTERFACE_CHANGED",
            Self::EnvironmentMetadataMissing => "ENVIRONMENT_METADATA_MISSING",
            Self::ContractInterfaceAdded => "CONTRACT_INTERFACE_ADDED",
            Self::ContractInterfaceRemoved => "CONTRACT_INTERFACE_REMOVED",
            Self::ContractSignatureChanged => "CONTRACT_SIGNATURE_CHANGED",
            Self::ContractEventChanged => "CONTRACT_EVENT_CHANGED",
            Self::ContractTypeChanged => "CONTRACT_TYPE_CHANGED",
            Self::StateSchemaChanged => "STATE_SCHEMA_CHANGED",
            Self::MigrationRequired => "MIGRATION_REQUIRED",
            Self::StateCompatibilityUnknown => "STATE_COMPATIBILITY_UNKNOWN",
            Self::AuthorizationSurfaceChanged => "AUTHORIZATION_SURFACE_CHANGED",
            Self::AuthorizationRemoved => "AUTHORIZATION_REMOVED",
            Self::RehearsalResultChanged => "REHEARSAL_RESULT_CHANGED",
            Self::RehearsalStateChanged => "REHEARSAL_STATE_CHANGED",
            Self::RehearsalEventChanged => "REHEARSAL_EVENT_CHANGED",
            Self::RehearsalFailed => "REHEARSAL_FAILED",
            Self::RehearsalErrorChanged => "REHEARSAL_ERROR_CHANGED",
            Self::RehearsalAuthorizationChanged => "REHEARSAL_AUTHORIZATION_CHANGED",
            Self::RehearsalResourceChanged => "REHEARSAL_RESOURCE_CHANGED",
            Self::RehearsalObservationIncomplete => "REHEARSAL_OBSERVATION_INCOMPLETE",
            Self::ResourceUsageChanged => "RESOURCE_USAGE_CHANGED",
        }
    }
}

impl fmt::Display for Rule {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The deterministic identity of a [`Finding`]: SHA-256 over its
/// category, rule, and subject. Human-readable text (summary, detail,
/// remediation) is deliberately excluded from identity, so rewording a
/// message never changes a finding's id.
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct FindingId([u8; 32]);

impl FindingId {
    fn compute(category: FindingCategory, rule: Rule, subject: &str) -> Self {
        let mut buffer = Vec::new();
        write_field(&mut buffer, category.as_str().as_bytes());
        write_field(&mut buffer, rule.as_str().as_bytes());
        write_field(&mut buffer, subject.as_bytes());

        let digest = Sha256::digest(&buffer);
        let mut out = [0u8; 32];
        out.copy_from_slice(&digest);
        Self(out)
    }

    /// The canonical textual representation: 64 lowercase hexadecimal
    /// characters.
    pub fn to_hex(&self) -> String {
        hex::encode(self.0)
    }
}

fn write_field(buffer: &mut Vec<u8>, field: &[u8]) {
    buffer.extend_from_slice(&(field.len() as u64).to_le_bytes());
    buffer.extend_from_slice(field);
}

impl fmt::Display for FindingId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.to_hex())
    }
}

impl fmt::Debug for FindingId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_tuple("FindingId").field(&self.to_hex()).finish()
    }
}

/// One result of a successful analysis operation: something the
/// analyzer observed or inferred about a proposed executable
/// replacement, together with how certain it is, how much it would
/// matter, and what evidence backs it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Finding {
    id: FindingId,
    category: FindingCategory,
    rule: Rule,
    subject: String,
    summary: String,
    detail: String,
    severity: Severity,
    confidence: Confidence,
    evidence: Vec<EvidenceId>,
    remediation: Option<String>,
}

impl Finding {
    /// Construct a finding. `subject` should name the specific entity
    /// the finding is about (a function name, `"executable"`, a type
    /// name); together with `category` and `rule` it determines the
    /// finding's deterministic id.
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        category: FindingCategory,
        rule: Rule,
        subject: impl Into<String>,
        summary: impl Into<String>,
        detail: impl Into<String>,
        severity: Severity,
        confidence: Confidence,
        evidence: Vec<EvidenceId>,
    ) -> Self {
        let subject = subject.into();
        let id = FindingId::compute(category, rule, &subject);
        Self {
            id,
            category,
            rule,
            subject,
            summary: summary.into(),
            detail: detail.into(),
            severity,
            confidence,
            evidence,
            remediation: None,
        }
    }

    /// Attach remediation or review guidance to this finding.
    pub fn with_remediation(mut self, remediation: impl Into<String>) -> Self {
        self.remediation = Some(remediation.into());
        self
    }

    pub fn id(&self) -> FindingId {
        self.id
    }

    pub fn category(&self) -> FindingCategory {
        self.category
    }

    pub fn rule(&self) -> Rule {
        self.rule
    }

    pub fn subject(&self) -> &str {
        &self.subject
    }

    pub fn summary(&self) -> &str {
        &self.summary
    }

    pub fn detail(&self) -> &str {
        &self.detail
    }

    pub fn severity(&self) -> Severity {
        self.severity
    }

    pub fn confidence(&self) -> Confidence {
        self.confidence
    }

    pub fn evidence(&self) -> &[EvidenceId] {
        &self.evidence
    }

    pub fn remediation(&self) -> Option<&str> {
        self.remediation.as_deref()
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::expect_used)]
mod tests {
    use super::*;

    #[test]
    fn confidence_and_severity_format_as_screaming_snake_case() {
        assert_eq!(Confidence::NotDeterminable.to_string(), "NOT_DETERMINABLE");
        assert_eq!(Severity::Critical.to_string(), "CRITICAL");
    }

    #[test]
    fn rule_formats_match_the_documented_catalog() {
        assert_eq!(
            Rule::ContractSignatureChanged.as_str(),
            "CONTRACT_SIGNATURE_CHANGED"
        );
        assert_eq!(Rule::MigrationRequired.as_str(), "MIGRATION_REQUIRED");
    }

    fn sample_finding(subject: &str) -> Finding {
        Finding::new(
            FindingCategory::Interface,
            Rule::ContractSignatureChanged,
            subject,
            "summary",
            "detail",
            Severity::Medium,
            Confidence::Detected,
            vec![],
        )
    }

    #[test]
    fn id_is_deterministic_for_same_category_rule_subject() {
        let a = sample_finding("transfer");
        let b = sample_finding("transfer");
        assert_eq!(a.id(), b.id());
    }

    #[test]
    fn id_differs_for_different_subject() {
        let a = sample_finding("transfer");
        let b = sample_finding("mint");
        assert_ne!(a.id(), b.id());
    }

    #[test]
    fn id_does_not_depend_on_human_readable_text() {
        let a = Finding::new(
            FindingCategory::Interface,
            Rule::ContractSignatureChanged,
            "transfer",
            "summary A",
            "detail A",
            Severity::Low,
            Confidence::Likely,
            vec![],
        );
        let b = Finding::new(
            FindingCategory::Interface,
            Rule::ContractSignatureChanged,
            "transfer",
            "a completely different summary",
            "a completely different detail",
            Severity::Critical,
            Confidence::NotDeterminable,
            vec![],
        );
        assert_eq!(a.id(), b.id());
    }

    #[test]
    fn id_formats_as_lowercase_hex_of_expected_length() {
        let finding = sample_finding("transfer");
        let hex = finding.id().to_hex();
        assert_eq!(hex.len(), 64);
        assert!(hex
            .chars()
            .all(|c| c.is_ascii_hexdigit() && !c.is_ascii_uppercase()));
    }

    #[test]
    fn remediation_is_optional() {
        let without = sample_finding("transfer");
        assert_eq!(without.remediation(), None);

        let with = sample_finding("transfer").with_remediation("review the diff");
        assert_eq!(with.remediation(), Some("review the diff"));
    }
}
