---
id: NFR-005
title: "Diagnostics inherit quire-rs FR-017 format"
type: NFR
quality_attribute: usability
relationships:
  - target: "ix://agent-ix/quire-rs/spec/functional/FR-017"
    type: "depends_on"
    cardinality: "1:1"
---

## Statement

Every diagnostic the CLI emits to stderr — including CLI-originated diagnostics
(`PathSafetyViolation`, `UnknownArchetype` lookup failure prior to engine call) —
SHALL be expressible as a `quire-rs::Diagnostic`. The CLI SHALL NOT invent a
parallel diagnostic format: human-readable rendering uses
`quire-rs::format_violation`; JSON rendering uses `serde_json::to_string` against
the `Diagnostic` type, gated by `--diagnostics-format=json`.

## Measurement and Evaluation

| Metric | Target | Threshold | Method |
|--------|--------|-----------|--------|
| stderr emissions in `src/` not produced via `quire-rs::Diagnostic` | 0 | 0 | Source audit |
| Known failure modes whose JSON stderr parses as `Diagnostic` | all | all | Integration test |

## Verification

A source audit confirms every stderr emission routes through
`quire-rs::format_violation` or `serde_json::to_string(&Diagnostic { ... })`, and
an integration test collects stderr from each known failure mode and confirms it
parses as `Diagnostic` JSON under `--diagnostics-format=json`.

## Acceptance Criteria

| ID | Criteria | Verification |
|----|----------|--------------|
| NFR-005-AC-1 | Every stderr emission in `src/` is produced by calling `quire-rs::format_violation` or `serde_json::to_string(&Diagnostic { ... })` | Inspection |
| NFR-005-AC-2 | An IT collects stderr from each known failure mode and confirms parseability as `Diagnostic` JSON when `--diagnostics-format=json` is set | Test |
