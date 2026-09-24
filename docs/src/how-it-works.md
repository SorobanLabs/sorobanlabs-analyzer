# How it works

Given a current executable, a candidate executable, and optionally a
migration manifest and a rehearsal input, the `analyze` command runs a
fixed pipeline:

1. Load and hash both executables (`analyzer-executable`).
2. Extract and diff their contract interfaces — functions, parameters,
   return types, events, and types (`analyzer-executable`).
3. Evaluate state compatibility, using any supplied migration manifest
   as a declared-not-proven signal (`analyzer-state`).
4. Extract and compare each entrypoint's authorization surface: which
   authorization primitives, if any, it directly calls
   (`analyzer-auth`).
5. If a rehearsal manifest was supplied, run the same invocations
   against both executables under a real, bounded Soroban host backend
   and record observed behavioral differences (`analyzer-rehearsal`).
6. Attach evidence to every meaningful finding (`analyzer-evidence`).
7. Render the canonical JSON report, or a terminal rendering of the same
   data (`analyzer-report`).

Every step after step 1 can only add findings and evidence — it cannot
suppress a finding another step already produced. The overall
[status](status.md) is derived from the full set of findings, not
computed by one step alone. See [Analysis pipeline](analysis-pipeline.md)
for what each step establishes and does not establish.
