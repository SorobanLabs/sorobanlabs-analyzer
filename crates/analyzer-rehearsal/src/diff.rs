use crate::observation::{
    EventObservation, ExecutionOutcome, InvocationObservation, ResourceUsage,
    StateAccessObservation,
};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "status", rename_all = "snake_case")]
pub enum Difference<T> {
    Unchanged,
    Changed { old: T, new: T },
    NotObservable,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct BehavioralDiff {
    pub invocation_label: String,
    pub outcome: Difference<ExecutionOutcome>,
    pub return_value: Difference<Option<String>>,
    pub events: Difference<Vec<EventObservation>>,
    pub state_reads: Difference<Vec<StateAccessObservation>>,
    pub state_writes: Difference<Vec<StateAccessObservation>>,
    pub resource_usage: Difference<ResourceUsage>,
}

pub fn diff_invocations(
    old: &InvocationObservation,
    new: &InvocationObservation,
) -> BehavioralDiff {
    let outcome = if old.outcome == new.outcome {
        Difference::Unchanged
    } else {
        Difference::Changed {
            old: old.outcome.clone(),
            new: new.outcome.clone(),
        }
    };

    let return_value = if old.return_value_xdr_hex == new.return_value_xdr_hex {
        Difference::Unchanged
    } else {
        Difference::Changed {
            old: old.return_value_xdr_hex.clone(),
            new: new.return_value_xdr_hex.clone(),
        }
    };

    // The current execution backend does not capture events or state.
    // We explicitly declare them NotObservable rather than falsely comparing empty lists.
    let events = Difference::NotObservable;
    let state_reads = Difference::NotObservable;
    let state_writes = Difference::NotObservable;

    let resource_usage = match (old.resource_usage, new.resource_usage) {
        (
            ResourceUsage {
                instructions_consumed: None,
                memory_bytes_consumed: None,
            },
            ResourceUsage {
                instructions_consumed: None,
                memory_bytes_consumed: None,
            },
        ) => Difference::NotObservable,
        (o, n) if o == n => Difference::Unchanged,
        (o, n) => Difference::Changed { old: o, new: n },
    };

    BehavioralDiff {
        invocation_label: old.invocation_label.clone(),
        outcome,
        return_value,
        events,
        state_reads,
        state_writes,
        resource_usage,
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::expect_used)]
mod tests {
    use super::*;

    fn dummy_observation(
        outcome: ExecutionOutcome,
        return_val: Option<&str>,
    ) -> InvocationObservation {
        InvocationObservation {
            invocation_label: "test".to_string(),
            outcome,
            return_value_xdr_hex: return_val.map(|s| s.to_string()),
            events: vec![],
            state_reads: vec![],
            state_writes: vec![],
            resource_usage: ResourceUsage::default(),
        }
    }

    #[test]
    fn identical_behavior() {
        let old = dummy_observation(ExecutionOutcome::Success, Some("00"));
        let new = dummy_observation(ExecutionOutcome::Success, Some("00"));
        let diff = diff_invocations(&old, &new);

        assert_eq!(diff.outcome, Difference::Unchanged);
        assert_eq!(diff.return_value, Difference::Unchanged);
        assert_eq!(diff.events, Difference::NotObservable);
    }

    #[test]
    fn changed_return_value() {
        let old = dummy_observation(ExecutionOutcome::Success, Some("00"));
        let new = dummy_observation(ExecutionOutcome::Success, Some("01"));
        let diff = diff_invocations(&old, &new);

        assert_eq!(diff.outcome, Difference::Unchanged);
        assert!(matches!(diff.return_value, Difference::Changed { .. }));
    }

    #[test]
    fn changed_error() {
        let old = dummy_observation(
            ExecutionOutcome::HostError {
                message: "err1".to_string(),
            },
            None,
        );
        let new = dummy_observation(
            ExecutionOutcome::HostError {
                message: "err2".to_string(),
            },
            None,
        );
        let diff = diff_invocations(&old, &new);

        assert!(matches!(diff.outcome, Difference::Changed { .. }));
        assert_eq!(diff.return_value, Difference::Unchanged);
    }

    #[test]
    fn matching_nonempty_events_are_still_not_observable() {
        // events/state_reads/state_writes must be NotObservable
        // unconditionally, never derived from comparing the vectors
        // themselves: the current backend does not populate them at
        // all, so an "unchanged" verdict here would claim an
        // observation that was never actually made, even if both sides
        // happen to hold matching (backend-unfilled) data.
        let event = EventObservation {
            topics_xdr_hex: vec!["01".to_string()],
            data_xdr_hex: "02".to_string(),
        };
        let access = StateAccessObservation {
            key_xdr_hex: "03".to_string(),
            durability: analyzer_state::Durability::Persistent,
        };

        let mut old = dummy_observation(ExecutionOutcome::Success, Some("00"));
        old.events = vec![event.clone()];
        old.state_reads = vec![access.clone()];
        old.state_writes = vec![access.clone()];

        let mut new = dummy_observation(ExecutionOutcome::Success, Some("00"));
        new.events = vec![event];
        new.state_reads = vec![access.clone()];
        new.state_writes = vec![access];

        let diff = diff_invocations(&old, &new);
        assert_eq!(diff.events, Difference::NotObservable);
        assert_eq!(diff.state_reads, Difference::NotObservable);
        assert_eq!(diff.state_writes, Difference::NotObservable);
    }

    #[test]
    fn reported_resource_usage_is_compared_not_marked_not_observable() {
        let mut old = dummy_observation(ExecutionOutcome::Success, Some("00"));
        old.resource_usage = ResourceUsage {
            instructions_consumed: Some(100),
            memory_bytes_consumed: Some(10),
        };
        let mut new = old.clone();
        new.resource_usage = ResourceUsage {
            instructions_consumed: Some(200),
            memory_bytes_consumed: Some(10),
        };

        let diff = diff_invocations(&old, &new);
        assert!(matches!(diff.resource_usage, Difference::Changed { .. }));

        let same_diff = diff_invocations(&old, &old);
        assert_eq!(same_diff.resource_usage, Difference::Unchanged);
    }

    #[test]
    fn candidate_execution_failure() {
        let old = dummy_observation(ExecutionOutcome::Success, Some("00"));
        let new = dummy_observation(
            ExecutionOutcome::Trap {
                message: "panic".to_string(),
            },
            None,
        );
        let diff = diff_invocations(&old, &new);

        assert!(matches!(diff.outcome, Difference::Changed { .. }));
        assert!(matches!(diff.return_value, Difference::Changed { .. }));
    }
}
