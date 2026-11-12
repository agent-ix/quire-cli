---
id: SR-20261110162003
title: "Base spec review of validate --okf --strict dangling references (PR #167)"
type: SpecReview
analysis: base
scope: "agent-ix/quire-cli; spec/functional/FR-014-validate-okf-bundle.md"
review_set: subset
---
# Base spec review of validate --okf --strict dangling references (PR #167)

## Summary

Ticket: PLAT-1407. The PR adds one row, FR-014-AC-11, to
spec/functional/FR-014-validate-okf-bundle.md. I checked it against
FR-014-AC-3 and FR-014-AC-10 in the same file, against FR-004 `--strict` (Behavior
step 5, FR-004-AC-11 and FR-004-AC-22), against quire-rs FR-038 on main (the
Strict posture and FR-038-AC-9), and against the companion quire-rs#646
(FR-038-AC-11). Read-only context: spec-objects-safety#30, which adds the
`mitigates` edge that the quire-rs#646 fixture uses.

The id is well formed and unique, and the verification method (Test) fits. The
exit code 3 matches the code path: `invalid_request!` in run_okf, and
`tests/cli_okf.rs` asserts `.code(3)` for a bundle error. The bare-id and
own-authority cases agree with FR-014-AC-10 and quire-rs FR-038-AC-9. "Without
`--strict`, FR-014-AC-3 is unchanged" correctly keeps the permissive path as it
is. `src/commands/validate.rs` run_okf ignores `args.strict` today, so
`--okf --strict` silently means `--okf`, and the AC describes a real change.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | medium | AC-11 switches the whole bundle to `BundlePosture::Strict` but only states the dangling-reference outcome. It does not say whether warnings that remain under Strict fail the run, which FR-004 `--strict` does. Two implementers would read it differently | spec/functional/FR-014-validate-okf-bundle.md:110 |
| FND-002 | low | "A target under a foreign authority stays skipped and exits 0" holds only for a bundle that is otherwise Strict-conformant (known types, archetype bodies, complete index with `okf_version`). The AC does not state this precondition | spec/functional/FR-014-validate-okf-bundle.md:110 |
| FND-003 | low | AC-11 lists only "bare id or own authority" as the cases that fail. A miss with no package declared, and a two-segment `ix://<org>/<id>`, are also dangling (FR-014-AC-10, quire-rs FR-038-AC-9) and would also exit 3 | spec/functional/FR-014-validate-okf-bundle.md:110 |

## Verdict

PASS with findings. The AC is sound, testable and consistent with FR-014-AC-10
and quire-rs FR-038-AC-9/AC-11. Fix FND-001 in this PR: either state that
`--okf --strict` fails on errors only under the Strict posture, or that it also
fails on any remaining warning, as FR-004 `--strict` does. Then name the
fixture precondition and the full set of dangling cases.

Pre-existing and outside this diff, so not counted: FR-014 §A still says
"Exit code: **1**" for a hard error, and FR-004 Behavior step 5 says 1, while
FR-014-AC-1, FR-004-AC-11 and the tests all use 3.

## Dispositions

| FND | outcome | reason |
| --- | --- | --- |
| FND-001 | fixed | f031077: AC-11 now says `--okf --strict` also applies FR-004's `--strict` escalation, so every finding, error or remaining warning, exits 3 |
| FND-002 | fixed | f031077: AC-11 now says a bundle that is otherwise Strict-conformant and has only foreign-authority targets exits 0 |
| FND-003 | fixed | f031077: AC-11 now says a miss with no package declared, and a two-segment `ix://<org>/<id>` link to an absent id, are dangling and exit 3 |
