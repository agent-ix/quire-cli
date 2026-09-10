---
id: NFR-009
title: "Native qualification boundary"
type: NFR
quality_attribute: maintainability
relationships:
  - target: "ix://agent-ix/quire-cli/FR-025"
    type: "constrains"
---

# NFR-009: Native qualification boundary

## Statement

The repository SHALL implement and qualify FR-025 with exact Rust 1.98.1,
locked dependency resolution, typed input models, canonical `ix-trace-rs`
bindings, and no shell-spawned qualification logic.

## Scope

- Applies to the four residual qualification scripts, duplicated inline
  benchmark parsing, their Rust callers, and their local Make orchestration.
- Excludes product CLI semantics, quire-rs parsing, hosted workflow redesign,
  Quire profile grammar, fixture data, and the npm distribution launcher.

## Rationale

Repository-specific assurance policy belongs with the repository, but one
native typed owner is required to prevent shell, Python, and Rust copies from
silently diverging. A real syntax parser prevents another ad hoc lexer from
becoming an unreviewed language implementation.

## Measurement and Evaluation

| Metric | Target | Threshold | Method |
|--------|--------|-----------|--------|
| Executable Python or shell qualification files outside fixture data | 0 | 0 | Static inventory (TC-839) |
| Shell processes spawned by Rust qualification tests | 0 | 0 | Process/static audit (TC-840) |
| Implemented FR-025 matrix rows lacking canonical `ix-trace-rs` bindings | 0 | 0 | Traceability audit (TC-840) |
| Hosted CI runs dispatched by this slice | 0 | 0 | Change inspection (TC-840) |

## Verification

Local qualification SHALL run sequentially with at most two Cargo build jobs:
format checking, strict all-target/all-feature Clippy, locked tests, strict
rustdoc, dependency license/advisory gates, and Quire validation of changed
specification artifacts. The implementation SHALL NOT dispatch hosted CI.

## Dependencies

- **Upstream:** [FR-025](../functional/FR-025-rust-qualification-tooling.md)
  defines the owned behaviors.
- **Upstream:** [NFR-007](./NFR-007-exact-qualified-rust-toolchain.md) selects
  exact Rust 1.98.1 for the workspace.
