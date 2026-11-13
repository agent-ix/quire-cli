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
  - target: "ix://agent-ix/quire-cli/spec/functional/FR-030"
    type: "extends"
---

# FR-041: quire relations design mode

## Description

`quire relations` ([FR-030](./FR-030-relations-subcommand.md)) SHALL accept `--design`, which renders
the design-element join of quire-rs [FR-086](ix://agent-ix/quire-rs/FR-086): for each requirement, the
design elements that realize it, and an explicit gap when there are none. The join, its rows, states
and `criteria` belong to quire-rs; the CLI resolves roots, applies path safety and renders
([StR-004](../stakeholder/StR-004-thin-boundary-over-quire-rs.md)). It holds no requirement or design
archetype name and no realizing verb: which relations are design relations is the loaded modules'
`traceability.design_relations`.

```
quire relations (--scope <DIR> | --repo <ORG/REPO>=<DIR>...) [--module <PATH>]... --design [--json]
```

`--design` combines with `--scope`, `--repo`, `--module` and `--json` exactly as FR-030 defines them,
so design elements held in a sibling repository join through `--repo` ([FR-085](ix://agent-ix/quire-rs/FR-085)).
`--design` replaces the relation selection, so it conflicts with `--relation`. The matrix design
column and the strict gate over it are separate requirements; `--strict` keeps FR-030's meaning.

## Behavior

- Human output is FR-030's, one block per design relation, one line per requirement: the requirement
  id, its `state`, then each design element as `id (kind)` with its repo when present, and the
  criterion ids it realizes. An `uncovered` requirement is marked `no design element` and names the
  relation's `check` token. Dangling and kind-mismatch entries print beneath the row as in FR-030.
- `--json` writes the quire-rs join on stdout and nothing else, and honours `--pretty`
  ([FR-008](./FR-008-json-output-encoding.md)).
- A module set declaring no `design_relations` is an invalid request: exit 3 naming the missing
  declaration, computing nothing. `--design` together with `--relation` exits 3 naming both flags.

## Acceptance Criteria

| ID | Criteria | Verification |
|----|----------|--------------|
| FR-041-AC-1 | `quire relations --scope <DIR> --module $M --design --json` over a fixture with one requirement realized by a design element and one with none emits only the join: the first row `covered` with the element's `id` and `archetype` in `linked`, the second `uncovered` with empty `linked`. | Test |
| FR-041-AC-2 | In the same run two executions are byte-identical, and a relation declared in the module but absent from `design_relations` produces no block. | Test |
| FR-041-AC-3 | Default human output prints each element as `id (kind)`, prints realized criterion ids, and marks an `uncovered` requirement `no design element` with the relation's `check` token. | Test |
| FR-041-AC-4 | `--repo o/a=<A> --repo o/b=<B> --design` over a fixture whose requirement in `o/a` is realized by an element in `o/b` shows that element with repo `o/b`; without the `o/b` flag the requirement is a gap and the reference prints under the row's dangling entries as authored. | Test |
| FR-041-AC-5 | A module set with no `design_relations` exits 3 naming the missing declaration, with no report on stdout. | Test |
| FR-041-AC-6 | `--design` with `--relation` exits 3 naming both flags. | Test |
| FR-041-AC-7 | The CLI source holds no requirement or design archetype name and no realizing verb, and delegates the join to quire-rs. | Inspection |

## Dependencies

- **Upstream**: [StR-004](../stakeholder/StR-004-thin-boundary-over-quire-rs.md); [FR-030](./FR-030-relations-subcommand.md); quire-rs [FR-086](ix://agent-ix/quire-rs/FR-086), [FR-085](ix://agent-ix/quire-rs/FR-085)
- **Downstream**: the matrix design column (PLAT-1529)
