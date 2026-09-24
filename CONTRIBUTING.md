# Contributing

Thank you for your interest in contributing to SorobanLabs Analyzer.

## Scope

This project analyzes the impact of replacing a specific deployed
Soroban contract's executable. Please read the "What the analyzer does
not do" section of the [README](README.md) before proposing new
functionality; contributions that turn the project into a generic
security scanner, a protocol compatibility checker, a hosted service, or
an automatic upgrade tool are out of scope.

## Prerequisites

- [rustup](https://rustup.rs/).
- The pinned toolchain declared in `rust-toolchain.toml`. Running
  `rustup show` from the repository root installs it automatically,
  including `rustfmt`, `clippy`, and the `wasm32v1-none` target.

No additional Soroban or Stellar tooling is required for normal
workspace development (building, testing, linting). The `stellar-cli`
toolchain listed in each fixture's README is only needed to reproduce
the checked-in WASM fixture artifacts; it is not part of the standard
development loop.

## Development setup

1. Clone the repository and enter the working directory.
2. Run `rustup show` from the repository root. This installs the exact
   toolchain pinned in `rust-toolchain.toml`, including `rustfmt`,
   `clippy`, and the `wasm32v1-none` target.
3. Build the workspace: `cargo build --workspace`.

## Required checks before submitting a change

```
cargo fmt --all --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
```

CI runs the same three checks (under the job named "format, lint, and
test") and also runs a separate MSRV build check (`cargo +1.84.0 build
--workspace`) to verify the workspace still compiles under the declared
minimum supported Rust version. Both CI jobs must pass before a pull
request can be merged.

## Architecture

The workspace has eight crates under `crates/`:

```
analyzer-core         domain model: errors, findings, confidence,
                      severity, status, upgrade plan
analyzer-executable   executable loading, WASM inspection, interface diff
analyzer-state        state snapshots and compatibility analysis
analyzer-auth         authorization surface extraction and diff
analyzer-rehearsal    controlled upgrade rehearsal (Soroban host backend)
analyzer-evidence     evidence model backing findings
analyzer-report       JSON report and terminal report rendering
analyzer-cli          command-line interface and pipeline orchestration
```

`analyzer-core` defines shared domain types that every other crate
depends on. `analyzer-cli` orchestrates the full analysis pipeline by
sequencing calls into the other crates; it lives here rather than in
`analyzer-core` to avoid a dependency cycle.

The analysis flow follows this path:

1. Load and hash the current and candidate executables
   (`analyzer-executable`).
2. Extract and diff their contract interfaces (`analyzer-executable`).
3. Evaluate state compatibility (`analyzer-state`).
4. Extract and compare authorization surfaces (`analyzer-auth`).
5. Run controlled rehearsal invocations when a rehearsal input is
   supplied (`analyzer-rehearsal`).
6. Attach evidence to each finding (`analyzer-evidence`).
7. Render the canonical JSON or terminal report (`analyzer-report`).

Supporting directories:

- `fixtures/` -- declarative test fixtures (see `fixtures/README.md`)
- `schemas/` -- versioned JSON schemas for the canonical report
- `tests/` -- workspace-level integration tests
- `scripts/` -- development and CI helper scripts
- `docs/` -- design and reference documentation
- `examples/` -- example inputs and outputs

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

Fixture source contracts (under `fixtures/executable-src/`) are not
workspace members and are not built by `cargo build --workspace`. They
have their own toolchain requirements documented in their respective
READMEs. Do not modify checked-in WASM artifacts without reproducing
them from source and recording the exact toolchain used.

## Git workflow

### Branches

- `main` is protected. All changes reach `main` through pull requests.
- Force pushes and direct pushes to `main` are blocked.
- Create a feature branch for your change. There is no enforced naming
  convention, but descriptive names are preferred (for example
  `feat/interface-event-diff` or `fix/rehearsal-timeout`).

### Commits

- Use conventional commit messages (for example
  `feat(executable): add wasm inspection`).
- Keep commits scoped to one logical unit of work.
- Do not create fake or empty commits.
- Do not split one logical change into artificial commits, and do not
  combine unrelated changes merely to reduce commit count.
- Do not introduce dependency churn solely to manufacture commit
  activity.
- Do not force-push or rewrite shared history.

### Staging

Before committing, review what you are about to stage:

1. Run `git status` to confirm which files changed.
2. Stage only the files belonging to the current logical change.
3. Review the staged diff with `git diff --cached`.
4. Run the required checks (`cargo fmt --all --check`, `cargo clippy
   --workspace --all-targets -- -D warnings`, `cargo test --workspace`)
   before committing.
5. After committing, verify the result with `git show --stat`.

### Pull requests

1. Push your branch.
2. Open a pull request targeting `main`.
3. CI runs automatically. Both required checks ("format, lint, and
   test" and "MSRV build (rustc 1.84.0)") must pass.
4. At least one independent approving review is required.
5. Stale approvals are dismissed when new commits are pushed, so
   changes after review may require re-approval.
6. Once CI is green and an approval is obtained, the PR may be merged.

## Security-sensitive areas

The following areas handle untrusted or externally-supplied data and
require particular care in review:

- **Executable/WASM parsing** (`analyzer-executable`): processes
  attacker-supplied WASM binaries.
- **Artifact loading** (`analyzer-executable`): reads files from disk.
- **Authorization analysis** (`analyzer-auth`): extracts authorization
  patterns from WASM; inaccuracies could mislead upgrade decisions.
- **RPC state access** (`analyzer-state`, specifically `rpc.rs`): makes
  outbound network requests to Soroban RPC endpoints.
- **Rehearsal execution** (`analyzer-rehearsal`): runs actual WASM code
  through the `soroban-env-host` backend under controlled limits.
- **Manifest and evidence provenance** (`analyzer-state`,
  `analyzer-evidence`): author-supplied manifests must never be
  represented as independently verified evidence.

For vulnerability reporting, see [SECURITY.md](SECURITY.md).

## When to update documentation

Update documentation alongside code changes when:

- User-visible CLI behavior changes (flags, exit codes, output format).
- Analyzer capabilities change (new analysis stages, new finding rules,
  new confidence or severity values).
- Architecture changes (new crates, changed crate responsibilities,
  changed pipeline flow).
- New limitations or changes to existing limitation disclosures.
- Evidence or provenance semantics change.

Changes that do not affect documented behavior (internal refactors,
performance improvements, test additions) do not require documentation
updates.

## Reporting issues

Please open an issue describing the observed behavior, the expected
behavior, and, where relevant, the executable or fixture that reproduces
it. See [SECURITY.md](SECURITY.md) for how to report security issues
specifically.
