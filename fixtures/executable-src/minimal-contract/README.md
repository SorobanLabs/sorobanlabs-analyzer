# minimal-contract

A real, minimal Soroban contract, used as the deterministic execution
fixture for `analyzer-rehearsal`'s host backend
(`crates/analyzer-rehearsal/src/host.rs`) and to sanity-check
`analyzer-executable`'s parsers against actual toolchain output.

This is not a workspace member (it has its own `[workspace]` table so
it resolves independently of the main workspace's dependency pins) and
is not built as part of `cargo build --workspace`. It exists so the
checked-in artifact can be reproduced and audited.

## Source

```rust
#![no_std]
use soroban_sdk::{contract, contractimpl, Env};

#[contract]
pub struct MinimalContract;

#[contractimpl]
impl MinimalContract {
    pub fn add(_env: Env, a: i32, b: i32) -> i32 {
        a + b
    }
}
```

## Toolchain used to build the checked-in artifact

- `soroban-sdk` `=28.0.0`
- `stellar-cli` `27.0.0` (`5a7c5fe76530bf4248477ac812fc757146b98cc4`)
- `rustc` `1.98.1` (`48a229cea 2026-09-01`)
- target `wasm32v1-none`

## Reproducing

```sh
stellar contract build --manifest-path fixtures/executable-src/minimal-contract/Cargo.toml --out-dir /tmp/out
sha256sum /tmp/out/minimal_contract.wasm
```

## Resulting artifact

- Path: `fixtures/executable/minimal-soroban-contract.wasm`
- SHA-256: `9ca7e87a2d06caf3100da345113e8acdb0c52fc821c6f697f2d9316bf64f8fce`
- Size: 479 bytes (optimized; `stellar contract build` reported the
  unoptimized size as 525 bytes)
- Exported functions: `add`

`stellar contract build` also embeds `contractmetav0` entries recording
the exact toolchain (`rsver`, `rssdkver`, `cliver`); those can be
inspected with `analyzer_executable::parse_contract_metadata` and are
consistent with the versions above.
