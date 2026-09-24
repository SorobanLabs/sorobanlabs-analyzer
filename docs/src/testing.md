# Testing

Run the full workspace test suite from the repository root:

```
cargo test --workspace
```

This runs unit tests, per-crate integration tests (including
`crates/analyzer-cli/tests/`, `crates/analyzer-report/tests/`, and
`crates/analyzer-rehearsal/tests/`), JSON schema validation tests, and
determinism/reproducibility tests — all in one command, all run by CI.

Required checks before submitting a change:

```
cargo fmt --all --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
```

CI runs the same three checks (job "format, lint, and test") plus a
separate MSRV build check (`cargo +1.84.0 build --workspace`, job "MSRV
build (rustc 1.84.0)") to verify the workspace still compiles under the
declared minimum supported Rust version. Both jobs must pass before a
pull request can be merged.

There is no separate top-level `tests/` directory in active use — see
[`tests/README.md`](https://github.com/SorobanLabs/sorobanlabs-analyzer/blob/main/tests/README.md)
for exactly where each category of test actually lives.
