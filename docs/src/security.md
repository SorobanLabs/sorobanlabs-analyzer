# Security

See [SECURITY.md](https://github.com/SorobanLabs/sorobanlabs-analyzer/blob/main/SECURITY.md)
in the repository for the full, current security policy and how to
report a vulnerability. Summary:

- All supplied WASM executables and state data are treated as
  untrusted input.
- The analyzer does not perform arbitrary, unbounded host execution of
  analyzed WASM. When a rehearsal manifest is supplied, both
  executables *are* actually run, invocation by invocation, but only
  through the project-selected `soroban-env-host` backend, inside its
  own panic-catching wrapper, subject to a CPU/memory budget, and never
  touching real network state. A panic or trap in the candidate is
  caught and reported as an observation, not propagated as a process
  crash.
- Malformed artifacts are rejected safely, with structured errors,
  rather than panicking or producing undefined behavior (outside the
  controlled rehearsal path above).
- No uncontrolled filesystem writes. Any Stellar RPC backend the
  analyzer uses is read-only. No transaction is ever automatically
  submitted, including during rehearsal.
- The analyzer does not request, accept, or handle private keys.

**Controlled rehearsal is not a guarantee of upgrade safety.** It only
verifies outcome and return value for the specific invocations supplied
— see [Controlled rehearsal](controlled-rehearsal.md) and
[Limitations](limitations.md) for exactly what it does not cover.
