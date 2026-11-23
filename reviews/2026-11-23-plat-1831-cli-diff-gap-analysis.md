---
id: SR-20261116045014
title: "PLAT-1831 quire-cli PR229 gap-analysis"
type: SpecReview
analysis: gap-analysis
scope: "agent-ix/quire-cli PR229 head 83a7028; spec/functional/FR-021-clauses-subcommand.md, src/commands/clauses.rs, tests/cli_clauses.rs"
review_set: subset
date: 2026-11-23
---

## Summary

Ticket: PLAT-1831. Planless gap analysis of FR-021 against `quoin matrix` (quoin 0.36.8) and the tests. Plan completion: not assessed. The matrix shows FR-021-AC-1..9 all `tagged`. it142 and it143 carry `#[trace("FR-021-AC-9")]`.

Red check, by reading (no edits were allowed in the worktree): every it142 assertion names strings that only the new render code emits (`Output catalog added:`, `output added\t`, `output_changed\t\t\t\t`) or the new JSON `outputCatalog` key, so reverting `clauses.rs` fails it. it143 fails if the engine stopped validating producers. The fixtures need the `producers` field, so neither test compiles against v0.62.2.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | medium | FR-021-AC-9 says `diff` and `evaluate` reject invalid producers; it143 runs only `diff`. The behaviour exists: a manual run of `evaluate` exits 3 with `outputCatalog "bad" producers must not be an empty list`. No test fails if `evaluate` stops rejecting. | tests/cli_clauses.rs:749 |
| FND-002 | low | The FR-021-AC-4 amendment (catalog entries added, removed, changed) is bound only through it138, which diffs two sets with identical catalogs. Catalog coverage exists in it142, but it142 is tagged AC-9 only. | spec/functional/FR-021-clauses-subcommand.md:43, tests/cli_clauses.rs:666 |

## Finding details

- FND-001 fix: loop it143 over `["evaluate", "diff"]` (as it141 already does at tests/cli_clauses.rs:506), asserting exit 3, empty stdout, and that stderr names the output.
- FND-002 fix: add `FR-021-AC-4` to it142's `#[trace]`, or drop the AC-4 amendment as redundant with AC-9 (see the spec-review SR).

## Scope examined

```yaml
scope:
- id: FR-021-AC-4
  path: spec/functional/FR-021-clauses-subcommand.md
  role: examined
  excerpt: "`diff` delegates to the engine and reports added, removed, and changed clauses between two exact versions of the same authority and set id, and the output-catalog entries added, removed, and changed between them."
- id: FR-021-AC-9
  path: spec/functional/FR-021-clauses-subcommand.md
  role: examined
  excerpt: "Human, TSV, and JSON `diff` views show each added, removed, and changed `outputCatalog` entry; an entry's producers are shown when present. JSON conforms to the engine's published diff schema. `diff` and `evaluate` reject a clause set whose catalog producers are invalid (empty list or a padded kind) with a diagnostic naming the output."
- id: FR-021-AC-5
  path: spec/functional/FR-021-clauses-subcommand.md
  role: context_only
  excerpt: "Human results and deterministic TSV are emitted on stdout; TSV escapes structural characters so one clause remains one row."
```

## Verdict

Changes requested on FND-001 (medium). No stubs or placeholder code, and no production code without an owning requirement: every render path added in `clauses.rs` serves FR-021-AC-9. The schema check reads `clause-diff-v1.schema.json` from the resolved quire-rs package, not a copy.

## New findings (disposition pass 1)

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-003 | low | The new `ignored_source_files` lines in the human views of `coverage` and `symbols` have no test and no owning AC. Reverting either line fails nothing. cli_coverage.rs:1738-1816 already has the twin test for `excluded_source_files`. | src/commands/coverage.rs:725, src/commands/symbols.rs:219 |

Fix for FND-003: add one focused test that writes a scope-root `.gitignore` excluding a `.rs` file, then asserts the `pruned by the scope root's ignore files` line in `quire symbols` human stderr and in `quire coverage` human stdout (and its absence with no ignore file). Tag it with the FR-017 row that owns the `excluded_source_files` line, extended to cover this count.

## Dispositions

| FND | outcome | reason |
| --- | --- | --- |
| FND-001 | fixed | 0595239: it143 runs both `evaluate` and `diff` over empty-list, padded, empty and duplicate kinds, asserting exit 3, empty stdout and stderr naming `bad-output`; now tagged FR-021-AC-10 |
| FND-002 | fixed | 0595239: it142 is tagged `FR-021-AC-4`, `FR-021-AC-9`, and its `after` set now also changes clauses, so the AC-4 binding sees both clause and catalog changes |
| FND-003 | fixed | d739c4c: cli_coverage::it144_ignored_source_count_reaches_coverage_and_symbols_human_views, tagged FR-017-AC-18 (extended in 99e20a9 to own the line), asserts the line is absent without a `.gitignore` and that `1 source file(s) pruned by the scope root's ignore files` appears with one, for both coverage and symbols. Red check by reasoning: the string appears only at coverage.rs:723 and symbols.rs:212 (not in quire-rs at 80d1f15), so removing either line fails that command's `with` assertion |
