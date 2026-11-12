---
id: SR-20261112034124
title: "Gap analysis of validate --okf --strict (PR #168)"
type: SpecReview
analysis: gap-analysis
scope: "agent-ix/quire-cli; spec/functional/FR-014-validate-okf-bundle.md, spec/functional/FR-004-validate-subcommand.md, spec/tests.md, src/commands/validate.rs, tests/cli_okf.rs"
review_set: subset
---
# Gap analysis of validate --okf --strict (PR #168)

## Summary

Ticket: PLAT-1408. Plan completion: not assessed. Examined FR-014-AC-11,
FR-014-AC-10, FR-014-AC-3 and FR-004 `--strict` (step 5, AC-11, AC-22) against
the diff. `quire matrix` binds FR-014-AC-11 to
`cli_okf::okf_strict_dangling_mitigates_exits_3_with_located_message` only.
FR-014-AC-3 (the non-strict path) and FR-014-AC-10 are unchanged and still
backed by IT-070, IT-178 and IT-179. The FR-004 per-file `--strict` path is
not touched.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | medium | No test reaches the new `args.strict && warnings > 0` branch. FR-014-AC-11 requires that a finding which stays a warning under Strict (for example the FR-040-AC-10 edge-target checks) exits 3. The only test exits 3 through the errors branch | src/commands/validate.rs:408-410 |
| FND-002 | medium | The authority clauses of FR-014-AC-11 are untested under `--strict`. Untested: an own-authority `ix://` miss exits 3; a foreign-authority target is skipped and an otherwise conformant bundle exits 0; a miss with no package declared exits 3; a two-segment `ix://<org>/<id>` link to an absent id exits 3; and a finding-free bundle exits 0. IT-251 covers only a bare-id target | spec/functional/FR-014-validate-okf-bundle.md:114 |
| FND-003 | low | The FR-014 coverage row in spec/tests.md still reads "AC-11 🚧 open: `--okf --strict` has no IT yet" and does not list IT-251, which this PR adds | spec/tests.md:74 |

## Verdict

FAIL with findings. Only the bare-id dangling slice of FR-014-AC-11 has a
test, and that test is vacuous (see the code review, FND-001). Add tests for
the warning-escalation branch and for the authority clauses, and update the
FR-014 row.

## Dispositions

| FND | outcome | reason |
| --- | --- | --- |
| FND-001 | fixed | 63e35a3 adds IT-257 (`okf_strict_fails_on_warning_only_finding`): a `[disallowed-edge-target]` warning exits 0 without `--strict` and 3 with it. Measured stderr: `1 OKF bundle warning(s) (--strict)`, rc=3, so the test reaches the warnings branch |
| FND-002 | fixed | 63e35a3 adds IT-252..IT-256: own-authority miss exits 3, foreign authority exits 0, no-package miss exits 3, two-segment link exits 3, finding-free bundle exits 0 with empty stderr. All pass |
| FND-003 | fixed | 63e35a3: the FR-014 row reads AC-1..11 and lists IT-251..IT-257 |
