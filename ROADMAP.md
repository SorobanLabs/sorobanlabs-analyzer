# Roadmap

## Current scope

The analyzer is a Rust CLI and library that reviews a proposed Soroban contract upgrade. It compares executable, interface, state, and authorization characteristics, runs a local rehearsal, and produces a canonical JSON report and a terminal report with the evidence behind each finding.

## Near-term maintenance

- Keep CI green.
- Keep dependencies compatible with the declared MSRV.
- Document currently reserved or unreachable rule identifiers (issue #12).
- Improve contributor templates and triage flow.

## Post-submission maintenance

- Expand real fixtures.
- Improve report documentation.
- Add more focused tests for edge cases.
- Continue tracking dependency updates that are blocked by MSRV (issue #14).

## Longer-term research

- Better state compatibility modeling.
- Richer rehearsal observations if the host backend can support them.
- More precise authorization analysis beyond direct calls, if it can be done without overstating confidence.

## Out of scope

- Hosted dashboard.
- Automatic contract upgrade execution.
- Private key handling.
- Generic vulnerability scanner.
- Protocol-wide compatibility checker.
