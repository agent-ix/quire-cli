---
id: FR-017
title: "quire coverage subcommand"
type: FR
relationships:
  - target: "ix://agent-ix/quire-cli/spec/stakeholder/StR-004"
    type: "implements"
    cardinality: "1:1"
---

## Description

The CLI SHALL provide a `coverage` subcommand that surfaces the quire-rs
declarative coverage rollup (upstream [FR-050](ix://agent-ix/quire-rs/FR-050))
over a repository, reconciling the trace ids a module's `traceability:` model
declares against the source symbols that carry trace tags.

```
quire coverage [--scope <DIR>] [--module <PATH>] [--json] [--strict]
```

This command shipped in v0.13.0 and was authored as a requirement only in
2026-08-16 (see the CR note below). Its criteria are **read off working code**,
not proposed: it is the most behavior-visible surface this CLI has added in
three releases, and it had no FR, no acceptance criteria and no matrix rows of
its own.

## Behavior

### §A — Two roots from one scope

`--scope` names the repository root. Two roots derive from it and are never
interchanged (quire-rs FR-050-AC-17, CR-045):

- the **document root** `<scope>/spec`, which the corpus is walked from;
- the **code root** `<scope>`, walked for source symbols with the document
  root excluded.

`<scope>` stays the relativization base for every path in the report, so a
compliant repository's output is byte-identical to a pre-split run. A `--scope`
holding no `spec/` is a named error (`MissingDocumentRoot`), never a silent
fallback to walking the scope. `spec/` is convention, not configuration.

### §B — The model is module data

The `traceability:` model comes from the discovered module set, or from
`--module`. With no model in scope the command **refuses**, naming the missing
declaration — it does not guess, since guessing is exactly the agent-grep
behavior the command replaces.

### §C — Report, not verdict

Default output is a human census on **stderr**; `--json` emits the
`CoverageReport` payload on **stdout**, which is the **stable interface** (the
human form may change). The split is deliberate: stdout carries the payload and
nothing else, so `quire coverage --json | jq` is safe with the census still
visible on a terminal. `--strict` exits 1 when any row is unbacked or any status is
contradicted; it is **off by default**, because whether a gap blocks is the
consuming workflow's policy, not this command's
([FR-050-CON-1](ix://agent-ix/quire-rs/FR-050)).

A model matching **zero** rows is reported as itself and fails `--strict`
separately from a clean run — "found nothing" and "all covered" are opposite
states (quire-rs FR-050-AC-14, CR-035).

## Acceptance Criteria

| ID | Criteria | Verification |
|----|----------|--------------|
| FR-017-AC-1 | `quire coverage --scope <DIR> --module $M` over a repository whose model matches rows exits **0** and renders a human census on **stderr**, leaving stdout empty so the `--json` payload is the only thing that ever reaches it | Test (IT-089) |
| FR-017-AC-2 | `--json` emits the `CoverageReport` payload on stdout, and two runs over identical inputs are byte-identical | Test (TC-740) |
| FR-017-AC-3 | The document root is `<DIR>/spec`: a perfectly typed matrix at the repository root or under `plan/` mints nothing, and repo-root `README.md`/`CHANGELOG.md` are never read as documents | Test (TC-810) |
| FR-017-AC-4 | The code walk excludes the document root: a trace-tagged source file under `<DIR>/spec` contributes no symbol, while the same file outside it does | Test (IT-087) |
| FR-017-AC-5 | A `--scope` holding no `spec/` exits non-zero with a diagnostic naming the missing document root by path, carrying machine `kind` `MissingDocumentRoot` under `--diagnostics-format json` | Test (TC-811, IT-086) |
| FR-017-AC-6 | With no `traceability:` model in scope the command exits non-zero naming the missing declaration, and computes nothing | Test (TC-740) |
| FR-017-AC-7 | `--strict` exits 1 when a row is unbacked or a status is contradicted, and exits 0 on the same repository without it | Test (IT-092) |
| FR-017-AC-8 | A model matching zero rows fails `--strict` with a diagnostic distinct from the unbacked-rows one, rather than reporting full coverage | Test (TC-797) |
| FR-017-AC-9 | (thin boundary) reconciliation is delegated entirely to quire-rs (`compute_coverage`, `extract_tree_scoped`, `trace::bind`); the CLI resolves the two roots, applies path-safety, and renders ([StR-004](../stakeholder/StR-004-thin-boundary-over-quire-rs.md)) | Inspection (TC-090) |
| FR-017-AC-10 | A row whose status value the module's `traceability.status` classes as nothing is reported on **both** surfaces — as an `undeclared_statuses` entry in `--json` and as a rendered line in the default human census — carrying the authored value verbatim. `--strict` does not gate on it: the exit code for a report whose only finding is an undeclared status is the same with and without the flag (CR-083) | Test (IT-107) |
| FR-017-AC-11 | A module-declared `traceability.source_exclude` glob reaches the source walk: a tagged file matching one contributes no symbol, a tagged file outside every glob still does, and a scope whose module declares none behaves exactly as before (CR-085) | Test (IT-108) |
| FR-017-AC-12 | Each human-census unbacked-row, status-lie and undeclared-status line leads with the row's own id when the record carries one (the declaration names a `row_id_column`) and keeps the reference kind visible in a bracketed trailer — `TC-123 (doc.md) has no backing symbol [traces-to]` — so two rows in the same document render distinguishable lines; a record without a row id renders the reference kind leading, exactly as before. The `--json` payload is unchanged (#51) | Test (IT-109, IT-107) |

> **CR note (authored after the fact, 2026-08-16):** this document did not
> exist while the command shipped, changed its default root (PR #27) and
> changed what it parses (PR #29). The gap was found by
> agent-ix/quire-cli#31: `coverage` and `properties` had no owning requirement
> anywhere in this repo's spec, so three releases of behavior change had
> nothing to be measured against and no matrix row to contradict. The criteria
> above are read off the shipped command and its existing tests rather than
> proposed, on the FR-019/FR-020/FR-022 backfill pattern from quire-rs CR-042.

## Dependencies

- **Upstream**: [StR-004](../stakeholder/StR-004-thin-boundary-over-quire-rs.md) thin boundary over quire-rs; quire-rs [FR-050](ix://agent-ix/quire-rs/FR-050) (declarative coverage computation), [FR-051](ix://agent-ix/quire-rs/FR-051) (source-symbol extraction).
- **Downstream**: the `gap-analysis` workflow, which consumes the `--json` payload as data and owns the verdict policy.

> **CR-011 note (2026-08-20):** AC-10 and AC-11 are new — the CLI carries the
> two capabilities quire-rs v0.41.0 added, in the same release rather than the
> next one.
>
> Both are pure reachability. `undeclared_statuses` (quire-rs CR-083) and
> `source_exclude` (CR-085) are **inert** until this crate renders the one and
> passes the other: an engine field nothing prints is a finding nobody reads,
> and a manifest key nothing forwards is a declaration with no effect whatsoever.
>
> That is the defect the previous programme phase kept finding — `quire-cli` five
> releases behind, so FR-057..061 were unreachable from any command line;
> `vocabulary_coverage` shipped and declared by no module. The mechanical tell it
> recommends is **grep for the symbol from the surface a user actually invokes**,
> and IT-107/IT-108 are that grep written down.
>
> `--strict` deliberately does **not** gate on an undeclared status. Shipping it
> as a gate would flip repositories red on an engine bump for a condition nobody
> has been told about; promotion is a separate, measured, user-gated decision
> once the corpus is clean. IT-107 asserts the exit code is unchanged, over a
> fixture whose rows are all backed so the undeclared status is the only finding
> — otherwise the assertion would pass on unbacked rows, which `--strict` gates
> on legitimately.

> **CR note (row-id-leading findings, 2026-08-21, #51):** AC-12 is new. The
> human renderer printed the reference **kind** per finding line (`traces-to
> (spec/tests.md) has no backing symbol`, four identical lines for four rows),
> while `UnbackedRow`, `StatusLie` and `UndeclaredStatus` all already carry
> `row_id: Option<String>` — the id was computed, serialized to `--json`, and
> never rendered. That omission is why downstream consumers parsed the
> megabyte-scale JSON payload to answer questions the human output had
> already computed. Of the row-id-carrying record kinds, `no_symbol_rows`
> still has no human renderer at all (JSON-only — part of #51's remaining
> scope, with the `path:line` prefix that waits on the engine). The stdout/
> stderr split of AC-1 is deliberately untouched here.

