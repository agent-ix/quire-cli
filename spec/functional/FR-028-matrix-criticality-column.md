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
command show it. The matrix is the requirement-level surface, and the trace-id lookup has no
requirement rows, so the column lands here.

> **CR note (PLAT-1412):** the originating ticket words this as a `quire trace` column. This
> requirement places it on `quire matrix` deliberately, for the reason above. FR-026 carries the
> matching amendment of its column lists.

### The column and its name

The CLI **SHALL** append a `Requirement Criticality` column to the Markdown requirement table and a
`requirement_criticality` column to the TSV record, repeating the requirement's value on each of its
criterion rows. The name states the level. quire-rs FR-053 also carries a per-criterion
`criticality`, which `quire coverage --format tsv` prints and leaves empty when absent. The matrix
does not show it, because the engine's criterion entries (FR-050-AC-48) do not carry it.

### Missing, present and odd values

The CLI **SHALL** render a requirement whose entry has no `criticality` key as the literal
`unassigned`. The word is a reporting label for a person-assigned field that was never assigned.
The per-criterion column is empty when absent because it is a module-declared optional column,
where empty means no value was declared. A key that is present with an empty string renders as an
empty cell, so it stays distinguishable from `unassigned`. The CLI **SHALL NOT** default, infer or
inherit a value.

A present value that is not a string (number, boolean, array, object) renders as its compact JSON
text. A present value renders through FR-026's existing cell rules: a `|` in Markdown is escaped to
`\|`, and a tab, newline or carriage return becomes a single space in both Markdown and TSV. The
value is never truncated.

### JSON and exit status

In `--format json` the CLI **SHALL** pass the engine's `requirements[]` through unmodified, so an
absent key stays absent and a present `criticality: "unassigned"` stays distinguishable from it.
Criticality has no effect on `--strict` or on the exit code.

## Acceptance Criteria

| ID | Criteria | Verification |
|----|----------|--------------|
| FR-028-AC-1 | The Markdown table header is `Criterion \| Statement \| Binders \| Status \| Requirement Criticality`, and each row's last cell is the requirement's engine value. | Test (CLI snapshot) |
| FR-028-AC-2 | The TSV header is `document\tcriterion\tstatus\tbinders\tstatement\trequirement_criticality`, and each record carries its requirement's value in the last column. | Test (CLI snapshot) |
| FR-028-AC-3 | A requirement whose engine entry has no `criticality` key renders `unassigned` in the Markdown and TSV cells. | Test (CLI snapshot) |
| FR-028-AC-4 | `--format json` carries the engine's `requirements[]` unmodified: an entry with no key has no key, and an entry with `criticality: "unassigned"` has that key present. | Test |
| FR-028-AC-5 | In a corpus with requirements valued `low`, `medium`, `high` and one missing, each requirement's rows show only its own value; the missing one shows `unassigned` and no row shows a value taken from another requirement. | Test (CLI snapshot) |
| FR-028-AC-6 | The zero-population output of FR-026-AC-9 is unchanged except that the TSV header alone is the six-column header of FR-028-AC-2. | Test |
| FR-028-AC-7 | A `criticality` value containing `\|` renders in Markdown as `\|` and keeps the row at five cells; the same value in TSV is unescaped. | Test (CLI snapshot) |
| FR-028-AC-8 | A `criticality` value containing a tab, newline or carriage return renders with each replaced by a single space in Markdown and TSV, leaving one record per criterion. | Test (CLI snapshot) |
| FR-028-AC-9 | A requirement whose key is present with an empty string renders an empty cell in Markdown and TSV, not `unassigned`. | Test (CLI snapshot) |
| FR-028-AC-10 | A requirement whose key is a non-string value (for example the number `3`) renders its compact JSON text (`3`) in Markdown and TSV. | Test (CLI snapshot) |
| FR-028-AC-11 | For the same corpus, `--strict` and the exit code are identical whether every requirement is `high`, every requirement is missing a value, or the values are mixed. | Test |

## Dependencies

- **Upstream**: [FR-026](./FR-026-matrix-subcommand.md), quire-rs FR-082 (the engine field; lands in the same change set)
