# Executable analysis

Implemented in `analyzer-executable`. This is the only stage that always
produces at least one finding.

- Loads both the current and candidate executable files and computes
  their SHA-256 hash and byte length (`ExecutableIdentity`).
- Compares the two hashes. If they differ,
  `EXECUTABLE_HASH_CHANGED` fires — this always fires when the bytes
  differ, and carries no judgment about the significance of the
  difference on its own.
- Validates that the candidate is structurally valid WASM and meets
  Soroban's known structural requirements (start section,
  component-model sections, memory64, shared memory). A structural
  incompatibility produces `ExecutableStructurallyIncompatible`
  (rule `EXECUTABLE_STRUCTURALLY_INCOMPATIBLE`).

The evidence for `EXECUTABLE_HASH_CHANGED` is a `derived_comparison`
record over both hashes — see [Evidence and confidence](evidence.md).
