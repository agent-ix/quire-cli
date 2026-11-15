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

`quire trace` and `quire symbols` SHALL render the engine's `formalizes` relation
([quire-rs FR-094](ix://agent-ix/quire-rs/FR-094), [quire-rs FR-077](ix://agent-ix/quire-rs/FR-077)):
a declaration in another language that names a requirement it formalizes. The relation
is neither evidence nor scope, so `trace` carries it in its own `claims.formalizes`
array beside `verifies` and `implements`, as a `formalized by <path>::<label>` line in
human output and as a `formalizes` record kind in TSV output. The CLI mints no relation:
it only drops those whose id the loaded corpus does not define.

## Acceptance Criteria

| ID | Criteria | Verification |
|----|----------|--------------|
| FR-045-AC-1 | `quire trace --id <ID> --json` over a scope with a QSL `formalizes` marker lists the declaration under `claims.formalizes` only, with `claims.verifies` and `claims.implements` empty and `resolved` true. | Test |
| FR-045-AC-2 | `quire trace --file <PATH> --json` lists each declaration in the file under `claims.formalizes` with its resolved id, and human output prints `formalized by <path>::<label>`. | Test |
| FR-045-AC-3 | An id the corpus does not define appears in neither `quire trace` nor `quire symbols --json` output. | Test |
| FR-045-AC-4 | `--format tsv` emits one `formalizes` record per relation, and `--prefix` queries include `formalizes` relations whose id matches the prefix. | Test |
| FR-045-AC-5 | When the scope has no `spec/` directory, a `trace` query succeeds and lists no `formalizes` relation, rather than failing on the missing document root. | Test |

## Dependencies

- **Upstream:** quire-rs FR-094 owns the relation and its id resolution; FR-077 owns the search result shape.
- **Local boundary:** [StR-004](../stakeholder/StR-004-thin-boundary-over-quire-rs.md).
