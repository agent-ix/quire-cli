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
