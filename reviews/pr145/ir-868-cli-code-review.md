---
id: SR-20261102086803
title: IR-868 CLI PR 145 code-review
type: SpecReview
analysis: code-review
scope: agent-ix/quire-cli; PR 145; src/commands/matrix.rs, spec/functional/FR-026-matrix-subcommand.md,
  Cargo.lock
review_set: subset
---

## Summary

Ticket: IR-868. One independent reviewer; model unknown. Reviewed typed changed units on the current-main dependency context. App: quire-cli 0.38.1; engine 0.51.0; Quoin 0.29.1. No applicable AssuranceProfile exists in spec/.

Exhaustive engine enum adoption and typed payload serialization are correct; no new unsafe, panics, flags, copied qualification logic or narrowed CI gates. Dependency refresh is confined to the existing engine-main dependency and its extraction package, under the authorized Cargo exception.

## Verdict

PASS for the scoped PR change. Final aggregate gate belongs to the lead and remains pending. Positive qualification coverage is the typed rendering/strict seam only; compute_report still calls legacy compute_coverage. No end-to-end owner qualification claim.

## Findings

| ID | Severity | Summary | Refs |
|---|---|---|---|
| FND-001 | low | No findings (placeholder) | - |

## Coverage

Plan completion: not assessed

Scoped FR-026 AC-1 through AC-15 inspected with tests/cli_matrix.rs and the matrix unit tests. No workflow, manifest or gitlink changes; Cargo.lock changes only normal engine-main resolution. Focused test and lint/fmt evidence is in the parent ir868-main directory; aggregate qualification is not asserted.

## Examined units

```yaml
scope:
- id: FR-026-AC-1
  path: spec/functional/FR-026-matrix-subcommand.md
  role: examined
  excerpt: '`quire matrix --scope <DIR> --module $M` over a repository whose module
    declares an `obligations:` source that derives a non-empty population exits 0
    and renders every requirement''s criteria in every format'
- id: FR-026-AC-2
  path: spec/functional/FR-026-matrix-subcommand.md
  role: examined
  excerpt: '`matrix` shares `coverage`''s refusals verbatim: no `spec/` under `--scope`
    exits 3 as `MissingDocumentRoot` (FR-017-AC-5); no `traceability:` model in scope
    exits 3 naming the missing declaration (FR-017-AC-6); `--module` is repeatable
    and closed exactly as FR-017-AC-20/21 specify, and `quire matrix --help` states
    the identical `--module` resolution-order text FR-017-AC-21 requires of `coverage
    --help`'
- id: FR-026-AC-3
  path: spec/functional/FR-026-matrix-subcommand.md
  role: examined
  excerpt: Markdown requirement tables render in the engine's own `requirements[]`
    order (sorted by `document`); the CLI performs no additional sort, group or alphabetization
- id: FR-026-AC-4
  path: spec/functional/FR-026-matrix-subcommand.md
  role: examined
  excerpt: Each requirement renders as the exact heading `## <document>` with the
    requirement's own `document` value verbatim (no link, no backticks), followed
    by one table with header `Criterion \| Statement \| Binders \| Status`, rows in
    the engine's own per-document criteria order
- id: FR-026-AC-5
  path: spec/functional/FR-026-matrix-subcommand.md
  role: examined
  excerpt: "A criterion's `Binders` cell lists each `(path, line, column)` binder\
    \ as `path:line:column`, separated by `, `, in the engine's own binder order \u2014\
    \ never collapsing two same-line binders that differ only by column; a binder\
    \ the engine marks `ignored` carries a trailing ` (ignored)`; a criterion with\
    \ zero binders renders the literal `(none)`"
- id: FR-026-AC-6
  path: spec/functional/FR-026-matrix-subcommand.md
  role: examined
  excerpt: "A `Statement` cell is the criterion's own `statement` field (never re-derived)\
    \ with every tab/newline/CR replaced by a space, truncated first \u2014 left intact\
    \ at \u226480 Unicode scalar values or cut to 77 scalar values plus a trailing\
    \ `...` above that \u2014 and only then pipe-escaped (`\\|`), so escaping never\
    \ splits the cut and never counts toward the 80-character budget; `--format json`/`--format\
    \ tsv` carry the untruncated, unescaped text for the identical criterion"
- id: FR-026-AC-7
  path: spec/functional/FR-026-matrix-subcommand.md
  role: examined
  excerpt: "`--format json` emits `{\"coverage_matrix\": <value>, \"engine\": {cli,\
    \ engine, capabilities}}` on stdout; the `coverage_matrix` key carries the engine's\
    \ own value unmodified \u2014 a bare array of requirement entries, never re-wrapped\
    \ in an object \u2014 and is absent when the engine omits it (zero population),\
    \ never present-but-empty; two runs over identical inputs are byte-identical"
- id: FR-026-AC-8
  path: spec/functional/FR-026-matrix-subcommand.md
  role: examined
  excerpt: "`--format tsv` emits header `document\\tcriterion\\tstatus\\tbinders\\\
    tstatement`, one untruncated, unescaped record per criterion in the JSON/markdown\
    \ order, `binders` comma-separated as `path:line:column` with the same `(ignored)`\
    \ marker and empty (not `(none)`) at zero; a zero-criteria report (\xA7B) emits\
    \ the header alone"
- id: FR-026-AC-9
  path: spec/functional/FR-026-matrix-subcommand.md
  role: examined
  excerpt: "The zero-population state \u2014 `coverage_matrix` absent, whether the\
    \ module declares no `obligations:` source or declares one that derives nothing\
    \ \u2014 renders in markdown as the single line `No obligations matched this scope.`\
    \ (no heading, no table) and in TSV as the header alone; `--format json` omits\
    \ the key, as the engine does"
- id: FR-026-AC-10
  path: spec/functional/FR-026-matrix-subcommand.md
  role: examined
  excerpt: "`--strict`'s pass condition requires **at least one** criterion, all `qualified-record`/`tagged`/`retired`/`method-without-symbol`:\
    \ it exits 1 on the zero-population state (\xA7B) exactly as it exits 1 on an\
    \ `untagged`/`tagged-by-ignored-test` criterion \u2014 a report measuring nothing\
    \ must not exit 0 (FR-050-AC-14/CR-035 argument, applied to the derived-obligation\
    \ population)"
- id: FR-026-AC-11
  path: spec/functional/FR-026-matrix-subcommand.md
  role: examined
  excerpt: "A criterion computing `method-without-symbol` never fails `--strict` on\
    \ its own, regardless of its binder count, and renders in every format exactly\
    \ like any other criterion \u2014 its own `Status`/`status` cell/field states\
    \ `method-without-symbol` verbatim"
- id: FR-026-AC-12
  path: spec/functional/FR-026-matrix-subcommand.md
  role: examined
  excerpt: '`matrix` does not accept `--severity` (an attempt is an ordinary clap
    argv error, exit 3, FR-007-AC-5) and runs none of the `unbacked-row`/`status-lie`/`untracked-symbol`/`undeclared-status`
    checks; a module''s `grammar_severity` promotion of any of those checks has no
    effect on `matrix`''s exit code, because `coverage_matrix` carries none of their
    record kinds. `matrix` also does not inherit `coverage`''s FR-017-AC-22 unread-measurement
    gate (`status-column-matches-nothing`/`hollow-denominator`): `matrix` consumes
    the engine''s computed status, including locus-scoped retirement metadata, without '
- id: FR-026-AC-13
  path: spec/functional/FR-026-matrix-subcommand.md
  role: examined
  excerpt: "`matrix` uses only the FR-007 exit codes \u2014 0, 1, 2, 3, 4 \u2014 across\
    \ every case above, including an unrecognized `--severity` flag (exit 3); the\
    \ shared taxonomy is authoritative"
- id: FR-026-AC-14
  path: spec/functional/FR-026-matrix-subcommand.md
  role: examined
  excerpt: "`matrix` never opens a file for writing: no flag names an output path,\
    \ and running it in every format against a fixture repository leaves every file\
    \ anywhere under `<scope>` \u2014 not only under `<scope>/spec` \u2014 byte-identical\
    \ before and after the run"
- id: FR-026-AC-15
  path: spec/functional/FR-026-matrix-subcommand.md
  role: examined
  excerpt: An engine-supplied `qualified-record` criterion renders that exact status
    in Markdown, JSON and TSV without fabricating a source binder; its nonempty population
    passes `--strict` by itself, while a mixed population containing an `untagged`
    or `tagged-by-ignored-test` criterion still exits 1. The CLI does not construct
    qualification records or add an evidence-input surface.
- id: strict_verdict
  path: src/commands/matrix.rs
  role: examined
  excerpt: CoverageMatrixStatus::QualifiedRecord | CoverageMatrixStatus::Tagged |
    CoverageMatrixStatus::Retired | CoverageMatrixStatus::MethodWithoutSymbol => {}
- id: status_token
  path: src/commands/matrix.rs
  role: examined
  excerpt: CoverageMatrixStatus::QualifiedRecord => "qualified-record"
- id: qualified_record_renders_verbatim_without_source_binders
  path: src/commands/matrix.rs
  role: examined
  excerpt: fn qualified_record_renders_verbatim_without_source_binders()
- id: qualified_record_passes_strict_without_hiding_live_gaps
  path: src/commands/matrix.rs
  role: examined
  excerpt: fn qualified_record_passes_strict_without_hiding_live_gaps()
```
