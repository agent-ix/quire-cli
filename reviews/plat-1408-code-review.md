---
id: SR-20261112034123
title: "Code review of validate --okf --strict (PR #168)"
type: SpecReview
analysis: code-review
scope: "agent-ix/quire-cli; src/commands/validate.rs, tests/cli_okf.rs, spec/tests.md, CHANGELOG.md"
review_set: subset
---
# Code review of validate --okf --strict (PR #168)

## Summary

Ticket: PLAT-1408. Code review with the rust-review lane over
`git diff origin/main...HEAD` at 8a68460. Built against quire-rs#647
(96d1ddf) through a local `--config patch`. `cargo test --test cli_okf`
passes 9/9. `cargo clippy --all-targets -D warnings` and `cargo fmt --check`
are clean.

Under `--strict`, `run_okf` selects `BundlePosture::Strict`, counts the
report's warnings, and fails with exit 3 when any error or warning is
present. The posture selection and exit logic match FR-014-AC-11.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | high | The IT-251 fixture uses `type: weird`. Under Strict that is an `[unknown-type]` error located at `a.md`, so the exit code 3 and the `a.md` substring are both satisfied by the unknown-type line alone. The test passes whether or not `dangling-reference` is escalated and whether or not quire-rs#647 locates it at the source path. Measured stderr: `.../a.md: unknown type 'weird' ... [unknown-type]` followed by `.../a.md: dangling reference 'HZ-9' (edge 'mitigates') ... [dangling-reference]`. Use a registered type in an otherwise Strict-conformant document, or assert on the single dangling-reference line (`a.md: dangling reference 'HZ-9' (edge 'mitigates')`) | tests/cli_okf.rs:332-363 |
| FND-002 | low | The `--strict` doc comment, which this PR edits, and the module doc still say that a `--strict` warning makes `validate` exit 1. Both the per-file path (FR-004-AC-11) and the new `--okf` path exit 3 through `invalid_request!` | src/commands/validate.rs:14, src/commands/validate.rs:70-74 |

## Verdict

FAIL until FND-001 is fixed. The implementation is correct and small. The test
that backs FR-014-AC-11 does not exercise the behaviour it names, so it cannot
catch a regression in dangling-reference escalation or in the quire-rs#647
source path. Rust lane: no idiom, panic, or ownership issues.
