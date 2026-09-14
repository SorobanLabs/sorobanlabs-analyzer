# Contributing

Thank you for your interest in contributing to SorobanLabs Analyzer.

## Scope

This project analyzes the impact of replacing a specific deployed
Soroban contract's executable. Please read the "What the analyzer does
not do" section of the [README](README.md) before proposing new
functionality; contributions that turn the project into a generic
security scanner, a protocol compatibility checker, a hosted service, or
an automatic upgrade tool are out of scope.

## Development setup

1. Install [rustup](https://rustup.rs/) if you do not already have it.
2. From the repository root, run `rustup show`. This installs the exact
   toolchain pinned in `rust-toolchain.toml`, including `rustfmt`,
   `clippy`, and the `wasm32v1-none` target.
3. Build the workspace: `cargo build --workspace`.

## Required checks before submitting a change

```
cargo fmt --all --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
```

CI runs the same checks, plus JSON schema validation and determinism
tests once those subsystems exist.

## Coding guidelines

- Production code must not use `unwrap()` or `expect()`. Use typed
  errors and explicit `Result` handling. This is enforced by a
  workspace-level clippy lint.
- Prefer the smallest correct abstraction. Do not add traits,
  configuration options, or crates that are not required by a concrete,
  current implementation need.
- The analyzer must be deterministic: given identical inputs it must
  produce the same canonical result. Sort collections before
  serialization where ordering matters, and never include timestamps,
  random identifiers, or machine-specific paths in canonical output.
- Every meaningful finding must carry an explicit confidence level
  (`DETECTED`, `LIKELY`, `POTENTIAL`, `NOT_DETERMINABLE`) and reference
  the evidence that supports it. Do not produce opaque findings.
- Do not silently downgrade an unsupported or undetermined case to a
  "compatible" or "safe" result.
- Do not use an em dash in documentation, code comments, commit
  messages, or generated reports unless it is genuinely required by the
  grammar; prefer commas, colons, parentheses, or separate sentences.

## Fixtures

Prefer real, minimal Soroban WASM fixtures over synthetic byte arrays
when a test needs an executable artifact. Each fixture should carry a
manifest describing its kind, description, and expected findings rather
than hiding expected behavior inside test code.

## Git workflow

- Use conventional commit messages (for example
  `feat(executable): add wasm inspection`).
- Keep commits scoped to one logical unit of work.
- Do not force-push or rewrite shared history.

## Reporting issues

Please open an issue describing the observed behavior, the expected
behavior, and, where relevant, the executable or fixture that reproduces
it. See [SECURITY.md](SECURITY.md) for how to report security issues
specifically.
