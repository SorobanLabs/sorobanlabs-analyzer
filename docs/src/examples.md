# Examples

- [`examples/upgrade-review`](https://github.com/SorobanLabs/sorobanlabs-analyzer/tree/main/examples/upgrade-review) —
  a full, real walkthrough: two real fixture executables, the exact
  `analyze` command run against them, the actual captured terminal and
  JSON output, and a plain-language explanation of each of the five
  findings it produces (including a real `CONTRACT_SIGNATURE_CHANGED`
  and `CONTRACT_TYPE_CHANGED` pair, resulting in overall status
  `REVIEW_REQUIRED`).

Nothing in that example is hypothetical — the command was actually run
against the checked-in fixture files, and the output shown is exactly
what it produced.
