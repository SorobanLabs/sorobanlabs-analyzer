# Changelog

## Unreleased

- Dependency maintenance after v0.1.0: thiserror 2.0.21, wasmparser 0.221.3, soroban-env-host 29.0.0, and indexmap 2.13.1. Updates for sha2 0.11, indexmap 2.14 and later, toml_edit 0.23, clap 4.6, toml 0.9, and ureq 3 were reviewed and not merged because they conflict with the declared MSRV (1.84.0) or break CI; see issue #14.
- Dependabot ignore rules for version ranges known to be incompatible with the MSRV.
- Repository hygiene additions for contribution flow: pull request template, issue templates, code of conduct, changelog, and roadmap.
- Submission evidence refresh.

## v0.1.0

- Initial public release.
- Rust workspace for executable, interface, state, authorization, rehearsal, evidence, report, and CLI analysis.
- Canonical JSON and terminal report output.
- Real upgrade-review example.
- Documentation site and evidence-led submission pack.
