---
id: FR-042
title: "quire matrix design column and design gate"
type: FR
relationships:
  - target: "ix://agent-ix/quire-cli/spec/stakeholder/StR-004"
    type: "implements"
    cardinality: "1:1"
  - target: "ix://agent-ix/quire-cli/spec/functional/FR-026"
    type: "extends"
  - target: "ix://agent-ix/quire-cli/spec/functional/FR-041"
    type: "references"
  - target: "ix://agent-ix/quire-rs/spec/functional/FR-092"
    type: "references"
  - target: "ix://agent-ix/quire-rs/spec/functional/FR-086"
    type: "references"
---

# FR-042: quire matrix design column and design gate

## Description

`quire matrix` ([FR-026](./FR-026-matrix-subcommand.md)) SHALL render the design column of quire-rs
[FR-092](ix://agent-ix/quire-rs/FR-092): for each requirement, the design elements that realize it, then
its criteria, their tests and results. The column, its blocks, `kind`, `realized_criteria` and the
per-criterion selection `design_elements` belong to quire-rs; the CLI resolves roots, renders and
decides the exit code ([StR-004](../stakeholder/StR-004-thin-boundary-over-quire-rs.md)). It holds no
requirement or design archetype name and no realizing verb. Which requirements the column and the gate
apply to is the loaded modules' `traceability.design_relations` with each relation's `from`, `where` and
`exclude` (quire-rs [FR-058](ix://agent-ix/quire-rs/FR-058)).

```
quire matrix (--scope <DIR> | --repo <ORG/REPO>=<DIR>...) [--module <PATH>]...
             [--format human|markdown|json|tsv] [--strict]
```

`--repo` resolves as in [FR-041](./FR-041-relations-design-mode.md), so design elements held in a
sibling repository join (quire-rs [FR-085](ix://agent-ix/quire-rs/FR-085)). With `--repo` the entries are
every member's, each labelled with its repo (quire-rs FR-092): Markdown and human print a line
`repo: <repo>` under the heading, TSV adds no column and prefixes `document` with `<repo>:`. `--scope`
with `--repo` is exit 3. `--format human` is new and `markdown` stays the default.

## Behavior

- **Computation.** `matrix` calls quire-rs `design_matrix` in place of `coverage_matrix` and renders the
  entries. When no loaded module declares `design_relations`, nothing below applies and output is
  FR-026's unchanged.
- **Design cell.** A criterion's design cell is its `design_elements`, each resolved to the entry's
  element and printed as FR-041 prints it (`id (kind)`, with its repo when present; `id` alone when the
  element has no `kind`). For each `uncovered` block the cell appends `no design element (<check>)`. For
  each `covered` block with no elements, the cell prints its `dangling` references as authored,
  `-> <reference> (dangling)`. An entry with no `design` key has an empty cell, so an empty cell means
  out of scope. Under the heading (which stays exactly `## <document>`, FR-026-AC-4) an entry with an
  `uncovered` block prints the line `Design gap: no design element (<check>)`. An entry with
  `criteria: []` prints its heading and that line, with no table.
- **Markdown.** `Design` is inserted between `Statement` and `Binders`:
  `Criterion | Statement | Design | Binders | Status | Requirement Criticality`. Truncation and
  escaping of [FR-026](./FR-026-matrix-subcommand.md) §C and §D apply to the cell.
- **Human.** Plain text, one block per requirement: the `document` path, then
  `  design: <elements, then no design element (<check>) per uncovered block | out of scope>`, then one line per criterion
  `  <criterion>  <status>  design: <cell>  tests: <binders or (none)>`. The `design:` line of the block is
  the union of the entry's elements. The format has no truncation.
- **JSON.** The engine's entries verbatim, each with its `design` key as the engine carries it, in
  FR-026's envelope. Element kind is `kind`. The key is absent for an entry out of scope.
- **TSV.** `design` and `design_state` are inserted after `criterion`:
  `document criterion design design_state status binders statement requirement_criticality`. `design` is
  the comma-separated `kind:id` of the criterion's elements, `kind:repo/id` with a repo (`id` or `repo/id` when the element
  has no `kind`), and empty when none. `design_state` is `uncovered` when any block is `uncovered`, otherwise `covered`, and empty
  for an entry with no `design` key. An entry with `criteria: []` emits one record with `document`,
  `design` and `design_state` set and every other field empty.
- **Zero population.** FR-026 §B's zero criteria means no criterion in any entry. When entries exist
  but all have `criteria: []`, Markdown and human print each entry's heading and gap and no empty-state
  line, TSV prints one record per entry, and `--strict` fails on the zero-criteria rule as well as the
  design gap. With no entry at all, FR-026's empty-state line and header-only TSV hold. In JSON the
  engine's array is passed through verbatim.

### Design gate

`--strict` additionally exits 1 when a **live** requirement has a block with `state: uncovered`. A
requirement is live unless it has criteria and every one computes `retired`; a retired requirement's gap
is still rendered and does not fail. Each failure names the requirement `document` and the block's
`check` token on stderr, one line per requirement, and stdout is the rendering. This sits beside
FR-026 §G's criterion rules, which are unchanged, and the verdict reads the entries alone. Without
`--strict`, the gap is in the output and the exit code is 0. A requirement with no `design` key never
fails the gate, so a module narrows the gate with `where` and holds the rule in data. With no
`design_relations`, `--strict` is FR-026's.

### Errors

A `design_relations` entry naming no declared relation is the quire-rs FR-086 error: exit 3
([FR-007](./FR-007-exit-codes.md)) naming the entry, with nothing on stdout. `--repo` with `--scope`
is exit 3 as in FR-030.

## Acceptance Criteria

| ID | Criteria | Verification |
|----|----------|--------------|
| FR-042-AC-1 | `quire matrix --scope <DIR> --module $M` over a fixture with one realized and one unrealized requirement prints, in Markdown, a `Design` column between `Statement` and `Binders`, with the realized requirement's rows listing `id (kind)` and the other's rows reading `no design element`. | Test |
| FR-042-AC-2 | `--format human` prints, per requirement, the `design:` line (elements, `no design element (<check>)` or `out of scope`) and one line per criterion with its status, design cell and tests. | Test |
| FR-042-AC-3 | `--format json` carries each entry's `design` key as quire-rs emits it, names each element's `kind`, and omits `design` for an out-of-scope entry; two runs are byte-identical. | Test |
| FR-042-AC-4 | `--format tsv` emits the header `document criterion design design_state status binders statement requirement_criticality`, a `kind:id` list in `design`, `uncovered` or `covered` in `design_state`, and empty values for an out-of-scope entry. | Test |
| FR-042-AC-5 | A requirement whose criteria are named by `realized_criteria` shows an element only on those criteria, and an element realizing the requirement as a whole shows on every criterion row. | Test |
| FR-042-AC-6 | A requirement with no acceptance criteria and no design element is printed in Markdown, human and TSV with the gap marker and `design_state` `uncovered`, and a covered requirement with no criteria prints no gap. | Test |
| FR-042-AC-7 | `--strict` exits 1 over a fixture with a live requirement and no design element, with a stderr line naming the requirement and the block's `check` token; the same fixture without `--strict` exits 0 and shows the gap. | Test |
| FR-042-AC-8 | `--strict` exits 0 over a fixture whose design relation's `where` selects only one value of a frontmatter field, when the only requirement lacking a design element holds another value; changing the module data to select it makes the gate exit 1 with no CLI change. | Test |
| FR-042-AC-9 | A requirement with criteria all `retired` and no design element is rendered with the gap and does not fail `--strict`; one live criterion beside it makes the gate exit 1. | Test |
| FR-042-AC-10 | A passing corpus, where every in-scope requirement has an element and every criterion is `tagged`, exits 0 under `--strict`; a corpus with a design gap and an `untagged` criterion exits 1 and the stderr names the design gap too. | Test |
| FR-042-AC-11 | A module with no `design_relations` yields the FR-026 output in every format, including a `--strict` verdict. | Test |
| FR-042-AC-12 | A `design_relations` entry naming no declared relation exits 3 naming the entry, with nothing on stdout. | Test |
| FR-042-AC-13 | `--repo o/a=<A> --repo o/b=<B>` over a fixture whose requirement in `o/a` is realized by an element in `o/b` shows it with repo `o/b`, and without the `o/b` flag the requirement is a gap. | Test |
| FR-042-AC-14 | Over a fixture whose relation, archetype, verb and criterion-id names are unrelated to requirements or design, all four formats are complete and the gate fires. | Test |
| FR-042-AC-15 | The CLI source holds no join, edge-matching or selection logic beyond rendering the quire-rs `design_matrix` entries and `design_elements`, and no archetype name or verb. | Inspection |
| FR-042-AC-17 | A requirement with one `covered` and one `uncovered` design relation prints its elements and `no design element (<check>)` in the same cell. | Test |
| FR-042-AC-18 | A `covered` block of a `to: []` relation with only a dangling edge prints `-> <reference> (dangling)`, not an empty cell. | Test |
| FR-042-AC-19 | A fixture whose only obliged requirements have no criteria prints each heading and `Design gap:` line, one TSV record each, and `--strict` exits 1. | Test |
| FR-042-AC-20 | An element with no `kind` prints `id` in Markdown and human output and `id` in TSV. | Test |
| FR-042-AC-16 | Running `matrix` in every format leaves every file under `<scope>` byte-identical ([FR-026](./FR-026-matrix-subcommand.md)-AC-14). | Test |

## Dependencies

- **Upstream**: [FR-026](./FR-026-matrix-subcommand.md); [FR-028](./FR-028-matrix-criticality-column.md); [FR-041](./FR-041-relations-design-mode.md); [FR-007](./FR-007-exit-codes.md); [StR-004](../stakeholder/StR-004-thin-boundary-over-quire-rs.md); quire-rs [FR-092](ix://agent-ix/quire-rs/FR-092), [FR-086](ix://agent-ix/quire-rs/FR-086), [FR-058](ix://agent-ix/quire-rs/FR-058)
- **Downstream**: `quoin matrix` (quoin FR-162)
