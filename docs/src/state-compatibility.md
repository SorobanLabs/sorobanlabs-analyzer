# State compatibility

Implemented in `analyzer-state`. The analyzer does not reconstruct a
contract's storage layout from raw WASM — that is not statically
recoverable in general. Instead, this stage works from two possible
signals:

- **No migration manifest supplied**: the stage reports
  `STATE_COMPATIBILITY_UNKNOWN` with confidence `NOT_DETERMINABLE`. This
  is the honest default — the analyzer has no basis to claim state
  compatibility either way.
- **A migration manifest supplied** (`--migration-manifest`): the
  manifest is an *author-supplied declaration*, not something the
  analyzer independently verified. If it declares a migration function
  or a schema change, the stage reports `MIGRATION_REQUIRED`. The
  finding detail and its evidence observation are both prefixed with
  `UNVERIFIED (author-supplied):` so a reader cannot mistake this for
  an established fact.

There is a separate, real, tested library module
(`analyzer_state::rpc`) for read-only Soroban RPC state access
(`getLedgerEntries`), but it is **not wired into the CLI**. Running
`analyze` never makes a network request. See
[Architecture](architecture.md#what-is-implemented-vs-library-only).
