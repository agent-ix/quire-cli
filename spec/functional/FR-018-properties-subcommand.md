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
| FR-018-AC-5 | Relative document paths resolve under `--scope` — **regardless of `--module`**, which pins the module set and never moves document resolution to the process directory (CR-011) — and a `..` or symlink-escape path is rejected by path-safety ([FR-005](./FR-005-path-safety.md)) before any load | Test (IT-097, IT-121) |
| FR-018-AC-7 | The command passes each document's **scope-relative path** to the engine, so an obligation source's `exclude:` globs bind this payload exactly as they bind `coverage --json` (upstream FR-053-AC-14); stdin passes no path, having no location a glob could match | Test (IT-098) |
| FR-018-AC-6 | (thin boundary) classification is delegated entirely to quire-rs; the CLI resolves paths, loads the module set, and renders ([StR-004](../stakeholder/StR-004-thin-boundary-over-quire-rs.md)) | Inspection (TC-090) |
| FR-018-AC-8 | The emitted `--json` envelope validates against quire-rs's **published** `schemas/output/properties-v1.schema.json`, read from the resolved dependency checkout rather than a vendored copy. The envelope is assembled here and nowhere else — quire-rs publishes the schema but never constructs one — so this is the only place the shape can be gated. The fixture must exercise the obligation branch, so conformance is not asserted over an empty payload | Test (IT-103) |
| FR-018-AC-9 | Against a module declaring **no obligation source**, every criterion record carries `obligation: null` and the payload still conforms to the published schema — the shape a corpus that has not adopted obligations sees | Test (IT-104) |
| FR-018-AC-10 | `--criteria` renders one block per criterion on stdout after the census — the row id, a `document:line` locus, the shape and extraction state, and each extraction span that was decomposed. A span that was not extracted prints nothing rather than an empty label. The default set is the actionable one (`extractable`, specific-shape); `--all` includes `example` and `unclassified`, and `--all` without `--criteria` is rejected by the parser (CR-012) | Test (IT-119) |

> **CR note (authored after the fact, 2026-08-16):** authored alongside
> [FR-017](./FR-017-coverage-subcommand.md) for the same reason — the command
> shipped with no owning requirement in this repo (agent-ix/quire-cli#31). Its
> `--json` payload is the interface the Phase C `spec-correctness` work keys
> on, so "no FR" meant the contract that work depends on was stated nowhere on
> this side of the boundary.

## Dependencies

- **Upstream**: [StR-004](../stakeholder/StR-004-thin-boundary-over-quire-rs.md) thin boundary over quire-rs; quire-rs [FR-052](ix://agent-ix/quire-rs/FR-052) (acceptance-criteria property classification), [FR-047](ix://agent-ix/quire-rs/FR-047) (the `ac` binding it shares).
- **Downstream**: the `spec-correctness` workflow, which turns these records into property tests keyed on `row_id`.

> **CR-012 note (2026-08-22):** AC-10 is new — the compact surface can drive
> `spec-correctness`. `agent-ix/quire-cli#59`; epic `agent-ix/quoin#197`.
>
> The entire default output was two lines — **869 bytes** on a 951-criterion
> corpus — and carried no `row_id`, `domain`, `precondition`, `oracle` or
> `signals`. Those fields were `--json`-only, and `--json` over the same corpus
> was **597,636 bytes (~149k tokens)**. quoin's `spec-correctness` skill
> consumes exactly the omitted fields, so the compact surface could not drive it
> at all and the only thing keeping the JSON tractable was the skill's own
> advice to scope per module.
>
> **The default set is the actionable one.** On that corpus 427 records were
> `example` — one scenario, `not-extractable` by construction — and 5 were
> `unclassified`. Rendering them is 432 of 951 blocks and none of the ones
> somebody could sit down and write a property for. `--all` is there for when
> the question is about the classifier rather than about the specification.
>
> **An unextracted span prints nothing**, rather than `domain:` with an empty
> value: an empty domain and an absent decomposition are different claims, and
> the second is the common one (quire-rs #241).

