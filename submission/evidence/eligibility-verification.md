# Drips Stellar Wave Eligibility Verification

## Previous Phase 34 assessment

SUPERSEDED

**Reason:** the earlier version of this file was produced before the
target program was identified. It evaluated Stellar Community Fund
(SCF) tracks and concluded BLOCKED on program identification. That
conclusion does not apply to the Drips Wave Program and must not be
read as the current result. The SCF research itself was accurate for
what it checked; it is simply the wrong program. It is preserved
verbatim in "Appendix: superseded SCF-based assessment" at the end of
this document for audit traceability, not deleted.

## Current assessment

Drips Wave Program, Stellar Wave — verified below.

## Verification date

2026-09-25

## Target program

Drips Wave Program, Stellar ecosystem Wave.

## Current Wave

**Stellar Wave 9.**

Source: [Stellar Wave](https://www.drips.network/wave/stellar) (live page, fetched 2026-09-25). The page states duration "Sep 23, 1:00 PM - Sep 30, 1:00 PM" and budget "$75000," and shows the Wave as "Active until September 30th."

Category: FACT (live official Drips source), not a playbook standard or inference. Year not printed on the page itself; read in the context of the current verification date (2026-09-25), which falls within the stated Sep 23-30 window.

## Official sources

| Source | URL | Verified | Relevant statement |
|---|---|---|---|
| Stellar Wave overview | https://www.drips.network/wave/stellar | 2026-09-25 | Wave 9, Sep 23-30, $75,000 budget, active until Sep 30 |
| Stellar Wave repository list | https://www.drips.network/wave/stellar/repos | 2026-09-25 | 824 approved repositories shown; no entry found for SorobanLabs or sorobanlabs-analyzer |
| Participating in a Wave | https://docs.drips.network/wave/maintainers/participating-in-a-wave/ | 2026-09-25 | Describes the sign-in, GitHub App install, sync, apply, and organizer-review process; states repositories must be public; no license, issue-structure, or demo-video requirement stated |
| Repo application limits | http://docs.drips.network/wave/maintainers/repo-application-limits/ | 2026-09-25 | Limits are per-Program-configurable, reset every Wave cycle, count against both a personal and an organizational pool simultaneously; exact current numeric limit not stated on this page |
| Drips Wave Terms and Rules | http://docs.drips.network/wave/terms-and-rules/ | 2026-09-25 | Maintainer eligibility: 18+/legal adult, no sanctioned/embargoed/AML-noncooperative jurisdiction, KYC/KYB by the natural person using the account; repository admission is at the sole discretion of the Association and Wave Program Organizer, no detailed repository criteria stated |
| Solving Issues & Earning Rewards | https://docs.drips.network/wave/contributors/solving-issues-and-earning-rewards/ | 2026-09-25 | No specific issue-eligibility criteria stated beyond a pointer to the Terms and Rules page for reward-formula/misbehavior adjustments |
| Lumen Loop: "What's New in Stellar Wave 6" (third-party changelog, not an official Drips domain) | https://lumenloop.com/news/changelog-new-stellar-wave-6 | 2026-09-25 (article published 2026-06-22) | States "repo applications are now capped at 5 per wave (per user and per org), and KYC is required to apply," introduced as of Wave 6; not independently re-confirmed on an official docs.drips.network page during this session, and no contradicting information was found either |

## Wave status

**ACTIVE.**

Category: FACT. Stellar Wave 9 is active per the live overview page, within its stated Sep 23-30 window, which contains the current verification date (2026-09-25).

Distinguishing note per this phase's own rule: "Wave active" is not the same fact as "repository applications are currently being accepted into this Wave." The fetched pages describe applications as an ongoing mechanism (apply, then await organizer review) rather than stating a separate applications-open/closed toggle distinct from the Wave's own active window. No evidence was found of applications being paused independent of the Wave's active status. This is recorded as UNKNOWN for the narrower question ("is the application mechanism itself currently open, as opposed to the Wave simply being active") rather than assumed identical to Wave-active status.

## Repository application status

**NOT FOUND** in the current approved Stellar Wave repository list.

Category: FACT, from a live source, with an explicit limitation. The [Stellar Wave repository list](https://www.drips.network/wave/stellar/repos) showed 824 approved repositories on 2026-09-25; neither "SorobanLabs" nor "sorobanlabs-analyzer" appeared among the entries surfaced by the fetch. This page is a JavaScript-rendered application; the fetch tool available in this session renders a snapshot but cannot guarantee it paginated or searched all 824 entries exhaustively via an in-page search box. A raw `curl` of the same URL (non-JS) returned no matching text either, which is expected for a client-rendered app and does not add independent confirmation.

Per instruction: **NOT FOUND does not mean rejected.** The three possibilities consistent with this evidence are: the repository has never been applied, an application exists but has not yet appeared in the public approved list (e.g., pending organizer review, which per the participating-in-a-wave page is a real intermediate state), or it was rejected. This evidence alone cannot distinguish between "never applied" and "pending," since only the *approved* list, not a pending-applications list, is publicly visible. Distinguishing these requires authenticated maintainer access (see "Organization onboarding status" below).

## Organization onboarding status

**UNKNOWN. Authenticated maintainer state required.**

Category: ACCOUNT-SPECIFIC UNKNOWN. This session has GitHub CLI access to the SorobanLabs organization's repository but no authenticated Drips account session. The following cannot be determined without logging into the Drips Wave app as a SorobanLabs-authorized maintainer:

- Whether the Drips Wave GitHub App is installed on the SorobanLabs organization
- Whether SorobanLabs is visible/onboarded in the Drips maintainer interface
- Whether `sorobanlabs-analyzer` has been synced as an available repository
- Whether an application to Stellar Wave 9 (or any prior Wave) already exists for this repository and its current status (none, pending, approved, rejected)
- Remaining per-user/per-organization application capacity for the current cycle

None of these are reported as PASS or FAIL. They are UNKNOWN pending authenticated access, per instruction.

## Current repository approval status

**NOT FOUND / UNKNOWN**, per the two sections above. Not APPROVED (absent from the public approved list), not confirmably REJECTED (no rejection evidence found and none is inferable from absence), not confirmably PENDING (would require authenticated access to verify).

## Current application limits

**UNKNOWN**, exact current numeric cap not independently confirmed on an official page during this session.

What is confirmed from official sources: limits are configured per Wave Program, apply per-user and per-organization simultaneously (whichever is exhausted first), and reset at the start of every Wave cycle. A third-party changelog (lumenloop.com, June 2026, describing Wave 6 changes) states a cap of "5 per wave (per user and per org)" and that KYC is required to apply; this is reported as a secondary-source finding, not verified against an official Drips page in this session. Actual current remaining capacity for the SorobanLabs account specifically is UNKNOWN, authenticated Drips account access required.

## Maintainer eligibility

| Requirement | Official source | Type | Status |
|---|---|---|---|
| 18 years old or legal adult in country of residence | Terms and Rules | B. Official Drips requirement | REQUIRES USER ATTESTATION |
| Not a citizen/resident/located in a comprehensively sanctioned or embargoed jurisdiction | Terms and Rules | B. Official Drips requirement | REQUIRES USER ATTESTATION |
| Not listed on, owned, or controlled by an entity on SECO/EU/US Commerce/Treasury/State sanctions lists | Terms and Rules | B. Official Drips requirement | REQUIRES USER ATTESTATION |
| Not resident/citizen/located in an AML-noncooperative jurisdiction | Terms and Rules | B. Official Drips requirement | REQUIRES USER ATTESTATION |
| Not domiciled in/organized under laws conflicting with the Drips Wave Programs | Terms and Rules | B. Official Drips requirement | REQUIRES USER ATTESTATION |
| KYC/KYB completed by the natural person actually using the account | Terms and Rules | B. Official Drips requirement | REQUIRES USER ATTESTATION (and ACCOUNT-SPECIFIC UNKNOWN whether it has actually been completed) |

None of the above are inferred from conversation context, nationality assumptions, or any other indirect source, per instruction. Each requires the maintainer's own attestation and/or completion of the Drips KYC/KYB flow.

## Repository requirements

| Requirement | Official source | Type | Current project state | Status |
|---|---|---|---|---|
| Repository must be public | Participating in a Wave | B. Official Drips requirement | Public (verified live, `visibility: public`) | PASS |
| GitHub organization onboarded, Drips Wave GitHub App installed | Participating in a Wave | B. Official Drips requirement | UNKNOWN (account-specific, see above) | UNKNOWN |
| Repository explicitly applied to the Stellar Wave Program | Participating in a Wave | B. Official Drips requirement | Not found in approved list; application state otherwise unknown | UNKNOWN |
| Repository admission at sole discretion of Association/Organizer | Terms and Rules | B. Official Drips requirement | Not something the repository can satisfy in advance; an organizer decision | NOT APPLICABLE to a repository-state check |
| Apache-2.0 license | Project fact, not a cited Drips requirement | A. Fact only | Apache-2.0, verified | PASS as a fact; not claimed as a Drips-mandated requirement, since no official source states a required license |
| Project banner, badges, contributor-credit artifact | Updated Drips-specific playbook (project standard) | C. Recommended/project-standard practice, not found stated in any official Drips source | Banner and badges present in README (see Phase 31 work); no dedicated contributor-credit artifact exists | C-level item; not scored against official eligibility since no official source requires it |

## Issue requirements

| Requirement | Official source | Type | Current project state | Status |
|---|---|---|---|---|
| Issue scope/quality/duplicate-prevention criteria for Wave inclusion | Solving Issues & Earning Rewards, Terms and Rules | Searched; no explicit official criteria found beyond a general pointer to reward-formula/misbehavior adjustments in Terms and Rules | Issues #12 and #14 are open, real, technically scoped, non-duplicate (independently re-verified in Phase 30-34) | UNKNOWN (no explicit official rule found to check against); project-level facts about #12/#14 remain PASS on their own terms |
| Issue complexity/points assignment (Trivial/Medium/High = 100/150/200 points) | Search-derived summary of Drips issue/points documentation | B. Official Drips mechanism, but this is a **post-approval maintainer action**, not a repository-application-time requirement | Neither #12 nor #14 has been assigned a complexity level or added to a Wave, since the repository itself has not been confirmed approved | NOT APPLICABLE until repository approval status is resolved |

Neither issue was modified, rewritten, or closed as part of this verification.

## Demo requirement

**Demo video: RECOMMENDED (project standard), NOT REQUIRED (official Drips repository-application rule, as far as found).**

Category breakdown, per this phase's explicit rule not to merge authorities:

- **A. Fact:** the project has a real, working CLI, real local WASM fixtures, a real captured execution, and a real JSON report; no public demo video exists.
- **B. Official Drips requirement:** none of the official pages fetched in this session (Stellar Wave overview, repo list, Participating in a Wave, Repo Application Limits, Terms and Rules, Solving Issues & Earning Rewards) mention a demo video as a repository-application requirement, optional item, or recommended item. A live web search for "Drips Wave Stellar repository application demo video requirement" likewise returned no official statement of a video requirement.
- **C. Recommended practice:** the *project's own* updated Drips-specific playbook (not an official Drips document) states a short demo video is a critical supporting submission element and that a submission without one "reads as unfinished." This is the project's internal quality bar, not a verified official Drips gate.
- **D. Project limitation:** no video currently exists (`submission/evidence/demo-verification.md`).

Because no official Drips source states a demo video is required to apply a repository, this is classified **NOT REQUIRED** for the official application step, and **RECOMMENDED** as this project's own internal standard. `submission/submission-pack.md`'s "Demo video: Not yet recorded" statement is preserved unchanged; it is not converted to BLOCKED, since the official gate found does not make it mandatory. This finding does not extend to whatever a specific Wave Program organizer might separately request during review, which is unverifiable in advance.

## Live application requirement

**NOT REQUIRED** (no official source found requiring one) / **NOT APPLICABLE** to current Analyzer architecture.

No official Drips page fetched in this session states that a repository applying to a Wave must have a deployed application. The current Analyzer has no deployed frontend, backend, live application, or public API (re-confirmed by repository-wide source search). The GitHub Pages documentation site (https://sorobanlabs.github.io/sorobanlabs-analyzer/) is documentation hosting, not an application, and is not presented as one here.

## Contract requirement

**NOT REQUIRED** (no official source found requiring one) / **NOT APPLICABLE**.

No official Drips page found states a repository must have a deployed Soroban contract, contract ID, or explorer link to apply. SorobanLabs Analyzer analyzes contract executables supplied as local files; it does not itself deploy or own a contract. No contract ID, network name, or explorer URL is claimed here or elsewhere in this repository.

## Transaction requirement

**NOT REQUIRED** (no official source found requiring one) / **NOT APPLICABLE**.

No official Drips page found states a repository must demonstrate a live network transaction to apply. The analyzer never submits a transaction (re-confirmed by repository-wide source search). No transaction hash, Testnet, or Mainnet evidence is claimed here.

## Documentation requirement

No official Drips page fetched in this session states specific documentation content requirements for Wave repository applications (beyond the repository being public). The updated project-specific playbook's documentation expectations are recorded here as project standard (Category C), verified against actual current content:

| Section | Status | Evidence |
|---|---|---|
| Introduction | IMPLEMENTED | `docs/src/introduction.md`, live at https://sorobanlabs.github.io/sorobanlabs-analyzer/introduction.html (re-verified HTTP 200, 2026-09-25) |
| Problem | IMPLEMENTED | `docs/src/problem.md` |
| How it works | IMPLEMENTED | `docs/src/how-it-works.md` |
| Architecture | IMPLEMENTED | `docs/src/architecture.md` |
| Lifecycle/state mechanics | NOT APPLICABLE | This is a stateless CLI analysis tool, not a stateful on-chain system; `docs/src/state-compatibility.md` covers the actual state-compatibility *analysis capability*, which is the relevant analog |
| End-user guidance | IMPLEMENTED | `docs/src/cli.md`, `docs/src/installation.md` |
| Developer setup | IMPLEMENTED | `docs/src/installation.md`, `CONTRIBUTING.md` |
| Environment variables | NOT APPLICABLE | No application environment variables exist (verified by repository-wide search, Phase 25) |
| SDK/API reference | NOT APPLICABLE | No SDK or network-facing API exists; the CLI itself is documented in `docs/src/cli.md` and the report schema in `docs/src/report-schema.md` |
| Testing | IMPLEMENTED | `docs/src/testing.md` |
| Contributing | IMPLEMENTED | `docs/src/contributing.md`, `CONTRIBUTING.md` |
| Security and limitations | IMPLEMENTED | `docs/src/security.md`, `docs/src/limitations.md`, `SECURITY.md` |

Live site re-verified 2026-09-25: root page and `introduction.html`, `cli.html`, `limitations.html` all HTTP 200 (same pages checked in Phase 32, re-checked here rather than assumed unchanged).

## Current project state

Re-verified live/fresh on 2026-09-25, not copied from prior reports:

| Fact | Verification | Result |
|---|---|---|
| Repository | SorobanLabs/sorobanlabs-analyzer | confirmed |
| Organization | SorobanLabs | confirmed |
| Visibility | `gh api .../repos/...` → `visibility: public` | public |
| License | `gh api .../license` → `spdx_id: Apache-2.0` | Apache-2.0 |
| Release | `gh api .../releases/tags/v0.1.0` → `draft: false, prerelease: false` | v0.1.0, published |
| Documentation | `curl` → HTTP 200 | https://sorobanlabs.github.io/sorobanlabs-analyzer/ |
| Real example | `examples/upgrade-review/` present on `main` | present |
| Real demo evidence | `submission/evidence/demo-verification.md` present on `main` | present |
| Demo video | no video file or link anywhere in the repository | not recorded |
| Live application | repository-wide source search, no listener/server code | none |
| Deployed contract | repository-wide source search, no deployment code | none |
| Transaction workflow | repository-wide source search, no transaction code | none |
| Open issues | `gh api .../issues/12`, `/14` → both `state: open` | #12 and #14, both open |

## Current submission materials

| Item | Current state | Status |
|---|---|---|
| Public GitHub repository | Yes | PASS |
| GitHub organization onboarded to Drips | Unknown | UNKNOWN |
| Drips Wave GitHub App installed | Unknown | UNKNOWN |
| Repository synced/selectable in Drips | Unknown | UNKNOWN |
| Repository applied to Stellar Wave 9 | Not found in approved list; application state otherwise unknown | UNKNOWN |
| Maintainer KYC/KYB completed | Unknown | UNKNOWN |
| Maintainer eligibility attestations (age, sanctions, jurisdiction) | Not evaluable by this audit | REQUIRES USER ATTESTATION |
| Project description | Present, accurate | PASS |
| Source code, license | Present, Apache-2.0 | PASS |
| Documentation | Live mdBook site, verified | PASS |
| Demo video | Not recorded; not an official requirement found | NOT YET AVAILABLE (not blocking) |
| Release | v0.1.0 published | PASS |
| Open issues suitable for Wave inclusion | #12, #14 open and real; no official quality gate found to check against | UNKNOWN vs. official rule / PASS on project-level facts |

## Link verification

All checked live 2026-09-25:

| Link | Result |
|---|---|
| https://github.com/SorobanLabs/sorobanlabs-analyzer | HTTP 200 |
| https://sorobanlabs.github.io/sorobanlabs-analyzer/ | HTTP 200 |
| https://github.com/SorobanLabs/sorobanlabs-analyzer/releases/tag/v0.1.0 | HTTP 200 |
| https://github.com/SorobanLabs/sorobanlabs-analyzer/issues/12 | HTTP 200 |
| https://github.com/SorobanLabs/sorobanlabs-analyzer/issues/14 | HTTP 200 |
| https://github.com/Hollujay | HTTP 200 |
| https://www.drips.network/wave/stellar | HTTP 200 (fetched via WebFetch, content confirmed) |
| https://www.drips.network/wave/stellar/repos | HTTP 200 (fetched via WebFetch, content confirmed) |
| https://docs.drips.network/wave/maintainers/participating-in-a-wave/ | HTTP 200 (after 301 redirect from the non-trailing-slash URL) |

## Current blockers

None that prevent *reading and verifying* the current rules. The following prevent a conclusive PASS/FAIL on application status specifically:

- Account-specific Drips state (organization onboarding, GitHub App installation, prior application status, KYC/KYB completion) cannot be determined without authenticated Drips maintainer access, which this session does not have.

## Current unknowns

- Whether SorobanLabs is onboarded to Drips and has the Wave GitHub App installed.
- Whether `sorobanlabs-analyzer` has ever been applied to any Stellar Wave, and if so its exact status (pending/rejected/withdrawn) versus simply never having been applied.
- The exact current numeric repository-application limit for Stellar Wave 9 specifically (a secondary source states 5 per user/org as of Wave 6; not officially re-confirmed for Wave 9).
- Whether the narrower "applications currently being accepted" state is distinct from "Wave 9 is active" (evidence found describes an ongoing apply-then-review mechanism, not a separate toggle).
- Whether the maintainer (Hollujay) meets the age/jurisdiction/sanctions/AML eligibility criteria and has completed KYC/KYB — these require the maintainer's own attestation, not something this audit can determine.

## Exact next action

The maintainer must log into the Drips Wave app (https://www.drips.network) with the GitHub account authorized for the SorobanLabs organization, confirm or complete organization onboarding and Drips Wave GitHub App installation, and check the Maintainers dashboard for `sorobanlabs-analyzer`'s actual current application status (never applied / pending / approved / rejected) before any further submission action is taken. This is an action only the authenticated maintainer can perform; it cannot be completed or simulated by this audit.

## Eligibility status

UNKNOWN

This reflects that no official Drips rule found blocks this repository from being eligible to apply (repository is public, which is the one stated repository-level requirement found; no demo, deployment, contract, or transaction requirement was found), but the actual current application/approval state cannot be verified without authenticated account access, and the maintainer's own personal eligibility attestations (age, jurisdiction, KYC/KYB) have not been made. This is not BLOCKED in the sense of "known to be disqualified," and it is not PASS in the sense of "confirmed approved" or "confirmed applied" — it is UNKNOWN pending the specific account-side actions listed above.

---

## Appendix: superseded SCF-based assessment (2026-09-25, original Phase 34 pass)

Preserved verbatim for audit traceability. This assessment targeted the
Stellar Community Fund, which has since been established as the wrong
program for this submission. Do not treat any conclusion below as
current.

> **Program**
>
> BLOCKED — exact target program not identified.
>
> No document in this repository (README.md, CONTRIBUTING.md,
> SECURITY.md, submission/submission-pack.md,
> submission/evidence/demo-verification.md, or any prior audit
> document) names a specific Stellar program, award, or submission
> process this project is intended for. "Stellar Project Approval
> Playbook" is not the name of any official Stellar Development
> Foundation program found during this research.
>
> Fresh research found multiple real, current, but distinct Stellar
> Community Fund (SCF) tracks (SCF Build Award: Open/Integration/RFP
> tracks; SCF Public Goods Award), each with materially different
> rules, and no evidence indicating which, if any, applied.
>
> **Eligibility status: BLOCKED**

The remainder of the original SCF-based tables (project-specific
verification, demo requirement research, repository status, link
verification, submission materials checklist) are superseded in full
by the Drips-specific sections above and are not reproduced again
here; they remain available in this repository's git history at
commit `6ff5178` for anyone who needs the original text.
