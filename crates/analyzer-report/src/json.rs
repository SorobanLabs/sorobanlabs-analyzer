//! Canonical JSON serialization of [`AnalysisReport`].
//!
//! "Canonical" here means: deterministic field order (struct field
//! declaration order, which `serde_json` preserves for
//! non-map/non-`BTreeMap` types), pretty-printed with a trailing
//! newline for stable diffs, and no nondeterministic values (no
//! timestamps, no random identifiers) anywhere in the document. Two
//! calls with equal [`AnalysisReport`] values always produce
//! byte-identical output.

use analyzer_core::{AnalyzerError, SerializationError};

use crate::canonical::AnalysisReport;

/// Serialize `report` to canonical, pretty-printed JSON with a trailing
/// newline.
pub fn to_canonical_json(report: &AnalysisReport) -> Result<String, AnalyzerError> {
    let mut json = serde_json::to_string_pretty(report).map_err(|source| {
        SerializationError::with_source("failed to serialize analysis report", source)
    })?;
    json.push('\n');
    Ok(json)
}

/// Parse a canonical JSON document back into an [`AnalysisReport`].
pub fn from_canonical_json(json: &str) -> Result<AnalysisReport, AnalyzerError> {
    serde_json::from_str(json).map_err(|source| {
        SerializationError::with_source("failed to parse analysis report", source).into()
    })
}

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::expect_used)]
mod tests {
    use super::*;
    use crate::canonical::ReportExecutableIdentity;
    use analyzer_core::AnalysisStatus;

    fn sample_report() -> AnalysisReport {
        AnalysisReport::new(
            "0.1.0",
            Some(28),
            ReportExecutableIdentity {
                hash: "a".repeat(64),
                byte_length: 8,
            },
            ReportExecutableIdentity {
                hash: "b".repeat(64),
                byte_length: 17,
            },
            AnalysisStatus::NoDetectedBlockers,
            &[],
            None,
            &[],
        )
    }

    #[test]
    fn serialization_is_deterministic_across_calls() {
        let report = sample_report();
        let first = to_canonical_json(&report).unwrap();
        let second = to_canonical_json(&report).unwrap();
        assert_eq!(first, second);
    }

    #[test]
    fn output_ends_with_a_single_trailing_newline() {
        let json = to_canonical_json(&sample_report()).unwrap();
        assert!(json.ends_with('\n'));
        assert!(!json.ends_with("\n\n"));
    }

    #[test]
    fn round_trips_through_json() {
        let report = sample_report();
        let json = to_canonical_json(&report).unwrap();
        let parsed = from_canonical_json(&json).unwrap();
        assert_eq!(report, parsed);
    }

    #[test]
    fn malformed_json_returns_structured_error_not_panic() {
        let result = from_canonical_json("not valid json");
        assert!(matches!(result, Err(AnalyzerError::Serialization(_))));
    }
}
