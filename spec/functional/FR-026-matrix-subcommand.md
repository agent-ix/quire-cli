---
id: FR-026
title: "quire matrix subcommand"
type: FR
relationships:
  - target: "ix://agent-ix/quire-cli/spec/stakeholder/StR-004"
    type: "implements"
    cardinality: "1:1"
  - target: "ix://agent-ix/quire-cli/spec/functional/FR-017"
    type: "extends"
    cardinality: "1:1"
---

## Description

The CLI SHALL provide a `matrix` subcommand that renders the quire-rs
**computed** coverage matrix (upstream
[FR-050](ix://agent-ix/quire-rs/FR-050)-AC-47..51, `CoverageReport.coverage_matrix`)
— requirement → criterion (derived obligation) → binder symbols → computed
status. This replaces the hand-written Test Matrix file: nobody authors a
matrix any more, `quire matrix` computes and renders one from the corpus and
its trace tags, on demand, every time.

```
quire matrix [--scope <DIR>] [--module <PATH>]...
             [--format markdown|json|tsv] [--strict]
             [--severity coverage:<check>=<level>]...
```

`--scope` and `--module` resolve exactly as they do for
[FR-017](./FR-017-coverage-subcommand.md) `coverage` (same two-roots-from-one-
scope split, same closed/repeatable module-set semantics) — one resolution
helper, shared, so the two commands never disagree about which module is in
scope for the same invocation.

**The matrix is a view, never an artifact.** `matrix` writes only to stdout.
It has no `--write` flag, no output-file argument, and no code path that opens
a file under `<scope>/spec` or anywhere else. A caller who wants the rendering
saved redirects stdout themselves; the command has no opinion about where a
shell redirect ends up.

## Behavior

### §A — Same engine call as `coverage`, one more field read

`matrix` computes the identical `CoverageReport` `coverage` does — same
`require_traceability_model` refusal when no module in scope declares a
`traceability:` model (FR-017-AC-6), same `MissingDocumentRoot` refusal when
`--scope` holds no `spec/` (FR-017-AC-5), same closed `--module` resolution
(FR-017-AC-20/21) — and renders `report.coverage_matrix` instead of the
unbacked/status-lie/untracked lists `coverage` renders. It is a second view
over one computation, not a second computation: the CLI reads no
document-reference declaration and scans no auxiliary trace-target source
itself (that is quire-rs's job, FR-050-AC-47) and introduces no matrix schema
of its own.

### §B — Zero population is a state, not silence

A module may declare no `obligations:` source, or declare one that derives no
obligations from the discovered corpus (FR-050-AC-51: `coverage_matrix` is
**omitted entirely** from the engine report in either case — "derives none" is
the one condition, regardless of which reason produced it). `matrix` renders
this as a fourth, non-error state, distinct from "every criterion computed
`untagged`" the same way FR-050-AC-14/CR-035 distinguish "found nothing" from
"all covered" for `coverage`: an absent `coverage_matrix` is reported as
itself in every format (AC-9), and — like the zero-rows-matched case it
mirrors — it fails `--strict` on its own, separately from an untagged
criterion (AC-10).

### §C — Markdown: one table per requirement, in the report's own order

Markdown groups criteria into one table per `requirements[]` entry
(FR-050-AC-48), in the order the engine already sorts them — by `document`,
the same scope-relative path every other report list sorts by. `matrix`
introduces no ordering of its own: it does not re-sort, re-group, or
alphabetize anything the engine handed it (AC-3).

Each requirement renders as a level-2 heading naming the requirement's
`document` path, followed by one table with four columns, in this order and
under these exact headers: `Criterion | Statement | Binders | Status`. Rows
follow the engine's own per-document criteria order (AC-4).

### §D — Binder rendering and the truncation rule

A criterion's `Binders` cell lists every `(path, line, column)` binder
(FR-050-AC-48) as `path:line`, ordered exactly as the engine orders them, each
entry separated by `, `. A binder the engine marks `ignored` (FR-051-AC-27)
renders with a trailing ` (ignored)` marker on that entry alone, so a
`tagged-by-ignored-test` criterion's evidence is visible in the same cell that
explains the status, without a reader cross-referencing JSON. A criterion with
zero binders renders the literal text `(none)` — never an empty cell, which
would be indistinguishable from a rendering bug (AC-5).

A criterion's `Statement` cell is the obligation's statement text with every
tab, newline and carriage return replaced by a single space (the same
structural-character guard `coverage --format tsv` already applies, FR-017-
AC-14), then truncated to **80 characters** measured in Unicode scalar values
(`chars().count()`, never bytes — the guard a multi-byte statement would
otherwise falsify): a statement at or under 80 characters renders in full; a
longer one is cut to the first 77 characters plus a trailing `...` marker, so
every truncated cell is exactly 80 characters wide and every reader can tell
truncation happened without opening `--format json` (AC-6). `--format json`
and `--format tsv` never truncate — 80 characters is a markdown-table
readability rule, not a data-loss rule (AC-6, AC-8).

### §E — JSON: the computed field verbatim, plus provenance

`--format json` emits, on stdout, `{"coverage_matrix": <value>, "engine":
{...}}` — the engine's own `coverage_matrix` value exactly as
`CoverageReport` carries it (present when the population is non-empty, the
`coverage_matrix` key **absent** when it is not, mirroring the engine's own
omission rather than coercing it to an empty array), plus the FR-008-CR-104
provenance block every JSON payload in this CLI carries. `matrix` introduces
no field of its own beside the envelope key and `engine` — the same
`engine::attach` call every other JSON surface uses (AC-7).

### §F — TSV: one flattened record per criterion

`--format tsv` emits a header naming five fixed columns —
`document criterion status binders statement` — then one record per
criterion, in the identical document/criterion order the markdown and JSON
forms use. `document` and `criterion` carry the requirement's document path
and the criterion's own `id`; `binders` flattens the same `path:line` list
markdown renders, comma-separated, with `(ignored)` on the same entries
markdown marks, and empty (not `(none)`) when there are zero; `statement` is
the full, untruncated, tab/newline-scrubbed text — TSV is a machine format,
and the 80-character rule is §D's markdown-only readability rule, not a wire
contract (AC-8). Zero-population renders the header alone, no data rows.

### §G — `--strict` and the coverage severity pack

`--strict` exits 1 when any criterion in `coverage_matrix` computes `untagged`
or `tagged-by-ignored-test`, or when `coverage_matrix` is itself absent
(§B); it exits 0 when every criterion computes `tagged` or
`method-without-symbol` and the field is present. `method-without-symbol`
**never** fails `--strict` on its own: FR-050-AC-49 defines it as the
declared-method exemption (mirroring `coverage`'s `no_source_symbol`
exemption, FR-050-AC-16) and wins over every other case regardless of binder
count — a criterion computing it is exempt from tagging by declaration, not
tagged and not a gap (AC-11).

`--severity coverage:<check>=<level>` is accepted and validated exactly as
`coverage` validates it (FR-017-AC-13, the closed four-check vocabulary,
rejected before any document is read) for command-surface consistency across
the two commands that share a scope/module/severity argument shape, but it
has **no effect** on `matrix`'s own output or `--strict` verdict:
`coverage_matrix` carries no `unbacked-row` / `status-lie` /
`untracked-symbol` / `undeclared-status` records to project — those are
`coverage`'s reference-based findings, a different population from the
obligation-derived one `matrix` renders. An `error`-promoted check with
findings in the underlying `CoverageReport` still fails the run (the same
promotion `coverage` applies to the full computation), because `matrix`
computes that same report even though it renders a different field of it
(AC-12).

### §H — Exit codes

`matrix` uses the FR-007 taxonomy unchanged: 0 success (including the
non-strict zero-population and fully-tagged cases), 1 for every refusal above
(`MissingDocumentRoot`, no `traceability:` model, a malformed or unknown
`--severity` entry, an `--strict` gate failure), 2 for an argv parse error. No
new exit code is introduced (AC-13).

## Acceptance Criteria

| ID | Criteria | Verification |
|----|----------|--------------|
| FR-026-AC-1 | `quire matrix --scope <DIR> --module $M` over a repository whose module declares an `obligations:` source that derives a non-empty population exits 0 and renders every requirement's criteria in every format | Test |
| FR-026-AC-2 | `matrix` shares `coverage`'s refusals verbatim: no `spec/` under `--scope` exits 1 as `MissingDocumentRoot` (FR-017-AC-5); no `traceability:` model in scope exits 1 naming the missing declaration (FR-017-AC-6); `--module` is repeatable and closed exactly as FR-017-AC-20/21 specify | Test |
| FR-026-AC-3 | Markdown requirement tables render in the engine's own `requirements[]` order (sorted by `document`); the CLI performs no additional sort, group or alphabetization | Test |
| FR-026-AC-4 | Each requirement is a level-2 heading naming its `document` path, followed by one table with header `Criterion \| Statement \| Binders \| Status`, rows in the engine's own per-document criteria order | Test |
| FR-026-AC-5 | A criterion's `Binders` cell lists each `(path, line, column)` binder as `path:line`, separated by `, `, in the engine's own binder order; a binder the engine marks `ignored` carries a trailing ` (ignored)`; a criterion with zero binders renders the literal `(none)` | Test |
| FR-026-AC-6 | A `Statement` cell has every tab/newline/CR replaced by a space, then is left intact at ≤80 Unicode scalar values or cut to 77 scalar values plus a trailing `...` above that — every rendered cell is ≤80 characters, and `--format json`/`--format tsv` carry the untruncated text for the identical criterion | Test |
| FR-026-AC-7 | `--format json` emits `{"coverage_matrix": <value>, "engine": {cli, engine, capabilities}}` on stdout; the `coverage_matrix` key is present with the engine's own value when the population is non-empty and **absent** (not `null`, not `[]`) when the engine omits it (FR-050-AC-51); two runs over identical inputs are byte-identical | Test |
| FR-026-AC-8 | `--format tsv` emits header `document\tcriterion\tstatus\tbinders\tstatement`, one untruncated record per criterion in the JSON/markdown order, `binders` comma-separated with the same `(ignored)` marker and empty (not `(none)`) at zero, `statement` tab/newline-scrubbed but never truncated; a zero-population report emits the header alone | Test |
| FR-026-AC-9 | When `coverage_matrix` is absent from the engine report (no `obligations:` source declared, or one declared that derives nothing), every format renders that state as itself — markdown emits a stated zero-population line and no requirement table, JSON omits the key, TSV emits the header alone — never fabricating an empty-but-present matrix | Test |
| FR-026-AC-10 | `--strict` exits 1 when `coverage_matrix` is absent, exactly as it exits 1 when a criterion computes `untagged` or `tagged-by-ignored-test` — "nothing to render" and "every criterion tagged" are opposite states and must not share an exit code (FR-050-AC-14/CR-035 argument, applied to the derived-obligation population) | Test |
| FR-026-AC-11 | A criterion computing `method-without-symbol` never fails `--strict` on its own, regardless of its binder count, and renders in every format exactly like any other criterion — its own `Status`/`status` cell/field states `method-without-symbol` verbatim | Test |
| FR-026-AC-12 | `--severity coverage:<check>=<level>` is parsed and validated identically to `coverage` (closed four-check vocabulary, rejected before any document is read) but changes nothing in `matrix`'s rendered output or `--strict` verdict, because `coverage_matrix` carries none of the four checks' record kinds; an `error`-promoted check with findings elsewhere in the same computed `CoverageReport` still fails the run | Test |
| FR-026-AC-13 | `matrix` uses only the FR-007 exit codes — 0, 1, 2 — across every case above; no new code is introduced | Test |
| FR-026-AC-14 | `matrix` never opens a file for writing: no flag names an output path, and running it against a fixture repository leaves every file under `<scope>/spec` byte-identical before and after the run | Test |

## Dependencies

- **Upstream**: [FR-017](./FR-017-coverage-subcommand.md) `coverage` (shared scope/module resolution, shared severity-pack validation, shared `CoverageReport` computation); [FR-008](./FR-008-json-output-encoding.md) JSON provenance envelope (`engine::attach`); [StR-004](../stakeholder/StR-004-thin-boundary-over-quire-rs.md) thin boundary; quire-rs [FR-050](ix://agent-ix/quire-rs/FR-050)-AC-47..51 (`coverage_matrix` computation, CR-187), [FR-051](ix://agent-ix/quire-rs/FR-051)-AC-27/28 (binder `ignored` marking, range-in-trace-tag).
- **Downstream**: none yet; this command supersedes the hand-written Test Matrix workflow — `spec/tests.md` in this and other repositories stops being hand-authored once a consuming workflow switches to `quire matrix` output. `agent-ix/quire-cli#79` (per-reference status columns) is superseded on landing: status is computed per criterion here, so a document-reference table no longer needs its own status column to make the same claim.

> **CR note (spec authored ahead of implementation, 2026-09-27, PLAT-1078):**
> this document is authored against quire-rs spec FR-050-AC-47..51/FR-051-AC-27/28
> (merged, quire-rs PR #494) while the quire-rs implementation itself (PLAT-1077)
> and this CLI's own implementation are still in progress. The ACs above are
> therefore proposed against a declared upstream contract, not read off shipped
> code — the reverse of FR-017/FR-018's backfill posture. `coverage_matrix`'s
> exact JSON key names are quoted from the upstream spec prose (`requirements[]`,
> `criteria[]`, `binders`, `status`); this FR describes the CLI's rendering
> obligations in terms robust to that shape rather than asserting an internal
> field layout no Rust struct yet exists to check.
