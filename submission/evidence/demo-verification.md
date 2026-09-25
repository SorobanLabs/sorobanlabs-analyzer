# Demo Verification

## Repository

SorobanLabs/sorobanlabs-analyzer

## Main SHA

`cdcfb709a7eda51a79eb51e62b94a5ac986fd679`

## Date

2026-09-25

## Real workflow

The demonstrated workflow is the analyzer's primary and only current
CLI workflow: given a current (deployed) contract executable and a
candidate replacement executable, determine what changed between them
across executable identity, contract interface, state compatibility,
and (when a rehearsal manifest is supplied) runtime behavior, and
produce an evidence-backed report with an explicit confidence level per
finding. No network access, no deployment, and no transaction
submission is part of this or any other current workflow.

This demonstration reuses `examples/upgrade-review/`, the repository's
own real, previously-verified example, rather than constructing a
second artificial one, because it already demonstrates the real
workflow end to end.

## Command

```
cargo run -p analyzer-cli -- analyze \
  --current examples/upgrade-review/current.wasm \
  --candidate examples/upgrade-review/candidate.wasm
```

Executed directly (via the built binary,
`./target/debug/sorobanlabs-analyzer analyze --current ... --candidate
...`) against `main` at the SHA above.

## Inputs

- `examples/upgrade-review/current.wasm` — byte-for-byte copy of the
  real workspace test fixture `fixtures/executable/v1.wasm`.
- `examples/upgrade-review/candidate.wasm` — byte-for-byte copy of the
  real workspace test fixture `fixtures/executable/v2_changed_error.wasm`.

Neither file is a deployed, production, testnet, or mainnet contract.
Both are local WASM files checked into this repository.

## Output

```
SorobanLabs Analyzer upgrade analysis report
schema version:   1.0.0
analyzer version: 0.1.0
protocol context: not specified

current executable:
  hash:        cbcdd56cf60eb516d89ea55103ad7e5182b71351f98a9498a8740b25a706cabe
  byte length: 939

candidate executable:
  hash:        d94fced32360e09f4081b36e834ef4a68cb3768d91b95e82b5f0d5e043bf88c5
  byte length: 1032

status: REVIEW_REQUIRED

rehearsal:
  requested:    false
  ran:          false
  backend:      (none)
  observed:     (none)
  not observed: (none)
  differences:  (none)
  unverified:   all execution behavior

findings (5):

[EXECUTABLE_HASH_CHANGED] [INFO/DETECTED] executable: executable
  summary: candidate executable bytes differ from the current executable
  detail:  current SHA-256 cbcdd56cf60eb516d89ea55103ad7e5182b71351f98a9498a8740b25a706cabe != candidate SHA-256 d94fced32360e09f4081b36e834ef4a68cb3768d91b95e82b5f0d5e043bf88c5
  evidence: a387f91aba594b49f0b5e5026066bcdf01e81930c7196e32816d6f648497f1e8

[CONTRACT_SIGNATURE_CHANGED] [HIGH/DETECTED] interface: cause_error
  summary: function output type changed
  detail:  'cause_error': return type changed
  evidence: cd58605ffba3e9d5db0f1ccb18502d5e2453de3d7950381db549320877fce9bc

[CONTRACT_TYPE_CHANGED] [MEDIUM/DETECTED] interface: Error
  summary: user-defined error enum type changed
  detail:  error enum 'Error' changed
  evidence: c90cdb740f635e004b7a79ba26a8cae7c2789c6b9fb2eac61d0377aeaefa9141

[STATE_COMPATIBILITY_UNKNOWN] [INFO/NOT_DETERMINABLE] state: state
  summary: state compatibility could not be determined from available evidence
  detail:  no migration manifest was supplied, and this analyzer does not reconstruct a contract's storage layout from raw WASM; static storage-access compatibility cannot be established from this evidence alone
  evidence: dcd43c90f3f8f8a4d1192ae0966f5b77ad04a28fbb071cf14f5f41bca52887e9

[REHEARSAL_FAILED] [INFO/NOT_DETERMINABLE] rehearsal: rehearsal
  summary: controlled rehearsal was not requested
  detail:  the analysis pipeline was run without a rehearsal manifest; no observation about runtime behaviour differences between the current and candidate executables was made
```

This is the exact, unedited terminal output of the command above,
re-run on 2026-09-25. It was diffed programmatically against the
"Actual output" block already checked into
`examples/upgrade-review/README.md` and matched exactly, character for
character. The canonical JSON form (`--format json`) was also re-run
and diffed as parsed JSON against the checked-in
`examples/upgrade-review/report.json`; it matched exactly, including
every evidence id (the evidence model is deterministic, so identical
inputs reproduce identical ids).

No discrepancy was found between the current implementation's output
and the previously checked-in example.

## Result

The analyzer correctly identified that the candidate executable is not
byte-identical to the current one, and specifically that the
`cause_error` function's return type changed and the `Error` error enum
changed, producing a `REVIEW_REQUIRED` overall status. Because no
migration manifest was supplied, state compatibility is reported as
`NOT_DETERMINABLE` rather than a guess in either direction. Because no
rehearsal manifest was supplied, no runtime-behavior observation was
made, and this is reported explicitly (`REHEARSAL_FAILED`,
`unverified: all execution behavior`) rather than silently omitted.
Every finding carries a confidence level (`DETECTED` or
`NOT_DETERMINABLE` here) and at least one evidence id resolvable in the
report's own `evidence` array.

## Evidence

- `examples/upgrade-review/README.md` — the pre-existing, now
  re-verified walkthrough.
- `examples/upgrade-review/report.json` — the pre-existing, now
  re-verified canonical JSON output.
- `examples/upgrade-review/current.wasm`, `candidate.wasm` — the actual
  input fixtures.
- `crates/analyzer-cli/src/cli.rs`, `commands.rs` — the real CLI
  argument surface and command implementation exercised by this run.
- `submission/submission-pack.md`, section 13 — the submission pack's
  own record of this example.

## Verification

This command was executed directly against a binary built from current
`main` at commit `cdcfb709a7eda51a79eb51e62b94a5ac986fd679` on
2026-09-25, not against a mock, stub, or previously cached output. The
workspace was also confirmed to build, format-check, and pass its full
227-test suite on the same commit immediately before this run (see the
Phase 33 report for the exact commands and results).

## Limitations

- This is a real local CLI demonstration, not a live Stellar network,
  testnet, or mainnet transaction demonstration. The analyzer does not
  submit transactions, does not require network access for this
  workflow, and did not use one here.
- No migration manifest or rehearsal manifest was supplied in this run,
  so state compatibility and runtime behavior are both correctly
  reported as not determined rather than as a positive or negative
  claim.
- Controlled rehearsal, when used, only captures execution outcome and
  return value; it does not observe events, state reads, state writes,
  or resource usage. This run did not invoke rehearsal at all.
- Authorization analysis was not exercised by this specific example
  (the fixtures used do not differ in authorization surface); it
  remains direct-call-only and never infers a principal or signer
  identity, as documented elsewhere in this repository.

## Demo video

Not yet recorded. No screen-recording capability (no `ffmpeg`,
`asciinema`, `termtosvg`, or equivalent tool) is available in the
environment this verification was performed in, and no video file was
produced. This CLI demonstration and its captured output are the
current evidence of the real workflow; a recorded video, if produced
later, is separate follow-up work.
