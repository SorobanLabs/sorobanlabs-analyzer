# Fixtures

Declarative test fixtures for the analyzer, organized by kind:

```
fixtures/
  executable/     WASM executables used in inspection/diff tests
  state/          local state snapshots
  authorization/  authorization-surface fixtures
  rehearsal/      controlled rehearsal fixtures (added with that subsystem)
```

Each fixture carries a manifest identifying its id, kind, description,
the current and candidate artifacts, and the expected findings, so
expected behavior is represented declaratively rather than hidden inside
test code. Fixtures are added alongside the test suites that exercise
them.
