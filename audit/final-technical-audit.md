# Final Technical Audit (2026-09-23 snapshot, superseded)

Date: 2026-09-23
Repository: SorobanLabs/sorobanlabs-analyzer
Branch: main
HEAD audited: `e64c7573ebdd3908efe9dbabc0c2b13c53f5329a`

**This document is a point-in-time snapshot, not the current audit
state.** Despite its title, it is not the final word on this
repository: five of the ten findings it describes below as open
(AUDIT-01, AUDIT-02, AUDIT-03, AUDIT-04, AUDIT-09) were fixed the same
day this audit was written, and a sixth (AUDIT-05) was fixed the
following day. See `finding-matrix.md` for the per-finding resolution
status, and see `evidence/index.md` and `claim-traceability.md` for
the current, actively maintained claim surface; those two documents,
not this one, are what a reviewer should treat as authoritative for
current state.

This audit was performed without changing product implementation. No source file under `crates/` was modified. The only changes made during this audit are this `audit/` directory itself.

See [`finding-matrix.md`](finding-matrix.md) for the itemized finding list and [`claim-traceability.md`](claim-traceability.md) for the claim-to-evidence table. This document summarizes methodology and conclusions.

## Scope and method

Evidence was gathered by: reading source files directly (not relying on prior audit reports where they could be checked against current source), running `cargo build --workspace`, `cargo fmt --all -- --check`, `cargo clippy --workspace --all-targets -- -D warnings`, and `cargo test --workspace` (all currently executed on 2026-09-23, all pass, 220 tests), running the real compiled CLI binary against real fixture WASM for every documented scenario, validating real generated JSON reports against `schemas/analysis-result.schema.json` with Python's `jsonschema` library, querying the live GitHub repository via `gh` (API and CLI), and cross-checking two external technical claims against current official sources (the Stellar `getLedgerEntries` RPC method documentation, and CVE-2026-33056 against `blog.rust-lang.org`).

Where a claim could not be checked from within this environment (chiefly: live Soroban RPC endpoint behavior, since this environment has no network path to a real Soroban RPC node), it is marked UNVERIFIED rather than assumed true or false. This matches the module's own self-disclosure in `analyzer-state::rpc`.

## Summary of what this audit confirmed as genuinely true

- The workspace builds, tests, formats, and lints cleanly under the pinned toolchain (1.98.1) today, and separately builds successfully under the declared MSRV (1.84.0), directly verified by compiling with `cargo +1.84.0 build --workspace` during this audit.
- The prior stabilization pass's core claims hold under direct re-verification: the state-compatibility overclaim (empty manifest + changed hash) is genuinely fixed and tested across all 8 documented branches; the rehearsal backend's documented scope (outcome/return-value only, no events/state/resources) matches its actual code exactly; `NotObservable` is never produced by comparing empty or matching vectors, it is unconditional; the authorization extractor's direct-call-only scope is consistently and honestly stated everywhere it appears in code; SECURITY.md's rehearsal-execution wording is accurate; evidence records are genuinely wired into findings (not empty placeholders) and are genuinely traceable from a finding's evidence id to a record in the report's own `evidence` array; the CLI's `--help` exit-code table matches `exit_code_for`'s actual mapping exactly, confirmed by running all 11 documented CLI scenarios against the real binary; historical AI co-author/session metadata is absent from `main` and present only on the intentionally retained `backup/pre-ai-metadata-cleanup-2026-09-22` branch (both locally and on `origin`); the final forward-fix commit's CI run is `completed`/`success` per the live GitHub Actions API.
- Two external, checkable technical claims embedded in source comments were independently verified against current official sources during this audit: CVE-2026-33056 (the `tar`-crate permission-manipulation vulnerability cited in `rust-toolchain.toml`, confirmed via `blog.rust-lang.org`, fixed in Rust 1.94.1) and the `getLedgerEntries` RPC method's read-only, explicit-keys-required shape (cited in `analyzer-state::rpc`, confirmed via `developers.stellar.org`). The `require_auth`/`require_auth_for_args` absence from `soroban-env-common/env.json` at tag v28.0.2 (cited in `analyzer-auth::extraction`) was also independently re-confirmed against the raw file on GitHub during this audit.

## Summary of what this audit found newly wrong or imprecise

Ten findings, detailed in `finding-matrix.md` (IDs AUDIT-01 through AUDIT-10): none classified Blocker, five classified Important Pre-Submission Fix (interface-finding evidence source imprecision; manifest-evidence losing its "unverified" framing in text; a false "principal" extraction claim in README; a false "MSRV checked in CI" claim; a rehearsal-coverage internal inconsistency omitting resource_usage from `remains_unverified`), four classified Stale/Cosmetic (two CONTRIBUTING.md statements, two placeholder-only directories), one classified Known Limitation (unreachable Rule identifiers), one classified Non-Blocking Backlog (GitHub repository metadata). None of these were present in, or contradict, the prior stabilization report's claims about what it fixed; they are new findings from re-reading the actual current source with fresh eyes, per this audit's explicit charge not to rely on the prior report where it could be checked against the repository.

**Current status as of 2026-09-24 (see `finding-matrix.md` for full detail): AUDIT-01, AUDIT-02, AUDIT-03, AUDIT-04, AUDIT-05, and AUDIT-10 are resolved. AUDIT-06 remains an accepted known limitation requiring no action. AUDIT-07 and AUDIT-08 remain open and are now tracked as GitHub issues #13 and #12 respectively.** None of the ten findings were ever classified Blocker; none changed the analyzer's actual behavior while open.

## What this audit did not attempt

No product code was changed. No dependency was upgraded. No new test infrastructure, state execution engine, resource accounting, or event capture was added. No git history was rewritten (the rewrite audited here was performed and pushed in the prior session, before this audit began). No later-phase submission artifacts (release, documentation site, external reviewer sign-off) were created, since none were requested and creating them was out of scope for an audit.

## Conclusion

This repository, at `e64c757`, builds, tests, and lints cleanly, and its highest-value claims (deterministic output, evidence traceability, honestly-scoped rehearsal, honestly-scoped authorization analysis, no transaction/key/network-mutation capability) hold under direct, current verification. It also contains ten concrete, real defects in documentation and evidence-source precision, none of which change the analyzer's actual behavior and none of which were found to be a blocker to further review. Six of the ten (AUDIT-01, 02, 03, 04, 05, 10) have since been resolved; see the banner at the top of this document and `finding-matrix.md` for current status. No claim in this document should be read as "production ready," "fully complete," "safe," or "fully verified"; those terms are not supported by, and are deliberately absent from, this audit's evidence.
