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

> **CR note (lazy-init delegation exception, 2026-06-19):** This NFR is scoped
> to *quire-cli's own process*. quire links no network client crate and opens no
> socket in its own process. The one documented exception is
> [FR-004](../functional/FR-004-validate-subcommand.md) scoped `validate`: when
> module discovery finds zero modules it MAY spawn `quoin plugin ensure-defaults`
> as a **child process**, whose network I/O bootstraps the default module set.
> The clone happens entirely in `quoin`, so the static-binary / no-HTTP-crate
> guarantee (AC-1, AC-3) is unchanged; only the absolute "no socket in any
> descendant" reading of AC-2 is relaxed for that one bootstrap path. See
> [ADR-0001](../assets/adr/0001-validate-lazy-init-module-bootstrap.md).

## Statement

The CLI SHALL NOT depend on any HTTP, gRPC, or other network client crate, and
SHALL NOT open any network socket **in its own process** at any point during any
subcommand's execution. The sole exception is the
[FR-004](../functional/FR-004-validate-subcommand.md) scoped-`validate` lazy-init,
which may spawn `quoin` as a child process to bootstrap modules (network confined
to that child; [ADR-0001](../assets/adr/0001-validate-lazy-init-module-bootstrap.md)).
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
| NFR-004-AC-2 | An IT runs each subcommand under `strace -fe network` (or equivalent) on its happy path (registry present — including scoped discovery that finds modules) and asserts zero `socket()` calls in quire's own process and descendants. The scoped-`validate` empty-discovery lazy-init (which spawns `quoin`) is the documented exception ([ADR-0001](../assets/adr/0001-validate-lazy-init-module-bootstrap.md)) and is out of this happy-path scope | Test |
| NFR-004-AC-3 | `Cargo.lock` audit (CI gate) lists no HTTP client crate | Inspection |
