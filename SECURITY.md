# Security Policy

## Scope and posture

SorobanLabs Analyzer treats all supplied WASM executables and state data
as untrusted input. The analyzer:

- Does not perform arbitrary host execution of analyzed WASM.
- Rejects malformed artifacts safely, with structured errors, rather
  than panicking or producing undefined behavior.
- Performs no uncontrolled filesystem writes.
- Never automatically submits a transaction or mutates network state.
  Any Stellar RPC backend used by the analyzer is read-only.
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
