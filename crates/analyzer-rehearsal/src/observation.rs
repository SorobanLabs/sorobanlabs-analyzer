//! Typed observations captured from one rehearsed invocation.
//!
//! Every field here is bounded, canonical XDR-hex, or a small scalar;
//! this module deliberately does not model an unrestricted execution
//! trace (arbitrary host call log, full memory dump, and so on). A real
//! execution backend (a later step) is responsible for enforcing the
//! bound when it fills these types in; this module only defines the
//! bounded shape.

use serde::{Deserialize, Serialize};

use analyzer_state::Durability;

/// How one rehearsed invocation concluded.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum ExecutionOutcome {
    /// The invocation completed and returned a value.
    Success,
    /// The invocation completed with a host-reported error (a
    /// contract-level error, not a panic or trap).
    HostError { message: String },
    /// The invocation trapped (a WASM trap: unreachable, out-of-bounds
    /// access, and so on).
    Trap { message: String },
    /// This invocation could not be safely or meaningfully executed by
    /// the current execution backend. See the crate-level docs for the
    /// backend's current boundary; this variant exists so a blocked
    /// invocation is represented explicitly rather than omitted or
    /// forced into a misleading `Success`/`HostError` shape.
    Blocked { reason: String },
}

/// One event observed during an invocation.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct EventObservation {
    /// Hex of each topic's canonical `ScVal` XDR encoding, in order.
    pub topics_xdr_hex: Vec<String>,
    /// Hex of the event's data `ScVal` XDR encoding.
    pub data_xdr_hex: String,
}

/// One contract data key read or written during an invocation.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct StateAccessObservation {
    /// Hex of the key's canonical `ScVal` XDR encoding.
    pub key_xdr_hex: String,
    pub durability: Durability,
}

/// Resource usage observed for one invocation, when the execution
/// backend provides trustworthy measurements. Every field is `None`
/// when the backend did not report it; this module never fabricates a
/// resource estimate.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct ResourceUsage {
    pub instructions_consumed: Option<u64>,
    pub memory_bytes_consumed: Option<u64>,
}

/// Everything observed from rehearsing one invocation against one
/// executable.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct InvocationObservation {
    /// The label of the [`crate::input::RehearsalInvocation`] this
    /// observation corresponds to.
    pub invocation_label: String,
    pub outcome: ExecutionOutcome,
    /// Hex of the returned value's canonical `ScVal` XDR encoding, when
    /// the invocation succeeded and returned a value.
    pub return_value_xdr_hex: Option<String>,
    pub events: Vec<EventObservation>,
    pub state_reads: Vec<StateAccessObservation>,
    pub state_writes: Vec<StateAccessObservation>,
    pub resource_usage: ResourceUsage,
}

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::expect_used)]
mod tests {
    use super::*;

    #[test]
    fn observation_serializes_and_round_trips() {
        let observation = InvocationObservation {
            invocation_label: "transfer zero".to_string(),
            outcome: ExecutionOutcome::Success,
            return_value_xdr_hex: Some("00".to_string()),
            events: vec![EventObservation {
                topics_xdr_hex: vec!["01".to_string()],
                data_xdr_hex: "02".to_string(),
            }],
            state_reads: vec![StateAccessObservation {
                key_xdr_hex: "03".to_string(),
                durability: Durability::Persistent,
            }],
            state_writes: vec![],
            resource_usage: ResourceUsage {
                instructions_consumed: Some(1000),
                memory_bytes_consumed: None,
            },
        };

        let json = serde_json::to_string(&observation).unwrap();
        let parsed: InvocationObservation = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed, observation);
    }

    #[test]
    fn blocked_outcome_is_distinct_from_host_error() {
        let blocked = ExecutionOutcome::Blocked {
            reason: "no execution backend available".to_string(),
        };
        let error = ExecutionOutcome::HostError {
            message: "contract error".to_string(),
        };
        assert_ne!(blocked, error);
    }

    #[test]
    fn resource_usage_defaults_to_unreported() {
        let usage = ResourceUsage::default();
        assert_eq!(usage.instructions_consumed, None);
        assert_eq!(usage.memory_bytes_consumed, None);
    }
}
