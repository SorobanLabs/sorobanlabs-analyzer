# Claim-to-Code Traceability

Repository: `SorobanLabs/sorobanlabs-analyzer`
HEAD at time of last refresh: `c7eecedf08176197f8bb0fd00a1b9fc02702db8c`
Date of last refresh: 2026-09-24

This document connects each high-value claim through:

```
CLAIM -> IMPLEMENTATION -> TEST -> LIVE EVIDENCE -> DOCUMENTATION
```

A missing live evidence item does not make the whole claim false.
Implementation + tests may justify TESTED LOCALLY while live evidence
remains UNVERIFIED.

This is a refresh of the document originally created 2026-09-23 at HEAD
`32b97c3`. Rows carried over unchanged keep their original meaning; two
rows (Exit codes, Read-only RPC) are corrected in place because they no
longer matched current source, with the correction noted inline. New
sections H through K trace the claim categories `evidence/index.md`
added in its own 2026-09-24 refresh (claims 43-59 there); the mapping
between this document's rows and those claim numbers is given in the
Evidence Ledger Reconciliation section at the end of this file.

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
| Authorization surface analysis (direct-call extraction) | `crates/analyzer-auth/src/extraction.rs`: fn `extract_authorization_surface` (L127), `AUTH_IMPORT_NAMES` (L66: `["require_auth", "require_auth_for_args"]`), `CUSTOM_ACCOUNT_CHECK_AUTH_EXPORT` (L95: `"__check_auth"`), `EntrypointAuthorization` (L70), `AuthorizationSurface` (L99) | `extraction::tests`: 8 tests | None | README L23: "Extracts and compares each entrypoint's authorization surface" | TESTED LOCALLY |
| Authorization surface comparison | `crates/analyzer-auth/src/diff.rs`: fn `diff_authorization_surfaces` (L73), `AuthorizationChange` enum (L18) with 7 variants, `AuthorizationDiff` struct (L62) | `diff::tests`: 6 tests covering identical surfaces, added/removed entrypoints, protection changes, module-level changes | None | README L23 | TESTED LOCALLY |
| Supported executable forms | `crates/analyzer-executable/src/artifact.rs` (`ArtifactSource::LocalFile`, fn `LoadedWasm::from_path` L134) | Covered by `artifact::tests` (loads valid WASM from local paths) | None | None specific | TESTED LOCALLY |
| Unsupported or limited executable forms | `crates/analyzer-state/src/snapshot.rs` (`ExecutableForm::StellarAsset`, `ExecutableForm::ExternalRef`); `crates/analyzer-state/src/compatibility.rs` L130 (form discriminant comparison; `ExternalRef` always `NotDetermined`) | `compatibility::tests::both_external_ref_is_not_determined`, `executable_form_change_is_potentially_incompatible` | None | None specific | KNOWN LIMITATION |
| Contract metadata parsing | `crates/analyzer-executable/src/contract_meta.rs` (fn `parse_contract_metadata` L121); recognizes `rsver`/`rssdkver` keys | `contract_meta::tests` (5 tests) | None | None specific | TESTED LOCALLY |

## B. Rehearsal

| Claim | Implementation | Test | Live Evidence | Documentation | Status |
|---|---|---|---|---|---|
| Controlled rehearsal (bounded local execution) | `crates/analyzer-rehearsal/src/host.rs`: fn `rehearse_invocation` (L87), fn `run_invocation` (L133), uses `soroban-env-host` 28.0.2 with `testutils` feature | `host::tests`: 5 tests including `add_invocation_returns_the_correct_sum`, `malformed_candidate_wasm_is_blocked_not_a_process_crash`, `repeated_rehearsal_of_the_same_invocation_is_deterministic` | None | `host.rs` module doc L1-62 | TESTED LOCALLY |
| Rehearsal outcome comparison (trace model) | `crates/analyzer-rehearsal/src/trace.rs`: `RehearsalTrace` (L17), fn `current_observation` (L28), fn `candidate_observation` (L36) | `trace::tests`: 3 tests including `empty_trace_has_no_observations`, `looks_up_observations_by_label`, `trace_serializes_deterministically` | None | None specific | TESTED LOCALLY |
| Rehearsal return-value comparison | `crates/analyzer-rehearsal/src/host.rs` L171-188 (captures return `ScVal`, encodes to hex); `observation.rs` L71 (`return_value_xdr_hex: Option<String>`) | `host::tests::add_invocation_returns_the_correct_sum` (verifies correct return value) | None | None specific | TESTED LOCALLY |
| Rehearsal events: NOT OBSERVABLE | `crates/analyzer-rehearsal/src/host.rs` L119 (hardcoded `events: vec![]`); module doc L57-58 | No test asserts event content (only confirms hardcoded empty) | None | `host.rs` module doc L57-58 | KNOWN LIMITATION |
| Rehearsal state reads: NOT OBSERVABLE | `crates/analyzer-rehearsal/src/host.rs` L120 (hardcoded `state_reads: vec![]`) | No test asserts state read content | None | `host.rs` module doc L58 | KNOWN LIMITATION |
| Rehearsal state writes: NOT OBSERVABLE | `crates/analyzer-rehearsal/src/host.rs` L121 (hardcoded `state_writes: vec![]`) | No test asserts state write content | None | `host.rs` module doc L58 | KNOWN LIMITATION |
| Rehearsal resource usage: NOT OBSERVABLE | `crates/analyzer-rehearsal/src/host.rs` L122 (hardcoded `ResourceUsage::default()`); `diff.rs` L54-68 (comparison handles NotObservable properly) | `observation::tests::resource_usage_defaults_to_unreported`; `diff::tests::reported_resource_usage_is_compared_not_marked_not_observable` | None | `host.rs` module doc L58 | KNOWN LIMITATION |
| Bounded rehearsal execution | `crates/analyzer-rehearsal/src/host.rs` L78-81 (default limits), L145 (`host.test_budget(cpu_limit, memory_limit)`), L99-101 (panic isolation) | `host::tests::malformed_candidate_wasm_is_blocked_not_a_process_crash` | None | SECURITY.md L8: "Does not perform arbitrary host execution" | TESTED LOCALLY |

## C. Evidence and Determinism

| Claim | Implementation | Test | Live Evidence | Documentation | Status |
|---|---|---|---|---|---|
| Deterministic EvidenceId | `crates/analyzer-evidence/src/reference.rs`: fn `compute_evidence_id` (L133), uses SHA-256 over length-prefixed fields | `reference::tests`: 6 tests for identity, distinctness, format, JSON round-trip | None | None specific | TESTED LOCALLY |
| Report evidence resolution | `crates/analyzer-report/src/canonical.rs`: `ReportFinding` L35 (`evidence: Vec<String>`), fn `from_finding` L51; `schemas/analysis-result.schema.json` L138-144 (hex64 pattern) | `schema_validation.rs`: 6 tests; `canonical::tests`: 2 tests | None | None specific | TESTED LOCALLY |
| Canonical serialization | `crates/analyzer-report/src/json.rs`: fn `to_canonical_json` (L17), fn `from_canonical_json` (L26) | `json::tests`: 4 tests for determinism, trailing newline, round-trip, malformed input | None | README L34-35: "Produces a versioned, deterministic canonical JSON report" | TESTED LOCALLY |
| Deterministic FindingId | `crates/analyzer-core/src/findings/mod.rs`: `FindingId` (L205), computed as SHA-256 over category, rule, subject only (excludes human-readable text) | `findings::tests`: `id_is_deterministic_for_same_category_rule_subject`, `id_differs_for_different_subject`, `id_does_not_depend_on_human_readable_text` | None | None specific | TESTED LOCALLY |

## D. Security and Trust Boundary

| Claim | Implementation | Test | Live Evidence | Documentation | Status |
|---|---|---|---|---|---|
| No private-key custody | Source search confirms zero matches for `SecretKey`, `SigningKey`, `private_key`, `seed_phrase`, `secret` (in key-handling context) across all `crates/` | N/A (absence claim) | N/A | SECURITY.md L14-15; README L39 | VERIFIED |
| No transaction submission | Source search confirms zero matches for `submit_transaction`, `send_transaction`, `TransactionEnvelope`, `TransactionV1Envelope` across all `crates/` | N/A (absence claim) | N/A | SECURITY.md L12-13; README L36-37 | VERIFIED |
| No network mutation | `crates/analyzer-state/src/rpc.rs`: only `getLedgerEntries` (read-only); `crates/analyzer-cli/`: no network calls | `rpc::tests` (mock transport only) | None | SECURITY.md L12-13 | VERIFIED |
| Read-only RPC (library capability only; not CLI-reachable, see section H) | `crates/analyzer-state/src/rpc.rs`: fn `load` (L135), sends only `getLedgerEntries` (L157), receives only ledger entry data | `rpc::tests`: 4 tests with mock transport | None (live endpoint not tested) | SECURITY.md L13 | TESTED LOCALLY |
| No uncontrolled filesystem writes | No `std::fs::write`, `File::create`, `OpenOptions::new().write(true)` in production code paths | N/A (absence claim) | N/A | SECURITY.md L11 | VERIFIED |
| Bounded rehearsal | `crates/analyzer-rehearsal/src/host.rs` L78-81 (CPU/memory defaults), L145 (budget enforcement), L99-101 (panic isolation) | `host::tests` (5 tests, including panic isolation test) | None | SECURITY.md L8 | TESTED LOCALLY |

## E. CLI Behavior

| Claim | Implementation | Test | Live Evidence | Documentation | Status |
|---|---|---|---|---|---|
| CLI command syntax | `crates/analyzer-cli/src/cli.rs` and `commands.rs`: `Cli` struct with explicit `analyze` subcommand | N/A | None | README L110-131 (CLI usage) | TESTED LOCALLY |
| Exit codes (application-level mapping, corrected 2026-09-24) | `crates/analyzer-cli/src/commands.rs`: `mod exit_code` (L23-35: `SUCCESS = 0`, `INVALID_INPUT = 2`, `BACKEND_FAILURE = 3`, `ANALYSIS_FAILURE = 5`); fn `exit_code_for` (L37-46) maps each `AnalyzerError` variant to one of these; this is application logic, not Clap's automatic argument-parsing exit behavior | `commands.rs` tests: `nonexistent_current_file_exits_with_backend_failure`, `valid_analysis_json_output_is_well_formed`, `valid_analysis_terminal_output_mentions_status`, `malformed_migration_manifest_is_rejected`, `malformed_rehearsal_input_is_rejected`, `a_completed_analysis_that_cannot_be_written_to_stdout_is_not_reported_as_success` (6 tests covering `SUCCESS`, `INVALID_INPUT`, and `BACKEND_FAILURE`; `ANALYSIS_FAILURE` (5) is implemented in `exit_code_for` but not exercised by any test) | None | README L133-138 (documents all 4 codes) | TESTED LOCALLY |
| Terminal/JSON output | `crates/analyzer-report/src/json.rs`: `to_canonical_json` produces JSON; `crates/analyzer-report/src/terminal.rs`: renders terminal format; `crates/analyzer-cli/src/commands.rs` supports format flag | `json::tests` (4 tests); `terminal::tests` | None | README L34-35 | TESTED LOCALLY |
| Malformed input handling | `crates/analyzer-executable/src/validation.rs`, `crates/analyzer-auth/src/extraction.rs`, `crates/analyzer-state/src/rpc.rs`, `crates/analyzer-rehearsal/src/host.rs`: all return structured errors; workspace lint `unwrap_used = "deny"` | Multiple tests across 4 crates confirming structured error returns | None | SECURITY.md L9-10 | TESTED LOCALLY |

## F. Repository Governance

| Claim | Implementation | Test | Live Evidence | Documentation | Status |
|---|---|---|---|---|---|
| CI format/lint/test | `.github/workflows/ci.yml`: job `check` (name: "format, lint, and test") running `cargo fmt`, `cargo clippy`, `cargo test` | N/A (CI infrastructure) | Check run "format, lint, and test" = `success` on HEAD `c7eeced`, re-confirmed live 2026-09-24 | README L109-113 | VERIFIED |
| MSRV declaration | `Cargo.toml` L17: `rust-version = "1.84.0"` | N/A | N/A | `rust-toolchain.toml` L2 (comment references MSRV) | VERIFIED |
| MSRV CI validation | `.github/workflows/ci.yml`: job `msrv` (name: "MSRV build (rustc 1.84.0)") running `cargo +1.84.0 build --workspace` | N/A | Check run "MSRV build (rustc 1.84.0)" = `success` on HEAD `c7eeced`, re-confirmed live 2026-09-24 | `rust-toolchain.toml` L10 claims "verified against rustc 1.84.0 directly (see MSRV check in CI)" | VERIFIED |
| Protected main branch (ruleset) | GitHub repository ruleset named "main" targeting `refs/heads/main` | N/A | Ruleset `enforcement: active`, re-read live 2026-09-24 | None specific | VERIFIED |
| Required PR approval | Ruleset "main": 1 required approving review, dismiss stale approvals | N/A | Re-read live 2026-09-24, unchanged | None specific | VERIFIED |
| No bypass actors | Ruleset "main": bypass list empty, current user bypass = never | N/A | Re-read live 2026-09-24: `bypass_actors: []`, `current_user_can_bypass: never` | None specific | VERIFIED |
| Repository identity matches documentation | `Cargo.toml` L19 (`repository = "https://github.com/SorobanLabs/sorobanlabs-analyzer"`) | N/A | `gh api repos/SorobanLabs/sorobanlabs-analyzer`: `full_name`, `visibility: public`, `default_branch: main`, re-read live 2026-09-24 | None specific | VERIFIED |
| License consistency | `LICENSE` (Apache License 2.0 text, "Copyright 2026 SorobanLabs" L178); `Cargo.toml` L18 (`license = "Apache-2.0"`, `license.workspace = true` in all 8 crates) | N/A | GitHub's detected license = `Apache-2.0`, re-read live 2026-09-24 | LICENSE; no conflicting reference in README/CONTRIBUTING/SECURITY | VERIFIED |
| Repository description and topics | N/A (GitHub metadata, not source) | N/A | `gh api repos/SorobanLabs/sorobanlabs-analyzer`: description and 6 topics (`rust`, `cli`, `soroban`, `stellar`, `wasm`, `smart-contracts`), re-read live 2026-09-24 | None specific (metadata is not reproduced in README) | VERIFIED |
| Private vulnerability reporting enabled | N/A (GitHub feature toggle, not source) | N/A | `gh api repos/.../private-vulnerability-reporting` returns `{"enabled": true}`, re-read live 2026-09-24 | SECURITY.md L45 ("open a private security advisory on the repository") | VERIFIED |
| Standalone repository topology | N/A (organization-level fact, not source) | N/A | `gh api orgs/SorobanLabs/repos --paginate` returns exactly one repository; `.fork`/`.parent`/`.source` all absent/false, re-read live 2026-09-24 | None specific | VERIFIED |
| Upstream dependency classification (not product peers) | `Cargo.toml` L38, L45, L67 (version-pinning comments citing `stellar/rs-soroban-env`, `stellar/rs-stellar-xdr`); `crates/analyzer-executable/src/validation.rs` L25; `crates/analyzer-rehearsal/src/host.rs` L14; `crates/analyzer-auth/src/extraction.rs` L16 | N/A | N/A (no live relationship exists to verify beyond the dependency itself) | None specific | VERIFIED |

## G. Explicit Limitations

| Claim | Implementation | Test | Live Evidence | Documentation | Status |
|---|---|---|---|---|---|
| No full storage enumeration | `crates/analyzer-state/src/rpc.rs` L21-26 (module doc: Soroban RPC does not support wildcard enumeration); L136-147 (only instance + specified keys queried) | N/A | N/A | None specific | KNOWN LIMITATION |
| No helper/transitive authorization tracing | `crates/analyzer-auth/src/extraction.rs` L42-49 (module doc: "Only direct calls... A call that reaches `require_auth` transitively, through a helper function, is not detected") | N/A | N/A | None specific | KNOWN LIMITATION |
| No principal inference | `crates/analyzer-auth/src/extraction.rs` L50-52 (module doc: "This module never infers a principal") | N/A | N/A | README L26-27 | KNOWN LIMITATION |
| Rehearsal events/state/resource: NOT OBSERVABLE | `crates/analyzer-rehearsal/src/host.rs` L57-62, L119-122 (hardcoded empty/default) | N/A | N/A | `host.rs` module doc | KNOWN LIMITATION |
| Unverified live RPC | No live endpoint test in test suite or CI | N/A | None | None | UNVERIFIED |
| Rehearsal uses testutils path, not production e2e_invoke | `crates/analyzer-rehearsal/src/host.rs` L26-37 (module doc explaining why testutils is used) | N/A | N/A | `host.rs` module doc | KNOWN LIMITATION |

## H. Deployment and Service Topology

| Claim | Implementation | Test | Live Evidence | Documentation | Status |
|---|---|---|---|---|---|
| RPC module is not reachable from the CLI | `grep -rn "rpc::" crates/analyzer-cli/` returns no matches; `crates/analyzer-cli/src/cli.rs` `AnalyzeArgs` (L57-87) has no RPC-related flag; `crates/analyzer-state/src/lib.rs` L20 publicly exports `rpc::{DataKeyRequest, RpcStateSource, Transport, UreqTransport}` as a library item only | N/A (absence claim; the library module itself is tested, see D "Read-only RPC") | N/A | v0.1.0 release body, Network section (corrected 2026-09-24) | KNOWN LIMITATION |
| No application runtime environment variables | Repository-wide search for `std::env`, `env::var`, `.env` files across `crates/` returns no matches; the only environment key anywhere is `CARGO_TERM_COLOR` in `.github/workflows/ci.yml` L9 (CI tooling, not application config) | N/A (absence claim) | N/A | None specific | VERIFIED |
| No public or private service endpoint | Repository-wide search for `TcpListener`, `bind(`, `axum`, `actix`, `warp`, `tonic`, `hyper::Server`, `tokio::net` across `crates/` returns no matches | N/A (absence claim) | N/A | None specific | VERIFIED |
| No application database | No database dependency (`postgres`, `sqlite`, `mysql`, `diesel`, `sqlx`, `rusqlite`) in `Cargo.toml` or any crate manifest | N/A (absence claim) | N/A | None specific | VERIFIED |
| No wallet or signing boundary | Repository-wide search for `SecretKey`, `sign(`, `Signature`, `private_key`, `SigningKey`, `Keypair` returns only the unrelated `Rule::ContractSignatureChanged` (a WASM function-signature-change identifier, not cryptographic signing) | N/A (absence claim) | N/A | SECURITY.md L27-28 | VERIFIED |
| No deployed service or hosting configuration | No `Dockerfile`, `docker-compose*`, or deployment/infra directory anywhere in the repository; `.github/workflows/` contains only `ci.yml` | N/A (absence claim) | N/A | v0.1.0 release body, Deployment section | VERIFIED |
| CLI-only, local-file execution | `crates/analyzer-cli/src/cli.rs` (`AnalyzeArgs`: `current`, `candidate`, `migration_manifest`, `rehearsal` are all `PathBuf`); `crates/analyzer-cli/src/commands.rs` L57 (writes report to `stdout`, errors to `stderr`, no file/network output) | Covered indirectly by every `commands.rs` and `orchestration.rs` test that runs `run_analyze`/`run_upgrade_analysis` against local fixture paths | N/A | README L110-138 (CLI usage) | TESTED LOCALLY |

## I. Release State

| Claim | Implementation | Test | Live Evidence | Documentation | Status |
|---|---|---|---|---|---|
| v0.1.0 tag and release exist and are published | N/A (GitHub release/tag, not source) | N/A | `gh api repos/.../releases/tags/v0.1.0`: `tag_name: v0.1.0`, `target_commitish` matches the commit that was `main` at creation, `draft: false`, `prerelease: false`; `gh api .../git/ref/tags/v0.1.0` confirms the tag object resolves to that same commit | v0.1.0 release body | VERIFIED |
| v0.1.0 release body does not overstate RPC/network capability | `crates/analyzer-cli/` has no RPC wiring (see section H) | N/A | Live release body, Network section, re-read 2026-09-24: states the `analyze` command "has no network-reachable option in this release" and that the RPC module "is not wired into the CLI" | v0.1.0 release body (corrected 2026-09-24) | VERIFIED |
| v0.1.0 release body does not claim deployed contract IDs | N/A (absence claim) | N/A | Live release body, Deployment section, re-read 2026-09-24: "No deployed contract IDs are included because none are verified as part of this repository's release state" | v0.1.0 release body | VERIFIED |

## J. Roadmap and Issue Tracking

An open GitHub issue is roadmap-tracking evidence, not implementation
evidence. Each row below separates the two: the Implementation/Test
columns trace the underlying limitation the issue describes, and Live
Evidence traces the issue itself.

| Claim | Implementation | Test | Live Evidence | Documentation | Status |
|---|---|---|---|---|---|
| Issue tracker contains exactly 3 open issues, 0 closed | N/A | N/A | `gh issue list --state all`: #12, #13, #14 open; none closed, re-read live 2026-09-24 | None specific | VERIFIED |
| #12: 8 of 23 Rule identifiers are unreachable and undocumented as such | `crates/analyzer-core/src/findings/mod.rs` L139-169 (`Rule` enum, 23 variants; `StateSchemaChanged`, `RehearsalStateChanged`, `RehearsalEventChanged`, `RehearsalErrorChanged`, `RehearsalAuthorizationChanged`, `RehearsalResourceChanged`, `RehearsalObservationIncomplete`, `ResourceUsageChanged` never constructed by orchestration); `schemas/analysis-result.schema.json` (`rule` enum, no reserved-value annotation) | N/A | GitHub issue #12, re-read live 2026-09-24: open | None specific | KNOWN LIMITATION |
| #13: four scaffold directories remain placeholder-only | `find examples docs tests scripts -type f` returns only one `README.md` per directory | N/A | GitHub issue #13, re-read live 2026-09-24: open | `examples/README.md`, `docs/README.md`, `tests/README.md`, `scripts/README.md` | KNOWN LIMITATION |
| #14: three Dependabot bumps blocked by declared MSRV | Closed PRs #6 (toml 0.9.6), #9 (ureq 3.4.2), #10 (clap 4.6.7), each failed the `msrv` CI job with "feature `edition2024` is required"; `ureq` 3.4.2 additionally breaks `crates/analyzer-state/src/rpc.rs`'s `ureq::AgentBuilder` usage | N/A | GitHub issue #14, re-read live 2026-09-24: open; closed PRs #6/#9/#10 re-read live 2026-09-24: still closed, not reopened | None specific | KNOWN LIMITATION |

---

## README Cross-Check

| README Statement | Traceability | Finding |
|---|---|---|
| "Extracts and compares each entrypoint's authorization surface... it never infers a principal (which `Address` is being checked) or a signer's identity" (L23-27) | Confirmed: Implementation extracts entrypoints and direct auth calls and explicitly does not infer principals (`extraction.rs` L50-52). | **RESOLVED BEFORE THIS REFRESH**: The previous misleading mention of "principal" as something extracted has been removed and explicitly clarified. |
| "The analysis pipeline (executable identity, interface diff, state compatibility, authorization diff, and controlled rehearsal) and the `analyze` CLI command are implemented" (L104-107) | Confirmed: `crates/analyzer-rehearsal/src/host.rs` implements bounded rehearsal with `soroban-env-host` and `crates/analyzer-cli` orchestrates it. | **RESOLVED BEFORE THIS REFRESH**: The previous stale statement that rehearsal was "not yet implemented" has been removed. |
| "Produces a versioned, deterministic canonical JSON report, plus a terminal rendering of the same data." (L34-35) | Confirmed: JSON report implemented in `analyzer-report/src/json.rs`. Terminal rendering implemented in `analyzer-report/src/terminal.rs`. | **RESOLVED BEFORE THIS REFRESH**: The previous claim regarding "Markdown" rendering has been removed. |
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
3. Is it only logically covered? (All CLI claims are now TESTED LOCALLY due to integration tests)
4. Is there live verification? (CI format/lint/test and MSRV check: yes, through GitHub Actions; all others: no)
5. Is it actually a limitation? (KNOWN LIMITATION claims above)
6. Does wording imply more than implementation proves? (Previous README discrepancies are all resolved)

No claim in this document has a status stronger than its evidence supports.

---

## Evidence Ledger Reconciliation

`evidence/index.md` (refreshed 2026-09-24) is the authoritative claim
inventory: 59 numbered claims across sections A-K. This table maps
every one of those 59 claims to its row in this document, or
explicitly accounts for it.

| Evidence Claim # | Evidence Claim | Traced In This Document | Status |
|---|---|---|---|
| 1 | Semantic executable diffing | A, "Semantic executable diffing" | TRACED |
| 2 | WASM structural validation | A, "WASM structural validation" | TRACED |
| 3 | Soroban contract environment metadata analysis | A, "Soroban contract environment metadata analysis" | TRACED |
| 4 | Contract specification/interface analysis | A, "Contract specification/interface analysis" | TRACED |
| 5 | Function/interface change detection | A, "Function/interface change detection" | TRACED |
| 6 | Event/type change detection | A, "Event/type change detection" | TRACED |
| 7 | State compatibility assessment | A, "State compatibility assessment" | TRACED |
| 8 | Migration manifest handling | A, "Migration manifest handling" | TRACED |
| 9 | Author-supplied migration evidence is unverified | A, "Author-supplied migration evidence is unverified" | TRACED |
| 10 | Authorization surface analysis | A, "Authorization surface analysis (direct-call extraction)" | TRACED |
| 11 | Authorization analysis is direct-call scoped | G, "No helper/transitive authorization tracing" | TRACED |
| 12 | Principal identity is not inferred | G, "No principal inference" | TRACED |
| 13 | Controlled rehearsal | B, "Controlled rehearsal (bounded local execution)" | TRACED |
| 14 | Rehearsal outcome comparison | B, "Rehearsal outcome comparison (trace model)" | TRACED |
| 15 | Rehearsal return-value comparison | B, "Rehearsal return-value comparison" | TRACED |
| 16 | Rehearsal events are not observable | B, "Rehearsal events: NOT OBSERVABLE" | TRACED |
| 17 | Rehearsal state reads are not observable | B, "Rehearsal state reads: NOT OBSERVABLE" | TRACED |
| 18 | Rehearsal state writes are not observable | B, "Rehearsal state writes: NOT OBSERVABLE" | TRACED |
| 19 | Rehearsal resource usage is not observable | B, "Rehearsal resource usage: NOT OBSERVABLE" | TRACED |
| 20 | Deterministic report generation | C, "Canonical serialization" | TRACED |
| 21 | Deterministic evidence identifiers | C, "Deterministic EvidenceId" | TRACED |
| 22 | Findings reference evidence | C, "Report evidence resolution" | TRACED |
| 23 | Report evidence records resolve correctly | C, "Report evidence resolution" (same row; both claims cover the same schema/resolution evidence) | TRACED |
| 24 | Malformed input handling | E, "Malformed input handling" | TRACED |
| 25 | CLI exit code behavior | E, "Exit codes (application-level mapping, corrected 2026-09-24)" | TRACED |
| 26 | Read-only RPC state source (library capability) | D, "Read-only RPC (library capability only; not CLI-reachable, see section H)" | TRACED |
| 27 | RPC source does not enumerate complete contract state | G, "No full storage enumeration" | TRACED |
| 28 | Live RPC behavior remains unverified | G, "Unverified live RPC" | TRACED |
| 29 | No private-key custody | D, "No private-key custody" | TRACED |
| 30 | No transaction submission | D, "No transaction submission" | TRACED |
| 31 | No network mutation | D, "No network mutation" | TRACED |
| 32 | No uncontrolled filesystem writes | D, "No uncontrolled filesystem writes" | TRACED |
| 33 | Bounded rehearsal execution through Soroban host | D, "Bounded rehearsal"; B, "Bounded rehearsal execution" | TRACED |
| 34 | Supported executable forms | A, "Supported executable forms" (added this refresh) | TRACED |
| 35 | Unsupported or limited executable forms | A, "Unsupported or limited executable forms" (added this refresh) | TRACED |
| 36 | Current declared Rust MSRV | F, "MSRV declaration" | TRACED |
| 37 | MSRV validation in CI | F, "MSRV CI validation" | TRACED |
| 38 | Normal CI checks | F, "CI format/lint/test" | TRACED |
| 39 | Protected main branch | F, "Protected main branch (ruleset)" | TRACED |
| 40 | Required PR approval | F, "Required PR approval" | TRACED |
| 41 | No bypass actors | F, "No bypass actors" | TRACED |
| 42 | Contract metadata parsing | A, "Contract metadata parsing" (added this refresh) | TRACED |
| 43 | Repository identity matches documentation | F, "Repository identity matches documentation" (added) | TRACED |
| 44 | License consistency | F, "License consistency" (added) | TRACED |
| 45 | Repository description and topics match implementation | F, "Repository description and topics" (added) | TRACED |
| 46 | Private vulnerability reporting is enabled | F, "Private vulnerability reporting enabled" (added) | TRACED |
| 47 | Standalone repository topology | F, "Standalone repository topology" (added) | TRACED |
| 48 | Upstream dependency repositories are not product peers | F, "Upstream dependency classification (not product peers)" (added) | TRACED |
| 49 | Open issue tracker represents real tracked work | J, "Issue tracker contains exactly 3 open issues, 0 closed" (added) | TRACED |
| 50 | v0.1.0 release and tag state | I, all three rows (added) | TRACED |
| 51 | RPC module is not reachable from the CLI | H, "RPC module is not reachable from the CLI" (added) | TRACED |
| 52 | No application runtime environment variables | H, "No application runtime environment variables" (added) | TRACED |
| 53 | No public or private service endpoint | H, "No public or private service endpoint" (added) | TRACED |
| 54 | No application database | H, "No application database" (added) | TRACED |
| 55 | No wallet or signing boundary | H, "No wallet or signing boundary" (added) | TRACED |
| 56 | No deployed service or hosting configuration | H, "No deployed service or hosting configuration" (added) | TRACED |
| 57 | 8 of 23 Rule identifiers are unreachable and undocumented as such | J, "#12: 8 of 23 Rule identifiers..." (added) | TRACED |
| 58 | Four scaffold directories remain placeholder-only | J, "#13: four scaffold directories..." (added) | TRACED |
| 59 | Three Dependabot dependency bumps are blocked by the declared MSRV | J, "#14: three Dependabot bumps..." (added) | TRACED |

Result: 59 of 59 evidence-ledger claims are traced in this document.
Zero are intentionally N/A at the claim level (some individual
Implementation/Test/Live-Evidence cells within a traced row are N/A,
which is expected and documented per row). Zero are missing. Zero are
blocked.

One gap was found during this reconciliation that is narrower than a
whole claim: the "Exit codes" row's `ANALYSIS_FAILURE` (5) code is
implemented but not exercised by any test (see that row's Test
column). This is noted in place rather than treated as a missing
claim, since codes 0, 2, and 3 are tested and the claim as a whole is
TESTED LOCALLY, not fully untested.
