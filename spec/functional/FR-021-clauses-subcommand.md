---
id: FR-021
title: "quire clauses subcommand"
type: FR
relationships:
  - target: "ix://agent-ix/quire-cli/spec/stakeholder/StR-004"
    type: "implements"
    cardinality: "1:1"
  - target: "ix://agent-ix/quire-rs/spec/functional/FR-040"
    type: "consumes"
    cardinality: "1:1"
---

# FR-021: Clause-set evaluation and comparison

## Description

The CLI SHALL expose the generic rights-aware clause-set model supplied by
`quire-rs` without embedding a publication, authority, rule inventory, or
domain classification in the binary.

```text
quire clauses evaluate --module <PATH> [--module <PATH>]... --authority <ID> --set <ID>
  --version <VERSION> [--context KEY=VALUE]... [--format human|json|tsv]

quire clauses diff --module <PATH> [--module <PATH>]... --authority <ID> --set <ID>
  --before-version <VERSION> --after-version <VERSION>
  [--format human|json|tsv]
```

Each command SHALL require one or more explicitly supplied `--module <PATH>`
roots. It SHALL strictly load those roots as one closed module set, merging the
active vocabulary across that set. It SHALL NOT discover or add modules from
ambient search paths or environment configuration.

## Acceptance Criteria

| ID | Criteria | Verification |
|----|----------|--------------|
| FR-021-AC-1 | Both commands require one or more `--module <PATH>` occurrences, path-check every supplied root, and strictly load exactly those roots as a closed module set. A malformed or unloadable supplied module exits non-zero before writing a result; neither command discovers or adds a module from ambient search paths or environment configuration. Authority, set id, and version are selected exactly; a missing version or invalid referenced set is a user error, never a nearest-version guess. | Test |
| FR-021-AC-2 | `evaluate` accepts repeatable `KEY=VALUE` context dimensions and delegates applicability to the engine. Duplicate or malformed context is rejected. | Test |
| FR-021-AC-3 | Evaluation preserves `binding`, `not_binding`, and `unresolved`; missing context remains unresolved with its reason. | Test |
| FR-021-AC-4 | `diff` delegates to the engine and reports added, removed, and changed clauses between two exact versions of the same authority and set id, including the output-catalog entries added, removed, and changed between them. | Test |
| FR-021-AC-5 | Human results and deterministic TSV are emitted on stdout; TSV escapes structural characters so one clause remains one row. | Test |
| FR-021-AC-6 | JSON carries tool provenance including the `clause_sets` capability and conforms to the pinned engine's published binding or diff schema. | Test |
| FR-021-AC-7 | The CLI contains only transport and rendering policy. Clause content, rights validation, applicability, and comparison remain engine/module concerns. | Test, Review |
| FR-021-AC-8 | Given a clause-set-owning module with an exact authority/set and two versions, plus an archetype whose authored `allowed_links` uses `registered-verb`, and a second module that registers `registered-verb` in `edge_types`, explicitly supplying both module roots makes `evaluate` and `diff` each emit a successful result for that authority/set. Supplying only the clause-set-owning root makes both commands exit 3 with an `UnknownEdgeType` diagnostic naming `registered-verb` and empty stdout, even when the vocabulary module is available through ambient module discovery. | Test |
| FR-021-AC-9 | The human, TSV, and JSON `diff` views show each added, removed, and changed `outputCatalog` entry, with its producers when present. A changed entry shows its before and after values in the human view, and the TSV `changed_fields` cell names which of `kind`, `description`, and `producers` changed, in that order, comma-separated. A TSV producers cell is the producer kinds as a JSON array, so a kind containing a comma stays distinct from two kinds; the human view uses the same JSON array form. | Test |
| FR-021-AC-10 | When a clause set's `outputCatalog` entry has a producers list that is empty, or that holds an empty, whitespace-padded, or duplicate producer kind, both `evaluate` and `diff` exit 3 with empty stdout and a diagnostic that names the output. | Test |

## Constraints

The repository and its fixtures SHALL contain only original synthetic clause
content. Restricted source material is not a CLI fixture and is not required
to exercise this interface.

## Dependencies

- **Upstream:** [quire-rs FR-073](ix://agent-ix/quire-rs/FR-073) owns the
  generic clause-set model, rights validation, applicability, and comparison.
- **Upstream:** [quire-rs FR-040](ix://agent-ix/quire-rs/spec/functional/FR-040)
  owns the merged `edge_types` and `roles` vocabularies used to strictly load
  explicitly supplied module sets.
- **Upstream:** [quire-rs FR-055](ix://agent-ix/quire-rs/FR-055) owns the
  published binding and diff output contracts.
- **Local boundary:** [StR-004](../stakeholder/StR-004-thin-boundary-over-quire-rs.md)
  keeps engine semantics outside the CLI.
- **Local process contract:** [FR-005](./FR-005-path-safety.md),
  [FR-006](./FR-006-io-contract.md), [FR-007](./FR-007-exit-codes.md), and
  [NFR-005](../non-functional/NFR-005-diagnostic-format.md) govern path checks,
  streams, typed outcomes, and diagnostic formatting.
