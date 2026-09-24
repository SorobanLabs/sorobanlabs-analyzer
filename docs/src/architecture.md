# Architecture

The workspace has eight crates under `crates/`:

```
analyzer-core         domain model: errors, findings, confidence,
                       severity, status, upgrade plan
analyzer-executable    executable loading, WASM inspection, interface diff
analyzer-state         state snapshots and compatibility analysis
analyzer-auth          authorization surface extraction and diff
analyzer-rehearsal     controlled upgrade rehearsal (Soroban host backend)
analyzer-evidence      evidence model backing findings
analyzer-report        JSON report and terminal report rendering
analyzer-cli           command-line interface and pipeline orchestration
```

`analyzer-core` defines shared domain types that every other crate
depends on. `analyzer-cli` orchestrates the full analysis pipeline by
sequencing calls into the other crates; it lives here rather than in
`analyzer-core` to avoid a dependency cycle, since
`analyzer-executable`/`-state`/`-auth`/`-report` all depend on
`analyzer-core`.

## What is implemented vs. library-only

Every crate above is real, compiled, and tested. But **not everything a
crate provides is reachable from the CLI**:

- `analyzer-state` includes an `rpc` module for read-only Soroban RPC
  state access (`getLedgerEntries`). This module is real and tested at
  the library level, but `analyzer-cli` never calls into it — there is
  no CLI flag that triggers a network request. State compatibility
  analysis as run by `analyze` today uses only the executables and an
  optional local migration manifest, never a live RPC endpoint.
- `analyzer-core::findings::Rule` declares 23 rule identifiers. As of
  this writing, 8 of them (all rehearsal-observation and one
  resource-usage rule) are never actually produced by the current
  pipeline, because the rehearsal backend does not yet capture events,
  state access, or resource usage. See [Limitations](limitations.md).

Architecture documentation in this book distinguishes these cases
explicitly rather than describing library-level code as a user-facing
capability.

## Repository layout

```
crates/      the eight crates above
fixtures/    declarative fixtures used by the test suite
schemas/     versioned JSON schemas for canonical output
audit/       point-in-time audit reports (see each file's own date and
             HEAD; evidence/index.md and audit/claim-traceability.md
             are the current, maintained claim surface)
evidence/    the current claim-to-evidence ledger
examples/    real, runnable examples (see examples/README.md)
docs/        source for this documentation book
tests/       intentionally empty; see tests/README.md for where real
             tests actually live
scripts/     intentionally empty; no helper scripts exist yet
```
