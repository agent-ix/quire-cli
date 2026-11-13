---
id: FR-030
title: "quire relations subcommand"
type: FR
relationships:
  - target: "ix://agent-ix/quire-cli/spec/stakeholder/StR-004"
    type: "implements"
    cardinality: "1:1"
  - target: "ix://agent-ix/quire-rs/spec/functional/FR-084"
    type: "references"
  - target: "ix://agent-ix/quire-rs/spec/functional/FR-085"
    type: "references"
---

# FR-030: quire relations subcommand

## Description

The CLI SHALL provide a `relations` subcommand that renders the two-way relation coverage report
of quire-rs [FR-084](ix://agent-ix/quire-rs/FR-084) over one repository or over several
repositories loaded as one corpus ([FR-085](ix://agent-ix/quire-rs/FR-085)).

```
quire relations (--scope <DIR> | --repo <ORG/REPO>=<DIR>...) [--module <PATH>]...
                [--relation <NAME>]... [--json] [--strict]
```

The subcommand is separate from `quire coverage` ([FR-017](./FR-017-coverage-subcommand.md)): that
command's `CoverageReport` payload is its stable interface and gains no row sets. The report
computation, the row states and the verification statuses belong to quire-rs; the CLI resolves
roots, applies path safety ([FR-005](./FR-005-path-safety.md)), and renders
([StR-004](../stakeholder/StR-004-thin-boundary-over-quire-rs.md)). It holds no archetype, verb or
chain name: which rows exist comes from the loaded modules' `traceability.required_relations`.

`--scope` names one repository, loaded as a corpus of one member. `--repo` is repeatable, each
naming a member identity and its root, and `--scope` and `--repo` together are an error. `--module`
and its closed, ordered semantics are those of FR-017. `--relation` is repeatable and restricts the
report to the named declarations.

## Behavior

- Human output goes to stdout: one block per relation, one line per row, leading with the document id,
  then `state`, `verification`, and the linked document ids with their own verification; an
  `uncovered` row is marked as a gap and names the relation's `check` token. Dangling and
  kind-mismatch entries print beneath the row, naming the reference as authored.
- `--json` writes the quire-rs report on stdout and nothing else, with rows carrying document id,
  linked ids and per-document verification as the engine produced them. It honours `--pretty`
  ([FR-008](./FR-008-json-output-encoding.md)).
- The command is report-only. With `--strict` it exits 1 when any row is `uncovered` or has
  `verification` `unverified` or `partial`; without it, such rows exit 0.
- A scope without `spec/`, a module set with no `required_relations`, and an unknown `--relation`
  name each exit non-zero naming the cause and compute nothing.

## Acceptance Criteria

| ID | Criteria | Verification |
|----|----------|--------------|
| FR-030-AC-1 | `quire relations --scope <DIR> --module $M --json` over a fixture corpus with an unmitigated risk, a mitigated-but-untested risk and a mitigated-and-tested risk emits rows whose `state` and `verification` are respectively `uncovered`, `covered`/`unverified`, `covered`/`verified`, each row carrying the document id, linked ids and per-linked-document verification, and stdout holds only the JSON. | Test |
| FR-030-AC-2 | The default human output lists each row with its state and verification, marks an `uncovered` row as a gap with the relation's `check` token, and lists a classified requirement with no linked risk document as a gap. | Test |
| FR-030-AC-3 | `--repo o/a=<A> --repo o/b=<B>` over a fixture where a requirement in `o/a` mitigates a hazard in `o/b` shows the hazard `covered`; the same command without `--repo o/b` shows the reference under the row's dangling entries, naming it as authored. | Test |
| FR-030-AC-4 | A fixture edge to a wrong-kind target prints beneath its row as a kind mismatch naming the target and its actual archetype, in human and `--json` output. | Test |
| FR-030-AC-5 | `--relation <NAME>` restricts the report to that declaration; an unknown name exits non-zero naming it with no report. | Test |
| FR-030-AC-6 | `--strict` exits 1 over a corpus with an `uncovered` or `unverified` row and exits 0 on the same corpus without it; a corpus whose rows are all `verified` exits 0 under `--strict`. | Test |
| FR-030-AC-7 | `--scope` together with `--repo`, a `--scope` holding no `spec/`, and a module set declaring no `required_relations` each exit non-zero with a diagnostic naming the cause. | Test |
| FR-030-AC-8 | Two runs over identical inputs produce byte-identical `--json` output, compact by default and indented under `--pretty` with the identical parsed value. | Test |
| FR-030-AC-9 | The CLI source holds no hazard, threat or safety-classification name, and delegates the report to quire-rs. | Inspection |

## Dependencies

- **Upstream**: [StR-004](../stakeholder/StR-004-thin-boundary-over-quire-rs.md); quire-rs [FR-084](ix://agent-ix/quire-rs/FR-084), [FR-085](ix://agent-ix/quire-rs/FR-085), [FR-058](ix://agent-ix/quire-rs/FR-058)
