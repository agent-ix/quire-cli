---
id: SR-070
title: "Gap analysis of PR #104 against FR-026 and IT-166..177 / TC-841"
type: SpecReview
analysis: gap-analysis
scope: "agent-ix/quire-cli@c9d022a31d8ba2779a6b2cd17777a5aed5fb60c8; spec/functional/FR-026-matrix-subcommand.md (AC-1..14 with the 2026-09-28 implementation CR note), spec/tests.md (FR-026 coverage row, IT-166..177, TC-841), src/commands/matrix.rs, src/commands/coverage.rs, tests/cli_matrix.rs"
review_set: subset
---

## Summary

Ticket: PLAT-1078. Plan completion: not assessed. The review is planless.

I built the PR head and ran `quire matrix` over this repository. It reports
all 14 FR-026 criteria as `tagged`, each bound to a test in
`tests/cli_matrix.rs`:

| AC | Test(s) |
| --- | --- |
| AC-1 | IT-166 |
| AC-2 | IT-167 |
| AC-3, AC-4 | IT-168 |
| AC-5 | IT-169 |
| AC-6 | IT-170 |
| AC-7 | IT-171 |
| AC-8 | IT-172 |
| AC-9 | IT-173 |
| AC-10, AC-11 | IT-174 |
| AC-11 | IT-175 |
| AC-12 | IT-176 |
| AC-13 | TC-841 |
| AC-14 | IT-177 |

Every IT-166..177 and TC-841 row in `spec/tests.md` names a test that exists
and passed in the `make ci` run at c9d022a. Across all 13 tests there are no
stubs, no tautologies and no guarded assertions that could skip.

The code maps to the spec. Nothing in `matrix.rs` lacks an owning AC. The
2026-09-28 CR note matches the engine at quire-rs a4f3a70, where
`coverage_matrix` is a `Vec<CoverageMatrixRequirement>` with
`skip_serializing_if = "Vec::is_empty"` and each requirement carries at least
one criterion. IT-171 and IT-173 assert equality with the engine's own
serialization rather than restating its shape.

Several oracles are strong. IT-168's worked example compares whole-file bytes
against a golden file identical to FR-026 §C. IT-176 proves its fixture is live
(`coverage` exits 1 on the promotion and reports
`status-column-matches-nothing`) before asserting that `matrix` is unaffected.
IT-177 snapshots every file under the scope.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | low | AC-5 and AC-6 carry clauses no test exercises. IT-169 never renders a `\|` in a Binders cell, and IT-170 never feeds a tab, newline or CR into a statement. Mutation confirms it: removing either the Binders escape or the Statement scrub leaves all of IT-166..177 and TC-841 green, so the ✅ on IT-169 and IT-170 overstates what those two clauses are verified by. Same root cause as SR-069 FND-001/FND-002. | spec/tests.md IT-169, IT-170 rows; tests/cli_matrix.rs:368-521 |

## Verdict

No coverage gap: all 14 ACs are traced and pass. There are no orphan traces,
no stubs and no untraced production code. One low gap in oracle strength does
not block merge.
