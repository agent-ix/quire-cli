---
id: FR-045
title: "quire trace and symbols report the formalizes relation"
type: FR
relationships:
  - target: "ix://agent-ix/quire-cli/spec/stakeholder/StR-004"
    type: "implements"
    cardinality: "1:1"
  - target: "ix://agent-ix/quire-rs/spec/functional/FR-094"
    type: "references"
  - target: "ix://agent-ix/quire-rs/spec/functional/FR-077"
    type: "references"
---
# FR-045: quire trace and symbols report the formalizes relation

## Description

The engine's `formalizes` relation ([quire-rs FR-094](ix://agent-ix/quire-rs/FR-094),
[quire-rs FR-077](ix://agent-ix/quire-rs/FR-077)) is a declaration in another language that
names a requirement it formalizes. Each statement below carries one obligation.

- `quire trace` SHALL carry the relation in its own `claims.formalizes` array beside `claims.verifies` and `claims.implements`.
- `quire trace` SHALL omit `claims.formalizes` when no formalizes relation is present, preserving its existing JSON bytes for modules that do not declare a formalizes marker form.
- `quire trace` SHALL print each relation in human output as `formalized by <path>::<label>`.
- `quire trace` SHALL emit each relation as a `formalizes` record kind in TSV output.
- `quire symbols` SHALL list a formalized id in the `formalizes` field of the declaring symbol.
- `quire trace` and `quire symbols` SHALL list a relation only when the loaded corpus defines its id.
- `quire trace` and `quire symbols` SHALL list no relation when the scope has no `spec/` directory.
- `quire trace` and `quire symbols` SHALL succeed when the scope has no `spec/` directory.

## Acceptance Criteria

| ID | Criteria | Verification |
|----|----------|--------------|
| FR-045-AC-1 | `quire trace --id <ID> --json` over a scope with a QSL `formalizes` marker lists the declaration under `claims.formalizes` only, with `claims.verifies` and `claims.implements` empty and `resolved` true. | Test |
| FR-045-AC-2 | `quire trace --file <PATH> --json` lists each declaration in the file under `claims.formalizes` with its resolved id. | Test |
| FR-045-AC-3 | An id the corpus does not define appears in neither `quire trace` nor `quire symbols --json` output. | Test |
| FR-045-AC-4 | `quire trace --format tsv` emits one `formalizes` record per relation. | Test |
| FR-045-AC-5 | When the scope has no `spec/` directory, a `quire trace` query succeeds and lists no `formalizes` relation, rather than failing on the missing document root. | Test |
| FR-045-AC-6 | Human `quire trace` output prints each relation as `formalized by <path>::<label>`. | Test |
| FR-045-AC-7 | `quire trace --prefix` includes the `formalizes` relations whose id matches the prefix. | Test |
| FR-045-AC-8 | `quire symbols --json` lists a formalized id, defined by the corpus, in the `formalizes` field of the declaring symbol. | Test |
| FR-045-AC-9 | When the scope has no `spec/` directory, `quire symbols` succeeds and lists no formalized id, rather than failing on the missing document root. | Test |
| FR-045-AC-10 | With a module that does not declare `trace_tags.formalizes` and a scope containing a `.native` file with no formalizes relation, `quire trace --id <ID> --json` omits `claims.formalizes` and emits the same bytes as the existing trace JSON contract. | Test |

## Dependencies

- **Upstream:** quire-rs FR-094 owns the relation and its id resolution; FR-077 owns the search result shape.
- **Local boundary:** [StR-004](../stakeholder/StR-004-thin-boundary-over-quire-rs.md).
