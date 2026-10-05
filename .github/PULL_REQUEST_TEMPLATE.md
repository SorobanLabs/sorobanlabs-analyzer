# Summary

Describe the change in a few sentences.

# Verification

List the commands you ran and their results.

- [ ] `cargo fmt --all --check`
- [ ] `cargo clippy --workspace --all-targets -- -D warnings`
- [ ] `cargo test --workspace`
- [ ] `cargo +1.84.0 build --workspace` if the change may affect MSRV

# Scope and limitations

State what this PR intentionally does not change.

# Documentation

- [ ] README updated, if user-visible behavior changed
- [ ] docs updated, if analyzer behavior or limitations changed
- [ ] schema or examples updated, if report output changed
- [ ] not applicable

# Checklist

- [ ] The change is one logical unit
- [ ] No unrelated formatting or dependency churn
- [ ] No generated attribution or co-author text
- [ ] No new unverified claims
