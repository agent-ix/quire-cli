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
its trace tags, on demand, every time. The rendered result goes on **stdout**;
refusal diagnostics go on **stderr** — the same split every other subcommand
uses (FR-006).

```
quire matrix [--scope <DIR>] [--module <PATH>]...
             [--format markdown|json|tsv] [--strict]
```

`--scope` and `--module` resolve exactly as they do for
[FR-017](./FR-017-coverage-subcommand.md) `coverage` (same two-roots-from-one-
scope split, same closed/repeatable module-set semantics) — one resolution
helper, shared, so the two commands never disagree about which module is in
scope for the same invocation. `matrix` does **not** accept `--severity`; §G
states why.

**The matrix is a view, never an artifact.** `matrix`'s rendered result goes to
stdout only. It has no `--write` flag, no output-file argument, and no code
path that opens a file anywhere under `<scope>` for writing. A caller who
wants the rendering saved redirects stdout themselves; the command has no
opinion about where a shell redirect ends up.

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

Every rendered field — `id`, `method`, `binders`, `status`, and `statement` —
is read from the engine's own criterion record and never re-derived.
`statement` is FR-050-AC-48 **as amended by PLAT-1077** (quire-rs, tracking
this same effort): the criterion entry carries the obligation's own statement
text, verbatim and untruncated — quire-rs FR-053 defines `Obligation.statement`
as the source cell's text verbatim, and the matrix criterion carries that same
value unchanged — alongside `id`/`method`/`binders`/`status`.
`matrix` reads that field verbatim; it does not call `obligation::derive`
itself, does not re-read the obligation's source document, and does not
reconstruct a statement from any other field or any other surface.

### §B — Zero population is zero criteria, and has one shape

`coverage_matrix` is a JSON array of requirement entries (the
`requirements[]` of FR-050-AC-47/48 is the array itself), and the engine omits
the key whenever that array would be empty (FR-050-AC-51: `Vec` with
`skip_serializing_if = "Vec::is_empty"` at quire-rs). The
zero-population state — **zero criteria** — therefore has exactly one shape:
the key is absent. Markdown renders it as the single empty-state line (§C),
TSV as the header alone (§F), `--strict` fails it (§G, AC-10), and
`--format json` omits the key exactly as the engine does (§E).

### §C — Markdown: one table per requirement, in the report's own order

Markdown groups criteria into one table per `requirements[]` entry
(FR-050-AC-48), in the order the engine already sorts them — by `document`,
the same scope-relative path every other report list sorts by. `matrix`
introduces no ordering of its own: it does not re-sort, re-group, or
alphabetize anything the engine handed it (AC-3).

Each requirement renders as the exact heading `## <document>`, where
`<document>` is the requirement's own `document` value verbatim, exactly as
upstream gives it — no link, no backticks, no relative-path rewriting. A table
with four columns follows, in this order and under these exact headers:
`Criterion | Statement | Binders | Status`. Rows follow the engine's own
per-document criteria order (AC-4).

Zero criteria (§B) renders no heading and no table; the entire stdout body is
the single literal line, followed by nothing else:

```
No obligations matched this scope.
```

This is the one markdown output that is not a table (AC-9).

A `|` in a `Statement` cell would otherwise split the row into extra table
columns, so every `|` remaining in a rendered cell is escaped to `\|`
(Markdown table syntax — this is escaping, not a data transformation: the
escaped text still reads as one `|`). Escaping happens **after** truncation
(§D): the raw statement is normalized and cut to its final length first, and
only then is every `|` in that already-cut string escaped — so an escape can
never itself be sliced in half by the 77-scalar-value cut, and the backslash
it adds is never counted toward the 80-character budget. Worked example, a
criterion whose full `statement` is `Requests carrying X-Debug|X-Trace
headers are logged verbatim` (61 characters, under the 80-character budget,
rendered whole):

```
## spec/functional/FR-009-debug-headers.md

| Criterion | Statement | Binders | Status |
|---|---|---|---|
| FR-009-AC-3 | Requests carrying X-Debug\|X-Trace headers are logged verbatim | src/debug.rs:41:9 | tagged |
```

### §D — Binder rendering and the truncation rule

A criterion's `Binders` cell lists every `(path, line, column)` binder
(FR-050-AC-48) as `path:line:column` — all three coordinates, never just
`path:line` — ordered exactly as the engine orders them, each entry separated
by `, `. The column is not optional: FR-050-AC-48 keeps two binders that open
on the same line (two `it(...)` calls whose callback opens on one line,
PLAT-882) distinct precisely by column, and a `path:line`-only rendering would
print `a.ts:5, a.ts:5` for them — collapsing the one piece of evidence
FR-050-AC-48 exists to keep apart. A binder the engine marks `ignored`
(FR-051-AC-27) renders with a trailing ` (ignored)` marker on that entry
alone, so a `tagged-by-ignored-test` criterion's evidence is visible in the
same cell that explains the status, without a reader cross-referencing JSON. A
criterion with zero binders renders the literal text `(none)` — never an
empty cell, which would be indistinguishable from a rendering bug (AC-5).

A criterion's `Statement` cell is the criterion's own `statement` field (§A)
with every tab, newline and carriage return replaced by a single space (the
same structural-character guard `coverage --format tsv` already applies,
FR-017-AC-14), **then** truncated to **80 characters** measured in Unicode
scalar values (`chars().count()`, never bytes — the guard a multi-byte
statement would otherwise falsify): a statement at or under 80 characters
renders in full; a longer one is cut to the first 77 scalar values plus a
trailing `...` marker, so every truncated cell is exactly 80 characters wide
before pipe-escaping (§C) is applied on top. **Truncation always happens
before escaping, never after** — the 80-character budget is spent entirely on
content, and an escape added afterward is never itself cut in half by it.
`--format json` and `--format tsv` never truncate and never pipe-escape — 80
characters and `\|` are markdown-table readability rules, not a data-loss
rule or a wire contract (AC-6, AC-8).

### §E — JSON: the computed field verbatim, plus provenance

`--format json` emits, on stdout, `{"coverage_matrix": <value>, "engine":
{...}}` — the engine's own `coverage_matrix` value exactly as `CoverageReport`
carries it: a bare array of `{document, criteria}` requirement entries, never
re-wrapped by this CLI in an object of its own. The key is **absent** when the
engine omits it (zero population, §B) and is never emitted present-but-empty.
The CLI adds no shape of its own: whatever the engine serializes for the
field is what this surface emits, verbatim. The envelope adds the
FR-008-CR-104 provenance block every JSON payload in this CLI carries, and
nothing else — the same `engine::attach` call every other JSON surface uses
(AC-7).

### §F — TSV: one flattened record per criterion

`--format tsv` emits a header naming five fixed columns —
`document criterion status binders statement` — then one record per
criterion, in the identical document/criterion order the markdown and JSON
forms use. `document` and `criterion` carry the requirement's document path
and the criterion's own `id`; `binders` flattens the same `path:line:column`
list markdown renders, comma-separated, with `(ignored)` on the same entries
markdown marks, and empty (not `(none)`) when there are zero; `statement` is
the full, untruncated, unescaped, tab/newline-scrubbed text — TSV is a machine
format, and §D/§C's readability rules are markdown-only, not a wire contract
(AC-8). Zero criteria (§B) renders the header alone,
no data rows.

### §G — `--strict`, and no severity pack

`matrix` does not run the `coverage` severity pack's checks (`unbacked-row`,
`status-lie`, `untracked-symbol`, `undeclared-status`) and does not accept
`--severity` — the flag is absent from its argv surface entirely, so passing
it is an ordinary clap unknown-argument failure (FR-007-AC-5, exit 3), not a
recognized-but-rejected flag. A module's own `grammar_severity` promotions of
those four checks have **no effect** on `matrix`'s exit code:
`coverage_matrix` carries none of those checks' record kinds to project or
promote, and `matrix` computes its verdict from `coverage_matrix` alone, never
from any other part of the same `CoverageReport`.

`matrix`'s entire verdict surface is:

- **`--strict`** exits 1 when any criterion in `coverage_matrix` computes
  `untagged` or `tagged-by-ignored-test`, or when the population is zero
  criteria (§B); it exits 0 only when there is **at
  least one** criterion and every one computes `tagged` or
  `method-without-symbol`. `method-without-symbol` **never** fails `--strict`
  on its own, regardless of its binder count: FR-050-AC-49 defines it as the
  declared-method exemption (mirroring `coverage`'s `no_source_symbol`
  exemption, FR-050-AC-16) and wins over every other case — a criterion
  computing it is exempt from tagging by declaration, not tagged and not a
  gap (AC-10, AC-11).
- the **FR-007** invalid-request exits: 3 for a load/resolution failure
  (`MissingDocumentRoot`, no `traceability:` model, a corrupt module), 2 for
  an argv parse error — identically to every other subcommand.

`matrix` deliberately does **not** inherit `coverage`'s FR-017-AC-22 unread-
measurement gate (`status-column-matches-nothing` / `hollow-denominator`):
that gate exists because `coverage` reconciles a document-reference **status
column** against test evidence, and a column nothing could read invalidates
that specific measurement. `coverage_matrix` reads no status column at all —
it is built solely from the module's `obligations:`-derived criteria and
their binders (FR-050-AC-47) — so there is no unread-column state for
`matrix` to have an opinion about (AC-12).

### §H — Exit codes

`matrix` uses the FR-007 taxonomy unchanged: 0 success (including the
non-strict zero-population and fully-tagged cases), 1 for every refusal above
(`MissingDocumentRoot`, no `traceability:` model, an `--strict` gate failure),
2 for an argv parse error — including an unrecognized `--severity` flag, which
`matrix` does not define (§G). No new exit code is introduced (AC-13).

## Acceptance Criteria

| ID | Criteria | Verification |
|----|----------|--------------|
| FR-026-AC-1 | `quire matrix --scope <DIR> --module $M` over a repository whose module declares an `obligations:` source that derives a non-empty population exits 0 and renders every requirement's criteria in every format | Test |
| FR-026-AC-2 | `matrix` shares `coverage`'s refusals verbatim: no `spec/` under `--scope` exits 3 as `MissingDocumentRoot` (FR-017-AC-5); no `traceability:` model in scope exits 3 naming the missing declaration (FR-017-AC-6); `--module` is repeatable and closed exactly as FR-017-AC-20/21 specify, and `quire matrix --help` states the identical `--module` resolution-order text FR-017-AC-21 requires of `coverage --help` | Test |
| FR-026-AC-3 | Markdown requirement tables render in the engine's own `requirements[]` order (sorted by `document`); the CLI performs no additional sort, group or alphabetization | Test |
| FR-026-AC-4 | Each requirement renders as the exact heading `## <document>` with the requirement's own `document` value verbatim (no link, no backticks), followed by one table with header `Criterion \| Statement \| Binders \| Status`, rows in the engine's own per-document criteria order | Test |
| FR-026-AC-5 | A criterion's `Binders` cell lists each `(path, line, column)` binder as `path:line:column`, separated by `, `, in the engine's own binder order — never collapsing two same-line binders that differ only by column; a binder the engine marks `ignored` carries a trailing ` (ignored)`; a criterion with zero binders renders the literal `(none)` | Test |
| FR-026-AC-6 | A `Statement` cell is the criterion's own `statement` field (never re-derived) with every tab/newline/CR replaced by a space, truncated first — left intact at ≤80 Unicode scalar values or cut to 77 scalar values plus a trailing `...` above that — and only then pipe-escaped (`\|`), so escaping never splits the cut and never counts toward the 80-character budget; `--format json`/`--format tsv` carry the untruncated, unescaped text for the identical criterion | Test |
| FR-026-AC-7 | `--format json` emits `{"coverage_matrix": <value>, "engine": {cli, engine, capabilities}}` on stdout; the `coverage_matrix` key carries the engine's own value unmodified — a bare array of requirement entries, never re-wrapped in an object — and is absent when the engine omits it (zero population), never present-but-empty; two runs over identical inputs are byte-identical | Test |
| FR-026-AC-8 | `--format tsv` emits header `document\tcriterion\tstatus\tbinders\tstatement`, one untruncated, unescaped record per criterion in the JSON/markdown order, `binders` comma-separated as `path:line:column` with the same `(ignored)` marker and empty (not `(none)`) at zero; a zero-criteria report (§B) emits the header alone | Test |
| FR-026-AC-9 | The zero-population state — `coverage_matrix` absent, whether the module declares no `obligations:` source or declares one that derives nothing — renders in markdown as the single line `No obligations matched this scope.` (no heading, no table) and in TSV as the header alone; `--format json` omits the key, as the engine does | Test |
| FR-026-AC-10 | `--strict`'s pass condition requires **at least one** criterion, all `tagged`/`method-without-symbol`: it exits 1 on the zero-population state (§B) exactly as it exits 1 on an `untagged`/`tagged-by-ignored-test` criterion — a report measuring nothing must not exit 0 (FR-050-AC-14/CR-035 argument, applied to the derived-obligation population) | Test |
| FR-026-AC-11 | A criterion computing `method-without-symbol` never fails `--strict` on its own, regardless of its binder count, and renders in every format exactly like any other criterion — its own `Status`/`status` cell/field states `method-without-symbol` verbatim | Test |
| FR-026-AC-12 | `matrix` does not accept `--severity` (an attempt is an ordinary clap argv error, exit 3, FR-007-AC-5) and runs none of the `unbacked-row`/`status-lie`/`untracked-symbol`/`undeclared-status` checks; a module's `grammar_severity` promotion of any of those checks has no effect on `matrix`'s exit code, because `coverage_matrix` carries none of their record kinds. `matrix` also does not inherit `coverage`'s FR-017-AC-22 unread-measurement gate (`status-column-matches-nothing`/`hollow-denominator`): `coverage_matrix` reads no status column, so there is no unread-column state for it to gate on | Test |
| FR-026-AC-13 | `matrix` uses only the FR-007 exit codes — 0, 1, 2, 3, 4 — across every case above, including an unrecognized `--severity` flag (exit 3); the shared taxonomy is authoritative | Test |
| FR-026-AC-14 | `matrix` never opens a file for writing: no flag names an output path, and running it in every format against a fixture repository leaves every file anywhere under `<scope>` — not only under `<scope>/spec` — byte-identical before and after the run | Test |

| FR-026-AC-15 | In every output format, `matrix` SHALL emit engine coverage diagnostics and unmatched annotation IDs on stderr with the authored file and line when available. A forbidden source-tag range retains its `range-in-trace-tag` explanation; an ID not read by the module's declared trace grammar is named with the symbol and guidance to use the declared form (comma-separated lists for `Trace:`). These advisories do not alter the engine's bindings, stdout matrix payload, or strict verdict. Correct comma-list controls emit neither range nor unmatched-tag warnings. | Test |

> **PLAT-1150 change note (2026-10-05):** At quire-driver `e7f6e89`, the
> engine already records source ranges and IDs dropped after `and`, but matrix
> selected `coverage_matrix` and discarded those explanations. Surface the
> existing facts rather than expanding the declared grammar: source ranges
> remain refused under quire-rs FR-050-AC-50 and FR-051-AC-28.

## Dependencies

- **Upstream**: [FR-017](./FR-017-coverage-subcommand.md) `coverage` (shared scope/module resolution, shared `CoverageReport` computation, shared help-text resolution-order requirement); [FR-008](./FR-008-json-output-encoding.md) JSON provenance envelope (`engine::attach`); [StR-004](../stakeholder/StR-004-thin-boundary-over-quire-rs.md) thin boundary; quire-rs [FR-050](ix://agent-ix/quire-rs/FR-050)-AC-47..51 (`coverage_matrix` computation, CR-187, `statement` field per PLAT-1077), [FR-051](ix://agent-ix/quire-rs/FR-051)-AC-27/28 (binder `ignored` marking, range-in-trace-tag).
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

> **CR note (review remediation, 2026-09-27, SR-065..068):** four review
> passes, four fix rounds folded into this one commit.
>
> - **SR-065 FND-001 (high).** The `Statement` column had no upstream source —
>   FR-050-AC-48 as originally merged carried only `id`/`method`/`binders`/
>   `status`. Fixed upstream: PLAT-1077 adds `statement` to the criterion
>   entry. §A/§D now cite it explicitly and state the CLI reads it and never
>   re-derives it.
> - **SR-066 FND-001 (medium).** Zero population is redefined as **zero
>   criteria**, covering both upstream shapes (an absent `coverage_matrix`, or
>   one present with an empty `requirements[]`) — rendered identically in
>   markdown/TSV, and both failing `--strict` (§B, AC-9/AC-10). `--format
>   json` still passes each shape through verbatim rather than collapsing
>   them (§E, AC-7).
> - **SR-066 FND-002 (medium).** §G is rewritten: `matrix` does not accept
>   `--severity`, runs none of the four `coverage` checks, is unaffected by
>   module `grammar_severity` promotion of them, and does not inherit
>   FR-017-AC-22's unread-measurement gate because it reads no status column.
>   AC-12 restates this as the command's entire severity story.
> - **SR-065 FND-002 (medium).** Pinned: the exact zero-population line text,
>   the `## <document>` heading form, pipe-escaping (`|` → `\|`, always
>   *after* truncation so an escape never splits and never counts toward the
>   80-character budget), and a worked example whose statement contains `|`
>   (§C/§D, AC-4/AC-6).
> - **Lows.** `requirements[]`'s empty-but-present shape is an object, never
>   called an "empty array" (§E, AC-7, SR-065 FND-003). The stdout/stderr
>   split is stated directly in the Description rather than left to imply
>   "stdout only" (SR-065 FND-004). Binders render `path:line:column`, never
>   collapsing two same-line binders upstream keeps distinct by column (§D,
>   AC-5/AC-8, SR-066 FND-003). `spec/tests.md`: IT-177/AC-14 now assert no
>   write anywhere under `<scope>`, not only under `<scope>/spec`; TC-841's
>   `Type` is corrected to match its `Test` verification method; IT-174 now
>   traces AC-11 as well as AC-10; AC-2 states that `matrix --help` itself
>   carries the FR-017-AC-21 resolution-order text (SR-067 FND-001).

> **CR note (implementation, 2026-09-28, PLAT-1078):** §B, §E and AC-7/8/9/10
> are amended to the shape quire-rs actually ships. The authored text
> read `coverage_matrix` as an object carrying `requirements[]` and allowed a
> present-but-empty shape. The engine serializes the field as a bare array of
> requirement entries (`Vec<CoverageMatrixRequirement>`) and omits it whenever
> it is empty, so neither the object shape nor the present-but-empty shape
> exists. Zero population now has one shape — key absent — and the
> verbatim-passthrough rule is unchanged. IT-171/IT-173/IT-174 in
> `spec/tests.md` are reworded to match; the tests already asserted equality
> with the engine's own serialization.
