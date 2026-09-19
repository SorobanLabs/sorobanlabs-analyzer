//! The overall outcome of an upgrade analysis run, and a human-oriented
//! summary of its findings ([`plan`]).

use std::fmt;

mod plan;

pub use plan::UpgradePlan;

/// The top-level result of analyzing a proposed executable replacement.
///
/// This is distinct from any individual [`crate::findings::Finding`]: it
/// summarizes what the analysis run as a whole established. It is
/// deliberately not a `SAFE`/`UNSAFE` verdict; it names what evidence
/// class was reached, nothing more.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum AnalysisStatus {
    /// The analysis ran to completion and detected no blocking
    /// condition. This does not mean the upgrade is safe in any
    /// absolute sense; it means the analyzer found nothing that meets
    /// its blocking criteria, given the evidence it had.
    NoDetectedBlockers,
    /// The analysis found one or more conditions that warrant human
    /// review before proceeding.
    ReviewRequired,
    /// The analysis established that a migration is required (or
    /// mechanically implied) before the candidate executable can safely
    /// replace the current one.
    MigrationRequired,
    /// The analysis could not reach a meaningful conclusion, typically
    /// because required evidence was unavailable or insufficient.
    Inconclusive,
    /// The analysis itself could not complete (an operational failure).
    /// This status exists for callers that need to represent "the
    /// pipeline did not finish" alongside an analytical outcome, even
    /// though the failure itself is carried as an
    /// [`crate::error::AnalyzerError`], not as analytical uncertainty.
    AnalysisError,
}

impl AnalysisStatus {
    /// The stable, `SCREAMING_SNAKE_CASE` textual form used in reports.
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::NoDetectedBlockers => "NO_DETECTED_BLOCKERS",
            Self::ReviewRequired => "REVIEW_REQUIRED",
            Self::MigrationRequired => "MIGRATION_REQUIRED",
            Self::Inconclusive => "INCONCLUSIVE",
            Self::AnalysisError => "ANALYSIS_ERROR",
        }
    }
}

impl fmt::Display for AnalysisStatus {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_status_formats_as_the_documented_constant() {
        assert_eq!(
            AnalysisStatus::NoDetectedBlockers.to_string(),
            "NO_DETECTED_BLOCKERS"
        );
        assert_eq!(
            AnalysisStatus::ReviewRequired.to_string(),
            "REVIEW_REQUIRED"
        );
        assert_eq!(
            AnalysisStatus::MigrationRequired.to_string(),
            "MIGRATION_REQUIRED"
        );
        assert_eq!(AnalysisStatus::Inconclusive.to_string(), "INCONCLUSIVE");
        assert_eq!(AnalysisStatus::AnalysisError.to_string(), "ANALYSIS_ERROR");
    }
}
