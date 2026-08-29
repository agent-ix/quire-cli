---
id: FR-020
title: "quire clauses subcommand"
type: FR
relationships:
  - target: "ix://agent-ix/quire-cli/spec/stakeholder/StR-004"
    type: "implements"
    cardinality: "1:1"
---

# FR-020: Clause-set evaluation and comparison

## Description

The CLI SHALL expose the generic rights-aware clause-set model supplied by
`quire-rs` without embedding a publication, authority, rule inventory, or
domain classification in the binary.

```text
quire clauses evaluate --module <PATH> --authority <ID> --set <ID>
  --version <VERSION> [--context KEY=VALUE]... [--format human|json|tsv]

quire clauses diff --module <PATH> --authority <ID> --set <ID>
  --before-version <VERSION> --after-version <VERSION>
  [--format human|json|tsv]
```

## Acceptance Criteria

| ID | Criteria | Verification |
|----|----------|--------------|
| FR-020-AC-1 | Both commands path-check and strictly load one explicit module and select authority, id, and version exactly; a missing version or invalid referenced set is a user error, never a nearest-version guess. | Test (IT-137, IT-140) |
| FR-020-AC-2 | `evaluate` accepts repeatable `KEY=VALUE` context dimensions and delegates applicability to the engine. Duplicate or malformed context is rejected. | Test (IT-137, IT-140) |
| FR-020-AC-3 | Evaluation preserves `binding`, `not_binding`, and `unresolved`; missing context remains unresolved with its reason. | Test (IT-137, IT-139) |
| FR-020-AC-4 | `diff` delegates to the engine and reports added, removed, and changed clauses between two exact versions of the same authority and set id. | Test (IT-138) |
| FR-020-AC-5 | Human results and deterministic TSV are emitted on stdout; TSV escapes structural characters so one clause remains one row. | Test (IT-139, TC-141) |
| FR-020-AC-6 | JSON carries tool provenance including the `clause_sets` capability and conforms to the pinned engine's published binding or diff schema. | Test (IT-137, IT-138) |
| FR-020-AC-7 | The CLI contains only transport and rendering policy. Clause content, rights validation, applicability, and comparison remain engine/module concerns. | Test (IT-140), Review |

## Constraints

The repository and its fixtures SHALL contain only original synthetic clause
content. Restricted source material is not a CLI fixture and is not required
to exercise this interface.
