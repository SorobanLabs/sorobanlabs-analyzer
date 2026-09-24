# Evidence and confidence

## Evidence

Every meaningful finding carries one or more evidence records. Each
evidence record has:

- `id` — a deterministic SHA-256 over the record's own content, so the
  same underlying fact always produces the same evidence id.
- `source` — what kind of thing was inspected: `local_artifact`,
  `custom_wasm_section`, `xdr_decoded_value`, `rpc_response`,
  `ledger_entry`, `execution_trace`, `rule_evaluation`,
  `migration_manifest`, or `derived_comparison` (a comparison across two
  or more other artifacts, identified by their hashes).
- `producer` — which parser, comparison, or rule produced this
  observation (for example `analyzer-executable::diff`).
- `location` — what location inside the source matters, if applicable
  (for example a function name), or `null`.
- `observation` — the bounded, human-readable fact this evidence
  establishes.

A finding's `evidence` array references these records by id, so a
reader can trace any finding back to exactly what was inspected to
produce it.

## Confidence

Every meaningful finding uses one of four confidence levels, reported
**separately** from severity:

- `DETECTED` — the analyzer directly established the condition from
  available evidence.
- `LIKELY` — available evidence strongly indicates the condition, but
  the analyzer cannot establish it with complete certainty.
- `POTENTIAL` — the change could create the condition, but available
  evidence is insufficient to establish the actual impact.
- `NOT_DETERMINABLE` — the analyzer does not have enough information to
  reach a meaningful conclusion. This is never treated as a negative
  finding — it means "unknown," not "bad."

## Severity

Independent of confidence: `INFO`, `LOW`, `MEDIUM`, `HIGH`, `CRITICAL`.
A `HIGH`/`NOT_DETERMINABLE` finding and a `HIGH`/`DETECTED` finding both
describe a potentially important condition, but with very different
certainty about whether it's actually true.
