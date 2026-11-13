---
id: FR-041
title: "quire relations design mode"
type: FR
relationships:
  - target: "ix://agent-ix/quire-cli/spec/stakeholder/StR-004"
    type: "implements"
    cardinality: "1:1"
  - target: "ix://agent-ix/quire-rs/spec/functional/FR-086"
    type: "references"
  - target: "ix://agent-ix/quire-rs/spec/functional/FR-084"
    type: "references"
  - target: "ix://agent-ix/quire-rs/spec/functional/FR-085"
    type: "references"
  - target: "ix://agent-ix/quire-cli/spec/functional/FR-030"
    type: "extends"
---

# FR-041: quire relations design mode

## Description

`quire relations` ([FR-030](./FR-030-relations-subcommand.md)) SHALL accept `--design`, which renders
the design-element join of quire-rs [FR-086](ix://agent-ix/quire-rs/FR-086): for each requirement, the
design elements that realize it, and an explicit gap when there are none. The join, its rows, states
and `realized_criteria` belong to quire-rs; the CLI resolves roots, applies path safety and renders
([StR-004](../stakeholder/StR-004-thin-boundary-over-quire-rs.md)). It holds no requirement or design
archetype name and no realizing verb: which relations are design relations is the loaded modules'
`traceability.design_relations`.

```
quire relations (--scope <DIR> | --repo <ORG/REPO>=<DIR>...) [--module <PATH>]... --design [--json]
```

`--design` combines with `--scope`, `--repo`, `--module` and `--json` exactly as FR-030 defines them,
so design elements held in a sibling repository join through `--repo` ([FR-085](ix://agent-ix/quire-rs/FR-085)).
`--design` replaces the relation selection, so it conflicts with `--relation`. It also conflicts
with `--strict`: the matrix design column and the strict gate over it are a separate requirement
(PLAT-1529), and FR-030's `--strict` rules (verification, `not-tracked`) are about the requirement's
tests, not about whether a design element exists, so applying them here would gate on the wrong thing.

## Behavior

- Human output is FR-030's, one block per design relation, one line per requirement, with FR-030's
  layout (requirement id, `state`, `verification`, linked documents) and two changes: each linked
  design element prints as `id (kind)` with its repo when present, followed by the criterion ids in its
  `realized_criteria`; and an `uncovered` requirement is marked `no design element` and names the
  relation's `check` token. Dangling and kind-mismatch entries print beneath the row as in FR-030.
  Each block ends with the relation's `summary` counts by `state` and `verification`, which equal the
  `summary` of the `--json` run over the same inputs.
- `--json` writes the quire-rs join on stdout and nothing else, and honours `--pretty`
  ([FR-008](./FR-008-json-output-encoding.md)).
- Each of these is an invalid request: exit 3 ([FR-007](./FR-007-exit-codes.md)) naming the cause and
  writing no report on stdout. A module set declaring no `design_relations`, naming the missing
  declaration; a `design_relations` entry naming no declared relation (the quire-rs FR-086 join
  error), naming the entry; `--design` with `--relation` or with `--strict`, naming both flags.

## Acceptance Criteria

| ID | Criteria | Verification |
|----|----------|--------------|
| FR-041-AC-1 | `quire relations --scope <DIR> --module $M --design --json` over a fixture with one requirement realized by a design element and one with none emits only the join: the first row `covered` with the element's `id` and `archetype` in `linked`, the second `uncovered` with empty `linked`. | Test |
| FR-041-AC-2 | In the same run, stdout holds only the JSON document, and two runs over identical inputs are byte-identical. | Test |
| FR-041-AC-3 | A relation declared in the module but absent from `design_relations` produces no block in the output. | Test |
| FR-041-AC-4 | Default human output prints each element as `id (kind)`, prints the criterion ids of its `realized_criteria`, marks an `uncovered` requirement `no design element` with the relation's `check` token, and prints each requirement's `state` and `verification`. | Test |
| FR-041-AC-5 | Each human block ends with the relation's summary, and its counts equal the `summary` of the `--json` run over the same inputs. | Test |
| FR-041-AC-6 | `--repo o/a=<A> --repo o/b=<B> --design` over a fixture whose requirement in `o/a` is realized by an element in `o/b` shows that element with repo `o/b`; without the `o/b` flag, the requirement authoring the inverse toward `o/b` is a gap and the reference prints under the row's dangling entries as authored. | Test |
| FR-041-AC-7 | A module set with no `design_relations` exits 3 naming the missing declaration, with no report on stdout. | Test |
| FR-041-AC-8 | A `design_relations` entry naming no declared relation exits 3 naming the entry, with nothing on stdout. | Test |
| FR-041-AC-9 | `--design` with `--relation` exits 3 naming both flags, and `--design` with `--strict` exits 3 naming both flags, each with nothing on stdout. | Test |
| FR-041-AC-10 | Over a fixture whose relation, archetype, verb, inverse and criterion-id names are unrelated to requirements or design, `--design` renders the complete join in human and `--json` output, so the CLI holds no such name. | Test |
| FR-041-AC-11 | The CLI source delegates the join to quire-rs: it holds no join, edge-matching or row logic. | Inspection |
## Dependencies

- **Upstream**: [StR-004](../stakeholder/StR-004-thin-boundary-over-quire-rs.md); [FR-030](./FR-030-relations-subcommand.md); [FR-007](./FR-007-exit-codes.md); [FR-008](./FR-008-json-output-encoding.md); quire-rs [FR-086](ix://agent-ix/quire-rs/FR-086), [FR-084](ix://agent-ix/quire-rs/FR-084), [FR-085](ix://agent-ix/quire-rs/FR-085)
- **Downstream**: the matrix design column (PLAT-1529)
