# Contributing

`quire-cli` is a thin process boundary over `quire-rs`. Pull requests must keep
parsing, validation, graph construction, assurance projection, and schema
ownership in the engine.

## Review checklist

- Does any new logic belong upstream in `quire-rs`?
- Does the change preserve stdout/stderr and exit-code contracts?
- Are new behavioral claims backed by executable tests and `make spec`?
- Were `make ci` and the relevant release build run locally?
