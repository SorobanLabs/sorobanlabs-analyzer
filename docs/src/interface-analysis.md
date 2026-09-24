# Interface analysis

Implemented in `analyzer-executable`. Extracts and normalizes each
executable's `contractspecv0` WASM section into a structured contract
interface — functions (with parameters and return types), events, and
user-defined types (structs, unions, enums) — then diffs the current and
candidate interfaces.

Possible findings:

| Rule | Fires when |
|---|---|
| `CONTRACT_INTERFACE_ADDED` | A function, event, or type exists in the candidate but not the current executable. |
| `CONTRACT_INTERFACE_REMOVED` | A function, event, or type exists in the current executable but not the candidate. |
| `CONTRACT_SIGNATURE_CHANGED` | A function's parameters or return type changed between versions. |
| `CONTRACT_EVENT_CHANGED` | An event's shape changed between versions. |
| `CONTRACT_TYPE_CHANGED` | A user-defined type's shape changed between versions. |

Interface findings do not, by themselves, say whether a change is
backward compatible for every caller — they report what changed in the
interface. A `CONTRACT_SIGNATURE_CHANGED` finding on a function means
exactly that: the function's signature is different. Whether that
matters depends on who calls it, which this stage does not attempt to
determine.

See [examples/upgrade-review](https://github.com/SorobanLabs/sorobanlabs-analyzer/tree/main/examples/upgrade-review)
in the repository for a real report containing `CONTRACT_SIGNATURE_CHANGED`
and `CONTRACT_TYPE_CHANGED` findings from an actual fixture pair.
