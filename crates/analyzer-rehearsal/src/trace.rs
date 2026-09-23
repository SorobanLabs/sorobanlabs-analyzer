//! A rehearsal's full, bounded trace: what was observed running the
//! same invocations against the current and candidate executables.
//!
//! This module models the *shape* of a completed rehearsal; comparing
//! the two sides to produce a behavioral diff is [`crate::diff`].
//! Keeping the trace and its comparison separate mirrors how
//! `analyzer_executable::interface` (normalization) and
//! `analyzer_executable::diff` (comparison) stay separate.

use serde::{Deserialize, Serialize};

use crate::observation::InvocationObservation;

/// The bounded trace of one rehearsal: one [`InvocationObservation`] per
/// requested invocation, per side, in request order.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct RehearsalTrace {
    /// Observations from rehearsing the current executable, in the same
    /// order as the request's invocations.
    pub current: Vec<InvocationObservation>,
    /// Observations from rehearsing the candidate executable, in the
    /// same order.
    pub candidate: Vec<InvocationObservation>,
}

impl RehearsalTrace {
    /// The current-side observation for `label`, if the trace has one.
    pub fn current_observation(&self, label: &str) -> Option<&InvocationObservation> {
        self.current
            .iter()
            .find(|observation| observation.invocation_label == label)
    }

    /// The candidate-side observation for `label`, if the trace has
    /// one.
    pub fn candidate_observation(&self, label: &str) -> Option<&InvocationObservation> {
        self.candidate
            .iter()
            .find(|observation| observation.invocation_label == label)
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::expect_used)]
mod tests {
    use super::*;
    use crate::observation::{ExecutionOutcome, ResourceUsage};

    fn observation(label: &str) -> InvocationObservation {
        InvocationObservation {
            invocation_label: label.to_string(),
            outcome: ExecutionOutcome::Success,
            return_value_xdr_hex: None,
            events: vec![],
            state_reads: vec![],
            state_writes: vec![],
            resource_usage: ResourceUsage::default(),
        }
    }

    #[test]
    fn empty_trace_has_no_observations() {
        let trace = RehearsalTrace::default();
        assert!(trace.current_observation("anything").is_none());
        assert!(trace.candidate_observation("anything").is_none());
    }

    #[test]
    fn looks_up_observations_by_label() {
        let trace = RehearsalTrace {
            current: vec![observation("transfer")],
            candidate: vec![observation("transfer")],
        };
        assert!(trace.current_observation("transfer").is_some());
        assert!(trace.candidate_observation("transfer").is_some());
        assert!(trace.current_observation("mint").is_none());
    }

    #[test]
    fn trace_serializes_deterministically() {
        let trace = RehearsalTrace {
            current: vec![observation("a"), observation("b")],
            candidate: vec![observation("a"), observation("b")],
        };
        let first = serde_json::to_string(&trace).unwrap();
        let second = serde_json::to_string(&trace).unwrap();
        assert_eq!(first, second);
    }
}
