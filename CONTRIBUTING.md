# Contributing

## Before you open a pull request: sign the CLA

Every contribution requires a signed Contributor License Agreement. Read it
at [`CLA.md`](CLA.md) in this repo before you start writing code.

When you open your first pull request, the CLA Assistant bot will comment
with instructions to sign electronically. Sign once and it's recorded for
future contributions across the org.

`quire-cli` is a thin process boundary over `quire-rs`. Pull requests must keep
parsing, validation, graph construction, assurance projection, and schema
ownership in the engine.

## Review checklist

- Does any new logic belong upstream in `quire-rs`?
- Does the change preserve stdout/stderr and exit-code contracts?
- Are new behavioral claims backed by executable tests and `make spec`?
- Does npm distribution policy remain in the Rust `quire-dist` tool, with the
  ADR-0002 Node launcher limited to transparent host selection and execution?
- Were `make ci` and the relevant release build run locally?
