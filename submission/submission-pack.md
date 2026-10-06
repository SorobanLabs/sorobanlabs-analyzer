# SorobanLabs Analyzer Submission Pack

All facts in this document were verified on 2026-10-06 against `main`
commit `13c8417b45a2bb67168aff878f5c6e12ae740b37` (verified base), the
live GitHub repository, and locally executed build/test commands. The
local commands were run on a branch that contains only documentation and
comment changes on top of that base (see section 14). The commit that
merges this pack is later than the verified base and is not recorded
here. This pack supersedes the earlier version, which described
`b7967ab84112ea6db52dec898fa7cd56641c7d66` (2026-10-05). It does not
declare submission readiness.

## 1. Submission Snapshot

- Project name: SorobanLabs Analyzer
- Organization: SorobanLabs
- Repository: SorobanLabs/sorobanlabs-analyzer
- Verified base SHA: `13c8417b45a2bb67168aff878f5c6e12ae740b37` (previous pack: `b7967ab`, superseded)
- Release: v0.1.0 (published, not draft, not prerelease)
- Documentation site: https://sorobanlabs.github.io/sorobanlabs-analyzer/ (live, verified 2026-10-06)
- Current project status: standalone Rust CLI/library workspace; post-v0.1.0 dependency maintenance; repository hygiene additions merged

This pack reflects the verified state as of the SHA above. It is
evidence preparation only, not a submission-readiness declaration.

## 2. Project Description

SorobanLabs Analyzer is a Rust CLI that determines what will change if
a deployed Soroban contract's executable is replaced. Given a current
and a candidate executable, it compares executable identity, contract
interface, state compatibility, and authorization surface, and can run
controlled rehearsal invocations against both under the real
`soroban-env-host` backend. Every finding carries an explicit
confidence level (`DETECTED`, `LIKELY`, `POTENTIAL`, `NOT_DETERMINABLE`)
and references the evidence it was derived from. It produces a
versioned, deterministic canonical JSON report and a terminal
rendering of the same data. It does not produce a SAFE/UNSAFE verdict,
does not perform real contract upgrades or submit transactions, and
does not request or handle private keys.

Source: `README.md`, current implementation under `crates/`.

## 3. Repository

- URL: https://github.com/SorobanLabs/sorobanlabs-analyzer (verified live, 2026-10-06)
- Organization: SorobanLabs
- Repository: sorobanlabs-analyzer
- Primary branch: `main`
- License: Apache-2.0 (verified against `LICENSE` and live GitHub-detected license, both agree)
- Release: v0.1.0

## 4. Documentation

Live GitHub Pages URL: https://sorobanlabs.github.io/sorobanlabs-analyzer/

Re-verified 2026-10-06, not trusted from a previous audit:

- Root page: `curl` HTTP 200.
- `introduction.html`: HTTP 200.
- `cli.html`: HTTP 200.
- `limitations.html`: HTTP 200.
- Live GitHub Pages configuration (`gh api repos/.../pages`): `build_type: workflow`, `public: true`, `https_enforced: true`.
- Deployment workflow (`.github/workflows/docs.yml`, job "Deploy documentation"): latest run `completed`/`success`, triggered by the merge that added the book (PR #24, 2026-09-24T21:37:11Z). The site was not redeployed since because `docs/`, `book.toml`, and the docs workflow have not changed since (`git log -- docs book.toml .github/workflows/docs.yml` shows no commit after the book was added). The final synchronization pull request changes `docs/src/limitations.md`, so merging it is expected to trigger one redeploy.

The book source lives at `docs/` (`book.toml`, `docs/src/*.md`, `docs/src/SUMMARY.md`) and covers: introduction, problem statement, how it works, architecture, analysis pipeline, executable/interface/state/authorization analysis, controlled rehearsal, evidence, report schema, CLI usage, installation, testing, security, limitations, status, examples, and contributing.

## 5. Release

- Tag: `v0.1.0`
- Tag target commit: `bf8b28599f617269958552f35f7315daeff04a54` (re-verified 2026-10-06 via `gh api .../releases/tags/v0.1.0` and `gh api .../git/ref/tags/v0.1.0`, both agree)
- Release state: published, `draft: false`, `prerelease: false` (re-verified live 2026-10-06)
- Release URL: https://github.com/SorobanLabs/sorobanlabs-analyzer/releases/tag/v0.1.0

No new release was created for this pack. The tag target predates the current verified `main` SHA (`bf8b285` vs. `13c8417`); this is expected release-tag aging, not a discrepancy, and is already recorded in the project's own audit trail (`audit/finding-matrix.md`, `audit/final-technical-audit.md`). Dependency updates merged since the tag (thiserror, wasmparser, soroban-env-host, indexmap) are not part of v0.1.0 and are listed under "Unreleased" in `CHANGELOG.md`.

## 6. Demo

Demo video: Not yet recorded.

No video file, hosted video link, or recording of any kind exists in this repository or is referenced by it. The real, runnable CLI example under `examples/upgrade-review/` (see section 13) is not a demo video and is not presented as one. Re-checked 2026-10-06: still no video file or link exists.

## 7. Repository Relationship

- Primary repository: SorobanLabs/sorobanlabs-analyzer
- Additional companion repositories: none verified

Re-verified live 2026-10-06: `gh api orgs/SorobanLabs/repos --paginate` returns exactly one repository, `SorobanLabs/sorobanlabs-analyzer` itself. No frontend, backend, API, SDK, indexer, or contract repository exists under the SorobanLabs organization. This project is a standalone repository.

## 8. Live Application

Live application: Not applicable for the current project state.

This is a CLI/library project with no server component. No listener, HTTP/REST/JSON-RPC/gRPC/WebSocket endpoint, or hosted service exists anywhere in the source (verified by repository-wide search during Phase 25/29, re-confirmed unchanged). The GitHub Pages documentation site (section 4) is static documentation hosting, not an application, and is not described as one anywhere in this pack.

## 9. Smart Contracts

Contract explorer links: Not applicable.

This repository does not deploy, own, or depend on a deployed Soroban contract for its own operation. It analyzes contract executables supplied to it as local files. No contract ID, network name, or explorer link is claimed anywhere in this repository, and none is fabricated here.

## 10. Transaction Links

Transaction links: Not applicable.

The analyzer never submits a transaction (verified: no transaction construction, signing, or submission code exists anywhere in `crates/`, confirmed by repository-wide search during Phase 25/29). No live network transaction workflow exists to link to.

## 11. Core Capabilities

| Capability | Current status | Evidence | Documentation |
|---|---|---|---|
| Executable analysis (loading, hashing, structural WASM validation) | Implemented | `crates/analyzer-executable/src/artifact.rs`, `validation.rs`; tests in same crate | `docs/src/executable-analysis.md` |
| Interface analysis (contract spec extraction and diff) | Implemented | `crates/analyzer-executable/src/interface.rs`, `diff.rs`; `diff::tests` (17 tests) | `docs/src/interface-analysis.md` |
| State compatibility analysis | Implemented, with disclosed limitation (no storage-layout reconstruction from raw WASM without a manifest) | `crates/analyzer-state/src/compatibility.rs`; `compatibility::tests` | `docs/src/state-compatibility.md` |
| Authorization analysis | Implemented, direct-call-only scope, no principal/signer inference (disclosed) | `crates/analyzer-auth/src/extraction.rs`, `diff.rs`; `extraction::tests` | `docs/src/authorization-analysis.md` |
| Controlled rehearsal | Implemented; captures outcome/return-value only, does not observe events/state/resource usage (disclosed) | `crates/analyzer-rehearsal/src/host.rs`; `host::tests` | `docs/src/controlled-rehearsal.md` |
| Evidence-backed findings | Implemented | `crates/analyzer-evidence/`; every finding carries at least one evidence id, verified traceable in generated reports | `docs/src/evidence.md` |
| Confidence reporting | Implemented, 4 levels (`DETECTED`, `LIKELY`, `POTENTIAL`, `NOT_DETERMINABLE`) | `crates/analyzer-core/src/findings/mod.rs` | `README.md`, `docs/src/evidence.md` |
| Structured report output (canonical JSON + terminal) | Implemented | `crates/analyzer-report/src/json.rs`, `terminal.rs`; schema at `schemas/analysis-result.schema.json` | `docs/src/report-schema.md` |

No planned-but-unimplemented capability is listed above as implemented.

## 12. CLI

Verified against `crates/analyzer-cli/src/cli.rs` (the actual `clap` argument definitions) and the real captured example (section 13), not from memory:

```
sorobanlabs-analyzer analyze \
  --current <PATH> \
  --candidate <PATH> \
  [--protocol <NUMBER>] \
  [--migration-manifest <PATH>] \
  [--rehearsal <PATH>] \
  [--format json|terminal]
```

`--current` and `--candidate` are required. `analyze` is the only subcommand. Documentation: `docs/src/cli.md`, `README.md` ("CLI usage" section).

## 13. Real Example

Location: `examples/upgrade-review/`

- Fixtures: `current.wasm` (byte-for-byte copy of `fixtures/executable/v1.wasm`), `candidate.wasm` (byte-for-byte copy of `fixtures/executable/v2_changed_error.wasm`).
- Command: `cargo run -p analyzer-cli -- analyze --current examples/upgrade-review/current.wasm --candidate examples/upgrade-review/candidate.wasm`
- Captured output: checked into `examples/upgrade-review/README.md` (terminal form) and `examples/upgrade-review/report.json` (canonical JSON form).
- Re-verified 2026-10-06: the CLI was rebuilt and run against these exact fixture files. The terminal output appears verbatim in `examples/upgrade-review/README.md`. The `--format json` output parses to the same JSON value as the checked-in `report.json`, including all evidence ids (deterministic). The two differ in whitespace only (the CLI emits 2-space indentation, the checked-in file uses 4), so this is a semantic match, not a byte-for-byte one.
- What it demonstrates: a candidate executable with a changed function return type and a changed error enum produces `REVIEW_REQUIRED` status, an `EXECUTABLE_HASH_CHANGED` finding, two interface-change findings (`CONTRACT_SIGNATURE_CHANGED`, `CONTRACT_TYPE_CHANGED`), a `STATE_COMPATIBILITY_UNKNOWN` finding (no migration manifest supplied), and a `REHEARSAL_FAILED` finding (no rehearsal manifest supplied), each with its own evidence id.

## 14. Test and Verification Evidence

All commands re-run 2026-10-06 on a documentation-only branch (`docs/final-synchronization`) that sits on verified base `13c8417`. The branch changes comments in `Cargo.toml`, `crates/analyzer-rehearsal/src/host.rs`, and `rust-toolchain.toml`, plus Markdown files; it changes no dependency and no executable code:

| Check | Command | Result |
|---|---|---|
| Formatting | `cargo fmt --all -- --check` | PASS |
| Workspace build | `cargo build --workspace` | PASS |
| Clippy | `cargo clippy --workspace --all-targets --all-features -- -D warnings` | PASS (no crate declares `[features]`, so `--all-features` is a no-op) |
| Workspace tests | `cargo test --workspace` | PASS, 227 tests, 0 failed |
| MSRV build | `cargo +1.84.0 build --workspace` | PASS |
| Targeted exit-code-5 test | `cargo test -p analyzer-cli --lib analysis_stage_failure_maps_to_analysis_failure_exit_code` | PASS |
| mdBook build | Not run locally (mdBook is not installed in this environment, and `docs/` did not change). The most recent "Deploy documentation" run is `completed`/`success` (2026-09-24) | Not re-run; no docs change to verify |
| Real example verification | Re-ran the example command; compared terminal output with the README and parsed JSON with `report.json` | PASS (terminal text identical; JSON semantically equal, whitespace differs) |

227 tests, 0 failed: confirmed by rerunning `cargo test --workspace` on 2026-10-06, not copied from a prior report. The count is the same as in the previous pack even though soroban-env-host (28.0.2 to 29.0.0) and indexmap (2.5.0 to 2.13.1) were updated in between.

## 15. CI and Repository Controls

Verified live 2026-10-06:

- CI workflow (`.github/workflows/ci.yml`): two jobs, "format, lint, and test" and "MSRV build (rustc 1.84.0)". Both jobs reported `success` on the push to `main` at `13c8417` (run 37426062230, 2026-10-06). CI has not yet run on the final synchronization branch at the time of writing.
- Required status checks (branch ruleset "main"): exactly these two job names, matching the workflow exactly, with the strict (up-to-date branch) policy on.
- Pull request requirement: active (ruleset rule `pull_request`, `required_approving_review_count: 1`, `dismiss_stale_reviews_on_push: true`).
- Branch protection: active as a GitHub ruleset (not classic branch protection), `enforcement: active`.
- Non-fast-forward protection: active (`non_fast_forward` rule; force pushes to `main` are rejected).
- Branch deletion protection: active (`deletion` rule).
- The ruleset also contains a `copilot_code_review` rule, which the previous pack did not list. It was not examined further.
- Bypass: `bypass_actors: []`, `current_user_can_bypass: never`: no one, including admins, can bypass these rules.
- Dependabot version updates: configured (`.github/dependabot.yml`, weekly checks for `cargo` and `github-actions` ecosystems). `ignore` rules on `main` (merged in PR #35) cover version ranges verified to conflict with the 1.84.0 MSRV or to break CI (clap >= 4.6, toml >= 0.9, ureq >= 3, sha2 >= 0.11, indexmap >= 2.14, toml_edit >= 0.23). Dependabot can still open PRs for versions below those ranges; PRs #38 (toml_edit 0.22.27) and #39 (clap 4.5.61) were opened after the rules landed, failed CI, and were closed unmerged on 2026-10-06. This is version-update automation, distinct from the items below.
- Dependabot security updates: **disabled** (re-verified 2026-10-06 via `gh api repos/.../` `security_and_analysis`, as an admin).
- Secret scanning (including push protection and validity checks): **disabled** (verified via the same call).

This pack does not claim GitHub's optional security services (Dependabot security updates, secret scanning) are enabled; they are not. Only the repository's own configured rules (branch ruleset, Dependabot version updates) are claimed as active.

## 16. Issues / Planned Work

Verified live 2026-10-06.

**Issue #12**
- Title: Document which Rule identifiers are currently unreachable
- Status: open
- URL: https://github.com/SorobanLabs/sorobanlabs-analyzer/issues/12
- Description: 8 of the 23 `Rule` identifiers defined in `crates/analyzer-core/src/findings/mod.rs` and in `schemas/analysis-result.schema.json` are never produced by any current analysis stage, and neither the enum's doc comments nor the schema say so (as recorded when the issue was opened; not re-counted in this refresh).
- Why it remains open: this is real, disclosed, in-scope documentation work that has not yet been done. It is also listed in `ROADMAP.md`.

**Issue #14**
- Title: clap, toml, and ureq dependency bumps are blocked by the declared 1.84.0 MSRV
- Status: open
- URL: https://github.com/SorobanLabs/sorobanlabs-analyzer/issues/14
- Description: Dependabot-proposed bumps cannot be merged because they require the `edition2024` Cargo feature or Rust newer than the declared MSRV (rustc 1.84.0). A comment added 2026-10-05 records three further blocked updates (sha2, indexmap, toml_edit, see below).
- Why it remains open: the MSRV decision is a maintainer policy choice. The interim decision to hold at 1.84.0 is implemented as Dependabot `ignore` rules, which are now on `main`. No maintainer has recorded a decision to close the issue, so it was left open and is not closed by this pack.

**Dependency pull requests since the previous pack**
- #30 thiserror 2.0.20 to 2.0.21: merged.
- #33 wasmparser 0.116.1 to 0.221.3: merged. `cargo tree` shows both versions in the lockfile: the analyzer crates (`analyzer-executable`, `analyzer-auth`) use 0.221.3 directly, while soroban-env-host and soroban-env-common 29.0.0 depend on 0.116.1 internally. The analyzer does not pass wasmparser types to the host, so the two versions do not need to match. The `Cargo.toml` comment describing this was corrected in PR #35 and updated for 29.0.0 in this pass.
- #36 soroban-env-host 28.0.2 to 29.0.0: merged. soroban-env-host 29.0.0 declares rust-version 1.84.0, the MSRV build passes, and it resolves to the same single stellar-xdr 28.0.0 already used by the workspace.
- #37 indexmap 2.5.0 to 2.13.1: merged. 2.13.1 is the version pinned in `Cargo.toml`; the 2.14 and later range is ignored (see #32 below).
- #31 sha2 to 0.11.0: closed unmerged. digest 0.11.x requires edition2024; the MSRV job failed.
- #32 indexmap 2.5.0 to 2.14.2: closed unmerged. indexmap 2.14.0 and later declare rust-version 1.85; the MSRV job failed.
- #34 toml_edit to 0.23.5: closed unmerged. Clippy failed against the current toml 0.8.19, and the MSRV job also failed.

- #38 toml_edit 0.22.20 to 0.22.27 and #39 clap 4.5.20 to 4.5.61: closed unmerged on 2026-10-06 after CI failed, with a comment that they can be revisited after submission. The cause of those CI failures was not investigated in this pass.

Each closed PR carries an explanatory comment. No pull request was open at the start of this pass (`gh pr list --state open` returned none).

Dependency baseline on `main` at `13c8417` (from `Cargo.toml` and `Cargo.lock`): soroban-env-host `=29.0.0`, stellar-xdr `=28.0.0`, wasmparser `=0.221.3` (plus 0.116.1 as a transitive dependency of soroban-env-host), indexmap `=2.13.1`, thiserror `=2.0.21`, tempfile `=3.27.0`, clap `=4.5.20`, toml `=0.8.19`, ureq `=2.12.1`, sha2 `=0.10.9`. Declared MSRV: 1.84.0.

## 17. Current Limitations

- Standalone CLI/library scope: no frontend, backend, API server, database, or wallet component exists (verified by repository-wide search, Phase 25/29).
- No deployed application (section 8).
- No deployed contract workflow (section 9).
- No live network submission workflow (section 10).
- `analyzer_state::rpc` exists as a tested library module (`crates/analyzer-state/src/rpc.rs`) but is not wired into the CLI; no flag, environment variable, or configuration path reaches it. The shipped CLI performs no network access.
- Controlled rehearsal (`crates/analyzer-rehearsal`) captures execution outcome and return value only; it does not observe events, state reads, state writes, or resource usage. This is disclosed in source (`host.rs` module docs) and in `docs/src/controlled-rehearsal.md` and `docs/src/limitations.md`.
- Authorization analysis is direct-call-only and never infers a principal or signer identity (disclosed in README and `docs/src/authorization-analysis.md`).
- Outstanding issues #12 and #14 (section 16) remain open.
- The MSRV is held at 1.84.0, so newer releases of clap (4.6 and later), toml (0.9 and later), ureq (3 and later), sha2 (0.11 and later), indexmap (2.14 and later), and toml_edit (0.23 and later) are deliberately not adopted until an MSRV decision is made.

These are current project boundaries, not defects awaiting silent correction.

## 18. Maintainer / Contact

- Maintainer: [@Hollujay](https://github.com/Hollujay), re-verified live 2026-10-06 (`gh api users/Hollujay` resolves to an active GitHub user account).
- GitHub: https://github.com/Hollujay
- Telegram: [@Hollujay21](https://t.me/Hollujay21): the public preview page returned HTTP 200 on 2026-10-06 (page content not inspected in this pass; the 2026-09-25 check saw a named contact, "Telegram: Contact @Hollujay21"). This is a public-page check, not an authenticated API verification.

No other contact channel is published for this project (confirmed in README's "Maintainer & community" section).

## 19. Contribution

Contribution mechanism: [CONTRIBUTING.md](../CONTRIBUTING.md), [CODE_OF_CONDUCT.md](../CODE_OF_CONDUCT.md), a pull request template, and issue templates for bug reports, documentation, analysis rules, and dependency policy (all on `main`, added in PR #35), plus GitHub [Issues](https://github.com/SorobanLabs/sorobanlabs-analyzer/issues) and [Pull Requests](https://github.com/SorobanLabs/sorobanlabs-analyzer/pulls) on this repository. No separate community forum, Discord, or Slack is claimed; README explicitly states none exists yet.

## 20. Security

Security policy: [SECURITY.md](../SECURITY.md).

Summary of what is documented there: the analyzer treats all supplied WASM and state data as untrusted input; rejects malformed artifacts with structured errors rather than panicking; performs no uncontrolled filesystem writes; never submits a transaction or mutates network state; does not request, accept, or handle private keys; and private vulnerability reporting is enabled on this repository (re-verified live 2026-10-06, `{"enabled": true}`).

No third-party security audit, bug bounty program, or security certification is claimed anywhere in this repository, and none is claimed here.

## 21. License

Apache-2.0.

Verified: `LICENSE` file contains the full Apache License 2.0 text with "Copyright 2026 SorobanLabs"; `Cargo.toml`'s `license = "Apache-2.0"` (inherited by all 8 workspace crates); live GitHub repository metadata reports detected license `Apache-2.0`. All three agree.

## 22. Evidence Matrix

| Claim | Evidence | Verification | Status |
|---|---|---|---|
| Analyzer is a Rust CLI | `crates/analyzer-cli/src/cli.rs`, `Cargo.toml` | source inspection | PASS |
| Repository is public, org SorobanLabs, license Apache-2.0 | live GitHub API | live request, 2026-10-06 | PASS |
| Documentation site is live | GitHub Pages + HTTP response | live request, 2026-10-06 | PASS |
| v0.1.0 exists and is published | GitHub Release/tag API | live GitHub, 2026-10-06 | PASS |
| 227 tests pass | `cargo test --workspace` | local execution, 2026-10-06 | PASS |
| Formatting, clippy, build, MSRV build all pass | respective `cargo` commands | local execution, 2026-10-06 | PASS |
| Exit code 5 is tested | `cargo test -p analyzer-cli --lib analysis_stage_failure_maps_to_analysis_failure_exit_code` | local execution, 2026-10-06 | PASS |
| Real example output matches current behavior | rebuilt binary vs. checked-in README/JSON | local execution, 2026-10-06 (JSON compared semantically) | PASS |
| Branch ruleset active, PR + review + both checks required, no bypass | GitHub ruleset API | live GitHub, 2026-10-06 | PASS |
| Dependabot security updates / secret scanning enabled | GitHub security_and_analysis API | live GitHub, 2026-10-06 | FAIL (both disabled; documented as such, not claimed enabled) |
| Standalone repository (no companion repos) | GitHub org repos API | live GitHub, 2026-10-06 | PASS |
| Live application exists | repository-wide source search | source inspection | NOT APPLICABLE |
| Contract deployment exists | repository-wide source search | source inspection | NOT APPLICABLE |
| Transaction workflow exists | repository-wide source search | source inspection | NOT APPLICABLE |
| Demo video exists | repository and link check | repository inspection, 2026-10-06 | NOT YET AVAILABLE |
| Maintainer GitHub/Telegram contact valid | GitHub user API, Telegram public preview | live request, 2026-10-06 | PASS |
| Issue #12 open | GitHub Issues API | live GitHub, 2026-10-06 | PASS |
| Issue #14 open, includes sha2/indexmap/toml_edit findings | GitHub Issues API | live GitHub, 2026-10-06 | PASS |
| Blocked dependency PRs #31, #32, #34, #38, #39 closed with explanation | GitHub PR API | live GitHub, 2026-10-06 | PASS |
| Cargo.toml dependency comments match dependency graph (wasmparser, soroban-env-host, stellar-xdr) | `cargo tree -i wasmparser@0.116.1`, `@0.221.3`, `stellar-xdr` | local execution, 2026-10-06 | PASS (on branch) |
| CI green on `main` at `13c8417` | GitHub Actions run 37426062230 | live GitHub, 2026-10-06 | PASS |
| CI green on the final synchronization branch | GitHub Actions | pending until the pull request is opened | PENDING |

The "FAIL" row above is an accurate negative result for an optional GitHub security service, not a defect in this submission pack; it is recorded so the pack does not overstate repository security posture.

## 23. Related evidence files

`submission/evidence/demo-verification.md` and `submission/evidence/eligibility-verification.md` are dated 2026-09-25 records. They were not edited in this refresh and are historical: for example, `demo-verification.md` records `main` at `cdcfb70`. Nothing in them was rewritten to match later commits. They are point-in-time evidence: the Drips Stellar Wave window cited in the eligibility record (Sep 23 to Sep 30) has ended per its own stated dates, and that eligibility check was not repeated here.
