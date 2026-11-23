---
id: SR-20261116045013
title: "PLAT-1831 quire-cli PR229 code-review"
type: SpecReview
analysis: code-review
scope: "agent-ix/quire-cli PR229 head 83a7028; CHANGELOG.md, Cargo.toml, Cargo.lock, src/commands/clauses.rs, tests/cli_clauses.rs"
review_set: subset
date: 2026-11-23
---

## Summary

Ticket: PLAT-1831. Code review with the rust-review lane folded in, scoped to `git diff origin/main...HEAD` (2 commits). The PR renders `ClauseSetDiff.output_catalog` in the human, TSV and JSON views of `quire clauses diff`, and moves quire-rs from tag v0.62.2 to rev 80d1f15.

The rev move ships 12 quire-rs commits, not only #714: #703 (0.62.3, dependency pins), #704 (qualified trace ids, NFR-140), #705/#707/#709/#706 (strict assurance JSON input), #708/#710 (deterministic source walk, symlink reporting, per-declaration collision attribution), #711 (hardening tests), #712 (spec only), #713/#714 (outputCatalog producers and catalog diff). Cargo.lock also moves quire-code-parse 0.2.2 to v0.2.3; that delta is test-tag and tooling only. `agent-ix-semantic-schema` moves from branch to tag at the same commit.

Gates: `cargo test --locked --test cli_clauses` 8 passed; `cargo clippy --locked --bin quire --test cli_clauses -- -D warnings` clean; `cargo fmt --check` clean.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | medium | The CHANGELOG names only the #714 dependency, but the rev move changes quire-cli behaviour in `symbols`, `trace`, `coverage` and `assurance` with no note and no quire-cli test. Reproduced: in a git repo whose root `.gitignore` lists `gen/`, `quire symbols --scope pkg --format tsv` now emits `gen/out.rs`, which quire 0.52.0 (engine 0.62.2) excluded. Ancestor, global and `.git/info/exclude` ignore files no longer apply, and a root `.gitignore` now applies outside a git checkout. Other unannounced changes: symlinked sources yield `symlink-not-followed` diagnostics, which quire-cli emits as `error` records; coverage JSON gains `ignored_source_files`; `ix://` and prefix-qualified `verifies` tags now bind or raise `unresolved-qualified-trace-id`; overlapping `trace_id_qualifiers` fail module load; the assurance reader rejects duplicate keys. | CHANGELOG.md:12, Cargo.toml:22, src/commands/symbols.rs:110, src/commands/trace.rs:296 |
| FND-002 | medium | TSV joins producer kinds with an unescaped `,`. The engine allows a comma inside a kind: it rejects only empty, padded and duplicate kinds. Reproduced: producers `["a,b"]` changed to `["a","b"]` renders `output_changed\t\t\t\to\ta,b\ta,b`, a changed row whose before and after cells are identical. The human view has the same collision with `", "` (`["a, b"]` vs `["a","b"]`). | src/commands/clauses.rs:404, src/commands/clauses.rs:328, src/commands/clauses.rs:336 |
| FND-003 | low | A TSV `output_changed` row carries only producers, so a change to only kind or description renders as a row with identical before and after cells. Reproduced: description `d` to `d2` gives `o\ta,b\ta,b`. A TSV reader cannot tell what changed. | src/commands/clauses.rs:385, src/commands/clauses.rs:408 |
| FND-004 | low | `coverage` and `symbols` human views print `excluded_source_files` but not the new `ignored_source_files`. The engine added the count so that ignore-file pruning is not silent (FR-050-AC-69), and the CLI human view drops it. | src/commands/coverage.rs:716, src/commands/symbols.rs:200 |
| FND-005 | low | it142's 7-cell check never sees a clause row, because `catalog_set` diffs identical clauses. The three empty trailing cells now appended to every clause row (`push_diff_row`) are untested, and no test asserts the new header text. | src/commands/clauses.rs:423, tests/cli_clauses.rs:730 |

## Finding details

- FND-001 fix: add CHANGELOG bullets for the engine changes this rev ships. At minimum cover the walk scope (only the scope root's own `.gitignore`/`.ignore` apply, now also outside git), symlink diagnostics, `ignored_source_files`, and qualified trace ids. Map `SymbolDiagnostic.severity` to `Record::warning` versus `Record::error` in symbols.rs and trace.rs, so a `symlink-not-followed` warning is not labelled an error.
- FND-002 fix: escape `\` and `,` inside each kind before joining in TSV (or emit the list as a JSON array cell), and add a comma-bearing kind to it142. In the human view, quote each kind, or use a separator the engine forbids.
- FND-003 fix: add `before_kind`/`after_kind` columns or a `changed_fields` cell. Otherwise state in FR-021 that TSV shows only producers for a changed entry.
- FND-004 fix: render `ignored_source_files` beside `excluded_source_files` in both human views.
- FND-005 fix: assert the exact header line, and run a mixed diff (changed clauses plus a changed catalog) that checks every row has 7 cells.

## Examined scope

```yaml
scope:
- id: Cargo.toml
  path: Cargo.toml
  role: examined
  excerpt: "quire-rs = { git = \"https://github.com/agent-ix/quire-rs\", rev = \"80d1f15939e5dd98b0a50403886e66833513f442\" }"
- id: CHANGELOG.md
  path: CHANGELOG.md
  role: examined
  excerpt: "TSV gains `output`, `before_producers` and `after_producers` columns. Depends on quire-rs at the PR #714 commit until the next release tag."
- id: src/commands/clauses.rs
  path: src/commands/clauses.rs
  role: examined
  excerpt: "render_diff_human / render_diff_tsv / push_output_row / producers_suffix / describe_output"
- id: tests/cli_clauses.rs
  path: tests/cli_clauses.rs
  role: examined
  excerpt: "it142_diff_shows_output_catalog_changes_in_every_view; it143_invalid_catalog_producers_are_rejected_naming_the_output"
- id: Cargo.lock
  path: Cargo.lock
  role: examined
  excerpt: "quire-rs 0.62.3 rev 80d1f15; quire-code-parse 0.2.3 tag v0.2.3; agent-ix-semantic-schema tag semantic-schema-v0.6.0 (same commit)"
```

## Verdict

Changes requested. FND-001 and FND-002 are medium and should be fixed in this PR; FND-003 to FND-005 are low.

Rust lane: clean apart from the findings. The new helpers are small, private and allocation-light, with no `unwrap` on a CLI path and no new public API. TSV cells for names and producers pass through `tsv_cell`. JSON is the engine's own serialization, and it142 validates it against `clause-diff-v1.schema.json`, read from the resolved quire-rs checkout through `cargo metadata`, not a copy. The diff adds no compatibility layer.

Dependency: nothing in quire-cli fails to compile or breaks the focused suite under 80d1f15. A rev pin on unreleased main is acceptable prerelease, and the CHANGELOG says the dependency returns to a tag at the next release.

No program in the agent-ix repos under ~/dev parses `clauses diff` TSV. A git grep found only prose mentions: quoin docs/assurance-workflows.md:24, spec-clauses-us-health FR-003 and spec.md, filament-docs rust-api.md, and quire-cli's README and spec. TSV is outside the SemVer contract (CHANGELOG.md:7-8), and the CHANGELOG states the column addition honestly.

## New findings (disposition pass 1)

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-006 | low | The CHANGELOG says `symlink-not-followed` diagnostics are "printed as warnings". Only `symbols` and `trace` map severity: `coverage` and `assurance` still emit every SymbolExtraction diagnostic through `Record::error`. The new mapping also turns the existing `symbol-id-collision` and `qsl-unbalanced-braces` warnings from error to warning records in `symbols` and `trace`, which the CHANGELOG does not mention. Exit codes are unaffected, because `emit` only writes stderr. The same 7-line match is now duplicated in two files. | src/commands/coverage.rs:309, src/commands/assurance.rs:111, src/commands/symbols.rs:112, src/commands/trace.rs:299, CHANGELOG.md:21 |

Fix for FND-006: add one shared `emit_extraction_diagnostics(ctx, &extraction)` helper that maps severity, use it at all four sites, and say in the CHANGELOG that warning-severity extraction diagnostics print as warnings.

## Dispositions

| FND | outcome | reason |
| --- | --- | --- |
| FND-001 | fixed | 0595239: CHANGELOG.md:18-26 adds an engine-changes bullet covering the ignore-file scope, symlink diagnostics, `ignored_source_files`, qualified trace ids, overlapping qualifiers and duplicate assurance keys; symbols.rs and trace.rs map Warning to `Record::warning` (remaining gap recorded as FND-006) |
| FND-002 | fixed | 0595239: `producers_json` emits the producer kinds as a JSON array in TSV and human views; it142 asserts `["a,b"]` vs `["a","b"]` stay distinct |
| FND-003 | fixed | 0595239: a `changed_fields` TSV column names kind/description/producers; it142 asserts a description-only row `desc\t\t\tdescription` |
| FND-004 | fixed | 0595239: coverage.rs:725 and symbols.rs:219 print `ignored_source_files` when non-zero (no test covers it, recorded in the gap-analysis SR as FND-003) |
| FND-005 | fixed | 0595239: it142 asserts the exact 8-column header, every row has 8 cells, and clause rows `added\tW-2...` and `changed\tW-1...` |
