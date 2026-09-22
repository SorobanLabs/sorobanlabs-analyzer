//! Typed inputs to a controlled upgrade rehearsal.
//!
//! A rehearsal compares the current and candidate executables' observed
//! behavior for a set of invocations. This module only models *what* a
//! rehearsal is asked to do; it does not execute anything (see the
//! crate-level docs for the current state of the execution boundary).
//!
//! # Which fields the current backend actually consumes
//!
//! [`host::rehearse_invocation`](crate::host::rehearse_invocation) takes
//! `(wasm_bytes, invocation, limits)`: only [`RehearsalInvocation`] and
//! [`ExecutionLimits`] reach it. The remaining [`RehearsalInput`] fields
//! are honest about that today:
//!
//! - `current_executable`/`candidate_executable`: **not consumed**.
//!   `analyzer-cli`'s orchestration rehearses the bytes it already
//!   loaded from `--current`/`--candidate`, not these fields; a caller
//!   that populates them with different bytes than the executables it
//!   separately supplies will not see that mismatch affect the
//!   rehearsal outcome. They exist so `RehearsalInput` is a
//!   self-contained record of what was rehearsed for a caller that
//!   invokes this crate directly (outside `analyzer-cli`); doing so
//!   consistently is the caller's responsibility, not something this
//!   crate currently checks.
//! - `state_snapshot`: **not consumed**. The backend does not apply any
//!   ledger state before invoking; every rehearsal currently runs
//!   against whatever empty/default footprint
//!   `Host::test_host_with_recording_footprint` starts with.
//! - `protocol_context`: **not consumed**. The backend always runs
//!   under `soroban-env-host`'s own fixed test-protocol version
//!   (verified against the `test_host_with_recording_footprint`
//!   implementation, which does not accept a protocol argument); this
//!   field is recorded for the caller's own bookkeeping only.
//! - `deterministic_seed`: **not consumed**. The backend
//!   (`test_host_with_recording_footprint`) has no source of
//!   caller-supplied randomness to seed; rehearsal is deterministic
//!   because the host path used has none, not because this field
//!   configures it.
//!
//! None of this is a promise that these fields will stay unused
//! forever, only an accurate statement of today's behavior. A caller
//! that needs state-aware, protocol-aware, or seeded rehearsal cannot
//! get it from the current backend regardless of what it puts in these
//! fields.

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

/// Everything a controlled rehearsal is asked to run. See this module's
/// top-level docs for which fields the current backend actually
/// consumes; several exist to make this a self-contained record even
/// though today's backend does not read them yet.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RehearsalInput {
    /// The current executable's exact bytes, for this record's own
    /// self-description. **Not consumed** by `analyzer-cli`'s
    /// orchestration today; see this module's top-level docs.
    pub current_executable: Vec<u8>,
    /// The candidate executable's exact bytes. **Not consumed** by
    /// `analyzer-cli`'s orchestration today; see this module's
    /// top-level docs.
    pub candidate_executable: Vec<u8>,
    /// The representative state both executables would be run against.
    /// **Not consumed** by the current backend: no state is applied
    /// before invocation. See this module's top-level docs.
    pub state_snapshot: Option<StateSnapshot>,
    /// The invocations to exercise, in order. Consumed.
    pub invocations: Vec<RehearsalInvocation>,
    /// The Soroban protocol number the caller intends to rehearse
    /// under. **Not consumed**: the current backend always runs under
    /// `soroban-env-host`'s own fixed test-protocol version. See this
    /// module's top-level docs.
    pub protocol_context: Option<u32>,
    /// Consumed by `analyzer-rehearsal::host`.
    pub execution_limits: ExecutionLimits,
    /// A seed for randomness a future host backend's execution might
    /// need. **Not consumed**: the current backend has no source of
    /// caller-supplied randomness to seed. See this module's top-level
    /// docs.
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
