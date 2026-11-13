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
  - target: "ix://agent-ix/quire-rs/spec/functional/FR-058"
    type: "references"
  - target: "ix://agent-ix/quire-cli/spec/functional/FR-004"
    type: "extends"
---

# FR-030: quire relations subcommand

## Description

The CLI SHALL provide a `relations` subcommand that renders the two-way relation coverage report
of quire-rs [FR-084](ix://agent-ix/quire-rs/FR-084) over one repository or over several
repositories loaded as one corpus ([FR-085](ix://agent-ix/quire-rs/FR-085)).

```
quire relations (--scope <DIR> | --repo <ORG/REPO>=<DIR>...) [--module <PATH>]...
                [--relation <NAME>]... [--json] [--strict]
quire validate --okf --repo <ORG/REPO>=<DIR>... --module <PATH>... [--severity ...]
```

The subcommand is separate from `quire coverage` ([FR-017](./FR-017-coverage-subcommand.md)): that
command's `CoverageReport` payload is its stable interface and gains no row sets. The report
computation, the row states and the verification statuses belong to quire-rs; the CLI resolves
roots, applies path safety ([FR-005](./FR-005-path-safety.md)), and renders
([StR-004](../stakeholder/StR-004-thin-boundary-over-quire-rs.md)). It holds no archetype, verb or
chain name: which rows exist comes from the loaded modules' `traceability.required_relations`.

### One corpus input for `relations` and `validate --okf`

`--repo <ORG/REPO>=<DIR>` is repeatable. Each occurrence names a member identity and its repository
root, and the members are loaded as one corpus by quire-rs [FR-085](ix://agent-ix/quire-rs/FR-085). For `validate --okf` the CLI calls the quire-rs FR-085 extension of [FR-038](ix://agent-ix/quire-rs/FR-038), `validate_corpus` (FR-085-AC-16 to AC-21), passing one `(document_root, reference_root)` pair per member in flag order and the posture of FR-014. For `--repo o/a=<DIR>` the pair is `(<DIR>/spec, <DIR>)`, the same pair the single-repository path of FR-014 derives from `--scope` (quire-rs FR-049-AC-9), so path-based declarations resolve against the member's repository root; the member identity in each finding is the engine's source-member `repo` field (FR-085-AC-19). The same flag, with the same meaning, is accepted by
`quire validate --okf` ([FR-004](./FR-004-validate-subcommand.md), [FR-014](./FR-014-validate-okf-bundle.md)),
so a finding that depends on an edge into a sibling repository, such as a classified NFR whose
mitigated hazard lives elsewhere, is reachable from `validate` and not only from `relations`. With
`--repo`, `validate --okf` validates every member's documents against the one corpus and each
finding names the member identity. `--repo` together with `--scope`, or with the positional bundle `<DIR>` of FR-014's `--okf` form, is an error, and `--repo` without `--okf` is an error on `validate`.

`--scope` names one repository, loaded as a corpus of one member with no identity. `--module` and
its closed, ordered semantics are those of FR-017. Without `--module`, ambient module discovery is
bounded by the first `--repo` member only (the member's own `manifest.yaml` or repo-local modules are
used when it is listed first and ignored otherwise), so `--repo` order decides which modules load. `--relation` is repeatable and restricts the
report to the named declarations.

An error in this section exits 3 ([FR-007](./FR-007-exit-codes.md)) with a diagnostic naming the
cause; a run that emits a complete report and then fails `--strict` exits 1.

## Behavior

- Human output goes to stdout: one block per relation, one line per row, leading with the
  document id, then `state`, `verification`, and the linked document ids with their own verification;
  an `uncovered` row is marked as a gap and names the relation's `check` token. Dangling and
  kind-mismatch entries print beneath the row, naming the reference as authored. Each block ends with
  the relation's `summary` counts by `state` and `verification`.
- `--json` writes the quire-rs report on stdout and nothing else, with rows carrying document id,
  linked ids and per-document verification as the engine produced them. It honours `--pretty`
  ([FR-008](./FR-008-json-output-encoding.md)).
- The command is report-only. With `--strict` it emits the complete report and then exits 1 when any
  row is `uncovered` or has `verification` `unverified` or `partial`, when the selected relations
  produce zero rows, or when rows exist and every one is `not-tracked`. The last two print a
  diagnostic distinct from the row failures and from each other (`empty-population`,
  `nothing-measured`): a gate that measured nothing is not a pass. Without `--strict` all of these
  exit 0.
- A scope without `spec/` is `MissingDocumentRoot` ([FR-017](./FR-017-coverage-subcommand.md)-AC-5), a module set with no
  `required_relations` and an unknown `--relation` name are each invalid requests (exit 3) that compute nothing.

## Acceptance Criteria

| ID | Criteria | Verification |
|----|----------|--------------|
| FR-030-AC-1 | `quire relations --scope <DIR> --module $M --json` over a fixture corpus with an unmitigated risk, a mitigated-but-untested risk and a mitigated-and-tested risk emits rows whose `state` and `verification` are respectively `uncovered`, `covered`/`unverified`, `covered`/`verified`, each row carrying the document id, linked ids and per-linked-document verification. | Test |
| FR-030-AC-2 | In the same run, stdout holds only the JSON document, and two runs over identical inputs are byte-identical. | Test |
| FR-030-AC-3 | The default human output lists each row with its state and verification, and marks an `uncovered` row of a forward relation as a gap carrying the relation's `check` token. | Test |
| FR-030-AC-4 | With a fixture module declaring a reverse relation (a classified requirement obliged to mitigate a risk document), the human output marks a classified requirement with no linked risk document as a gap carrying that relation's `check` token. | Test |
| FR-030-AC-5 | Each human relation block ends with the relation's summary, and its counts equal the `summary` of the `--json` run over the same inputs. | Test |
| FR-030-AC-6 | `--repo o/a=<A> --repo o/b=<B>` over a fixture where a requirement in `o/a` mitigates a hazard in `o/b` shows the hazard `covered`; the same command without the `o/b` flag shows the reference under the row's dangling entries, naming it as authored. | Test |
| FR-030-AC-7 | A fixture edge to a wrong-kind target prints beneath its row as a kind mismatch naming the target and its actual archetype, in human and `--json` output. | Test |
| FR-030-AC-8 | `--relation <NAME>` restricts the report to that declaration. | Test |
| FR-030-AC-9 | An unknown `--relation` name exits 3, naming it on stderr, with no report on stdout. | Test |
| FR-030-AC-10 | Over a corpus with an `uncovered` row, `--strict` emits the complete report and exits 1, and the same invocation without `--strict` exits 0. | Test |
| FR-030-AC-11 | Over a corpus whose only non-covered-and-verified row is `unverified`, `--strict` exits 1; over one whose only such row is `partial`, `--strict` exits 1; the invocation without `--strict` exits 0 in both. | Test |
| FR-030-AC-12 | Over a corpus whose rows are all `verified`, `--strict` exits 0. | Test |
| FR-030-AC-13 | With `--strict`, a selection producing zero rows exits 1 with an `empty-population` diagnostic, and a corpus whose every row is `not-tracked` (a module declaring no `obligations:` source) exits 1 with a `nothing-measured` diagnostic; both exit 0 without `--strict`. | Test |
| FR-030-AC-14 | A `--scope` holding no `spec/` exits 3 with machine `kind` `MissingDocumentRoot` under `--diagnostics-format json`, naming the document root by path. | Test |
| FR-030-AC-15 | A module set declaring no `required_relations` exits 3 naming the missing declaration, and computes nothing. | Test |
| FR-030-AC-16 | `--scope` together with `--repo` exits 3 naming both flags. | Test |
| FR-030-AC-17 | A `--repo` whose root holds no `spec/` exits 3 with `MissingDocumentRoot` naming that root and the member identity. | Test |
| FR-030-AC-18 | A `--repo` root that fails path safety ([FR-005](./FR-005-path-safety.md)) exits 2 and reads nothing from it. | Test |
| FR-030-AC-19 | Two `--repo` flags with one identity exit 3 naming the identity (quire-rs FR-085-AC-9), and a `--repo` value that is not `<ORG/REPO>=<DIR>` exits 3 naming the value. | Test |
| FR-030-AC-20 | `quire validate --okf --repo o/a=<A> --repo o/b=<B> --module $M` over a fixture where a classified requirement in `o/a` mitigates a hazard in `o/b` carries no finding bearing the fixture relation's own `check` token for it, and with the `o/b` flag removed carries that finding, naming the member identity `o/a`. | Test |
| FR-030-AC-21 | `quire validate --repo ...` without `--okf` exits 3 naming the missing flag. | Test |
| FR-030-AC-23 | `quire validate --okf --repo o/a=<A> --scope <DIR>` exits 3 naming both flags, and `quire validate <DIR> --okf --repo o/a=<A>` (a positional bundle directory) exits 3 naming the positional argument and `--repo`. | Test |
| FR-030-AC-24 | `quire validate --okf --repo o/a=<A> --repo o/b=<B> --module $M` over a fixture where the module declares an `exclude: ['spec/fixtures/**']` glob on a trace target, with a broken fixture under `spec/fixtures/` in both members: the glob excludes the fixture in `o/a` and in `o/b` when it resolves against each member's own `<DIR>`, and without the glob each member reports its own finding, so each member's declarations resolve against its own `<DIR>`, not `<DIR>/spec`. | Test |
| FR-030-AC-25 | `quire relations --repo o/a=<A> --repo o/b=<B>` without `--module`, where `<A>` and `<B>` each hold a `manifest.yaml` declaring a differently named relation, reports only the relation of the first `--repo` member, in either order. | Test |
| FR-030-AC-22 | The CLI source holds no hazard, threat or safety-classification name, and delegates the report to quire-rs. | Inspection |

## Dependencies

- **Upstream**: [StR-004](../stakeholder/StR-004-thin-boundary-over-quire-rs.md); quire-rs [FR-084](ix://agent-ix/quire-rs/FR-084), [FR-085](ix://agent-ix/quire-rs/FR-085), [FR-058](ix://agent-ix/quire-rs/FR-058); [FR-007](./FR-007-exit-codes.md), [FR-005](./FR-005-path-safety.md)
- **Downstream**: `agent-ix/spec-objects-safety` FR-009 and `agent-ix/spec-objects-security` FR-008, whose cross-repository acceptance criteria run through `validate --okf --repo`
