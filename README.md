# SorobanLabs Analyzer

SorobanLabs Analyzer is a Rust-first analysis tool that answers one
specific question:

> What will change if I replace the executable of this deployed Soroban
> contract?

The analyzer determines what can be established from the available
executable, contract interface, state, authorization surface, protocol
context, and controlled execution evidence, and reports each finding with
an explicit confidence level.

## What the analyzer does

- Loads and hashes a current and a candidate contract executable.
- Extracts and normalizes the Soroban contract interface (functions,
  parameters, return types, events, types).
- Diffs the current and candidate interfaces and reports explicit,
  rule-identified findings.
- Represents known contract state and compares state requirements
  between the current and candidate executables.
- Extracts and compares the authorization surface (entrypoint,
  authorization requirement, principal, check).
- Attaches evidence to every meaningful finding.
- Produces a versioned, deterministic canonical JSON report, plus
  Markdown and terminal renderings.

## What the analyzer does not do

- It does not produce a simplistic SAFE/UNSAFE verdict.
- It is not a generic smart contract security scanner or static
  vulnerability scanner.
- It does not duplicate protocol-wide compatibility checking
  responsibilities (that is a separate product's job).
- It does not automatically perform real contract upgrades, submit
  transactions, or execute migrations.
- It does not operate as a hosted service or web dashboard.
- It does not ask for or handle private keys.

## Confidence model

Every meaningful finding uses one of four confidence levels:

- `DETECTED`: the analyzer directly established the condition from
  available evidence.
- `LIKELY`: available evidence strongly indicates the condition, but the
  analyzer cannot establish it with complete certainty.
- `POTENTIAL`: the change could create the condition, but available
  evidence is insufficient to establish the actual impact.
- `NOT_DETERMINABLE`: the analyzer does not have enough information to
  reach a meaningful conclusion.

`NOT_DETERMINABLE` is never treated as a negative finding. Confidence
(how certain the analyzer is) is always reported separately from
severity (how important the condition is if confirmed).

## Analysis status

Top-level analysis status is one of:

- `NO_DETECTED_BLOCKERS`
- `REVIEW_REQUIRED`
- `MIGRATION_REQUIRED`
- `INCONCLUSIVE`
- `ANALYSIS_ERROR`

## Repository layout

```
crates/
  analyzer-core        domain model and orchestration
  analyzer-executable   executable loading, WASM inspection, interface diff
  analyzer-state        state snapshots and compatibility analysis
  analyzer-auth         authorization surface extraction and diff
  analyzer-evidence     evidence model backing findings
  analyzer-report       JSON/Markdown/terminal report rendering
  analyzer-cli          command-line interface (orchestration only)
fixtures/    declarative fixtures used by the test suite
schemas/     versioned JSON schemas for canonical output
examples/    example inputs and outputs
docs/        additional documentation
tests/       workspace-level integration tests
scripts/     development and CI helper scripts
```

## Status

This repository is in early, active development. The crate structure and
toolchain are established; the analysis pipeline, CLI commands, and
report schema are implemented incrementally. Controlled upgrade rehearsal
(running old and candidate executables against representative state and
comparing observable behavior) is a planned major subsystem and is not
yet implemented.

## Development setup

Install the pinned toolchain (this repository's `rust-toolchain.toml`
pins the exact version, including `rustfmt`, `clippy`, and the
`wasm32v1-none` target used to build test fixture contracts):

```
rustup show
```

Build and test:

```
cargo build --workspace
cargo test --workspace
cargo fmt --all --check
cargo clippy --workspace --all-targets -- -D warnings
```

See [CONTRIBUTING.md](CONTRIBUTING.md) for more detail and
[SECURITY.md](SECURITY.md) for the project's security posture.

## License

Licensed under the Apache License, Version 2.0. See [LICENSE](LICENSE).
