# Claim-to-Code Traceability

Repository: `SorobanLabs/sorobanlabs-analyzer`
HEAD at time of audit: `064507b3e730194d2a02b103345ff90acb39ff3e`
Date of audit: 2026-09-23

This document connects each high-value claim through:

```
CLAIM -> IMPLEMENTATION -> TEST -> LIVE EVIDENCE -> DOCUMENTATION
```

A missing live evidence item does not make the whole claim false.
Implementation + tests may justify TESTED LOCALLY while live evidence
remains UNVERIFIED.

---

## A. Core Analyzer Capability

| Claim | Implementation | Test | Live Evidence | Documentation | Status |
|---|---|---|---|---|---|
| Semantic executable diffing | `crates/analyzer-executable/src/diff.rs`: fn `diff_interfaces` (L181), fn `diff_by_name` (L251), fn `diff_functions` (L288), fn `diff_function_signature` (L302), fn `diff_events` (L360) | `diff::tests`: 17 tests covering added/removed/changed for all 6 categories (functions, structs, unions, enums, error enums, events) | None | README L19-20: "Diffs the current and candidate interfaces and reports explicit, rule-identified findings." | TESTED LOCALLY |
| WASM structural validation | `crates/analyzer-executable/src/validation.rs`: fn `validate_generic_wasm` (L122), fn `check_soroban_structural_compatibility` (L155), `SorobanStructuralViolation` enum (L50) with 6 variants | `validation::tests`: 10 tests including `malformed_bytes_are_rejected`, `structurally_valid_but_soroban_incompatible_detects_violations` | None | SECURITY.md L9-10: "Rejects malformed artifacts safely, with structured errors" | TESTED LOCALLY |
| Soroban contract environment metadata analysis | `crates/analyzer-executable/src/environment_meta.rs`: fn `parse_environment_metadata` (L132), `EnvironmentInterfaceVersion` struct (L40), `EnvironmentMetaReport` struct (L56) | `environment_meta::tests`: 6 tests including missing section, duplicate sections, conflicting versions | None | None specific | TESTED LOCALLY |
| Contract specification/interface analysis | `crates/analyzer-executable/src/contract_spec.rs`: fn `parse_contract_spec` (L90); `crates/analyzer-executable/src/interface.rs`: fn `normalize_interface` (L220), `NormalizedInterface` (L200) and 16 associated types | `contract_spec::tests`: 5 tests; `interface::tests`: 9 tests including proptest for type normalization | None | README L17-18: "Extracts and normalizes the Soroban contract interface" | TESTED LOCALLY |
| Function/interface change detection | `crates/analyzer-executable/src/diff.rs`: `FunctionChange` enum (L23) with 6 variants (Added, Removed, InputCountChanged, InputOrderChanged, InputChanged, OutputChanged) | `diff::tests`: tests for each `FunctionChange` variant | None | README L19-20 | TESTED LOCALLY |
| Event/type change detection | `crates/analyzer-executable/src/diff.rs`: `EventChange` enum (L127) with 5 variants; `StructChange` (L63), `UnionChange` (L79), `EnumChange` (L95), `ErrorEnumChange` (L111) each with 3 variants | `diff::tests`: tests for struct/union/enum/error_enum/event changes | None | None specific | TESTED LOCALLY |
| State compatibility assessment | `crates/analyzer-state/src/compatibility.rs`: fn `assess_state_compatibility` (L100), `StateCompatibility` enum (L30) with 4 variants | `compatibility::tests`: 10 tests covering identical hash, changed hash, manifest signals, form changes, priority rules | None | README L21: "Represents known contract state and compares state requirements" | TESTED LOCALLY |
| Migration manifest handling | `crates/analyzer-state/src/migration.rs`: `MigrationManifest` struct (L65), fn `parse_json` (L82), fn `to_canonical_json` (L91), fn `content_hash` (L102), fn `declares_migration_function` (L112), fn `declares_schema_change` (L119), fn `to_evidence` (L130) | `migration::tests`: 8 tests including parse, round-trip, hash determinism, evidence labeling | None | None specific | TESTED LOCALLY |
| Author-supplied migration evidence is unverified | `crates/analyzer-state/src/migration.rs`: L5-12 (module doc: "treated as evidence, not proof"), fn `to_evidence` L130-156 (prefixes observation with "UNVERIFIED (author-supplied)") | `migration::tests::evidence_observation_is_explicitly_labeled_unverified` | None | None specific | VERIFIED |
| Authorization surface analysis (direct-call extraction) | `crates/analyzer-auth/src/extraction.rs`: fn `extract_authorization_surface` (L127), `AUTH_IMPORT_NAMES` (L66: `["require_auth", "require_auth_for_args"]`), `CUSTOM_ACCOUNT_CHECK_AUTH_EXPORT` (L95: `"__check_auth"`), `EntrypointAuthorization` (L70), `AuthorizationSurface` (L99) | `extraction::tests`: 8 tests | None | README L23: "Extracts and compares the authorization surface" | TESTED LOCALLY |
| Authorization surface comparison | `crates/analyzer-auth/src/diff.rs`: fn `diff_authorization_surfaces` (L73), `AuthorizationChange` enum (L18) with 7 variants, `AuthorizationDiff` struct (L62) | `diff::tests`: 6 tests covering identical surfaces, added/removed entrypoints, protection changes, module-level changes | None | README L23 | TESTED LOCALLY |

## B. Rehearsal

| Claim | Implementation | Test | Live Evidence | Documentation | Status |
|---|---|---|---|---|---|
| Controlled rehearsal (bounded local execution) | `crates/analyzer-rehearsal/src/host.rs`: fn `rehearse_invocation` (L87), fn `run_invocation` (L133), uses `soroban-env-host` 28.0.2 with `testutils` feature | `host::tests`: 5 tests including `add_invocation_returns_the_correct_sum`, `malformed_candidate_wasm_is_blocked_not_a_process_crash`, `repeated_rehearsal_of_the_same_invocation_is_deterministic` | None | `host.rs` module doc L1-62 | TESTED LOCALLY |
| Rehearsal outcome comparison (trace model) | `crates/analyzer-rehearsal/src/trace.rs`: `RehearsalTrace` (L17), fn `current_observation` (L28), fn `candidate_observation` (L36) | `trace::tests`: 3 tests including `empty_trace_has_no_observations`, `looks_up_observations_by_label`, `trace_serializes_deterministically` | None | None specific | TESTED LOCALLY |
| Rehearsal return-value comparison | `crates/analyzer-rehearsal/src/host.rs` L171-188 (captures return `ScVal`, encodes to hex); `observation.rs` L71 (`return_value_xdr_hex: Option<String>`) | `host::tests::add_invocation_returns_the_correct_sum` (verifies correct return value) | None | None specific | TESTED LOCALLY |
| Rehearsal events: NOT OBSERVABLE | `crates/analyzer-rehearsal/src/host.rs` L119 (hardcoded `events: vec![]`); module doc L57-58 | No test asserts event content (only confirms hardcoded empty) | None | `host.rs` module doc L57-58 | KNOWN LIMITATION |
| Rehearsal state reads: NOT OBSERVABLE | `crates/analyzer-rehearsal/src/host.rs` L120 (hardcoded `state_reads: vec![]`) | No test asserts state read content | None | `host.rs` module doc L58 | KNOWN LIMITATION |
| Rehearsal state writes: NOT OBSERVABLE | `crates/analyzer-rehearsal/src/host.rs` L121 (hardcoded `state_writes: vec![]`) | No test asserts state write content | None | `host.rs` module doc L58 | KNOWN LIMITATION |
| Rehearsal resource usage: NOT OBSERVABLE | `crates/analyzer-rehearsal/src/host.rs` L122 (hardcoded `ResourceUsage::default()`); `observation.rs` L57-58 (both fields `None`) | `observation::tests::resource_usage_defaults_to_unreported` | None | `host.rs` module doc L58 | KNOWN LIMITATION |
| Bounded rehearsal execution | `crates/analyzer-rehearsal/src/host.rs` L78-81 (default limits), L145 (`host.test_budget(cpu_limit, memory_limit)`), L99-101 (panic isolation) | `host::tests::malformed_candidate_wasm_is_blocked_not_a_process_crash` | None | SECURITY.md L8: "Does not perform arbitrary host execution" | TESTED LOCALLY |

## C. Evidence and Determinism

| Claim | Implementation | Test | Live Evidence | Documentation | Status |
|---|---|---|---|---|---|
| Deterministic EvidenceId | `crates/analyzer-evidence/src/reference.rs`: fn `compute_evidence_id` (L133), uses SHA-256 over length-prefixed fields | `reference::tests`: 6 tests for identity, distinctness, format, JSON round-trip | None | None specific | TESTED LOCALLY |
| Report evidence resolution | `crates/analyzer-report/src/canonical.rs`: `ReportFinding` L35 (`evidence: Vec<String>`), fn `from_finding` L51; `schemas/analysis-result.schema.json` L138-144 (hex64 pattern) | `schema_validation.rs`: 6 tests; `canonical::tests`: 2 tests | None | None specific | TESTED LOCALLY |
| Canonical serialization | `crates/analyzer-report/src/json.rs`: fn `to_canonical_json` (L17), fn `from_canonical_json` (L26) | `json::tests`: 4 tests for determinism, trailing newline, round-trip, malformed input | None | README L26-27: "versioned, deterministic canonical JSON report" | TESTED LOCALLY |
| Deterministic FindingId | `crates/analyzer-core/src/findings/mod.rs`: `FindingId` (L205), computed as SHA-256 over category, rule, subject only (excludes human-readable text) | `findings::tests`: `id_is_deterministic_for_same_category_rule_subject`, `id_differs_for_different_subject`, `id_does_not_depend_on_human_readable_text` | None | None specific | TESTED LOCALLY |

## D. Security and Trust Boundary

| Claim | Implementation | Test | Live Evidence | Documentation | Status |
|---|---|---|---|---|---|
| No private-key custody | Source search confirms zero matches for `SecretKey`, `SigningKey`, `private_key`, `seed_phrase`, `secret` (in key-handling context) across all `crates/` | N/A (absence claim) | N/A | SECURITY.md L14-15; README L39 | VERIFIED |
| No transaction submission | Source search confirms zero matches for `submit_transaction`, `send_transaction`, `TransactionEnvelope`, `TransactionV1Envelope` across all `crates/` | N/A (absence claim) | N/A | SECURITY.md L12-13; README L36-37 | VERIFIED |
| No network mutation | `crates/analyzer-state/src/rpc.rs`: only `getLedgerEntries` (read-only); `crates/analyzer-cli/`: no network calls | `rpc::tests` (mock transport only) | None | SECURITY.md L12-13 | VERIFIED |
| Read-only RPC | `crates/analyzer-state/src/rpc.rs`: fn `load` (L135), sends only `getLedgerEntries` (L157), receives only ledger entry data | `rpc::tests`: 4 tests with mock transport | None (live endpoint not tested) | SECURITY.md L13 | TESTED LOCALLY |
| No uncontrolled filesystem writes | No `std::fs::write`, `File::create`, `OpenOptions::new().write(true)` in production code paths | N/A (absence claim) | N/A | SECURITY.md L11 | VERIFIED |
| Bounded rehearsal | `crates/analyzer-rehearsal/src/host.rs` L78-81 (CPU/memory defaults), L145 (budget enforcement), L99-101 (panic isolation) | `host::tests` (5 tests, including panic isolation test) | None | SECURITY.md L8 | TESTED LOCALLY |

## E. CLI Behavior

| Claim | Implementation | Test | Live Evidence | Documentation | Status |
|---|---|---|---|---|---|
| CLI command syntax | `crates/analyzer-cli/src/main.rs`: `Cli` struct with `#[derive(Parser)]` (L13); no subcommands wired | N/A | None | README L108-113 (build/test commands) | TESTED LOCALLY |
| Exit codes | `crates/analyzer-cli/src/main.rs` L17-20: prints message, exits 0; Clap handles `--help` (0), invalid args (2) | No explicit exit code tests | None | None specific | LOGICALLY COVERED |
| Terminal/JSON output | `crates/analyzer-report/src/json.rs`: `to_canonical_json` produces JSON; `crates/analyzer-cli/src/main.rs`: currently prints plain text only | `json::tests` (4 tests) | None | README L26-27 | TESTED LOCALLY |
| Malformed input handling | `crates/analyzer-executable/src/validation.rs`, `crates/analyzer-auth/src/extraction.rs`, `crates/analyzer-state/src/rpc.rs`, `crates/analyzer-rehearsal/src/host.rs`: all return structured errors; workspace lint `unwrap_used = "deny"` | Multiple tests across 4 crates confirming structured error returns | None | SECURITY.md L9-10 | TESTED LOCALLY |

## F. Repository Governance

| Claim | Implementation | Test | Live Evidence | Documentation | Status |
|---|---|---|---|---|---|
| CI format/lint/test | `.github/workflows/ci.yml`: job `check` (name: "format, lint, and test") running `cargo fmt`, `cargo clippy`, `cargo test` | N/A (CI infrastructure) | GitHub Actions runs on push/PR | README L109-113 | VERIFIED |
| MSRV declaration | `Cargo.toml` L17: `rust-version = "1.84.0"` | N/A | N/A | `rust-toolchain.toml` L2 (comment references MSRV) | VERIFIED |
| MSRV CI validation | No CI job or step builds against rustc 1.84.0 | N/A | None | `rust-toolchain.toml` L10 claims "verified against rustc 1.84.0 directly (see MSRV check in CI)" but no such CI step exists. GitHub ruleset requires a check named "MSRV build (rustc 1.84.0)" that no workflow job produces. | UNVERIFIED |
| Protected main branch (ruleset) | GitHub repository ruleset named "main" targeting `refs/heads/main` | N/A | GitHub configuration | None specific | VERIFIED |
| Required PR approval | Ruleset "main": 1 required approving review, dismiss stale approvals | N/A | GitHub configuration | None specific | VERIFIED |
| No bypass actors | Ruleset "main": bypass list empty, current user bypass = never | N/A | GitHub configuration | None specific | VERIFIED |

## G. Explicit Limitations

| Claim | Implementation | Test | Live Evidence | Documentation | Status |
|---|---|---|---|---|---|
| No full storage enumeration | `crates/analyzer-state/src/rpc.rs` L21-26 (module doc: Soroban RPC does not support wildcard enumeration); L136-147 (only instance + specified keys queried) | N/A | N/A | None specific | KNOWN LIMITATION |
| No helper/transitive authorization tracing | `crates/analyzer-auth/src/extraction.rs` L42-49 (module doc: "Only direct calls... A call that reaches `require_auth` transitively, through a helper function, is not detected") | N/A | N/A | None specific | KNOWN LIMITATION |
| No principal inference | `crates/analyzer-auth/src/extraction.rs` L50-52 (module doc: "This module never infers a principal") | N/A | N/A | None specific | KNOWN LIMITATION |
| Rehearsal events/state/resource: NOT OBSERVABLE | `crates/analyzer-rehearsal/src/host.rs` L57-62, L119-122 (hardcoded empty/default) | N/A | N/A | `host.rs` module doc | KNOWN LIMITATION |
| Unverified live RPC | No live endpoint test in test suite or CI | N/A | None | None | UNVERIFIED |
| Rehearsal uses testutils path, not production e2e_invoke | `crates/analyzer-rehearsal/src/host.rs` L26-37 (module doc explaining why testutils is used) | N/A | N/A | `host.rs` module doc | KNOWN LIMITATION |

---

## README Cross-Check

| README Statement | Traceability | Finding |
|---|---|---|
| "Extracts and compares the authorization surface (entrypoint, authorization requirement, principal, check)" (L23-24) | Implementation extracts entrypoints and direct auth calls. **Principal is NOT inferred** (`extraction.rs` L50-52). | **DISCREPANCY**: The word "principal" in this README sentence implies the analyzer extracts principal information. The implementation explicitly does not infer principals. This overclaim appears to persist from before the previous audit correction. |
| "Controlled upgrade rehearsal... is a planned major subsystem and is not yet implemented" (L92-94) | `crates/analyzer-rehearsal/src/host.rs` implements bounded rehearsal with `soroban-env-host`. 5 tests pass. Rehearsal is partially implemented (execution + return value observable; events/state/resources not observable). | **DISCREPANCY**: README Status section says rehearsal "is not yet implemented" but the rehearsal module exists with working host execution. The statement is outdated. |
| "Produces a versioned, deterministic canonical JSON report, plus Markdown and terminal renderings" (L26-27) | JSON report: implemented (`analyzer-report`). Markdown rendering: **not found** in crate source. Terminal rendering: **not found** in crate source. | **DISCREPANCY**: README claims Markdown and terminal renderings exist but only canonical JSON is implemented. |
| All other README claims | Traceable to implementation and tests | No discrepancy |

## SECURITY.md Cross-Check

| SECURITY.md Statement | Traceability | Finding |
|---|---|---|
| "Does not perform arbitrary host execution of analyzed WASM" (L8) | `crates/analyzer-rehearsal/src/host.rs` does execute WASM through the official Soroban host, but it is bounded (CPU/memory limits), isolated (panic hook suppression), and controlled (specific function invocations with specified arguments). The execution is not "arbitrary" in the sense of unrestricted execution. | **BORDERLINE**: The statement is defensible because rehearsal execution is bounded and controlled, not arbitrary. However, WASM IS executed. An external reviewer might reasonably expect this nuance to be more explicit. No correction required, but a note that the analyzer does perform bounded, controlled contract invocation through the official Soroban host would be more precise. |
| "Rejects malformed artifacts safely" (L9-10) | Confirmed: malformed input tests exist in 4 crates; `unwrap_used = "deny"` lint | No discrepancy |
| "Performs no uncontrolled filesystem writes" (L11) | Confirmed: no production write operations found | No discrepancy |
| "Never automatically submits a transaction" (L12) | Confirmed: no transaction submission code | No discrepancy |
| "Any Stellar RPC backend used by the analyzer is read-only" (L13) | Confirmed: only `getLedgerEntries` method used | No discrepancy |
| "Does not request, accept, or handle private keys" (L14-15) | Confirmed: no private key types or operations | No discrepancy |

---

## Claim Strength Review

For each claim, I asked:

1. Is it directly established? (VERIFIED claims above)
2. Is it only tested locally? (TESTED LOCALLY claims above)
3. Is it only logically covered? (CLI exit codes: implementation exists but no explicit test)
4. Is there live verification? (CI format/lint/test: yes, through GitHub Actions; all others: no)
5. Is it actually a limitation? (KNOWN LIMITATION claims above)
6. Does wording imply more than implementation proves? (README discrepancies noted above)

No claim in this document has a status stronger than its evidence supports.
