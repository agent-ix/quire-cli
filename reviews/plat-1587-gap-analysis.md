---
id: SR-20261113142747
title: "PLAT-1587 PLAT-1589 quire-cli PR195 gap-analysis"
type: SpecReview
analysis: gap-analysis
scope: "agent-ix/quire-cli PR195 head 6f5d180; spec/functional/FR-043-matrix-design-column.md, src/commands/matrix.rs, src/commands/matrix_design.rs, tests/cli_matrix_design.rs"
review_set: subset
---

## Summary

Tickets: PLAT-1587 (CLI half) and PLAT-1589. This is a planless gap pass over FR-043's 26 acceptance criteria, scoped to the PR diff. Plan completion: not assessed.

I computed the matrix with `quire matrix --scope . --format tsv` (quire 0.44.0, engine 0.59.1) on the PR head. AC-1..14 and AC-16..26 compute `tagged`, and AC-15 (Inspection) computes `method-without-symbol`. AC-15 was inspected by hand: the diff holds no join, edge-matching, archetype name or verb. AC-9 is tagged only by a unit test, and AC-22 by a unit test plus an integration test.

Mutation spot-check on a scratch copy of the head, running the bin unit tests plus `cli_matrix_design` and `cli_matrix`:

- 11 of 13 mutants were killed: marker order, `is_live`, `design_state` swap, gap/repo line order, gate repo prefix, kind-mismatch archetype, repo separator, per-criterion element selection, Design-cell truncation, TSV repo column position, and the strict gate ignoring design gaps.
- 2 survived: dangling markers printed for covered blocks that have elements (FND-001), and a looser `parse_repo` (code-review SR-20261113142746 FND-001).

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | low | Behavior "Design cell" limits `-> <reference> (dangling)` to "a `covered` block with no elements". No test has a covered block with an element and a dangling reference, so the mutant that drops `b.elements.is_empty()` survived. FR-043-AC-18 covers only the no-element case. | src/commands/matrix_design.rs:90, tests/cli_matrix_design.rs:654 |
| FND-002 | low | FR-043-AC-9 (Test) asks for a rendered gap and a `--strict` exit code. It is tagged only by a unit test of `gate_failures` and `gap_lines`, so no run checks that the engine's retired status reaches the CLI with the block attached and that `quire matrix --strict` exits 0 and then 1. A retired fixture is feasible: `tests/cli_coverage_strict.rs:140` builds one with `vocabularies.retired`. | src/commands/matrix_design.rs:273-286 |
| FND-003 | low | FR-043-AC-16 says every file under `<scope>` stays byte-identical in every format. The test compares only `spec/GLY-001.md`, and it discards the four runs' exit codes. A run that writes a new file, or changes another document or `src/lib.rs`, would pass. | tests/cli_matrix_design.rs:245-252 |
| FND-004 | low | FR-043-AC-24 says two members with the same document path are distinguishable "on stdout and in the strict stderr line". The test asserts only the stderr lines for `o/a` and `o/c`; stdout for that run is never checked. | tests/cli_matrix_design.rs:603-623 |
| FND-005 | low | Trace: FR-043-AC-25 has three forms. The `n/a` human `design:` line is asserted only in `tsv_inserts_design_and_design_state_after_criterion`, which is tagged AC-4 and AC-20, not AC-25. The `(none)` human `design:` line is never asserted. The AC-25 test covers only the `no design element` form. | tests/cli_matrix_design.rs:257, tests/cli_matrix_design.rs:359 |

## Coverage

- FR-043 criteria: 26. Tagged: 25. Method-without-symbol (Inspection): 1 (AC-15, inspected by hand, clean). Untagged: 0.
- Partially evidenced: AC-9 (unit test only), AC-16, AC-24 and AC-25 (one clause each unasserted).
- Reverse gap: none. Every added behavior (`--repo`, `--format human`, Design column, design gate) is owned by FR-043.
- Stubs or coverage inflation: none found.
- Semantic review: skipped (not requested).
- Plan completion: not assessed

## Examined scope

```yaml
scope:
- {id: FR-043-AC-1, role: examined, excerpt: "Markdown Design column between Statement and Binders; id (kind) and no design element (<check>)"}
- {id: FR-043-AC-2, role: examined, excerpt: "--format human design: line, Design gap: lines, one line per criterion"}
- {id: FR-043-AC-3, role: examined, excerpt: "--format json carries design, names kind, omits out of scope; byte-identical runs"}
- {id: FR-043-AC-4, role: examined, excerpt: "TSV header with design design_state; empty for out-of-scope"}
- {id: FR-043-AC-5, role: examined, excerpt: "realized_criteria limits element rows; whole-requirement element on every row"}
- {id: FR-043-AC-6, role: examined, excerpt: "no criteria and no element printed with gap; covered no-criteria prints no gap"}
- {id: FR-043-AC-7, role: examined, excerpt: "--strict exit 1 with stderr line; without --strict exit 0"}
- {id: FR-043-AC-8, role: examined, excerpt: "where selector narrows the gate as module data"}
- {id: FR-043-AC-9, role: examined, excerpt: "all-retired requirement renders gap and does not fail --strict"}
- {id: FR-043-AC-10, role: examined, excerpt: "passing corpus exit 0; gap plus untagged exit 1 naming design gap"}
- {id: FR-043-AC-11, role: examined, excerpt: "no design_relations gives FR-026 output; --repo adds repo lines and TSV column"}
- {id: FR-043-AC-12, role: examined, excerpt: "undeclared relation exit 3 naming entry, empty stdout"}
- {id: FR-043-AC-13, role: examined, excerpt: "sibling repo element shows repo o/b; without o/b a gap"}
- {id: FR-043-AC-14, role: examined, excerpt: "unrelated vocabulary, all formats complete, gate fires"}
- {id: FR-043-AC-15, role: examined, excerpt: "CLI holds no join, edge-matching, selection, archetype or verb"}
- {id: FR-043-AC-16, role: examined, excerpt: "every file under scope byte-identical"}
- {id: FR-043-AC-17, role: examined, excerpt: "covered and uncovered blocks share one cell"}
- {id: FR-043-AC-18, role: examined, excerpt: "dangling-only covered block prints -> <reference> (dangling)"}
- {id: FR-043-AC-19, role: examined, excerpt: "only criteria-less obliged requirements: headings, gap lines, TSV records, --strict 1"}
- {id: FR-043-AC-20, role: examined, excerpt: "untyped element prints id"}
- {id: FR-043-AC-21, role: examined, excerpt: "--scope with --repo exit 3 naming both"}
- {id: FR-043-AC-22, role: examined, excerpt: "n/a, (none), untruncated marker"}
- {id: FR-043-AC-23, role: examined, excerpt: "kind mismatch: <id> (<archetype>) beside gap marker"}
- {id: FR-043-AC-24, role: examined, excerpt: "--repo repo: before Design gap:, leading repo column, <repo>/<id>, same-path members distinguishable"}
- {id: FR-043-AC-25, role: examined, excerpt: "human design: line three forms"}
- {id: FR-043-AC-26, role: examined, excerpt: "two uncovered relations give two stderr lines in block order"}
```

## Verdict

CONDITIONAL. No criterion is untagged and there is no high finding. Five low gaps remain: one surviving mutant on an untested Behavior clause, and four ACs with one clause unasserted or asserted under another tag.
