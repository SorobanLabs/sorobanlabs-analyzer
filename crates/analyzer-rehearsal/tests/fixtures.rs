#![allow(clippy::unwrap_used, clippy::expect_used)]

use analyzer_executable::LoadedWasm;
use std::path::PathBuf;

fn fixture_path(name: &str) -> PathBuf {
    let mut path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    path.push("../../fixtures/executable");
    path.push(name);
    path
}

fn load_fixture(name: &str) -> LoadedWasm {
    let path = fixture_path(name);
    match LoadedWasm::from_path(&path) {
        Ok(exec) => exec,
        Err(err) => panic!("failed to load fixture {}: {err}", path.display()),
    }
}

#[test]
fn test_rehearsal_corpus_loads() {
    let fixtures = vec![
        "v1.wasm",
        "v2_identical.wasm",
        "v2_changed_return.wasm",
        "v2_changed_error.wasm",
        "v2_changed_event.wasm",
        "v2_changed_state.wasm",
        "v2_fails.wasm",
    ];

    for fixture in fixtures {
        let exec = load_fixture(fixture);
        // Minimal assertions to prove it loaded and validated
        assert!(!exec.bytes().is_empty());
        // It has a valid hash
        assert_eq!(exec.hash().to_string().len(), 64);
    }
}

#[test]
fn v2_fails_is_blocked_or_host_error() {
    use analyzer_rehearsal::{
        rehearse_invocation, ExecutionLimits, ExecutionOutcome, RehearsalInvocation,
    };
    use soroban_env_host::xdr::{Limits, ScVal, WriteXdr};

    let wasm = std::fs::read(fixture_path("v2_fails.wasm")).unwrap();
    let invocation = RehearsalInvocation {
        label: "probe".to_string(),
        function_name: "add".to_string(),
        arguments_xdr_hex: vec![
            hex::encode(ScVal::I32(1).to_xdr(Limits::none()).unwrap()),
            hex::encode(ScVal::I32(2).to_xdr(Limits::none()).unwrap()),
        ],
    };
    let obs = rehearse_invocation(&wasm, &invocation, &ExecutionLimits::default());
    eprintln!("v2_fails outcome: {:?}", obs.outcome);
    assert!(
        matches!(
            obs.outcome,
            ExecutionOutcome::Blocked { .. } | ExecutionOutcome::HostError { .. }
        ),
        "expected Blocked or HostError, got {:?}",
        obs.outcome
    );
}
