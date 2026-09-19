//! A human-oriented summary of an upgrade analysis, derived from its
//! findings.
//!
//! [`UpgradePlan`] sorts a set of [`Finding`]s into five buckets so a
//! reviewer can see, at a glance, what needs attention and why. It is a
//! **decision-support summary, not a safety guarantee**: an empty
//! `detected_blockers` list means no finding met this analyzer's
//! blocker criteria, not that the upgrade has been proven safe. Every
//! bucket holds [`FindingId`]s, not copies of the findings themselves,
//! so the plan stays a thin index into the authoritative finding list
//! rather than a second, possibly-diverging copy of it.

use crate::findings::{Confidence, Finding, FindingCategory, FindingId, Rule, Severity};

/// A human-oriented summary of an upgrade analysis's findings, sorted
/// into actionable buckets.
///
/// Bucketing is by explicit priority, so each finding lands in exactly
/// one bucket (or none, for purely informational findings):
///
/// 1. A finding for [`Rule::MigrationRequired`] is a migration
///    requirement.
/// 2. A finding in [`FindingCategory::Rehearsal`] is a rehearsal
///    requirement.
/// 3. A finding with [`Confidence::NotDeterminable`] is an unresolved
///    question.
/// 4. A [`Severity::Critical`] finding (with determinable confidence)
///    is a detected blocker.
/// 5. A [`Severity::High`] or [`Severity::Medium`] finding (with
///    determinable confidence) is a review item.
/// 6. [`Severity::Info`] or [`Severity::Low`] findings are not bucketed;
///    they remain in the full finding list but do not, by themselves,
///    warrant a reviewer's attention.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct UpgradePlan {
    /// Findings this analyzer treats as blocking: critical severity,
    /// established with determinable confidence.
    pub detected_blockers: Vec<FindingId>,
    /// Findings indicating a state migration is required.
    pub migration_requirements: Vec<FindingId>,
    /// Findings warranting human review: high or medium severity,
    /// established with determinable confidence.
    pub review_items: Vec<FindingId>,
    /// Findings the analyzer could not resolve from available evidence
    /// ([`Confidence::NotDeterminable`]), regardless of severity.
    pub unresolved_questions: Vec<FindingId>,
    /// Findings indicating controlled rehearsal is required or
    /// incomplete.
    pub rehearsal_requirements: Vec<FindingId>,
}

impl UpgradePlan {
    /// Build a plan from a set of findings.
    pub fn from_findings(findings: &[Finding]) -> Self {
        let mut plan = Self::default();

        for finding in findings {
            if finding.rule() == Rule::MigrationRequired {
                plan.migration_requirements.push(finding.id());
            } else if finding.category() == FindingCategory::Rehearsal {
                plan.rehearsal_requirements.push(finding.id());
            } else if finding.confidence() == Confidence::NotDeterminable {
                plan.unresolved_questions.push(finding.id());
            } else if finding.severity() == Severity::Critical {
                plan.detected_blockers.push(finding.id());
            } else if matches!(finding.severity(), Severity::High | Severity::Medium) {
                plan.review_items.push(finding.id());
            }
        }

        plan
    }

    /// True if no bucket holds anything: nothing in the finding set
    /// warranted blocking, migration, review, or clarification. This
    /// is not a claim that the upgrade is safe; see the module-level
    /// docs.
    pub fn is_clear(&self) -> bool {
        self.detected_blockers.is_empty()
            && self.migration_requirements.is_empty()
            && self.review_items.is_empty()
            && self.unresolved_questions.is_empty()
            && self.rehearsal_requirements.is_empty()
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::expect_used)]
mod tests {
    use super::*;
    use crate::findings::FindingCategory;

    fn finding(
        rule: Rule,
        category: FindingCategory,
        severity: Severity,
        confidence: Confidence,
    ) -> Finding {
        Finding::new(
            category,
            rule,
            "subject",
            "summary",
            "detail",
            severity,
            confidence,
            vec![],
        )
    }

    #[test]
    fn empty_findings_produce_a_clear_plan() {
        let plan = UpgradePlan::from_findings(&[]);
        assert!(plan.is_clear());
    }

    #[test]
    fn migration_required_rule_takes_priority() {
        let f = finding(
            Rule::MigrationRequired,
            FindingCategory::State,
            Severity::Critical,
            Confidence::Detected,
        );
        let plan = UpgradePlan::from_findings(std::slice::from_ref(&f));
        assert_eq!(plan.migration_requirements, vec![f.id()]);
        assert!(plan.detected_blockers.is_empty());
    }

    #[test]
    fn rehearsal_category_is_a_rehearsal_requirement_even_at_high_severity() {
        let f = finding(
            Rule::RehearsalFailed,
            FindingCategory::Rehearsal,
            Severity::High,
            Confidence::Detected,
        );
        let plan = UpgradePlan::from_findings(std::slice::from_ref(&f));
        assert_eq!(plan.rehearsal_requirements, vec![f.id()]);
        assert!(plan.review_items.is_empty());
    }

    #[test]
    fn not_determinable_confidence_is_an_unresolved_question_regardless_of_severity() {
        let f = finding(
            Rule::StateCompatibilityUnknown,
            FindingCategory::State,
            Severity::Critical,
            Confidence::NotDeterminable,
        );
        let plan = UpgradePlan::from_findings(std::slice::from_ref(&f));
        assert_eq!(plan.unresolved_questions, vec![f.id()]);
        assert!(plan.detected_blockers.is_empty());
    }

    #[test]
    fn critical_severity_with_determinable_confidence_is_a_blocker() {
        let f = finding(
            Rule::AuthorizationRemoved,
            FindingCategory::Authorization,
            Severity::Critical,
            Confidence::Likely,
        );
        let plan = UpgradePlan::from_findings(std::slice::from_ref(&f));
        assert_eq!(plan.detected_blockers, vec![f.id()]);
    }

    #[test]
    fn high_and_medium_severity_are_review_items() {
        let high = finding(
            Rule::ContractSignatureChanged,
            FindingCategory::Interface,
            Severity::High,
            Confidence::Detected,
        );
        let medium = finding(
            Rule::ContractEventChanged,
            FindingCategory::Interface,
            Severity::Medium,
            Confidence::Detected,
        );
        let plan = UpgradePlan::from_findings(&[high.clone(), medium.clone()]);
        assert_eq!(plan.review_items.len(), 2);
        assert!(plan.review_items.contains(&high.id()));
        assert!(plan.review_items.contains(&medium.id()));
    }

    #[test]
    fn info_and_low_severity_findings_are_not_bucketed() {
        let info = finding(
            Rule::ExecutableHashChanged,
            FindingCategory::Executable,
            Severity::Info,
            Confidence::Detected,
        );
        let plan = UpgradePlan::from_findings(&[info]);
        assert!(plan.is_clear());
    }

    #[test]
    fn each_finding_lands_in_exactly_one_bucket() {
        let findings = vec![
            finding(
                Rule::MigrationRequired,
                FindingCategory::State,
                Severity::High,
                Confidence::Likely,
            ),
            finding(
                Rule::RehearsalFailed,
                FindingCategory::Rehearsal,
                Severity::Info,
                Confidence::NotDeterminable,
            ),
            finding(
                Rule::StateCompatibilityUnknown,
                FindingCategory::State,
                Severity::Medium,
                Confidence::NotDeterminable,
            ),
            finding(
                Rule::AuthorizationRemoved,
                FindingCategory::Authorization,
                Severity::Critical,
                Confidence::Detected,
            ),
            finding(
                Rule::ContractSignatureChanged,
                FindingCategory::Interface,
                Severity::Medium,
                Confidence::Detected,
            ),
        ];
        let plan = UpgradePlan::from_findings(&findings);
        let total = plan.detected_blockers.len()
            + plan.migration_requirements.len()
            + plan.review_items.len()
            + plan.unresolved_questions.len()
            + plan.rehearsal_requirements.len();
        assert_eq!(total, findings.len());
    }
}
