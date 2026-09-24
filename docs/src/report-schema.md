# Report schema

The canonical JSON report is versioned independently of the analyzer's
own release version (currently schema version `1.0.0`). The full,
authoritative schema lives at
[`schemas/analysis-result.schema.json`](https://github.com/SorobanLabs/sorobanlabs-analyzer/blob/main/schemas/analysis-result.schema.json)
in the repository; this page summarizes it — the schema file is the
source of truth, not this page.

## Top-level fields

| Field | Type | Notes |
|---|---|---|
| `schema_version` | string | Currently the constant `"1.0.0"`. |
| `analyzer_version` | string | The analyzer release version that produced this report. |
| `protocol_context` | integer or `null` | The `--protocol` value, or `null` if not supplied. Recorded for context only — not yet compared against observed environment interface versions. |
| `current_executable` / `candidate_executable` | object | `{ hash, byte_length }` — `hash` is lowercase hex SHA-256. |
| `status` | string | See [Analysis status](status.md). |
| `rehearsal` | object or `null` | See below. `null` only for reports predating rehearsal reporting. |
| `evidence` | array | Every evidence record any finding references, in encounter order. |
| `findings` | array | See below. |

## `rehearsal`

Required fields: `requested`, `ran`, `backend_used`,
`observations_captured`, `observations_unavailable`,
`behavioral_differences_established`, `remains_unverified`. See
[Controlled rehearsal](controlled-rehearsal.md) for what these mean in
practice.

## `evidence[]`

Required fields: `id`, `source`, `producer`, `location`, `observation`.
`source.kind` is one of: `local_artifact`, `custom_wasm_section`,
`xdr_decoded_value`, `rpc_response`, `ledger_entry`, `execution_trace`,
`rule_evaluation`, `migration_manifest`, `derived_comparison`. See
[Evidence and confidence](evidence.md).

## `findings[]`

Required fields: `id`, `category`, `rule`, `subject`, `summary`,
`detail`, `severity`, `confidence`, `evidence`, `remediation`.

- `category` is one of: `executable`, `environment`, `interface`,
  `state`, `authorization`, `rehearsal`, `resource`.
- `rule` is one of the 23 identifiers declared in the schema (see the
  full list in the schema file). Not all 23 are currently reachable —
  see [Limitations](limitations.md) for exactly which 8 are not yet
  produced, and why.
- `severity` is one of `INFO`, `LOW`, `MEDIUM`, `HIGH`, `CRITICAL`.
- `confidence` is one of `DETECTED`, `LIKELY`, `POTENTIAL`,
  `NOT_DETERMINABLE`.
- `evidence` is an array of evidence record ids (see above).
- `remediation` is a string or `null` — not every finding carries a
  remediation suggestion.

## A real example

See the [Examples](examples.md) page, or
[`examples/upgrade-review/report.json`](https://github.com/SorobanLabs/sorobanlabs-analyzer/blob/main/examples/upgrade-review/report.json)
directly, for a complete, real report matching this schema exactly.
