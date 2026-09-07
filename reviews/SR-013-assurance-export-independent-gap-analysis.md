---
id: SR-013
title: "Gap analysis — PR #75 deterministic assurance export"
type: SpecReview
analysis: gap-analysis
scope: "plan/plan.md, spec/tests.md, FR-020, US-006, StR-004, IT-136 through IT-145, TC-814"
review_set: subset
relationships:
  - target: ix://agent-ix/quire-cli/spec/functional/FR-020
    type: reviews
  - target: ix://agent-ix/quire-cli/spec/tests.md
    type: references
---
# SR-013: Gap analysis — PR #75 deterministic assurance export

## Summary

Independent gap analysis of the assurance-export work at `c5f8c43f`: matrix
backing, criterion-level traceability, and code with no owning requirement.
All eleven new test-case rows (IT-136…IT-145, TC-814) are genuinely backed by
tagged, compiled, passing tests. The gap is one level down — four FR-020
criteria and all four US-006 criteria do not resolve — and one level up, where
no gate re-runs the traceability check at all.

## Verdict

**FAIL** — `spec/tests.md` marks FR-020 `AC-1..9 ✅` and US-006 `AC-1..4 ✅`
while the engine backs 5 of 9 FR-020 criteria and cannot see any US-006
criterion.

## Findings

| ID | Severity | Summary | Refs |
|---|---|---|---|
| FND-030 | high | `US-006-AC-1` through `AC-4` are absent from `minted_targets` entirely. The section is `## Acceptance` with prose bullets; the model's `acceptance-criterion` target reads `section: Acceptance Criteria` with `id_column: ID`. Nothing the tests tag or the matrix claims for US-006 can bind. | spec/usecase/US-006-export-assurance-facts.md:19 |
| FND-031 | high | FR-020 measures 5/9 backed. `FR-020-AC-4`, `-AC-5`, `-AC-8`, `-AC-9` carry `backed: false` although IT-141, IT-142, IT-144 and IT-145 all cite them in `Traces To` and are themselves backed. | spec/functional/FR-020-assurance-export-subcommand.md:88 |
| FND-032 | high | No repository gate runs `quire validate` or `quire coverage`; there is no `make spec` equivalent. The PR's traceability evidence is a one-off manual run that CI, `make ci`, and any future change will not reproduce. | Makefile:124 |
| FND-033 | medium | Nine criterion-level tag instances on the new tests appear in `unmatched_tags`: `FR-020-AC-1/3/4/5/8/9` and `US-006-AC-1/2/3`. The `// IT-NNN, FR-020-AC-N.` comment form between `#[test]` and `fn` is identical on tests whose criteria do bind, so the tag form is not the differentiator and the cause needs the engine's declaration origins to pin down. | tests/cli_assurance.rs:189 |
| FND-034 | medium | `spec/tests.md:43` marks the StR-004 row `✅` citing TC-814 and IT-136. StR-004's validation criteria are `StR-004-VC-1..3`, and VC-3 ("the code review checklist includes: does any new logic belong upstream?") is a process claim no test can discharge. | spec/tests.md:43 |
| FND-035 | low | 23 rows are unbacked repo-wide, including all six FR-001 criteria and NFR-001/002/006. All pre-existing on `main`; none introduced here. Recording so the 286/337 rollup is not read as a regression from this PR. | spec/functional/FR-001-render-subcommand.md:72 |
| FND-036 | low | Repo-wide `quire validate` reports 39/47 grammar-clean with 19 findings. Every one is in a document this PR does not touch; the 8 changed spec documents are 8/8 clean. | spec/non-functional/NFR-001-render-latency-budget.md:1 |

## Coverage

| Measure | Value |
|---|---|
| New test-case rows added | 11 (IT-136…IT-145, TC-814) |
| New rows backed by a tagged, compiled test | 11 of 11 |
| FR-020 criteria backed | 5 of 9 |
| US-006 criteria minted | 0 of 4 |
| `quire coverage` rollup | 286/337 backed; 0 status lies; 0 untracked symbols |
| Unbacked rows | 23, all pre-existing |
| Unmatched tag instances on new tests | 9 |
| `make ci` | exit 0; 213 tests |
| Changed spec documents validated | 8/8 grammar-clean |

No underspecified code was found. `src/commands/assurance.rs` is the only new
source module and every function in it traces to FR-020; the two `#[cfg(test)]`
unit tests cover the premise parsers. The optional semantic review was folded
into SR-012 rather than run as a separate pass.
