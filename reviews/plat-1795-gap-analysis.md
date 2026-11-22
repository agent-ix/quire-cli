---
id: SR-20261116045012
title: "PLAT-1795 quire-cli gap analysis"
type: SpecReview
analysis: gap-analysis
scope: "agent-ix/quire-cli; PLAT-1795 changes to assurance cross-language checks and qualification JSON decoding; FR-020 and FR-025"
review_set: subset
relationships:
  - { target: "ix://agent-ix/quire-cli/FR-020", type: references }
  - { target: "ix://agent-ix/quire-cli/FR-025", type: references }
---

## Summary

Ran the computed matrix and inspected the changed behavior against FR-020/FR-025. The qualification EOF change is owned by FR-025-AC-2 and has a failing-before/positive-after assertion shape; the assurance corpus exercises real Rust and Python schema/identity checks, while its Node path only checks JSON syntax.

## Verdict

**CONDITIONAL** — the required Node semantic acceptance/refusal behavior is absent from the supported reader path; the project has no Node assurance schema reader/dependency to run.

## Coverage

- Plan completion: not assessed.
- `quoin matrix --repo . --json` completed with Quoin 0.36.8 and `spec-artifacts-process@v0.36.0`.
- Repository matrix: 509 criteria; 405 tagged, 49 untagged, 34 method-without-symbol, 21 tagged-by-ignored-test. The matrix has no bound run evidence in this checkout.
- Relevant FR-020 rows: AC-1..5 and AC-7..9 tagged; AC-6 has method-without-symbol; CON-5 is untagged. FR-020-AC-8 is tagged by both cross-language tests, but its Node assertion remains syntax-only.
- Relevant FR-025 rows: AC-1..7 tagged. FR-025-AC-2 is tagged by the new end-of-input test and existing malformed-input tests.
- Reverse-gap/stub inspection of changed production code found no unowned decoder behavior or stub. `CoverageTrailingData` is covered by FR-025-AC-2. No applicable AssuranceProfile was present. Optional semantic review was not run.
- Reviewer focused tests: one decoder test passed (5 filtered); one cross-language corpus test passed (1 filtered). `cargo fmt --all -- --check`, focused clippy for `quire-qualify --test coverage`, and focused clippy for `assurance_cross_language` passed.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | medium | The shared corpus does not exercise semantic acceptance/refusal in Node: unsupported versions, duplicate module/schema identities, unknown fields, and unaccepted module premises are treated as syntactically valid. No supported Node assurance reader exists in this repository, so PLAT-1795 cannot claim evidence for that reader. | tests/assurance_cross_language.rs:236-255 |

## Dispositions

| FND | outcome | reason |
| --- | --- | --- |
| FND-001 | deferred | No supported Node assurance reader exists in this repository scope. The existing JSON.parse probe supplies syntax-only evidence and does not satisfy the Node acceptance/refusal obligation; PLAT-1795 remains blocked pending an actual supported reader. |
