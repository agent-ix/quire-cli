---
id: SR-20261113083704
title: "PLAT-1529 quire-cli PR188 spec-review ears"
type: SpecReview
analysis: ears-conformance
scope: "agent-ix/quire-cli PR188; spec/functional/FR-042-matrix-design-column.md statements"
review_set: subset
relationships:
  - target: ix://agent-ix/quire-cli/FR-042
    type: references
---

## Summary

Ticket: PLAT-1529. Checks the FR-042 requirement statements for EARS grammar.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | low | The design gate, the gate's core obligation, carries no SHALL: "`--strict` additionally exits 1 when a live requirement has a block with `state: uncovered`" is description, not an event-driven "When ..., `matrix` SHALL exit 1" statement | spec/functional/FR-042-matrix-design-column.md:79 |

## Examined scope

```yaml
scope:
- {id: FR-042, path: spec/functional/FR-042-matrix-design-column.md, role: examined, excerpt: "quire matrix (FR-026) SHALL render the design column of quire-rs FR-092"}
- {id: FR-042-gate, path: spec/functional/FR-042-matrix-design-column.md, role: examined, excerpt: "--strict additionally exits 1 when a live requirement has a block with state: uncovered."}
```

## Verdict

Changes requested (1 low). The primary statement is a ubiquitous SHALL; behavior bullets follow the house prose style of FR-026/FR-041.

## Dispositions

| FND | outcome | reason |
| --- | --- | --- |
| FND-001 | fixed | Gate restated event-driven: "When `--strict` is given and a live requirement has a block with `state: uncovered`, `matrix` SHALL exit 1" (FR-043:94-95) |
