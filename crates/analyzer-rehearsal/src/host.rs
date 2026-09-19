//! A bounded local execution backend for controlled rehearsal, using the
//! official `soroban-env-host` implementation.
//!
//! # Feasibility (verified, not assumed)
//!
//! Before writing this module, a focused feasibility check was run
//! against the actual official source and an actual local build,
//! rather than inferred from memory or documentation summaries alone:
//!
//! - `soroban-env-host` 28.0.2 is the released version matching this
//!   workspace's existing `stellar-xdr` (`=28.0.0`) and `wasmparser`
//!   (`=0.116.1`) pins exactly, and is what `soroban-sdk` 28.0.0 (the
//!   current released SDK line) itself depends on. Verified against
//!   <https://github.com/stellar/rs-soroban-env/blob/v28.0.2/Cargo.toml>
//!   and the published crate's own dependency metadata.
//! - It builds under this workspace's declared MSRV, rustc 1.84.0,
//!   after five additional transitive `Cargo.lock` pins (documented in
//!   the workspace `Cargo.toml`, next to the dependency declaration);
//!   none are in `soroban-env-host`'s own code.
//! - A real minimal contract, built with the actual `soroban-sdk` 28.0.0
//!   and the official `stellar-cli` 27.0.0 toolchain (see
//!   `fixtures/executable-src/minimal-contract/`), was invoked through
//!   this exact dependency in this environment, and returned the
//!   correct result. This is `TESTED_LOCALLY` evidence, not an
//!   inference from documentation.
//! - The production embedder entry point is
//!   `soroban_env_host::e2e_invoke::invoke_host_function`, which takes
//!   fully-encoded XDR (resources, footprint, ledger entries, auth
//!   entries) matching what `stellar-core` itself supplies. This module
//!   instead uses the `testutils`-gated [`Host::register_test_contract_wasm`]
//!   and [`Host::test_host_with_recording_footprint`] path: it is the
//!   officially-supported way to load and invoke a WASM contract
//!   without also constructing that full transaction-level footprint,
//!   which this analyzer's rehearsal use case does not need (there is
//!   no real transaction, no real fee accounting, no real network).
//!   `testutils` is a real Cargo feature of the official crate, not a
//!   substitute or generic runtime.
//!
//! # Untrusted input handling
//!
//! The candidate executable is treated as untrusted: `Host`'s own
//! `register_test_contract_wasm` panics internally on certain malformed
//! inputs (it is designed for a test suite that controls its own
//! fixtures, not for arbitrary untrusted WASM). This module runs
//! registration and invocation inside
//! [`soroban_env_host::testutils::call_with_suppressed_panic_hook`]
//! (the official crate's own sanctioned mechanism for isolating a
//! panicking contract call, also suppressing the default panic hook's
//! console output) and converts a caught panic into
//! [`ExecutionOutcome::Blocked`] rather than letting it crash this
//! process.
//!
//! # Current scope
//!
//! This is the smallest real integration needed to prove and use the
//! host: it captures the invocation's success/error/trap/blocked
//! outcome and its return value. It does **not** yet capture emitted
//! events, state reads/writes, or resource usage (all left at their
//! default/empty values); those require decoding the host's `Events`
//! and footprint types into this crate's observation shapes, which is
//! deferred to the rehearsal-fixtures and behavioral-diff steps that
//! actually need them.

use std::panic::AssertUnwindSafe;

use soroban_env_host::{
    testutils::call_with_suppressed_panic_hook,
    xdr::{HostFunction, InvokeContractArgs, Limits, ReadXdr, ScSymbol, ScVal, StringM, WriteXdr},
    Host,
};

use crate::input::{ExecutionLimits, RehearsalInvocation};
use crate::observation::{ExecutionOutcome, InvocationObservation, ResourceUsage};

/// Default CPU instruction budget when the caller does not configure
/// one. This is an analyzer-chosen bound for rehearsal, not a claim
/// about any Soroban network's actual per-transaction limit.
const DEFAULT_CPU_INSTRUCTION_LIMIT: u64 = 100_000_000;
/// Default memory budget, in bytes, when the caller does not configure
/// one.
const DEFAULT_MEMORY_LIMIT_BYTES: u64 = 41_943_040;

/// Rehearse one invocation against one executable's WASM bytes.
///
/// This never panics: a panic inside the host (from malformed candidate
/// WASM) is caught and reported as [`ExecutionOutcome::Blocked`].
pub fn rehearse_invocation(
    wasm_bytes: &[u8],
    invocation: &RehearsalInvocation,
    limits: &ExecutionLimits,
) -> InvocationObservation {
    let cpu_limit = limits
        .max_instructions
        .unwrap_or(DEFAULT_CPU_INSTRUCTION_LIMIT);
    let memory_limit = limits
        .max_memory_bytes
        .unwrap_or(DEFAULT_MEMORY_LIMIT_BYTES);

    let outcome = call_with_suppressed_panic_hook(AssertUnwindSafe(|| {
        run_invocation(wasm_bytes, invocation, cpu_limit, memory_limit)
    }));

    match outcome {
        Ok(Ok(observation)) => observation,
        Ok(Err(reason)) => blocked(invocation, reason),
        Err(_panic_payload) => blocked(
            invocation,
            "the execution backend panicked while registering or invoking the candidate WASM"
                .to_string(),
        ),
    }
}

fn blocked(invocation: &RehearsalInvocation, reason: String) -> InvocationObservation {
    InvocationObservation {
        invocation_label: invocation.label.clone(),
        outcome: ExecutionOutcome::Blocked { reason },
        return_value_xdr_hex: None,
        events: vec![],
        state_reads: vec![],
        state_writes: vec![],
        resource_usage: ResourceUsage::default(),
    }
}

/// The actual registration + invocation, run inside the panic-catching
/// wrapper in [`rehearse_invocation`]. Returns `Err` for a problem this
/// function can detect without needing to execute anything (a malformed
/// invocation argument or function name); host-level failures during
/// execution itself become `Ok` with a non-success
/// [`ExecutionOutcome`], matching how `soroban-env-host` itself
/// distinguishes "could not run this at all" from "ran and failed".
fn run_invocation(
    wasm_bytes: &[u8],
    invocation: &RehearsalInvocation,
    cpu_limit: u64,
    memory_limit: u64,
) -> Result<InvocationObservation, String> {
    let args = invocation
        .arguments_xdr_hex
        .iter()
        .map(|hex_arg| decode_scval(hex_arg))
        .collect::<Result<Vec<ScVal>, String>>()?;

    let host = Host::test_host_with_recording_footprint().test_budget(cpu_limit, memory_limit);
    let address_object = host.register_test_contract_wasm(wasm_bytes);

    let contract_address = host
        .scaddress_from_address(address_object)
        .map_err(|source| {
            format!("failed to resolve the registered contract's address: {source}")
        })?;
    let function_name = ScSymbol(
        StringM::try_from(invocation.function_name.as_str()).map_err(|source| {
            format!(
                "invalid function name '{}': {source}",
                invocation.function_name
            )
        })?,
    );
    let encoded_args = args
        .try_into()
        .map_err(|_| "too many arguments for a single invocation".to_string())?;

    let invoke_args = InvokeContractArgs {
        contract_address,
        function_name,
        args: encoded_args,
    };

    let result = host.invoke_function(HostFunction::InvokeContract(invoke_args));

    let (outcome, return_value_xdr_hex) = match result {
        Ok(return_value) => {
            let hex = encode_scval(&return_value)?;
            (ExecutionOutcome::Success, Some(hex))
        }
        Err(host_error) => (
            ExecutionOutcome::HostError {
                message: host_error.to_string(),
            },
            None,
        ),
    };

    Ok(InvocationObservation {
        invocation_label: invocation.label.clone(),
        outcome,
        return_value_xdr_hex,
        events: vec![],
        state_reads: vec![],
        state_writes: vec![],
        resource_usage: ResourceUsage::default(),
    })
}

fn decode_scval(hex_str: &str) -> Result<ScVal, String> {
    let bytes =
        hex::decode(hex_str).map_err(|source| format!("invalid hex in argument: {source}"))?;
    ScVal::from_xdr(bytes, Limits::none())
        .map_err(|source| format!("invalid ScVal XDR in argument: {source}"))
}

fn encode_scval(value: &ScVal) -> Result<String, String> {
    let bytes = value
        .to_xdr(Limits::none())
        .map_err(|source| format!("failed to encode return value as XDR: {source}"))?;
    Ok(hex::encode(bytes))
}

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::expect_used)]
mod tests {
    use super::*;

    const FIXTURE_PATH: &str = concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../fixtures/executable/minimal-soroban-contract.wasm"
    );

    fn fixture_bytes() -> Vec<u8> {
        std::fs::read(FIXTURE_PATH)
            .expect("real minimal-soroban-contract.wasm fixture must be present")
    }

    fn i32_arg(value: i32) -> String {
        let scval = ScVal::I32(value);
        encode_scval(&scval).unwrap()
    }

    #[test]
    fn add_invocation_returns_the_correct_sum() {
        let wasm = fixture_bytes();
        let invocation = RehearsalInvocation {
            label: "add 2 and 3".to_string(),
            function_name: "add".to_string(),
            arguments_xdr_hex: vec![i32_arg(2), i32_arg(3)],
        };

        let observation = rehearse_invocation(&wasm, &invocation, &ExecutionLimits::default());

        assert_eq!(observation.outcome, ExecutionOutcome::Success);
        let returned = decode_scval(observation.return_value_xdr_hex.as_deref().unwrap()).unwrap();
        assert_eq!(returned, ScVal::I32(5));
    }

    #[test]
    fn calling_a_nonexistent_function_is_a_host_error_not_a_panic() {
        let wasm = fixture_bytes();
        let invocation = RehearsalInvocation {
            label: "call missing function".to_string(),
            function_name: "definitely_not_exported".to_string(),
            arguments_xdr_hex: vec![],
        };

        let observation = rehearse_invocation(&wasm, &invocation, &ExecutionLimits::default());
        assert!(matches!(
            observation.outcome,
            ExecutionOutcome::HostError { .. }
        ));
    }

    #[test]
    fn malformed_candidate_wasm_is_blocked_not_a_process_crash() {
        // Bytes that pass this crate's own boundary but are not
        // anything soroban-env-host can register as a contract.
        let malformed = b"not actually wasm".to_vec();
        let invocation = RehearsalInvocation {
            label: "malformed".to_string(),
            function_name: "add".to_string(),
            arguments_xdr_hex: vec![],
        };

        let observation = rehearse_invocation(&malformed, &invocation, &ExecutionLimits::default());
        assert!(matches!(
            observation.outcome,
            ExecutionOutcome::Blocked { .. }
        ));
    }

    #[test]
    fn invalid_argument_hex_is_blocked_not_a_panic() {
        let wasm = fixture_bytes();
        let invocation = RehearsalInvocation {
            label: "bad args".to_string(),
            function_name: "add".to_string(),
            arguments_xdr_hex: vec!["not hex".to_string()],
        };

        let observation = rehearse_invocation(&wasm, &invocation, &ExecutionLimits::default());
        assert!(matches!(
            observation.outcome,
            ExecutionOutcome::Blocked { .. }
        ));
    }

    #[test]
    fn repeated_rehearsal_of_the_same_invocation_is_deterministic() {
        let wasm = fixture_bytes();
        let invocation = RehearsalInvocation {
            label: "add 10 and 20".to_string(),
            function_name: "add".to_string(),
            arguments_xdr_hex: vec![i32_arg(10), i32_arg(20)],
        };

        let first = rehearse_invocation(&wasm, &invocation, &ExecutionLimits::default());
        let second = rehearse_invocation(&wasm, &invocation, &ExecutionLimits::default());
        assert_eq!(first.outcome, second.outcome);
        assert_eq!(first.return_value_xdr_hex, second.return_value_xdr_hex);
    }
}
