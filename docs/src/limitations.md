# Limitations

## What the analyzer does not do

- It does not produce a simplistic SAFE/UNSAFE verdict.
- It is not a generic smart contract security scanner or static
  vulnerability scanner.
- It does not duplicate protocol-wide compatibility checking
  responsibilities — that is a separate product's job.
- It does not automatically perform real contract upgrades, submit
  transactions, or execute migrations.
- It does not operate as a hosted service or web dashboard.
- It does not ask for or handle private keys.

## Specific, currently-true limitations

- **State compatibility**: the analyzer does not reconstruct a
  contract's storage layout from raw WASM. Without a migration
  manifest, state compatibility is reported as
  `STATE_COMPATIBILITY_UNKNOWN` (`NOT_DETERMINABLE`), not guessed. With
  one, the manifest is a declaration, not proof — see
  [State compatibility](state-compatibility.md).
- **Authorization surface**: only *direct* calls from an entrypoint's
  own function body are seen; calls through helper functions are not
  traced, and no principal/signer identity is ever inferred. "No direct
  call observed" is not the same as "unprotected" — see
  [Authorization analysis](authorization-analysis.md).
- **Controlled rehearsal**: only outcome and return value are captured
  today. Events, state reads/writes, and resource usage are not yet
  observed by the rehearsal backend — see
  [Controlled rehearsal](controlled-rehearsal.md).
- **8 of 23 `Rule` identifiers are defined but never produced** by the
  current pipeline (`StateSchemaChanged`, `RehearsalStateChanged`,
  `RehearsalEventChanged`, `RehearsalErrorChanged`,
  `RehearsalAuthorizationChanged`, `RehearsalResourceChanged`,
  `RehearsalObservationIncomplete`, `ResourceUsageChanged`), because the
  capabilities they'd report on (independent state-schema comparison;
  rehearsal event/state/resource observation) are not yet implemented.
  Tracked in
  [issue #12](https://github.com/SorobanLabs/sorobanlabs-analyzer/issues/12).
- **`analyzer_state::rpc` is library-only.** It is real, tested, and
  read-only, but not wired into the CLI — `analyze` never makes a
  network request. See
  [Architecture](architecture.md#what-is-implemented-vs-library-only).
- **`examples/`, `docs/`, `scripts/`, and `tests/` scaffold
  completeness** is tracked in
  [issue #13](https://github.com/SorobanLabs/sorobanlabs-analyzer/issues/13)
  and is not necessarily fully resolved at any given point in time —
  check that issue's current state rather than assuming completeness
  from this book's existence.

## Repository status

This repository is in early, active development. The analysis pipeline
described in this book and the `analyze` CLI command are implemented;
further CLI commands and report formats are added incrementally as they
are needed. No claim in this book should be read as "production ready,"
"fully complete," or "fully verified" beyond what is stated explicitly.
