# Evidence Ledger

Repository: `SorobanLabs/sorobanlabs-analyzer`
HEAD at time of audit: `064507b3e730194d2a02b103345ff90acb39ff3e`
Date of audit: 2026-09-23

This ledger records what an external reviewer can verify and from where.
It is not a marketing document; it is a verification record.

## Evidence States

| State | Meaning |
|---|---|
| VERIFIED | Directly established from available evidence during this audit |
| TESTED LOCALLY | Established by local test execution; no live/production verification |
| LOGICALLY COVERED | Implementation exists and is structurally sound but not directly exercised during this audit |
| UNVERIFIED | Claim cannot be established from current evidence |
| KNOWN LIMITATION | Explicitly acknowledged scope boundary |
| BLOCKED | Cannot be verified due to an external dependency or missing capability |

---

## A. Core Analyzer Capability

| # | Claim | Evidence Type | Source | What the Evidence Shows | Date Checked | Status | Notes |
|---|---|---|---|---|---|---|---|
| 1 | Semantic executable diffing | source, test | `crates/analyzer-executable/src/diff.rs` (fn `diff_interfaces`, L181-L392); tests: `diff::tests` (17 test functions covering function/struct/union/enum/error_enum/event changes) | Compares two `NormalizedInterface` structs by name-matched categories (functions, structs, unions, enums, error enums, events). Reports added, removed, and changed items with exact field-level detail. | 2026-09-23 | TESTED LOCALLY | 167 workspace tests pass at HEAD `064507b`. Diff covers 6 interface element categories. |
| 2 | WASM structural validation | source, test | `crates/analyzer-executable/src/validation.rs` (fn `validate_generic_wasm` L122, fn `check_soroban_structural_compatibility` L155); tests: `validation::tests` (10 tests) | Two-stage validation: (1) `wasmparser::Validator` checks core WASM well-formedness; (2) Soroban-specific checks for start functions, component-model sections, tags, memory64, shared memory. Reports `SorobanStructuralViolation` variants. Size bounded to `MAX_WASM_BYTES` = 64 MiB. | 2026-09-23 | TESTED LOCALLY | Tests include malformed bytes, structurally valid but Soroban-incompatible modules, and the 6 violation types. |
| 3 | Soroban contract environment metadata analysis | source, test | `crates/analyzer-executable/src/environment_meta.rs` (fn `parse_environment_metadata` L132); tests: `environment_meta::tests` (6 tests) | Parses `contractenvmetav0` custom WASM sections. Decodes `ScEnvMetaEntry` XDR to extract `EnvironmentInterfaceVersion` (protocol number, pre-release indicator). Detects missing, duplicate, and conflicting sections. | 2026-09-23 | TESTED LOCALLY | |
| 4 | Contract specification/interface analysis | source, test | `crates/analyzer-executable/src/contract_spec.rs` (fn `parse_contract_spec` L90); `crates/analyzer-executable/src/interface.rs` (fn `normalize_interface` L220); tests: `contract_spec::tests` (5 tests), `interface::tests` (9 tests including proptest) | Parses `contractspecv0` custom WASM sections. Decodes `ScSpecEntry` XDR. Normalizes into `NormalizedInterface` covering functions, structs, unions, enums, error enums, events with full type information. | 2026-09-23 | TESTED LOCALLY | Includes proptest for type normalization round-trip stability. |
| 5 | Function/interface change detection | source, test | `crates/analyzer-executable/src/diff.rs` (fn `diff_functions` L288, fn `diff_function_signature` L302); tests: `diff::tests::detects_function_added_and_removed`, `detects_function_input_count_changed`, `detects_function_output_changed`, etc. | Detects added/removed functions, input count changes, input order changes, individual input parameter changes, and output type changes. | 2026-09-23 | TESTED LOCALLY | |
| 6 | Event/type change detection | source, test | `crates/analyzer-executable/src/diff.rs` (fn `diff_events` L360, `diff_by_name` for structs/unions/enums/error_enums); tests: `diff::tests::detects_event_added`, `detects_struct_fields_changed`, etc. | Detects added/removed/changed events (topics, parameters, data format), structs (fields), unions (cases), enums (cases), error enums (cases). | 2026-09-23 | TESTED LOCALLY | |
| 7 | State compatibility assessment | source, test | `crates/analyzer-state/src/compatibility.rs` (fn `assess_state_compatibility` L100); tests: `compatibility::tests` (10 tests) | Evaluates compatibility based on executable form comparison and optional migration manifest signals. Categories: Compatible, RequiresMigration, PotentiallyIncompatible, NotDetermined. Manifest signals take priority over hash comparison. | 2026-09-23 | TESTED LOCALLY | Does not infer storage layout from WASM bytecode. |
| 8 | Migration manifest handling | source, test | `crates/analyzer-state/src/migration.rs` (`MigrationManifest` L65, fn `parse_json` L82, fn `to_evidence` L130); tests: `migration::tests` (8 tests) | Parses author-supplied JSON migration manifests. Records schema versions, key families, migration functions, prerequisites. Produces deterministic content hashes. | 2026-09-23 | TESTED LOCALLY | |
| 9 | Author-supplied migration evidence is unverified | source, test | `crates/analyzer-state/src/migration.rs` L5-12 (module doc), fn `to_evidence` L130-156 (observation string prefixed with `UNVERIFIED (author-supplied)`) ; test: `evidence_observation_is_explicitly_labeled_unverified` | The `to_evidence()` method explicitly prefixes the observation string with "UNVERIFIED (author-supplied)". Module documentation states manifest content is treated as evidence, not proof, and the analyzer does not independently verify manifest claims. | 2026-09-23 | VERIFIED | Directly established from source code and test. |
| 10 | Authorization surface analysis | source, test | `crates/analyzer-auth/src/extraction.rs` (fn `extract_authorization_surface` L127); tests: `extraction::tests` (8 tests) | Parses WASM bytecode to identify imports of `require_auth` and `require_auth_for_args`, detects `__check_auth` custom auth hook export, and maps direct calls from exported function bodies to recognized auth primitives. | 2026-09-23 | TESTED LOCALLY | Direct-call scoped only. |
| 11 | Authorization analysis is direct-call scoped | source | `crates/analyzer-auth/src/extraction.rs` L42-49 (module doc: "Only **direct** calls from an exported function's own body are detected") | Only `Operator::Call` instructions within an exported function's own code body are scanned. No interprocedural traversal, no `call_indirect` resolution, no transitive helper tracing. | 2026-09-23 | KNOWN LIMITATION | Explicitly documented in source. |
| 12 | Principal identity is not inferred | source | `crates/analyzer-auth/src/extraction.rs` L50-52 (module doc: "This module never infers a principal") | The module establishes that a call to the primitive exists but never infers which `Address` was passed. Arguments and stack operands preceding `Operator::Call` are uninspected. | 2026-09-23 | KNOWN LIMITATION | Explicitly documented in source. |

## B. Rehearsal

| # | Claim | Evidence Type | Source | What the Evidence Shows | Date Checked | Status | Notes |
|---|---|---|---|---|---|---|---|
| 13 | Controlled rehearsal | source, test | `crates/analyzer-rehearsal/src/host.rs` (fn `rehearse_invocation` L87); tests: `host::tests` (5 tests) | Invokes contract functions through `soroban-env-host` 28.0.2 `testutils` path. Uses `Host::test_host_with_recording_footprint()` and `Host::register_test_contract_wasm()`. | 2026-09-23 | TESTED LOCALLY | Uses the official Soroban host, not arbitrary execution. |
| 14 | Rehearsal outcome comparison | source, test | `crates/analyzer-rehearsal/src/trace.rs` (`RehearsalTrace` L17, fn `current_observation` L28, fn `candidate_observation` L36); `crates/analyzer-rehearsal/src/observation.rs` (`InvocationObservation` L64); tests: `trace::tests` (3 tests) | `RehearsalTrace` holds parallel `Vec<InvocationObservation>` for current and candidate executables, enabling per-label observation lookup. Outcome (`ExecutionOutcome`) records Success, HostError, Trap, or Blocked. | 2026-09-23 | TESTED LOCALLY | Behavioral diff logic not yet implemented; trace model is in place. |
| 15 | Rehearsal return-value comparison | source, test | `crates/analyzer-rehearsal/src/host.rs` L171-188 (captures `ScVal` return, encodes to hex); `crates/analyzer-rehearsal/src/observation.rs` L71 (`return_value_xdr_hex: Option<String>`); test: `add_invocation_returns_the_correct_sum` | On successful invocation, the return value is captured as canonical XDR hex. On failure/blocked, it is `None`. | 2026-09-23 | TESTED LOCALLY | |
| 16 | Rehearsal events are not observed | source | `crates/analyzer-rehearsal/src/host.rs` L119 (`events: vec![]`); module doc L57-58 | Events field is hardcoded to empty `vec![]`. Module documentation explicitly states events are not yet captured. | 2026-09-23 | KNOWN LIMITATION | NOT OBSERVABLE in current implementation. |
| 17 | Rehearsal state reads are not observed | source | `crates/analyzer-rehearsal/src/host.rs` L120 (`state_reads: vec![]`); module doc L58 | State reads field is hardcoded to empty `vec![]`. | 2026-09-23 | KNOWN LIMITATION | NOT OBSERVABLE in current implementation. |
| 18 | Rehearsal state writes are not observed | source | `crates/analyzer-rehearsal/src/host.rs` L121 (`state_writes: vec![]`); module doc L58 | State writes field is hardcoded to empty `vec![]`. | 2026-09-23 | KNOWN LIMITATION | NOT OBSERVABLE in current implementation. |
| 19 | Rehearsal resource usage is not observed | source | `crates/analyzer-rehearsal/src/host.rs` L122 (`resource_usage: ResourceUsage::default()`); observation.rs L57-58 (both fields `None`) | Resource usage is hardcoded to default (both `instructions_consumed` and `memory_bytes_consumed` are `None`). | 2026-09-23 | KNOWN LIMITATION | NOT OBSERVABLE in current implementation. |

## C. Evidence and Determinism

| # | Claim | Evidence Type | Source | What the Evidence Shows | Date Checked | Status | Notes |
|---|---|---|---|---|---|---|---|
| 20 | Deterministic report generation | source, test | `crates/analyzer-report/src/json.rs` (fn `to_canonical_json` L17); tests: `json::tests::serialization_is_deterministic_across_calls`, `output_ends_with_a_single_trailing_newline`, `round_trips_through_json` | Uses `serde_json::to_string_pretty` on structs with only ordered fields (no `HashMap`). No timestamps, UUIDs, or local paths in output. Trailing newline appended. | 2026-09-23 | TESTED LOCALLY | |
| 21 | Deterministic evidence identifiers | source, test | `crates/analyzer-evidence/src/reference.rs` (fn `compute_evidence_id` L133); tests: `identical_inputs_produce_identical_id`, `different_observation_produces_different_id`, `different_producer_produces_different_id`, `different_location_produces_different_id` | SHA-256 over length-prefixed concatenation of: source kind, source JSON, producer, location, observation. No timestamps or randomness. | 2026-09-23 | TESTED LOCALLY | |
| 22 | Findings reference evidence | source, test, schema | `crates/analyzer-core/src/findings/mod.rs` L260 (`evidence: Vec<EvidenceId>`); `crates/analyzer-report/src/canonical.rs` L35 (`evidence: Vec<String>`), L51 (conversion via `id.to_string()`); `schemas/analysis-result.schema.json` L138-144 (pattern `^[0-9a-f]{64}$`) | Each `Finding` holds `Vec<EvidenceId>`. Each `ReportFinding` holds `Vec<String>` of 64-char hex IDs. Schema enforces the hex64 pattern. | 2026-09-23 | TESTED LOCALLY | |
| 23 | Report evidence records resolve correctly | test, schema | `crates/analyzer-report/tests/schema_validation.rs` (6 tests validating structure against schema); `crates/analyzer-report/src/canonical.rs` tests (2 tests) | Schema validation tests confirm required keys, enum values, hex patterns, and schema version const match. Finding conversion test confirms all fields including evidence IDs are preserved. | 2026-09-23 | TESTED LOCALLY | No runtime JSON Schema validator linked; validation is structural test-time checking. |

## D. Input Handling and CLI

| # | Claim | Evidence Type | Source | What the Evidence Shows | Date Checked | Status | Notes |
|---|---|---|---|---|---|---|---|
| 24 | Malformed input handling | source, test | `crates/analyzer-executable/src/validation.rs` (returns `UnsupportedArtifactError`); `crates/analyzer-auth/src/extraction.rs` test `malformed_bytes_return_structured_error_not_panic`; `crates/analyzer-state/src/rpc.rs` test `malformed_response_body_is_a_structured_serialization_error`; `crates/analyzer-rehearsal/src/host.rs` test `malformed_candidate_wasm_is_blocked_not_a_process_crash`; workspace lint `unwrap_used = "deny"`, `expect_used = "deny"` | Malformed WASM, malformed JSON, malformed XDR, and malformed RPC responses all produce structured `AnalyzerError` variants rather than panics. Clippy lints deny `unwrap` and `expect` in production code. | 2026-09-23 | TESTED LOCALLY | |
| 25 | CLI exit code behavior | source | `crates/analyzer-cli/src/main.rs` L17-20 | CLI currently has no subcommands implemented. Prints placeholder message and exits 0. Clap exits 0 on `--help`/`--version` and 2 on invalid arguments. No custom exit codes exist. | 2026-09-23 | TESTED LOCALLY | CLI subcommands are not yet wired up. |

## E. Security and Trust Boundary

| # | Claim | Evidence Type | Source | What the Evidence Shows | Date Checked | Status | Notes |
|---|---|---|---|---|---|---|---|
| 26 | Read-only RPC state source | source, test | `crates/analyzer-state/src/rpc.rs` (fn `load` L135; uses only `getLedgerEntries` JSON-RPC method, L157); tests: `rpc::tests` (4 tests with mock transport) | Only the `getLedgerEntries` method is called. No mutating RPC methods. HTTP POST with `Content-Type: application/json` only. | 2026-09-23 | TESTED LOCALLY | Tests use mock transport, not live endpoints. |
| 27 | RPC source does not enumerate complete contract state | source | `crates/analyzer-state/src/rpc.rs` L21-26 (module doc), L136-147 (implementation) | Soroban RPC `getLedgerEntries` requires exact key specification. Only the contract instance entry and caller-specified `DataKeyRequest` keys are queried. Module documentation explicitly states this does not hold a complete contract data footprint. | 2026-09-23 | KNOWN LIMITATION | Inherent Soroban RPC design constraint. |
| 28 | Live RPC behavior remains unverified | manual observation | No live endpoint test exists in the test suite; all RPC tests use `MockTransport` | No test or CI step exercises a real Soroban RPC endpoint. The `UreqTransport` implementation exists but has not been verified against a live endpoint during this audit. | 2026-09-23 | UNVERIFIED | |
| 29 | No private-key custody | source | Full repository search: `grep -r "secret\|private.key\|seed.phrase\|SigningKey\|SecretKey" crates/ --include="*.rs"` yields zero matches (excluding comments about the absence of key handling) | No private key types, arguments, storage, or operations exist anywhere in the codebase. | 2026-09-23 | VERIFIED | Confirmed by source search on 2026-09-23. |
| 30 | No transaction submission | source | Full repository search: no `submit_transaction`, `send_transaction`, `TransactionEnvelope`, or equivalent submission code exists | No transaction construction, signing, simulation, or submission logic exists. | 2026-09-23 | VERIFIED | Confirmed by source search on 2026-09-23. |
| 31 | No network mutation | source | `crates/analyzer-state/src/rpc.rs` (read-only `getLedgerEntries` only); `crates/analyzer-cli/` (no network calls) | The only network interaction is the read-only RPC client. No mutation path exists. | 2026-09-23 | VERIFIED | |
| 32 | No uncontrolled filesystem writes | source | `crates/analyzer-cli/src/orchestration.rs` (reads `LoadedWasm::from_path` only; no file creation/write calls); no `std::fs::write`, `File::create`, or `OpenOptions` with write in production crate code | Production code reads WASM files and produces in-memory `AnalysisReport`. No files are written. Test code uses `tempfile` for temporary fixtures only. | 2026-09-23 | VERIFIED | |
| 33 | Bounded rehearsal execution through Soroban host | source, test | `crates/analyzer-rehearsal/src/host.rs` L78-81 (`DEFAULT_CPU_INSTRUCTION_LIMIT = 100_000_000`, `DEFAULT_MEMORY_LIMIT_BYTES = 41_943_040`), L145 (`host.test_budget(cpu_limit, memory_limit)`); L99-101 (panic isolation via `call_with_suppressed_panic_hook`) | CPU instruction limit (default 100M) and memory byte limit (default ~40 MiB) are passed to `Host::test_budget()`. Panic isolation catches host panics from malformed WASM and converts them to `ExecutionOutcome::Blocked`. | 2026-09-23 | TESTED LOCALLY | Test `malformed_candidate_wasm_is_blocked_not_a_process_crash` confirms panic isolation. |

## F. Supported Executable Forms

| # | Claim | Evidence Type | Source | What the Evidence Shows | Date Checked | Status | Notes |
|---|---|---|---|---|---|---|---|
| 34 | Supported executable forms | source | `crates/analyzer-executable/src/artifact.rs` (`ArtifactSource::LocalFile`, fn `LoadedWasm::from_path` L134) | Supports loading WASM executables from local filesystem paths. `LoadedWasm` reads bytes, computes SHA-256 hash, runs generic WASM validation and Soroban structural compatibility check. | 2026-09-23 | TESTED LOCALLY | |
| 35 | Unsupported or limited executable forms | source | `crates/analyzer-state/src/snapshot.rs` (`ExecutableForm::StellarAsset`, `ExecutableForm::ExternalRef`); `crates/analyzer-state/src/compatibility.rs` L130 (form discriminant comparison) | `StellarAsset` and `ExternalRef` forms are modeled in the state snapshot but not loadable as WASM artifacts. `ExternalRef` compatibility always returns `NotDetermined`. Form changes between types are `PotentiallyIncompatible`. | 2026-09-23 | KNOWN LIMITATION | |

## G. Repository Governance

| # | Claim | Evidence Type | Source | What the Evidence Shows | Date Checked | Status | Notes |
|---|---|---|---|---|---|---|---|
| 36 | Current declared Rust MSRV | source | `Cargo.toml` L17 (`rust-version = "1.84.0"`); `rust-toolchain.toml` L12-13 (development/CI toolchain `channel = "1.98.1"`) | Workspace declares MSRV 1.84.0. Development/CI uses 1.98.1. The comment in `rust-toolchain.toml` L10 claims "The resolved dependency graph is still verified against rustc 1.84.0 directly (see MSRV check in CI)". | 2026-09-23 | TESTED LOCALLY | MSRV declaration exists. See claim 37 for CI verification. |
| 37 | MSRV validation in CI | CI | `.github/workflows/ci.yml` (complete file, 42 lines) | The CI workflow contains exactly 1 job (`check`, named "format, lint, and test") running on `ubuntu-latest`. It installs the toolchain from `rust-toolchain.toml` (1.98.1) and runs `cargo fmt --check`, `cargo clippy`, `cargo test`. There is **no dedicated MSRV check job or step** that builds against rustc 1.84.0. The `rust-toolchain.toml` comment claims an MSRV check in CI, but no such step exists in the workflow file. | 2026-09-23 | UNVERIFIED | The GitHub ruleset requires a check named "MSRV build (rustc 1.84.0)" but no CI job produces this check. This is a discrepancy between the ruleset configuration and the CI workflow. |
| 38 | Normal CI checks | CI | `.github/workflows/ci.yml` job `check` (name: "format, lint, and test"); commands: `cargo fmt --all --check`, `cargo clippy --workspace --all-targets -- -D warnings`, `cargo test --workspace` | CI runs format check, clippy with `-D warnings`, and workspace tests on every push to main and every PR. | 2026-09-23 | VERIFIED | Verified from workflow file content. Actual CI pass status depends on GitHub Actions runs. |
| 39 | Protected main branch | GitHub configuration | GitHub repository ruleset named "main", targeting `refs/heads/main` | Main branch is protected by an active GitHub ruleset. Force pushes blocked. Branch deletion blocked. | 2026-09-23 | VERIFIED | Repository ruleset (not classic branch protection). |
| 40 | Required PR approval | GitHub configuration | Ruleset "main": required approving reviews = 1, dismiss stale approvals = enabled | One independent approving review is required before merge to main. Stale approvals are dismissed on new pushes. | 2026-09-23 | VERIFIED | |
| 41 | No bypass actors | GitHub configuration | Ruleset "main": bypass list empty, current user bypass = never | No users or teams can bypass the ruleset requirements. | 2026-09-23 | VERIFIED | |

## H. Contract Metadata

| # | Claim | Evidence Type | Source | What the Evidence Shows | Date Checked | Status | Notes |
|---|---|---|---|---|---|---|---|
| 42 | Contract metadata parsing | source, test | `crates/analyzer-executable/src/contract_meta.rs` (fn `parse_contract_metadata` L121); tests: `contract_meta::tests` (5 tests) | Parses `contractmetav0` custom WASM sections. Decodes `ScMetaEntry` XDR to extract key-value pairs. Recognizes `rsver` (Rust version) and `rssdkver` (SDK version) keys. Reports decode errors without panicking. | 2026-09-23 | TESTED LOCALLY | |

---

## Quality Checks Performed

1. Every source path listed above exists in the repository at HEAD `064507b`.
2. Every test name listed was confirmed by `cargo test --workspace` (167 tests, 0 failures, 2026-09-23).
3. CI workflow job name "format, lint, and test" matches `.github/workflows/ci.yml` exactly.
4. GitHub ruleset claims match the governance state provided for this audit.
5. No duplicate claims with conflicting statuses exist.
6. No claim has a status stronger than its evidence supports.
7. No invented deployment, Testnet, or live network data exists in this ledger.
8. No claim has been marked VERIFIED solely because documentation says so.
9. KNOWN LIMITATION entries all reflect explicitly documented scope boundaries in source code.
10. UNVERIFIED entries identify claims that cannot be established from current evidence.
