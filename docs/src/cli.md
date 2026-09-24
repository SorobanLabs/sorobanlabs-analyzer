# CLI reference

This page is generated from the CLI's own real `--help` output (not
retyped from memory) at the current source revision.

## Top level

```
$ sorobanlabs-analyzer --help
```

```
SorobanLabs Analyzer determines what can be established about replacing a deployed Soroban contract's executable: changes to its identity, interface, state compatibility, and authorization surface, plus, when a rehearsal manifest is supplied, observed behavioral differences under controlled execution.

Every finding carries a confidence (DETECTED, LIKELY, POTENTIAL, or NOT_DETERMINABLE) and a severity, reported separately. The analyzer does not produce a SAFE/UNSAFE verdict, and it never performs a real contract upgrade: it only reads the executables and any state/manifest files it is given, and, for rehearsal, runs bounded local execution through the existing soroban-env-host backend. It does not access the network, does not require a wallet or private key, and does not submit transactions.

Usage: sorobanlabs-analyzer <COMMAND>

Commands:
  analyze  Run the full upgrade analysis pipeline against a current and candidate executable and print a report
  help     Print this message or the help of the given subcommand(s)

Options:
  -h, --help
          Print help (see a summary with '-h')

  -V, --version
          Print version
```

## Exit codes

| Code | Meaning |
|---|---|
| `0` | The analysis pipeline ran to completion. The report's own `status` field (`NO_DETECTED_BLOCKERS`, `REVIEW_REQUIRED`, `MIGRATION_REQUIRED`, or `INCONCLUSIVE`) carries the analytical outcome; none of those statuses is itself treated as a CLI failure. |
| `2` | Invalid input: bad command-line arguments, an empty executable path, an executable whose bytes are not valid WASM, a malformed migration manifest, a malformed rehearsal input file, or a report serialization failure. This is about the *content* of what was supplied. |
| `3` | A required input file (executable, migration manifest, or rehearsal input) could not be read: missing file, permission denied, or the path names a directory. This is about being unable to *read* what was supplied, before its content is even examined. |
| `5` | An internal analysis-stage failure (the artifact loaded, but an analysis operation itself could not complete). |

## `analyze`

```
$ sorobanlabs-analyzer analyze --help
```

```
Run the full upgrade analysis pipeline against a current and candidate executable and print a report

Usage: sorobanlabs-analyzer analyze [OPTIONS] --current <CURRENT> --candidate <CANDIDATE>

Options:
      --current <CURRENT>
          Path to the current (deployed) contract executable (.wasm)

      --candidate <CANDIDATE>
          Path to the candidate replacement contract executable (.wasm)

      --protocol <PROTOCOL>
          The Soroban protocol number to record the analysis against. Carried through to the report; omit if you don't know it or it doesn't apply

      --migration-manifest <MIGRATION_MANIFEST>
          Path to an author-supplied migration manifest (JSON). Treated as a declaration, not proof; see the report's rehearsal/state sections for what the analyzer itself established

      --rehearsal <REHEARSAL>
          Path to a rehearsal input file (JSON, the analyzer_rehearsal RehearsalInput format) describing invocations to run against both executables under the real soroban-env-host backend. Omit to skip controlled rehearsal

      --format <FORMAT>
          Output format
          
          [default: terminal]

          Possible values:
          - json:     The canonical JSON report, written to stdout
          - terminal: A human-readable plain-text report, written to stdout

  -h, --help
          Print help (see a summary with '-h')
```

Required: `--current` and `--candidate`. Everything else is optional.
See [Examples](examples.md) for a full real run with real output.
