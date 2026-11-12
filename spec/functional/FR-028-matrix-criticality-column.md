---
id: FR-028
title: "quire matrix renders requirement criticality"
type: FR
relationships:
  - target: "ix://agent-ix/quire-cli/spec/functional/FR-026"
    type: "extends"
    cardinality: "1:1"
  - target: "ix://agent-ix/quire-rs/spec/functional/FR-082"
    type: "references"
---
# FR-028: quire matrix renders requirement criticality

## Description

`quire matrix` ([FR-026](./FR-026-matrix-subcommand.md)) renders the engine's `coverage_matrix`.
The engine carries an optional requirement-level `criticality` on each `requirements[]` entry
([quire-rs FR-082](ix://agent-ix/quire-rs/spec/functional/FR-082)). This requirement makes the
command show it.

The matrix is the requirement-level surface, so the column lands here and not in the trace-id
lookup, which has no requirement rows. The originating ticket words this as a `quire trace`
column; that wording is a deviation this requirement makes deliberately.

The CLI **SHALL** append a `Criticality` column to the Markdown requirement table and a
`criticality` column to the TSV record, repeating the requirement's value on each of its
criterion rows. The CLI **SHALL** render a requirement whose entry has no `criticality` key as the
literal `unassigned`. The CLI **SHALL NOT** default, infer or inherit a value. In `--format json`
the CLI passes the engine's value through unmodified, so an absent key stays absent and a present
`criticality: "unassigned"` stays distinguishable from it.

## Acceptance Criteria

| ID | Criteria | Verification |
|----|----------|--------------|
| FR-028-AC-1 | The Markdown table header is `Criterion \| Statement \| Binders \| Status \| Criticality` (FR-026-AC-4 plus the appended column), and each row's `Criticality` cell is the requirement's engine value verbatim. | Test (CLI snapshot) |
| FR-028-AC-2 | The TSV header is `document\tcriterion\tstatus\tbinders\tstatement\tcriticality`, and each record carries its requirement's value in the last column. | Test (CLI snapshot) |
| FR-028-AC-3 | A requirement whose engine entry has no `criticality` key renders `unassigned` in the Markdown and TSV `criticality` cells. | Test (CLI snapshot) |
| FR-028-AC-4 | `--format json` carries the engine's `requirements[]` unmodified: an entry with no key has no key, and an entry with `criticality: "unassigned"` has that key present. | Test |
| FR-028-AC-5 | In a corpus with requirements valued `low`, `medium`, `high` and one missing, each requirement's rows show only its own value; the missing one shows `unassigned` and no row shows a value taken from another requirement. | Test (CLI snapshot) |
| FR-028-AC-6 | The zero-population output of FR-026-AC-9 is unchanged: the single line in Markdown, the header alone (now with the `criticality` column) in TSV. | Test |

## Dependencies

- **Upstream**: [FR-026](./FR-026-matrix-subcommand.md), quire-rs FR-082 (the engine field; lands in the same change set)
