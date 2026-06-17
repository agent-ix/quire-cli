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

## Statement

The CLI SHALL NOT depend on any HTTP, gRPC, or other network client crate, and
SHALL NOT open any network socket at any point during any subcommand's execution.
`quire-rs` already pins `jsonschema` with `default-features = false, features =
["resolve-file"]` to drop `reqwest`; this CLI SHALL maintain that constraint and
add no network dependencies of its own.

## Measurement and Evaluation

| Metric | Target | Threshold | Method |
|--------|--------|-----------|--------|
| HTTP/network client crates in dependency tree | 0 | 0 | `cargo deny check bans` + `Cargo.lock` audit |
| `socket()` calls observed across all subcommands | 0 | 0 | `strace -e network` runtime audit |

## Verification

`cargo deny check bans` rejects known HTTP client crates and a `Cargo.lock` audit
confirms none are present (cross-platform); on Linux, each subcommand is run under
`strace -e network` and asserted to make zero `socket()` calls.

## Acceptance Criteria

| ID | Criteria | Verification |
|----|----------|--------------|
| NFR-004-AC-1 | `cargo deny check bans` rejects `reqwest`, `hyper`, `tonic`, `surf`, `ureq` | Inspection |
| NFR-004-AC-2 | An IT runs each subcommand under `strace -e network` (or equivalent) and asserts zero `socket()` calls | Test |
| NFR-004-AC-3 | `Cargo.lock` audit (CI gate) lists no HTTP client crate | Inspection |
