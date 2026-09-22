# Security Policy

## Scope and posture

SorobanLabs Analyzer treats all supplied WASM executables and state data
as untrusted input. The analyzer:

- Does not perform arbitrary, unbounded host execution of analyzed WASM.
  This is a distinction, not a denial: when a rehearsal manifest is
  supplied (`analyze --rehearsal`), the candidate and current
  executables ARE actually run, invocation by invocation. That
  execution is bounded and controlled: it goes only through the
  project-selected `soroban-env-host` backend (`analyzer-rehearsal`),
  runs inside that backend's own panic-catching wrapper, is subject to
  a caller-configurable (and otherwise analyzer-defaulted) CPU/memory
  budget, and never touches real network state. A panic or trap inside
  the candidate is caught and reported as an observation
  (`Blocked`/`Trap`/`HostError`), not propagated as a crash of this
  process.
- Rejects malformed artifacts safely, with structured errors, rather
  than panicking or producing undefined behavior, outside of the
  controlled rehearsal path described above.
- Performs no uncontrolled filesystem writes.
- Never automatically submits a transaction or mutates network state,
  including during rehearsal. Any Stellar RPC backend used by the
  analyzer is read-only.
- Does not request, accept, or handle private keys. No workflow in this
  project requires a private key.
- Avoids logging sensitive configuration values.

The analyzer is not itself a general-purpose security scanner. It
reports what it can establish about the impact of replacing a contract
executable, with explicit confidence levels; it does not render a
security judgment about the contract as a whole. See the README for the
full scope statement.

## Reporting a vulnerability

If you discover a security vulnerability in this project (for example, a
way for a malformed artifact to cause memory unsafety, a panic on
untrusted input outside of a controlled `Result`-returning path, or an
unintended write or network mutation), please report it privately rather
than opening a public issue.

Open a private security advisory on the repository, or contact the
maintainers directly, with:

- A description of the issue and its impact.
- Steps or a minimal artifact to reproduce it.
- The affected version or commit.

Please allow reasonable time for a fix before public disclosure.
