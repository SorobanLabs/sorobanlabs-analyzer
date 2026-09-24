# Controlled rehearsal

Implemented in `analyzer-rehearsal`. Optional — only runs when
`--rehearsal <PATH>` is supplied.

A rehearsal takes a set of invocations and runs each one against both
the current and candidate executable under a real, bounded
`soroban-env-host` backend, then compares what was observed.

## What is actually captured today

The backend currently captures **outcome and return value only**:
success, error, trap, or `Blocked` (an invocation that could not be
safely or meaningfully executed). It does **not** currently capture
events, state reads/writes, or resource usage — these are represented
as `NotObservable` rather than compared, and are unconditionally listed
in the report's `remains_unverified` field alongside `authorization`.

A panic or trap inside the candidate is caught by the backend's own
panic-catching wrapper and reported as an observation, not propagated
as a crash of the analyzer process. Execution is subject to a
caller-configurable (otherwise analyzer-defaulted) CPU/memory budget,
and never touches real network state.

## Possible findings

| Rule | Fires when |
|---|---|
| `REHEARSAL_FAILED` | No rehearsal manifest was supplied — this is not an error, it's the "rehearsal was skipped" case, reported so the report always states plainly that runtime behavior is unverified. |
| `REHEARSAL_RESULT_CHANGED` | An invocation's outcome or return value differs between current and candidate. |

Several other rehearsal-related rule identifiers exist in the schema
(`RehearsalStateChanged`, `RehearsalEventChanged`,
`RehearsalErrorChanged`, `RehearsalAuthorizationChanged`,
`RehearsalResourceChanged`, `RehearsalObservationIncomplete`,
`ResourceUsageChanged`) but are **not currently produced** by the
pipeline, because the backend does not yet observe the corresponding
categories. See [Limitations](limitations.md).
