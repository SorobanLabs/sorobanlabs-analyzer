# Analysis pipeline

The `analyze` command runs five analysis stages in a fixed order, each
covered in its own page:

1. [Executable analysis](executable-analysis.md) — identity and
   structural validity.
2. [Interface analysis](interface-analysis.md) — contract interface
   diff.
3. [State compatibility](state-compatibility.md) — declared vs.
   established state compatibility.
4. [Authorization analysis](authorization-analysis.md) — direct-call
   authorization surface diff.
5. [Controlled rehearsal](controlled-rehearsal.md) — optional, real
   bounded execution of both executables.

Every finding from every stage is attached to [evidence](evidence.md),
and the full set of findings determines the report's overall
[status](status.md).
