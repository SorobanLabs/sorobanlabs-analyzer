# Workspace integration tests

This top-level directory is intentionally empty of test files. The
cross-crate integration tests, fixture-driven analysis tests, JSON
schema validation tests, and determinism/reproducibility tests this
directory's name suggests all exist today, but live under each crate's
own `tests/` directory instead:

- `crates/analyzer-cli/tests/` -- end-to-end CLI behavior (exit codes,
  `--format` output, error handling) and pipeline integration tests.
- `crates/analyzer-report/tests/` -- JSON schema validation and report
  determinism tests.
- `crates/analyzer-rehearsal/tests/` -- rehearsal-backend integration
  tests against the real `soroban-env-host`.

Run the full suite from the repository root with:

```
cargo test --workspace
```

There is no separate top-level test suite here, and none is planned;
consolidating the per-crate tests into this directory would duplicate
existing coverage without adding any.
