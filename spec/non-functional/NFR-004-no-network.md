---
id: NFR-004
title: "No network dependencies, no network calls"
type: NFR
quality_attribute: security
relationships:
  - target: "ix://agent-ix/quire-cli/spec/stakeholder/StR-001"
    type: "implements"
    cardinality: "1:1"
  - target: "ix://agent-ix/quire-rs/spec/stakeholder/StR-001"
    type: "consumes"
    cardinality: "1:1"
---

## Constraint

The CLI SHALL NOT depend on any HTTP, gRPC, or other network client crate. It SHALL NOT open any network socket at any point during any subcommand's execution.

`quire-rs` already pins `jsonschema` with `default-features = false, features = ["resolve-file"]` to drop `reqwest`. This CLI SHALL maintain that constraint and add no network dependencies of its own.

## Acceptance

- **NFR-004-AC-1**: `cargo deny check bans` rejects `reqwest`, `hyper`, `tonic`, `surf`, `ureq`.
- **NFR-004-AC-2**: An IT runs each subcommand under `strace -e network` (or equivalent) and asserts zero `socket()` calls.
- **NFR-004-AC-3**: `Cargo.lock` audit (CI gate) lists no HTTP client crate.
