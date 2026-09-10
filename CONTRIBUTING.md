# Contributing

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
