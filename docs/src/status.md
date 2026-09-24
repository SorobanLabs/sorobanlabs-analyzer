# Analysis status

The report's top-level `status` field is one of:

- `NO_DETECTED_BLOCKERS` — no finding rose to a level the pipeline
  treats as requiring review or migration.
- `REVIEW_REQUIRED` — at least one finding (for example an interface or
  authorization change) warrants human review before proceeding.
- `MIGRATION_REQUIRED` — a migration manifest declared a migration
  function or schema change, or the pipeline otherwise determined state
  migration is needed.
- `INCONCLUSIVE` — the pipeline could not reach a confident overall
  conclusion from the available findings.
- `ANALYSIS_ERROR` — the analysis pipeline itself could not complete
  (distinct from a CLI-level error; see the [CLI reference](cli.md) exit
  code table for how a failure to complete maps to a process exit code).

`status` is a summary of the full finding set, not a verdict computed by
any single pipeline stage. It is never a SAFE/UNSAFE judgment — see
[Limitations](limitations.md).
