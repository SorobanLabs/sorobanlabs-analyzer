# The problem

Soroban contracts are deployed as WASM executables that can be upgraded
in place. Replacing a deployed contract's executable is risky in ways
that are easy to miss by eye:

- A function's signature can change in a way that breaks an existing
  caller.
- A shared type (an error enum, an event, a struct) can change shape.
- The new executable might expect state the old one never wrote, or
  interpret existing state differently.
- The new executable might call authorization primitives differently
  than the old one, changing who is allowed to invoke what.
- All of the above can be true even when the two executables' actual
  runtime behavior, under real invocations, turns out to match — or
  differ in ways a static diff alone cannot show.

Reviewing an upgrade manually means cross-referencing the interface
diff, the state layout, the authorization surface, and (ideally) actual
execution, by hand, for every entrypoint. SorobanLabs Analyzer automates
that comparison and reports each finding with an explicit confidence
level, so a reviewer knows not just what changed, but how certain the
tool is about it.
