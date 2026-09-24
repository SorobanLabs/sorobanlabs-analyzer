# Example: reviewing a contract upgrade with a changed error type

This is a real, runnable example. `current.wasm` and `candidate.wasm` are
byte-for-byte copies of two real workspace test fixtures
(`fixtures/executable/v1.wasm` and `fixtures/executable/v2_changed_error.wasm`
respectively) -- they are not hand-crafted for this example. The command
below and its output were captured by actually running the built CLI
against these two files; nothing here is hypothetical.

## Command

```
cargo run -p analyzer-cli -- analyze \
  --current examples/upgrade-review/current.wasm \
  --candidate examples/upgrade-review/candidate.wasm
```

## Actual output

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

The canonical JSON form of this exact run is checked in as
[`report.json`](report.json) (generated with `--format json`, pretty-printed
for readability; the CLI itself emits compact JSON on one line).

## What these findings mean

- **`EXECUTABLE_HASH_CHANGED`** (`INFO`/`DETECTED`) -- the two WASM files
  are different. This always fires when the bytes differ; it carries no
  judgment on its own.
- **`CONTRACT_SIGNATURE_CHANGED`** (`HIGH`/`DETECTED`) -- the candidate's
  `cause_error` function returns a different type than the current
  version's. This is the actionable finding: any caller of `cause_error`
  that depends on the old return type needs to be reviewed before
  upgrading.
- **`CONTRACT_TYPE_CHANGED`** (`MEDIUM`/`DETECTED`) -- the shared
  `Error` enum itself changed shape between versions, which is *why*
  the signature above changed.
- **`STATE_COMPATIBILITY_UNKNOWN`** (`INFO`/`NOT_DETERMINABLE`) -- no
  `--migration-manifest` was supplied, and the analyzer does not
  reconstruct a contract's storage layout from raw WASM on its own, so
  it correctly reports that it cannot determine state compatibility
  rather than guessing.
- **`REHEARSAL_FAILED`** (`INFO`/`NOT_DETERMINABLE`) -- no `--rehearsal`
  manifest was supplied, so no real invocation was run against either
  executable; runtime behavior remains entirely unverified. This is not
  a failure of the tool, it is an accurate statement of what wasn't
  checked.

## Overall result

`status: REVIEW_REQUIRED` -- the analyzer detected a concrete,
`HIGH`-severity interface change (`cause_error`'s return type) that a
human should review before approving this upgrade. This is not a
SAFE/UNSAFE verdict; see the top-level README's confidence model for why.

## Where the generated report goes

Nothing is written to disk unless you redirect stdout yourself. To save
the JSON report shown above:

```
cargo run -p analyzer-cli -- analyze \
  --current examples/upgrade-review/current.wasm \
  --candidate examples/upgrade-review/candidate.wasm \
  --format json > report.json
```
