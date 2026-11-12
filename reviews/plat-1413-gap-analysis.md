---
id: SR-20261112032006
title: "PLAT-1413 quire-cli PR169 gap-analysis"
type: SpecReview
analysis: gap-analysis
scope: "agent-ix/quire-cli PR169 head 6031b6f; spec/functional/FR-028-matrix-criticality-column.md, spec/functional/FR-026-matrix-subcommand.md (CR note PLAT-1412), spec/tests.md, src/commands/matrix.rs, tests/cli_matrix.rs"
review_set: subset
---

## Summary

Ticket: PLAT-1413. This is a planless gap pass scoped to the diff. Plan completion: not assessed.

The computed matrix was read from `quire matrix --scope .`, using the PR head's own binary built against the locally bumped quire-rs 0575fd4. The manual AC-to-test check found that each of FR-028-AC-1..11 has a test that asserts it, and all 7 `fr028_*` tests pass. The FR-026 CR note's column amendments are reflected in IT-168, IT-172, IT-173 and the worked-example snapshot.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | medium | In the repo's computed matrix, all 11 FR-028 criteria compute `untagged`. Each new test emits `criterion-tagged-outside-row` ("no matrix row cited by that test references it"). The cause is that the repo's traceability model binds a criterion tag only through a same-test IT-row tag, and spec/tests.md has no FR-028 rows: no IT rows and no feature-table row. The fix is to add IT rows (IT-183 onward) citing FR-028-AC-1..11 and an FR-028 feature row, then tag each test `#[trace("IT-NNN", "FR-028-AC-n")]`. | spec/tests.md:84, tests/cli_matrix.rs:1099-1217 |

## Examined scope

```yaml
scope:
- {id: FR-028-AC-1, path: spec/functional/FR-028-matrix-criticality-column.md, role: examined, excerpt: "The Markdown table header is `Criterion | Statement | Binders | Status | Requirement Criticality`, and each row's last cell is the requirement's engine value."}
- {id: FR-028-AC-2, path: spec/functional/FR-028-matrix-criticality-column.md, role: examined, excerpt: "The TSV header is `document\\tcriterion\\tstatus\\tbinders\\tstatement\\trequirement_criticality`, and each record carries its requirement's value in the last column."}
- {id: FR-028-AC-3, path: spec/functional/FR-028-matrix-criticality-column.md, role: examined, excerpt: "A requirement whose engine entry has no `criticality` key renders `unassigned` in the Markdown and TSV cells."}
- {id: FR-028-AC-4, path: spec/functional/FR-028-matrix-criticality-column.md, role: examined, excerpt: "`--format json` carries the engine's `requirements[]` unmodified: an entry with no key has no key, and an entry with `criticality: \"unassigned\"` has that key present."}
- {id: FR-028-AC-5, path: spec/functional/FR-028-matrix-criticality-column.md, role: examined, excerpt: "In a corpus with requirements valued `low`, `medium`, `high` and one missing, each requirement's rows show only its own value; the missing one shows `unassigned` and no row shows a value taken from another requirement."}
- {id: FR-028-AC-6, path: spec/functional/FR-028-matrix-criticality-column.md, role: examined, excerpt: "The zero-population output of FR-026-AC-9 is unchanged except that the TSV header alone is the six-column header of FR-028-AC-2."}
- {id: FR-028-AC-7, path: spec/functional/FR-028-matrix-criticality-column.md, role: examined, excerpt: "A `criticality` value containing `|` renders in Markdown as `\\|` and keeps the row at five cells; the same value in TSV is unescaped."}
- {id: FR-028-AC-8, path: spec/functional/FR-028-matrix-criticality-column.md, role: examined, excerpt: "A `criticality` value containing a tab, newline or carriage return renders with each replaced by a single space in Markdown and TSV, leaving one record per criterion."}
- {id: FR-028-AC-9, path: spec/functional/FR-028-matrix-criticality-column.md, role: examined, excerpt: "A requirement whose key is present with an empty string renders an empty cell in Markdown and TSV, not `unassigned`."}
- {id: FR-028-AC-10, path: spec/functional/FR-028-matrix-criticality-column.md, role: examined, excerpt: "A requirement whose key is a non-string value (for example the number `3`) renders its compact JSON text (`3`) in Markdown and TSV."}
- {id: FR-028-AC-11, path: spec/functional/FR-028-matrix-criticality-column.md, role: examined, excerpt: "For the same corpus, `--strict` and the exit code are identical whether every requirement is `high`, every requirement is missing a value, or the values are mixed."}
- {id: FR-026, path: spec/functional/FR-026-matrix-subcommand.md, role: context_only, excerpt: "CR note (requirement criticality column, PLAT-1412): FR-028 appends one column to each Markdown requirement table, `Requirement Criticality` ... The TSV of §F and AC-8 has six columns, the last `requirement_criticality`."}
```

## Verdict

Changes requested on traceability only. The behaviour matches FR-028 for every AC, including these:

- A missing key renders `unassigned`, while an empty string renders empty.
- A non-string value renders as compact JSON.
- `|` is escaped, and tab, LF and CR are scrubbed.
- JSON passes through unmodified.
- `--strict` is unaffected.

The tests are not reachable from the matrix the repo computes, so the matrix still shows FR-028 entirely untagged. FND-001 closes that gap. There is no unowned production code in the diff.
