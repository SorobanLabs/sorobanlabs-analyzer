# Scripts

There are currently no development or CI helper scripts in this
repository. Everything a contributor needs is a plain `cargo` command,
documented in [CONTRIBUTING.md](../CONTRIBUTING.md):

```
cargo fmt --all --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
```

CI (`.github/workflows/ci.yml`) runs these same commands directly; it
does not invoke any script from this directory.

If a real helper script (for example, one that regenerates the checked-in
WASM test fixtures) becomes genuinely useful, it will be added here. This
file will be updated at that point; until then, this directory is
correctly empty.
