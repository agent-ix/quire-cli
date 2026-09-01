---
id: SR-015
title: "Gap analysis remediation — PR #75"
type: SpecReview
analysis: gap-analysis
scope: "PR #75 external findings FND-030 through FND-036; FR-020, US-006, StR-004, IT-136..145, TC-814"
review_set: subset
relationships:
  - target: ix://agent-ix/quire-cli/spec/functional/FR-020
    type: reviews
  - target: ix://agent-ix/quire-cli/reviews/SR-013
    type: references
---

# SR-015: Gap analysis remediation — PR #75

## Summary

This closing analysis reconciles the independent gap report with the
authoritative traceability model. Binding acceptance belongs to FR-020, not the
US artifact; the source-comment delimiter defect is repaired; and the local
composite gate now reproduces grammar and coverage checks. The repository-wide
historical baseline remains explicitly outside issue #74.

## Verdict

**PASS after fixes.** FR-020 is 9/9 backed, IT-136..145 and TC-814 are backed,
issue #74 has no unmatched trace tags, and the assurance rows introduce no
status lie.

## Assurance Context

- **Measured scope:** FR-020, US-006, StR-004-VC-2/3, IT-136..145, TC-814,
  changed assurance review artifacts, and the gates that reproduce their state.
- **Model boundary:** US examples remain illustrative under the installed
  module pair; only FR acceptance and StR validation criteria are minted as
  binding.
- **Baseline posture:** unrelated pre-existing unbacked rows and grammar
  findings are not reclassified as issue-#74 regressions.

## Findings

| ID | Severity | Summary | Refs |
|----|----------|---------|------|
| FND-030 | high | **FIXED by removing the invalid claim, not inventing targets.** US-006 examples are explicitly illustrative and its matrix cell delegates to FR-020. | US-006; spec/tests.md |
| FND-031 | high | **FIXED.** FR-020-AC-1..9 are all minted and backed after canonical trace comments. | tests/cli_assurance.rs; tests/assurance_cross_language.rs; tests/cli_assurance_contract.rs; tests/audit_static.rs |
| FND-032 | high | **FIXED.** `make spec` is reproducible and included in `make ci`. | Makefile; scripts/check_assurance_traceability.py |
| FND-033 | medium | **FIXED.** The root cause was the period after the final ID: it is not a declared delimiter, so the final token failed to bind. Canonical trace lines remove the ambiguity. | tests/*.rs |
| FND-034 | medium | **FIXED.** The StR-004 rollup is returned to in-progress because VC-1 remains repository-wide work. VC-3 is now real: CONTRIBUTING contains the exact review question and TC-814 gates it. | spec/tests.md; CONTRIBUTING.md; scripts/check_thin_boundary.sh |
| FND-035 | low | **RECORDED, unchanged/out of scope.** Historical unbacked rows remain outside issue #74; the assurance gate checks the exact new target set and global status lies. | scripts/check_assurance_traceability.py |
| FND-036 | low | **RECORDED, unchanged/out of scope.** The targeted changed assurance artifact set is grammar-gated; unrelated legacy documents are not relabeled. | Makefile |

## Closing coverage contract

The checker requires FR-020-AC-1..9, IT-136..145, TC-814, and
StR-004-VC-2/3 to be minted and backed; forbids pseudo `US-006-AC-*` targets and
unmatched issue tags; and rejects any global status lie or untracked source
symbol. This turns the review's one-off coverage inspection into a maintained
regression gate. Five negative mutation probes confirm each failure class exits
non-zero rather than producing a vacuous pass.
