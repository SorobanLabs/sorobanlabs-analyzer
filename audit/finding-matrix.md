# Finding Matrix

Audit date: 2026-09-23. Repository: SorobanLabs/sorobanlabs-analyzer, branch main, HEAD `e64c7573ebdd3908efe9dbabc0c2b13c53f5329a`. All evidence below was directly observed in this repository's source, tests, or the live GitHub repository during this audit; no result is inferred from intent or from a prior report.

This document is a point-in-time snapshot dated 2026-09-23 at the HEAD
above. It is not re-run automatically and is not the current claim
surface. `evidence/index.md` and `audit/claim-traceability.md` are the
current, actively maintained record of what is verified true today;
consult those first. Each finding below now carries a `resolution`
line stating its actual current status as of this correction
(2026-09-24), so this document does not present a fixed finding as
still open. The original finding text is preserved unchanged below the
resolution line for historical record.

Classification key: A = Blocker, B = Stale/Cosmetic, C = Known Limitation, D = Important Pre-Submission Fix, E = Non-Blocking Backlog.

## AUDIT-01

- resolution: RESOLVED 2026-09-23 by commit `93cfaf7` (fix(cli): correct interface evidence provenance). Interface evidence now names the correct side per change direction (candidate for Added, current for Removed, both for Changed via `DerivedComparison`), with regression tests. See `evidence/index.md` claims 1-6 and `audit/claim-traceability.md` section A.
- classification: D
- title: Interface-finding evidence source always cites the current executable, even for facts about the candidate
- affected area: `crates/analyzer-cli/src/orchestration.rs`, `synthesize_interface_findings`'s `spec_evidence` closure
- evidence: the closure builds `EvidenceSource::CustomWasmSection { artifact_hash: current.hash().to_hex(), section_name: "contractspecv0" }` unconditionally, including for `FunctionChange::Added`, `EventChange::Added`, and every struct/union/enum "Added" case, all of which are facts about the **candidate's** contractspecv0 section, not the current one's.
- why it matters: the evidence record's `source` field is supposed to name what was actually inspected to establish the fact; for "Added" cases it names the wrong artifact, weakening the finding-to-evidence-to-source chain the evidence model exists to provide.
- current status: TESTED LOCALLY (the code path is exercised by existing interface-diff tests, but no test asserts the evidence source's artifact_hash matches the side the fact is actually about)
- recommended next action: use `DerivedComparison { inputs: [current_hash, candidate_hash] }` (as identity/environment/state/authorization findings already do) or select the correct single-artifact hash per change direction.
- submission-critical: no

## AUDIT-02

- resolution: RESOLVED 2026-09-23 by commit `05e0b5a` (fix(state): preserve manifest evidence provenance). Both the finding detail and evidence observation now carry the literal `UNVERIFIED (author-supplied):` prefix for manifest-derived reasons. See `evidence/index.md` claim 9 and `audit/claim-traceability.md` section A.
- classification: D
- title: Manifest-derived state-compatibility evidence loses the "UNVERIFIED (author-supplied)" framing
- affected area: `crates/analyzer-cli/src/orchestration.rs`, `synthesize_state_findings`; contrast with `crates/analyzer-state/src/migration.rs`, `MigrationManifest::to_evidence`
- evidence: running `analyze` with a manifest declaring a migration function produces a MIGRATION_REQUIRED finding with `detail: "migration manifest declares a migration function"` and an evidence record with `observation: "migration manifest declares a migration function"`. Neither string carries the "UNVERIFIED (author-supplied)" prefix that `MigrationManifest::to_evidence()` (unused here) produces. Confirmed by direct CLI execution during this audit (see Claim Traceability, migration manifest handling).
- why it matters: the only signal that this finding rests on an unverified author declaration is `confidence: LIKELY` and `evidence[0].source.kind == "migration_manifest"`; a reader of the finding/evidence text alone, without cross-referencing what `migration_manifest` as a source kind means, could read it as an established fact.
- current status: TESTED LOCALLY (CLI smoke test run during this audit, 2026-09-23)
- recommended next action: either call `MigrationManifest::to_evidence()` for this evidence record, or prefix the `reason_detail` text itself with an unverified-declaration marker when the outcome is manifest-derived.
- submission-critical: no

## AUDIT-03

- resolution: RESOLVED 2026-09-23 by commit `55ea097` (docs(auth): correct authorization principal claim). README's authorization bullet no longer claims principal extraction and explicitly states principal/signer identity is never inferred. See `audit/claim-traceability.md` section G.
- classification: D
- title: README's authorization bullet claims "principal" is extracted, which the extractor explicitly does not do
- affected area: `README.md` line 24; `crates/analyzer-auth/src/extraction.rs`
- evidence: README: "Extracts and compares the authorization surface (entrypoint, authorization requirement, principal, check)." `extraction.rs`'s module docs state "This module never infers a principal (which `Address`)... is protected; only a direct call..."; `EntrypointAuthorization` (the extractor's only output type) has fields `export_name: String` and `direct_calls: Vec<String>` only. No principal/address field exists anywhere in the crate.
- why it matters: this is a directly checkable, currently false claim about analyzer capability, in the project's primary public-facing document.
- current status: VERIFIED false (source inspected directly, `grep -n "principal" crates/analyzer-auth/` returns nothing)
- recommended next action: remove "principal" from the README bullet, or replace with an explicit statement that principal/address identity is never inferred.
- submission-critical: no (does not affect implementation correctness, but is a factual accuracy defect in the primary README)

## AUDIT-04

- resolution: RESOLVED 2026-09-23 by commit `d26cbe8` (ci: validate declared Rust MSRV). `.github/workflows/ci.yml` now has an `msrv` job ("MSRV build (rustc 1.84.0)") that installs rustc 1.84.0 and runs `cargo +1.84.0 build --workspace`; this check is required by the branch ruleset and passes live. See `evidence/index.md` claim 37 and `audit/claim-traceability.md` section F.
- classification: D
- title: rust-toolchain.toml claims an MSRV check exists in CI; it does not
- affected area: `rust-toolchain.toml` (comment), `.github/workflows/ci.yml`
- evidence: comment reads "The resolved dependency graph is still verified against rustc 1.84.0 directly (see MSRV check in CI)". `.github/workflows/ci.yml` contains exactly one job (`check`: checkout, `rustup show`, cache, `cargo fmt --all --check`, `cargo clippy ... -D warnings`, `cargo test --workspace`) run under the toolchain pinned by `rust-toolchain.toml` (1.98.1). There is no step that installs or builds against rustc 1.84.0.
- why it matters: the underlying MSRV claim is currently true (this audit ran `cargo +1.84.0 build --workspace` directly and it succeeded, 2026-09-23, ~2m15s), but nothing in CI would catch a future regression against 1.84.0; the comment asserts a safety net that does not exist.
- current status: VERIFIED false (workflow file read directly; MSRV build itself CURRENTLY EXECUTED and confirmed passing separately)
- recommended next action: either add an MSRV job to CI, or correct the comment to state the graph is only spot-checked manually/ad hoc.
- submission-critical: no

## AUDIT-05

- resolution: RESOLVED 2026-09-24 by commit `22eb282` (docs(contributing): correct stale CI claim and add missing contributor sections, PR #11). CONTRIBUTING.md no longer describes schema validation/determinism tests as future work; it now states they run as part of `cargo test --workspace`.
- classification: B
- title: CONTRIBUTING.md describes JSON schema validation and determinism tests as future work; both already exist and already run
- affected area: `CONTRIBUTING.md`
- evidence: "CI runs the same checks, plus JSON schema validation and determinism tests once those subsystems exist." `crates/analyzer-report/tests/schema_validation.rs` (6 tests) and multiple determinism-oriented tests (for example `orchestration::tests::identical_analysis_inputs_produce_identical_evidence_ids`, `json::tests::serialization_is_deterministic_across_calls`) exist today and run as part of `cargo test --workspace`, which `ci.yml` already executes.
- why it matters: understates current test coverage to a reader deciding whether to trust the project's CI.
- current status: stale documentation; underlying capability VERIFIED present and passing (this audit re-ran both test files, 2026-09-23)
- recommended next action: reword to state these checks are already part of the unified `cargo test --workspace` step, not separate future CI steps.
- submission-critical: no

## AUDIT-06

- classification: B
- title: CONTRIBUTING.md's "do not force-push or rewrite shared history" was violated once, by explicit instruction, without a documented exception
- affected area: `CONTRIBUTING.md`; git history (main force-pushed from `c6fc8b4` to `e64c757` during the prior session's AI-metadata-cleanup)
- evidence: `CONTRIBUTING.md` states this rule unconditionally. `git log` shows the rewrite; a `backup/pre-ai-metadata-cleanup-2026-09-22` branch documents it, but CONTRIBUTING.md itself has no carve-out for maintainer-authorized, backup-preserved history cleanup.
- why it matters: internal consistency between stated project policy and an action actually taken on the repository; a future contributor citing this document would have grounds to call the rewrite a policy violation.
- current status: KNOWN LIMITATION (the rewrite was a one-time, explicitly authorized, backed-up exception, not a recurring practice)
- recommended next action: none required before submission; optionally add a narrow exception clause if this kind of cleanup is expected to recur.
- submission-critical: no

## AUDIT-07

- resolution: STILL OPEN as of 2026-09-24. Now formally tracked as GitHub issue #13 ("Populate or correct placeholder scaffold directories"), which also extends this finding to `tests/` and `scripts/` (not originally covered here). Not resolved by this correction pass; do not treat this finding as fixed.
- classification: B
- title: examples/ and docs/ directories contain only placeholder READMEs describing content that does not exist yet
- affected area: `examples/README.md`, `docs/README.md`
- evidence: `examples/README.md`: "Example inputs and generated reports demonstrating end-to-end analyzer usage. Populated alongside the CLI and fixture corpus." The CLI and fixture corpus both now exist; `examples/` contains only this one file. `docs/README.md` lists "the confidence and finding model, rule references, the report schema, and fixture structure" as directory contents; `docs/` contains only this one file.
- why it matters: a reviewer following either README's stated purpose into the directory finds nothing; not a functional defect but an unfulfilled documentation promise now overdue given the CLI and fixtures both exist.
- current status: NOT PRESENT
- recommended next action: populate with real content, or soften the wording to avoid implying imminent delivery.
- submission-critical: no

## AUDIT-08

- resolution: STILL OPEN as of 2026-09-24. Now formally tracked as GitHub issue #12 ("Document which Rule identifiers are currently unreachable"). Not resolved by this correction pass; do not treat this finding as fixed.
- classification: C
- title: 8 of 23 `Rule` identifiers are defined (in code and in the JSON schema) but never produced
- affected area: `crates/analyzer-core/src/findings/mod.rs` (`Rule` enum), `schemas/analysis-result.schema.json` (`rule` enum), `crates/analyzer-cli/src/orchestration.rs` (actual production sites)
- evidence: `grep -n "Rule::" crates/analyzer-cli/src/orchestration.rs` (excluding the `overall_status` match on `MigrationRequired`) shows 15 distinct rules actually constructed: `ExecutableHashChanged`, `ExecutableStructurallyIncompatible`, `EnvironmentInterfaceChanged`, `EnvironmentMetadataMissing`, `ContractInterfaceAdded`, `ContractInterfaceRemoved`, `ContractSignatureChanged`, `ContractEventChanged`, `ContractTypeChanged`, `MigrationRequired`, `StateCompatibilityUnknown`, `AuthorizationSurfaceChanged`, `AuthorizationRemoved`, `RehearsalFailed`, `RehearsalResultChanged`. Never produced: `StateSchemaChanged`, `RehearsalStateChanged`, `RehearsalEventChanged`, `RehearsalErrorChanged`, `RehearsalAuthorizationChanged`, `RehearsalResourceChanged`, `RehearsalObservationIncomplete`, `ResourceUsageChanged`. All 8 unused rules are consistent with the disclosed rehearsal-backend limitation (no event/state/resource observation) and the fact that state schema comparison is not independently implemented.
- why it matters: a consumer reading the schema's `rule` enum could reasonably expect all 23 values to be reachable; 8 currently are not.
- current status: KNOWN LIMITATION, consistent with disclosed scope elsewhere in the codebase (`analyzer-rehearsal` module docs already state events/state/resource usage are not captured)
- recommended next action: note explicitly in schema or docs which rule identifiers are reserved for future capability.
- submission-critical: no

## AUDIT-09

- resolution: RESOLVED 2026-09-23 by commit `32b97c3` (fix(rehearsal): report resource usage as unobservable). `remains_unverified` now includes `resource_usage` alongside `events`, `state`, and `authorization`. See `evidence/index.md` claim 19 and `audit/claim-traceability.md` section B.
- classification: D
- title: `remains_unverified` omits "resource_usage" even though `observations_unavailable` includes it
- affected area: `crates/analyzer-cli/src/orchestration.rs`, the `Some(analyzer_report::ReportRehearsal { ... })` construction for the "rehearsal ran" branch
- evidence: `observations_unavailable: vec!["events", "state_reads", "state_writes", "resource_usage"]` but `remains_unverified: vec!["events", "state", "authorization"]` (resource usage absent from the second list).
- why it matters: `remains_unverified` is the field meant to tell a report reader what rehearsal did not establish about runtime behavior; omitting resource usage there, while the sibling field correctly lists it as unavailable, is an internal inconsistency that could read as "resource usage was verified."
- current status: VERIFIED by direct source read; confirmed present in a real generated report (this audit's CLI smoke test JSON output)
- recommended next action: add "resource_usage" to `remains_unverified` in the rehearsal-ran branch.
- submission-critical: no

## AUDIT-10

- resolution: RESOLVED 2026-09-24 via live GitHub repository metadata (description and topics `rust`, `cli`, `soroban`, `stellar`, `wasm`, `smart-contracts` set directly; not a source commit). See `evidence/index.md` claim 45.
- classification: E
- title: GitHub repository metadata is minimal (no description, no topics, no homepage)
- affected area: GitHub repository settings (not source-controlled)
- evidence: `gh repo view` returned `"description":"","homepageUrl":"","repositoryTopics":null`.
- why it matters: reduces discoverability/context for an external reviewer landing on the repository page before reading README.
- current status: VERIFIED via GitHub API, 2026-09-23
- recommended next action: add a one-line description and relevant topics (rust, soroban, stellar, static-analysis) via repository settings.
- submission-critical: no
