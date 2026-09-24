# Installation

SorobanLabs Analyzer is not published to crates.io. Build it from
source.

## Prerequisites

- [rustup](https://rustup.rs/).
- The pinned toolchain declared in `rust-toolchain.toml`. Running
  `rustup show` from the repository root installs it automatically,
  including `rustfmt`, `clippy`, and the `wasm32v1-none` target.

No Soroban or Stellar CLI tooling is required for normal development
(building, testing, running `analyze`). The `stellar-cli` toolchain
listed in each fixture's README is only needed to reproduce the
checked-in WASM fixture artifacts.

## Build

```
git clone https://github.com/SorobanLabs/sorobanlabs-analyzer.git
cd sorobanlabs-analyzer
rustup show
cargo build --workspace
```

## Run

```
cargo run -p analyzer-cli -- analyze \
  --current path/to/current.wasm \
  --candidate path/to/candidate.wasm
```

See [CLI reference](cli.md) for the full command surface.
