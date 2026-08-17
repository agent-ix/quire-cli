---
id: FR-018
title: "quire properties subcommand"
type: FR
relationships:
  - target: "ix://agent-ix/quire-cli/spec/stakeholder/StR-004"
    type: "implements"
    cardinality: "1:1"
---

## Description

The CLI SHALL provide a `properties` subcommand that surfaces the quire-rs
acceptance-criteria **property classification** (upstream
[FR-052](ix://agent-ix/quire-rs/FR-052)) for one or more documents: for each
binding criterion, its property shape, whether a generator can extract a
property from it, and the `{domain, precondition, oracle}` spans when it is
universally quantified.

```
quire properties <DOC_OR_GLOB>... [--scope <DIR>] [--module <PATH>]
                 [--archetype <NAME>] [--json]
```

Like [FR-017](./FR-017-coverage-subcommand.md), this shipped without an owning
requirement; the criteria below are read off working code.

## Behavior

### §A — Per-criterion records

`--json` emits, on stdout, a `{documents: [{document, archetype, criteria:
[...]}]}` envelope. Each criterion record carries `row_id`, the classified
`shape` and `property`, the `extraction` outcome and its `extractable` boolean,
the `statement` text, its `line`, the `signals` that fired, and the
`{domain, precondition, oracle}` spans when the criterion is universally
quantified (`null` when it is not).

This payload is the **stable interface** — it is what the downstream
`spec-correctness` workflow consumes to key generated property tests on
`row_id`. The default human output is a census on stderr and may change.

### §B — Classification is data, never a verdict

A criterion that classifies as `Unclassified`, or whose `extraction` outcome is
`not-extractable`, is **reported, never failed**. The command has no `--strict`
and constructs no `GrammarFinding`: quire-rs FR-052-CON-1 forbids the shape
classification from being addressable by the severity registry, precisely so
authors are not steered into rewording criteria to satisfy a checker. StR-shaped
criteria legitimately score low and that is a description, not a defect
(quire-rs CR-020).

### §C — Module data supplies the vocabulary

The `property_idioms` registry comes from the discovered module set or
`--module`. A repository declaring nothing still gets full extraction reach: a
missed idiom degrades a *label*, never drops a criterion (quire-rs
FR-052-CON-4).

## Acceptance Criteria

| ID | Criteria | Verification |
|----|----------|--------------|
| FR-018-AC-1 | `quire properties <DOC> --module $M` over a document with binding criteria exits **0** and renders a per-criterion census on **stderr**, leaving stdout for the `--json` payload | Test (IT-093) |
| FR-018-AC-2 | `--json` emits a `{documents: [{document, archetype, criteria}]}` envelope with one record per binding criterion, each carrying `row_id`, `shape`, `property`, `extraction` and `extractable`, and two runs over identical inputs are byte-identical | Test (IT-094) |
| FR-018-AC-3 | A document whose archetype binds no criteria yields an empty record set and still exits **0** | Test (IT-095) |
| FR-018-AC-4 | An unclassifiable or non-extractable criterion is reported in the payload and never changes the exit code — the command has no failure mode driven by classification | Test (IT-096) |
| FR-018-AC-5 | Relative document paths resolve under `--scope`, and a `..` or symlink-escape path is rejected by path-safety ([FR-005](./FR-005-path-safety.md)) before any load | Test (IT-097) |
| FR-018-AC-7 | The command passes each document's **scope-relative path** to the engine, so an obligation source's `exclude:` globs bind this payload exactly as they bind `coverage --json` (upstream FR-053-AC-14); stdin passes no path, having no location a glob could match | Test (IT-098) |
| FR-018-AC-6 | (thin boundary) classification is delegated entirely to quire-rs; the CLI resolves paths, loads the module set, and renders ([StR-004](../stakeholder/StR-004-thin-boundary-over-quire-rs.md)) | Inspection (TC-090) |

> **CR note (authored after the fact, 2026-08-16):** authored alongside
> [FR-017](./FR-017-coverage-subcommand.md) for the same reason — the command
> shipped with no owning requirement in this repo (agent-ix/quire-cli#31). Its
> `--json` payload is the interface the Phase C `spec-correctness` work keys
> on, so "no FR" meant the contract that work depends on was stated nowhere on
> this side of the boundary.

## Dependencies

- **Upstream**: [StR-004](../stakeholder/StR-004-thin-boundary-over-quire-rs.md) thin boundary over quire-rs; quire-rs [FR-052](ix://agent-ix/quire-rs/FR-052) (acceptance-criteria property classification), [FR-047](ix://agent-ix/quire-rs/FR-047) (the `ac` binding it shares).
- **Downstream**: the `spec-correctness` workflow, which turns these records into property tests keyed on `row_id`.
