---
id: NFR-005
title: "Diagnostics inherit quire-rs FR-017 format"
type: NFR
quality_attribute: usability
relationships:
  - target: "ix://agent-ix/quire-rs/spec/functional/FR-017"
    type: "consumes"
    cardinality: "1:1"
---

## Constraint

Every diagnostic the CLI emits to stderr — including CLI-originated diagnostics (`PathSafetyViolation`, `UnknownArchetype` lookup failure prior to engine call) — SHALL be expressible as a `quire-rs::Diagnostic`.

The CLI SHALL NOT invent a parallel diagnostic format. Human-readable rendering uses `quire-rs::format_violation`. JSON rendering uses `serde_json::to_string` against the `Diagnostic` type, gated by `--diagnostics-format=json`.

## Acceptance

- **NFR-005-AC-1**: Every stderr emission in `src/` is produced by calling `quire-rs::format_violation` or `serde_json::to_string(&Diagnostic { ... })`.
- **NFR-005-AC-2**: An IT collects stderr from each known failure mode and confirms parseability as `Diagnostic` JSON when `--diagnostics-format=json` is set.
