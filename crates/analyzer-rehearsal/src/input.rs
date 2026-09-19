//! Typed inputs to a controlled upgrade rehearsal.
//!
//! A rehearsal compares the current and candidate executables' observed
//! behavior under the same representative state and the same set of
//! invocations. This module only models *what* a rehearsal is asked to
//! do; it does not execute anything (see the crate-level docs for the
//! current state of the execution boundary).

use serde::{Deserialize, Serialize};

use analyzer_state::StateSnapshot;

/// One function call to rehearse, identically, against both the current
/// and candidate executable.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RehearsalInvocation {
    /// A caller-assigned name for this invocation, used to correlate
    /// observations back to the request (for example, `"transfer with
    /// zero amount"`). Not the contract function name by itself, since
    /// a rehearsal may want to exercise the same function multiple
    /// times with different arguments.
    pub label: String,
    /// The contract function to call.
    pub function_name: String,
    /// The function's arguments, each as hex of its canonical `ScVal`
    /// XDR encoding, in call order. Kept as XDR hex rather than a typed
    /// value, consistent with how this workspace represents contract
    /// data elsewhere (see `analyzer_state::ContractDataEntrySnapshot`).
    pub arguments_xdr_hex: Vec<String>,
}

/// Bounds on one rehearsal execution, so a hostile or pathological
/// candidate cannot consume unbounded resources.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct ExecutionLimits {
    /// Maximum CPU instructions the host may execute for one
    /// invocation, if the host exposes such a limit. `None` means no
    /// limit was configured by the caller; it does not mean execution
    /// is unbounded, since a real host backend may impose its own
    /// default limit regardless.
    pub max_instructions: Option<u64>,
    /// Maximum memory, in bytes, the host may allocate for one
    /// invocation.
    pub max_memory_bytes: Option<u64>,
}

/// Everything a controlled rehearsal needs: the two executables, the
/// state to run them against, the invocations to exercise, and the
/// bounds to enforce.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RehearsalInput {
    /// The current executable's exact bytes.
    pub current_executable: Vec<u8>,
    /// The candidate executable's exact bytes.
    pub candidate_executable: Vec<u8>,
    /// The representative state both executables are run against. Both
    /// sides start from the identical snapshot; a rehearsal only
    /// establishes behavior for the state and invocations selected
    /// here, never for a contract's full state space.
    pub state_snapshot: Option<StateSnapshot>,
    /// The invocations to exercise, in order.
    pub invocations: Vec<RehearsalInvocation>,
    /// The Soroban protocol number to rehearse under, when the host
    /// backend needs one to select behavior.
    pub protocol_context: Option<u32>,
    pub execution_limits: ExecutionLimits,
    /// A seed for any randomness a real host backend's execution needs,
    /// so that two rehearsals of the same input are reproducible.
    pub deterministic_seed: Option<u64>,
}

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::expect_used)]
mod tests {
    use super::*;

    #[test]
    fn execution_limits_default_to_no_configured_bound() {
        let limits = ExecutionLimits::default();
        assert_eq!(limits.max_instructions, None);
        assert_eq!(limits.max_memory_bytes, None);
    }

    #[test]
    fn rehearsal_input_serializes_deterministically() {
        let input = RehearsalInput {
            current_executable: vec![0x00, 0x61, 0x73, 0x6d, 0x01, 0x00, 0x00, 0x00],
            candidate_executable: vec![0x00, 0x61, 0x73, 0x6d, 0x01, 0x00, 0x00, 0x00],
            state_snapshot: None,
            invocations: vec![RehearsalInvocation {
                label: "noop".to_string(),
                function_name: "hello".to_string(),
                arguments_xdr_hex: vec![],
            }],
            protocol_context: Some(28),
            execution_limits: ExecutionLimits::default(),
            deterministic_seed: Some(1),
        };

        let first = serde_json::to_string(&input).unwrap();
        let second = serde_json::to_string(&input).unwrap();
        assert_eq!(first, second);

        let parsed: RehearsalInput = serde_json::from_str(&first).unwrap();
        assert_eq!(parsed, input);
    }
}
