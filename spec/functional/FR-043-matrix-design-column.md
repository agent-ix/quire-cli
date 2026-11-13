---
id: FR-043
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
  - target: "ix://agent-ix/quire-rs/spec/functional/FR-085"
    type: "references"
  - target: "ix://agent-ix/quire-rs/spec/functional/FR-058"
    type: "references"
  - target: "ix://agent-ix/quire-cli/spec/functional/FR-007"
    type: "references"
  - target: "ix://agent-ix/quire-cli/spec/functional/FR-028"
    type: "references"
  - target: "ix://agent-ix/quire-cli/spec/functional/FR-030"
    type: "references"
---

# FR-043: quire matrix design column and design gate

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
quire matrix [--scope <DIR> | --repo <ORG/REPO>=<DIR>...] [--module <PATH>]...
             [--format human|markdown|json|tsv] [--strict]
```

`--repo` resolves as in [FR-041](./FR-041-relations-design-mode.md), so design elements held in a
sibling repository join (quire-rs [FR-085](ix://agent-ix/quire-rs/FR-085)). `--scope` keeps its default of `.` when `--repo` is absent, and `--scope` with `--repo` is exit 3. With
`--repo` the entries are every member's, each labelled with its repo (quire-rs FR-092): Markdown and
human print a line `repo: <repo>` under the heading, and TSV gains a first column `repo`. This holds
whether or not any module declares `design_relations`; the criteria rows then are FR-026's for each member. `--format human` is new and `markdown` stays the default.

## Behavior

- **Computation.** `matrix` calls quire-rs `design_matrix` in place of `coverage_matrix` and renders the
  entries. When no loaded module declares `design_relations`, nothing below about design applies and, for a
  single `--scope`, output is FR-026's unchanged.
- **Design cell.** A criterion's design cell is its `design_elements`, each resolved to the entry's
  element and printed as FR-041 prints it (`id (kind)`, with its repo when present; `id` alone when the
  element has no `kind`). Then, in order: `no design element (<check>)` for each `uncovered` block;
  `-> <reference> (dangling)` for each `dangling` reference of a `covered` block with no elements; and
  `kind mismatch: <id> (<archetype>)` for each `kind_mismatch` entry, as FR-041 reports it. The cell has
  three distinct empty forms: `n/a` for an entry with no `design` key (out of scope); `(none)` for a
  criterion with empty `design_elements` under a `covered` entry with nothing to append; the elements
  and markers otherwise.
- **Gap lines.** Under the heading, which stays exactly `## <document>` (FR-026-AC-4), Markdown and
  human print in this order: `repo: <repo>` when the entry has a repo, then one line
  `Design gap: no design element (<check>)` per `uncovered` block. An entry with `criteria: []` prints
  its heading and these lines, with no table.
- **Markdown.** `Design` is inserted between `Statement` and `Binders`:
  `Criterion | Statement | Design | Binders | Status | Requirement Criticality`. The `Statement` cell follows
  [FR-026](./FR-026-matrix-subcommand.md) §C and §D. The `Design` cell is never truncated, so no marker is
  cut; only the `|` escaping applies to it.
- **Human.** Plain text, one block per requirement: the `document` path, the gap lines above, then
  `  design: <union of the entry's elements, then the design cell's markers | n/a | (none)>`, with `n/a` for an entry with no `design` key and `(none)` when the union is empty and nothing is appended, then one line per criterion
  `  <criterion>  <status>  design: <cell>  tests: <binders or (none)>`. The format has no truncation.
- **JSON.** The engine's entries verbatim, each with its `design` key as the engine carries it, in
  FR-026's envelope. Element kind is `kind`. The key is absent for an entry out of scope.
- **TSV.** `design` and `design_state` are inserted after `criterion`:
  `document criterion design design_state status binders statement requirement_criticality`. `design` is
  the comma-separated `[kind:]<id>` of the criterion's elements, `[kind:]<repo>/<id>` for an element with
  a repo (`/` is the only repo separator), and empty when none. `design_state` is `uncovered` when any
  block is `uncovered`, otherwise `covered`, and empty for an entry with no `design` key. With `--repo` the
  first column `repo` precedes `document` (empty when a member has no identity). An entry with `criteria: []` emits one record with `document`,
  `design` and `design_state` set and every other field empty.
- **Zero population.** FR-026 §B's zero criteria means no criterion in any entry. When entries exist
  but all have `criteria: []`, Markdown and human print each entry's heading and gap and no empty-state
  line, TSV prints one record per entry, and `--strict` fails on the zero-criteria rule as well as the
  design gap. With no entry at all, FR-026's empty-state line and header-only TSV hold. In JSON the
  engine's array is passed through verbatim.

### Design gate

When `--strict` is given and a **live** requirement has a block with `state: uncovered`, `matrix` SHALL
exit 1. A requirement is live unless it has criteria and every one computes `retired`; a retired
requirement's gap is still rendered and does not fail. `matrix` SHALL name each failing requirement on
stderr, one line per failing requirement and uncovered block, as `design gap: [<repo>:]<document> <check>`, so a requirement with two uncovered relations gets two lines, in block order, with stdout the rendering. This sits
beside FR-026 §G's criterion rules, which are unchanged, and the verdict reads the entries alone. Without
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
| FR-043-AC-1 | `quire matrix --scope <DIR> --module $M` over a fixture with one realized and one unrealized requirement prints, in Markdown, a `Design` column between `Statement` and `Binders`, with the realized requirement's rows listing `id (kind)` and the other's rows reading `no design element (<check>)`. | Test |
| FR-043-AC-2 | `--format human` prints, per requirement, the `design:` line (elements, or `out of scope`), the `Design gap:` lines, and one line per criterion with its status, design cell and tests. | Test |
| FR-043-AC-3 | `--format json` carries each entry's `design` key as quire-rs emits it, names each element's `kind`, and omits `design` for an out-of-scope entry; two runs are byte-identical. | Test |
| FR-043-AC-4 | `--format tsv` emits the header `document criterion design design_state status binders statement requirement_criticality`, a `[kind:]<id>` list in `design`, `uncovered` or `covered` in `design_state`, and empty values for an out-of-scope entry. | Test |
| FR-043-AC-5 | A requirement whose criteria are named by `realized_criteria` shows an element only on those criteria, and an element realizing the requirement as a whole shows on every criterion row. | Test |
| FR-043-AC-6 | A requirement with no acceptance criteria and no design element is printed in Markdown, human and TSV with the gap marker and `design_state` `uncovered`, and a covered requirement with no criteria prints no gap. | Test |
| FR-043-AC-7 | `--strict` exits 1 over a fixture with a live requirement and no design element, with a stderr line naming the requirement and the block's `check` token; the same fixture without `--strict` exits 0 and shows the gap. | Test |
| FR-043-AC-8 | `--strict` exits 0 over a fixture whose design relation's `where` selects only one value of a frontmatter field, when the only requirement lacking a design element holds another value; changing the module data to select it makes the gate exit 1 with no CLI change. | Test |
| FR-043-AC-9 | A requirement with criteria all `retired` and no design element is rendered with the gap and does not fail `--strict`; one live criterion beside it makes the gate exit 1. | Test |
| FR-043-AC-10 | A passing corpus, where every in-scope requirement has an element and every criterion is `tagged`, exits 0 under `--strict`; a corpus with a design gap and an `untagged` criterion exits 1 and the stderr names the design gap too, as `design gap: <document> <check>`, with `<repo>:` before the document under `--repo`. | Test |
| FR-043-AC-11 | A module with no `design_relations` yields the FR-026 output in every format under `--scope`, including a `--strict` verdict, and under `--repo` the same entries labelled with `repo` lines and a TSV `repo` column. | Test |
| FR-043-AC-12 | A `design_relations` entry naming no declared relation exits 3 naming the entry, with nothing on stdout. | Test |
| FR-043-AC-13 | `--repo o/a=<A> --repo o/b=<B>` over a fixture whose requirement in `o/a` is realized by an element in `o/b` shows it with repo `o/b`, and without the `o/b` flag the requirement is a gap. | Test |
| FR-043-AC-14 | Over a fixture whose relation, archetype, verb and criterion-id names are unrelated to requirements or design, all four formats are complete and the gate fires. | Test |
| FR-043-AC-15 | The CLI source holds no join, edge-matching or selection logic beyond rendering the quire-rs `design_matrix` entries and `design_elements`, and no archetype name or verb. | Inspection |
| FR-043-AC-16 | Running `matrix` in every format leaves every file under `<scope>` byte-identical ([FR-026](./FR-026-matrix-subcommand.md)-AC-14). | Test |
| FR-043-AC-17 | A requirement with one `covered` and one `uncovered` design relation prints its elements and `no design element (<check>)` in the same cell. | Test |
| FR-043-AC-18 | A `covered` block of a `to: []` relation with only a dangling edge prints `-> <reference> (dangling)`, not an empty cell. | Test |
| FR-043-AC-19 | A fixture whose only obliged requirements have no criteria prints, in Markdown and human formats, each heading and `Design gap:` line with no empty-state line, prints one TSV record each, and `--strict` exits 1. | Test |
| FR-043-AC-20 | An element with no `kind` prints `id` in Markdown and human output and `id` in TSV. | Test |
| FR-043-AC-21 | `--scope` with `--repo` exits 3 naming both flags, with nothing on stdout. | Test |
| FR-043-AC-22 | The Design cell shows `n/a` for an entry with no `design` key and `(none)` for a criterion with empty `design_elements` under a `covered` entry, and a `no design element (<check>)` marker survives intact beside elements long enough that a Statement cell would be truncated. | Test |
| FR-043-AC-23 | A block left `uncovered` by a kind mismatch prints `kind mismatch: <id> (<archetype>)` beside the gap marker. | Test |
| FR-043-AC-24 | Under `--repo`, Markdown and human print `repo: <repo>` under the heading before the `Design gap:` lines, TSV has a leading `repo` column, an element in another repo prints `[kind:]<repo>/<id>`, and two members with the same document path are distinguishable on stdout and in the strict stderr line. | Test |
| FR-043-AC-25 | In `--format human`, the `design:` line of an `uncovered` entry with no elements shows `no design element (<check>)`, of an entry with no `design` key shows `n/a`, and of a `covered` entry with no elements shows `(none)`. | Test |
| FR-043-AC-26 | A requirement with two `uncovered` design relations fails `--strict` with two stderr lines, one per `<check>`, in block order. | Test |

## Dependencies

- **Upstream**: [FR-026](./FR-026-matrix-subcommand.md); [FR-028](./FR-028-matrix-criticality-column.md); [FR-041](./FR-041-relations-design-mode.md); [FR-007](./FR-007-exit-codes.md); [FR-030](./FR-030-relations-subcommand.md); [StR-004](../stakeholder/StR-004-thin-boundary-over-quire-rs.md); quire-rs [FR-092](ix://agent-ix/quire-rs/FR-092), [FR-086](ix://agent-ix/quire-rs/FR-086), [FR-085](ix://agent-ix/quire-rs/FR-085), [FR-058](ix://agent-ix/quire-rs/FR-058)
- **Downstream**: `quoin matrix` (quoin FR-165)
