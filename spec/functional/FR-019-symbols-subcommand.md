---
id: FR-019
title: "quire symbols subcommand"
type: FR
relationships:
  - target: "ix://agent-ix/quire-cli/spec/stakeholder/StR-004"
    type: "implements"
    cardinality: "1:1"
---

## Description

The CLI SHALL provide a `symbols` subcommand that reports the **extracted
source symbol table** as the engine built it (upstream
[FR-051](ix://agent-ix/quire-rs/FR-051), `agent-ix/quire-rs#309`).

```
quire symbols [--scope <DIR>] [--module <PATH>] [--language <LANG>]
              [--json | --format human|json|tsv]
```

## Why this exists

`extract` reads **documents**. `coverage` reports which symbols **bound**,
which is several transformations downstream of the walk. So no surface reported
which symbols the scanner *found*, and the only available method for sizing a
scanner defect was to reimplement `quire-rs`'s `src/symbols/python.rs` and diff
it against `ast.parse`.

That was done three times while sizing `agent-ix/quire-rs#274`, and produced
three answers over one tree — **386**, **490** and **5,263** lost declarations.
The loss figure is stable only under bare-name comparison; everything else
moves by an order of magnitude with how the port qualifies nested names, which
is precisely the part of the scanner under test.

**A defect in the scanner cannot be sized by a reimplementation of the
scanner.** The ports disagree exactly where the original is wrong.

## Behavior

### §A — The record

`--json` emits `{symbols: [...], by_language: [...], diagnostics: [...],
excluded_source_files, files}` on stdout. Each symbol carries its `path`,
qualified `symbol`, `kind`, `language`, `line`, `leading_line`, `end_line`,
`container`, identity `id`, and the two capability flags that decide its fate:
`binds_trace_ids` and `carries_implements`.

`leading_line` is reported beside `line` because a marker that failed to match
is written at the **annotation block**, not at the declaration, and that is the
line a reader has to edit.

### §B — The module is optional, and the difference is stated

`coverage` bails without a `traceability:` model because it has nothing to
reconcile. "What did the scanner find" is a question about the walk alone, so
this command answers it without one.

With a module, each record also carries the ids it bound. Without one, no
record does — and an unbound run and a repository nobody tagged otherwise
produce the same empty `trace_ids`. The human surface therefore says binding
was **not asked** rather than leaving a reader to infer it from a column of
zeroes.

### §C — Both denominators

`by_language` reports `symbols` examined **and** `binding_kinds` — the count of
symbols whose kind can bind a trace id at all. A binding rate drawn over the
first reads a tree of containers as untagged, which is the shape of the figure
EPIC `agent-ix/quire-rs#264` was opened on.

## Acceptance Criteria

| ID | Criteria | Verification |
|----|----------|--------------|
| FR-019-AC-1 | `quire symbols --scope <DIR>` exits **0** with no module supplied, rendering a per-language census on **stderr** and leaving stdout for the payload | Test (IT-130) |
| FR-019-AC-2 | `--json` emits one record per extracted symbol carrying `path`, `symbol`, `kind`, `language`, `line`, `leading_line`, `end_line`, `container`, `id`, `binds_trace_ids` and `carries_implements`; two runs over identical inputs are byte-identical | Test (IT-131) |
| FR-019-AC-3 | With `--module`, each record additionally carries the `trace_ids` it bound; **without** one, the human surface states that binding was not asked rather than reporting zero — an unbound run and an untagged repository are different facts | Test (IT-132) |
| FR-019-AC-4 | `by_language` carries both `symbols` and `binding_kinds`, so a binding rate is never drawn over the population of every symbol | Test (IT-133) |
| FR-019-AC-5 | Extraction diagnostics reach **stderr** in every output format: a file the extractor could not read is indistinguishable from a file with no declarations, and dropping it would silently shrink the table | Test (IT-134) |
| FR-019-AC-6 | Module resolution is the resolution `coverage` performs, shared rather than restated, so the two commands cannot disagree about which module is in scope for one invocation | Test (IT-135) |

## Dependencies

- **Upstream**: `agent-ix/quire-rs` FR-051 (source symbol extraction), FR-045
  (the record-id convention the `id` field follows)
- **Related**: [FR-017](./FR-017-coverage-subcommand.md), which reports the
  downstream half — what bound — over the same walk and the same declaration

## Constraints

| ID | Constraint | Type | Validation |
|----|------------|------|------------|
| FR-019-CON-1 | The command performs no analysis of its own. It reports the extraction and, when asked, the binding — both computed by the engine. A second scanner living in the CLI would reintroduce exactly the divergence this command exists to end | Design | Review |
