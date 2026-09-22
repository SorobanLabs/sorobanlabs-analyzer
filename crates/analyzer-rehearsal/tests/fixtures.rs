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
