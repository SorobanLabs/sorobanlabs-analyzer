# rehearsal-corpus

A collection of minimal Soroban contract fixtures to exercise the `analyzer-rehearsal` subsystem. These are not in the main workspace to prevent them from interfering with standard compilation. They provide deterministic behavior for testing behavioral differences during upgrades.

## Fixtures included:
- `v1`: The baseline contract with baseline behaviors (add, cause_error, emit_event, set_state, auth_test).
- `v2_identical`: Functionally identical to `v1`.
- `v2_changed_return`: Changes the return value of `add`.
- `v2_changed_error`: Changes the error behavior of `cause_error`.
- `v2_changed_event`: Changes the emitted event value in `emit_event`.
- `v2_changed_state`: Changes the state value stored in `set_state`.
- `v2_fails`: Panics on all invocations to simulate a candidate that fails during rehearsal.

All fixtures have explicit `TESTED_LOCALLY` status.

## Toolchain used to build the checked-in artifacts

- `soroban-sdk` `=28.0.0`
- `stellar-cli` `27.1.0` (`8e402ea28202950b272fbabc34caad4d2f64fe87`)
- `rustc` `1.98.1` (`48a229cea 2026-09-01`)
- target `wasm32v1-none`

## Reproducing

```sh
stellar contract build --manifest-path fixtures/executable-src/rehearsal-corpus/Cargo.toml --out-dir fixtures/executable
```

## Expected Observable Outcomes (Current Host Limitations)

The host implementation currently captures execution success/failure/blocked outcomes and return values. Events, state changes, and authorization require further host integrations and will be reported as `NOT_OBSERVABLE` or empty until those host features are exposed.
