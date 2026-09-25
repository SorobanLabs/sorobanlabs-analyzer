# Eligibility Verification

## Verification date

2026-09-25

## Program

**BLOCKED — exact target program not identified.**

No document in this repository (`README.md`, `CONTRIBUTING.md`,
`SECURITY.md`, `submission/submission-pack.md`,
`submission/evidence/demo-verification.md`, or any prior audit
document) names a specific Stellar program, award, or submission
process this project is intended for. Nothing in this project's own
history establishes it either. "Stellar Project Approval Playbook" is
not the name of any official Stellar Development Foundation program
found during this research; live web search for that exact phrase
returned no matching official document.

Fresh research (2026-09-25) found multiple real, current, but distinct
Stellar Community Fund (SCF) tracks, each with materially different
rules, and no evidence indicating which (if any) applies here:

- **SCF Build Award** (Open Track, Integration Track, RFP Track): a
  funding application for *future* development, structured around
  three funding tranches tied to deliverables ("ready to begin
  development as soon as the award is granted"). This does not
  obviously match a project that is already built and functioning.
- **SCF Public Goods Award**: currently invite-only during a
  "Soft-Launch" stage, requires SCF verified membership, KYC and due
  diligence, and requires the project have no existing/planned revenue
  model. Materially different eligibility from Build.

Per this phase's explicit instruction, this is recorded as BLOCKED
rather than guessed. **A human decision is required**: which program,
if any, is the actual submission target. Everything below that depends
on the specific program's rules is scoped accordingly.

## Official sources

- [Welcome to the SCF Handbook](https://stellar.gitbook.io/scf-handbook) — general SCF process overview, referenced "SCF 7.0," rounds "SCF #43"/"SCF #44," no explicit current window status on this page.
- [SCF Build Award: Submission Criteria](https://stellar.gitbook.io/scf-handbook/scf-awards/build-award/submission-criteria) — Build Award eligibility and submission requirements.
- [SCF Public Goods Award: Official Rules](https://stellar.gitbook.io/scf-handbook/supporting-programs/public-goods-award/official-rules) — Public Goods Award eligibility, currently invite-only/soft-launch.
- [Stellar Community Fund site](https://communityfund.stellar.org/) — lists "Instawards," "Build" (up to $150,000 XLM across 3 award tracks), and "Grow" pathways; this page is JavaScript-rendered and did not yield current round/deadline data through static fetch.
- [Stellar Community Fund: Submission Best Practices](https://stellar.org/blog/ecosystem/stellar-community-fund-soroban-submission-best-practices) — general submission guidance blog post; no publication date shown on the fetched content.
- [Stellar grants and funding](https://stellar.org/grants-and-funding) — top-level entry point, not individually fetched in depth beyond search-result summary.

All fetched live 2026-09-25.

## Submission window

**UNKNOWN.** The SCF handbook references specific numbered rounds
(e.g., "SCF #43," "SCF #44") but the fetched pages did not state
whether a round is currently open, its opening/closing dates, or
timezone. The live submission site (`communityfund.stellar.org`) is a
JavaScript-rendered application; static content fetching did not
surface the current round status. This would need either a program
confirmation (to know which application to check) or a
JavaScript-capable browser check of the live site, neither of which
was available to definitively resolve in this environment.

## Eligibility requirements

Because the target program is unconfirmed (BLOCKED above), no single
authoritative requirement list applies. The two most relevant
candidate SCF tracks found are recorded here for reference, not as
confirmed applicable rules:

**SCF Build Award (Open Track), as stated in its submission-criteria page:**

| Requirement | Evidence | Current project state | Status |
|---|---|---|---|
| Participant/submission eligibility requirements met | "You must meet all Participant and Submission Eligibility Requirements..." | Not independently verifiable without the full Participant Eligibility page (not fetched) | UNKNOWN |
| Product-market fit / validated need | Significant user traction or validated need required | This is a completed, functioning tool with 0 GitHub stargazers and no reported external users | UNKNOWN — depends on reviewer judgment, not a fact this audit can settle |
| Team ready to begin development immediately | Tranche-based funding model implies proposing *future* work | This project's core implementation is already built, not proposed future work | Mismatch noted, not scored |
| Clear use case for Stellar, technical architecture outline | Required in submission | `README.md`, `docs/src/architecture.md`, `submission/submission-pack.md` provide this | PASS (if this track applies) |
| Smart-contract projects must have a clear open-source plan | Required where applicable | Not applicable: this project analyzes contracts, it does not ship one; already Apache-2.0 licensed and public | NOT APPLICABLE / PASS |
| Entities funded through Matching Fund or Enterprise Fund are ineligible | Stated restriction | No evidence this project has received such funding | PASS (absence, not independently verified against an SDF funding registry) |

**SCF Public Goods Award, as stated in its official-rules page:**

| Requirement | Evidence | Current project state | Status |
|---|---|---|---|
| Submitter must be an SCF verified member, Pilot role or higher | Stated requirement | Unknown; not established anywhere in this repository or conversation | UNKNOWN |
| Program currently invite-only (Soft-Launch) | Stated on the official-rules page | No invitation evidence exists in this repository | BLOCKED if this is the intended track |
| Project must not have an existing/planned revenue model | Stated requirement | This project has no revenue model of any kind (open-source tool, no monetization anywhere in source or docs) | PASS (if this track applies) |
| No currently active SDF grant funding | Stated requirement | No evidence of any SDF grant funding anywhere in this repository | PASS (absence, not independently verified against an SDF funding registry) |

## Project-specific verification

Current, freshly re-verified facts about SorobanLabs/sorobanlabs-analyzer, independent of which program applies:

| Fact | Verification | Status |
|---|---|---|
| Public GitHub repository | `gh api repos/.../` → `private: false`, `archived: false`, `disabled: false` | PASS |
| Apache-2.0 license | `LICENSE` file + `Cargo.toml` + GitHub-detected license, all agree | PASS |
| Rust implementation | `Cargo.toml`, `crates/` (8 crates) | PASS |
| Soroban-focused analyzer | `README.md`, `stellar-xdr`/`soroban-env-host` dependencies | PASS |
| Standalone repository | `gh api orgs/SorobanLabs/repos --paginate` → exactly 1 repository | PASS |
| Live documentation | `https://sorobanlabs.github.io/sorobanlabs-analyzer/` → HTTP 200 | PASS |
| v0.1.0 release | `gh api .../releases/tags/v0.1.0` → published, not draft/prerelease | PASS |
| Real CLI example | `examples/upgrade-review/`, re-verified Phase 33, byte-for-byte match | PASS |
| Real CLI demonstration | `submission/evidence/demo-verification.md`, re-run 2026-09-25 | PASS |
| No deployed web application | repository-wide source search, no listener/server code | NOT APPLICABLE (project category) |
| No deployed contract workflow | repository-wide source search, no deployment code | NOT APPLICABLE (project category) |
| No transaction submission workflow | repository-wide source search, no transaction code | NOT APPLICABLE (project category) |
| No public demo video | `submission/evidence/demo-verification.md` | NOT YET AVAILABLE |
| Issue #12 open | `gh api .../issues/12` → `state: open`, verified live | PASS |
| Issue #14 open | `gh api .../issues/14` → `state: open`, verified live | PASS |

## Demo requirement

**UNKNOWN**, pending program confirmation, with the following general finding on record: the general SCF submission best-practices guidance ([stellar.org blog post](https://stellar.org/blog/ecosystem/stellar-community-fund-soroban-submission-best-practices), no publication date shown) states applicants should "showcase any code you already have on GitHub or **in a demo video**," phrasing a demo video as one optional way to demonstrate work, alongside GitHub code, not as a mandatory item. No official SCF page found in this research explicitly states a demo video is a hard submission requirement. Some already-approved SCF projects have video requirements as post-award *milestone completion* criteria (e.g., proving a specific on-chain transaction happened), which is a different thing from an initial-submission requirement and does not apply to this project's current status.

This is recorded as UNKNOWN, not PASS, because the exact application form that would actually govern this submission has not been identified (see "Program" above). The submission pack's "Demo video: Not yet recorded" statement is not altered on the basis of this finding, per instruction not to infer "optional" as "required."

## Repository status

Verified live 2026-09-25:

| Check | Result |
|---|---|
| Repository exists | PASS |
| Repository is public | PASS |
| Default branch | `main` |
| Archived | No (PASS) |
| Disabled | No (PASS) |
| Issues enabled, 2 open (#12, #14) | PASS |
| Release v0.1.0 published | PASS |
| Documentation site live | PASS |
| Branch ruleset active (PR + 1 review + 2 required checks, no bypass, force-push/deletion blocked) | PASS |
| Open PRs | 0 (none currently open) |
| Unpublished local changes | None; local `main` == `origin/main` at time of this check |

## Submission limits

**UNKNOWN.** No per-person, per-organization, concurrent-project, or duplicate-submission limit was found stated on any SCF page fetched during this research. This does not mean no such limit exists; it means this research did not locate one on the pages actually reachable. Resolving this fully would require identifying the specific program (see "Program" above) and reading its complete official rules page, which was not conclusively identified.

## Approval / duplication status

Public approval registry not found. No publicly browsable, current registry of "already-approved SCF projects" or "already-submitted applications" was located during this research that could be queried for "SorobanLabs Analyzer" or "SorobanLabs" by name. This audit does not claim "project has never been approved"; it states only that no public registry was found to check against.

## Link verification

All checked live 2026-09-25 via direct HTTP request:

| Link | Result |
|---|---|
| https://github.com/SorobanLabs/sorobanlabs-analyzer | HTTP 200 |
| https://github.com/SorobanLabs/sorobanlabs-analyzer/releases/tag/v0.1.0 | HTTP 200 |
| https://sorobanlabs.github.io/sorobanlabs-analyzer/ | HTTP 200 |
| https://github.com/Hollujay | HTTP 200 |
| https://github.com/SorobanLabs/sorobanlabs-analyzer/issues/12 | HTTP 200 |
| https://github.com/SorobanLabs/sorobanlabs-analyzer/issues/14 | HTTP 200 |
| https://t.me/Hollujay21 | HTTP 200 (public preview page; not an authenticated check) |
| https://communityfund.stellar.org/ | HTTP 200 |
| https://stellar.gitbook.io/scf-handbook | HTTP 200 |

No official program submission portal URL is included above as "the correct current submission entry point," because the specific program (and therefore its specific application URL) is not confirmed.

## Submission materials checklist

Based on `submission/submission-pack.md` content and generally-observed SCF submission guidance (not a confirmed program-specific list, since the program is unconfirmed):

| Item | Current project state | Status |
|---|---|---|
| Repository | Public, Apache-2.0, standalone | PASS |
| Project description | Present in README and submission pack | PASS |
| Source code | Present, builds, 227 tests pass | PASS |
| License | Apache-2.0, consistent everywhere checked | PASS |
| Documentation | README + live mdBook site | PASS |
| Demo (video) | Not yet recorded | NOT YET AVAILABLE |
| Deployment | None; not applicable to this project category | NOT APPLICABLE |
| Contract IDs | None; not applicable | NOT APPLICABLE |
| Transaction links | None; not applicable | NOT APPLICABLE |
| Release | v0.1.0 published | PASS |
| Applicant/contact details | Maintainer GitHub + Telegram published and verified | PASS |
| Issue summary | #12 and #14 documented accurately | PASS |
| Limitations | Documented in README, docs site, and submission pack | PASS |

## Outstanding blockers

- **Exact target Stellar program not identified.** Without this, submission window status, the complete applicable eligibility rule set, submission limits, and the correct submission portal URL cannot be conclusively verified. This is the single blocking gap in this verification.

## Eligibility status

BLOCKED
