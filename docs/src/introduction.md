# Introduction

SorobanLabs Analyzer is a Rust command-line tool that answers one
specific question:

> What will change if I replace the executable of this deployed Soroban
> contract?

It is not a generic smart-contract security scanner, a protocol
compatibility checker, or a hosted service. It reads a current and a
candidate contract executable (plus, optionally, a state manifest and a
rehearsal input), and reports what it can actually establish about the
difference between them, each finding carrying an explicit confidence
level rather than a single SAFE/UNSAFE verdict.

This book documents the current, implemented state of the project. If a
capability isn't described here, it doesn't exist yet — see
[Limitations](limitations.md) for what's explicitly out of scope or not
yet built.
