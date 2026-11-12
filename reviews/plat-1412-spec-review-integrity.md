---
id: SR-20261112032002
title: "Integrity review of FR-028 matrix criticality column (quire-cli PR 166)"
type: SpecReview
analysis: integrity
scope: "agent-ix/quire-cli PR166 head 0ad25b1; spec/functional/FR-028-matrix-criticality-column.md, spec/functional/FR-026-matrix-subcommand.md (context)"
review_set: subset
relationships:
  - target: ix://agent-ix/quire-cli/FR-028
    type: references
---

# Integrity review of FR-028 matrix criticality column (quire-cli PR 166)

## Summary

Ticket: PLAT-1412. Checked FR-028 for consistency with FR-026, which it extends, and for completeness of its statement against the command's other contracts. FR-026 is not touched by the diff.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | high | FR-026-AC-4 and §C fix the Markdown table as "four columns ... under these exact headers: `Criterion \| Statement \| Binders \| Status`". FR-028-AC-1 requires a fifth `Criticality` column. Both cannot pass: IT-168 (`spec/tests.md:286`) and `tests/cli_matrix.rs:324` assert the four-column header. FR-028 says "FR-026-AC-4 plus the appended column" but FR-026 is not amended. Fix: add a CR note to FR-026 amending §C and AC-4 to the five-column header (this repo's convention for amending a merged FR). | spec/functional/FR-026-matrix-subcommand.md (§C, FR-026-AC-4), FR-028-AC-1, spec/tests.md:286 |
| FND-002 | high | FR-026 §F names "five fixed columns" and FR-026-AC-8 fixes the TSV header as `document\tcriterion\tstatus\tbinders\tstatement`. FR-028-AC-2 and AC-6 require a sixth `criticality` column. Same contradiction as FND-001: IT-172 (`spec/tests.md:290`) and `tests/cli_matrix.rs:645` assert the old header. Fix: amend FR-026 §F, AC-8 and AC-9's TSV header in the same CR note. | spec/functional/FR-026-matrix-subcommand.md (§F, FR-026-AC-8), FR-028-AC-2, FR-028-AC-6 |
| FND-003 | low | FR-028 does not say that criticality has no effect on `--strict` or the exit code. FR-026 §G defines the verdict from criterion status alone, so this is implied, but a reader of a criticality FR in a regulated-software project could expect `high` + `untagged` to be treated differently. Fix: one sentence: criticality does not change the `--strict` verdict or the exit code. | spec/functional/FR-028-matrix-criticality-column.md:25-30, FR-026 §G |

## Verdict

FAIL until FR-026 is amended (FND-001, FND-002). The fix is a CR note on FR-026; FR-028 itself is consistent with FR-082. Clean: FR-028-AC-6 matches FR-026-AC-9's zero-population shapes once the header is amended; AC-1..AC-6 ids are sequential and each Verification cell names a method.

## Dispositions

| FND | outcome | reason |
| --- | --- | --- |
| FND-001 | fixed | Round 1 (2a9935b): FR-026 CR note amends §C and AC-4 to the five-column header ending `Requirement Criticality`, matching FR-028-AC-1. |
| FND-002 | fixed | Round 1 (2a9935b): the same CR note amends §F, AC-8 and AC-9's TSV header to six columns ending `requirement_criticality`, matching FR-028-AC-2/AC-6. |
| FND-003 | fixed | Round 1 (2a9935b): "Criticality has no effect on `--strict` or on the exit code", tested by FR-028-AC-11. |
