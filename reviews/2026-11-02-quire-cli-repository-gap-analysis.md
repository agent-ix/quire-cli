---
id: SR-14565
title: "Gap analysis — quire-cli repository audit"
type: SpecReview
analysis: gap-analysis
scope: "spec/, src/, tests/, tools/"
review_set: all
relationships:
  - target: "ix://agent-ix/quire-cli/FR-009"
    type: references
---

## Summary

Repository-driven audit of the full quire-cli specification, source, and test tree. Plan completion was not assessed. The computed matrix reports 51 untagged acceptance criteria and no run evidence for 271 criteria.

## Verdict

**FAIL** — untagged criteria and no-run-evidence findings remain in this repository-wide, planless audit.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | high | FR-001-AC-1 has no test or symbol bound by the computed matrix | FR-001-AC-1 |
| FND-002 | high | FR-001-AC-2 has no test or symbol bound by the computed matrix | FR-001-AC-2 |
| FND-003 | high | FR-001-AC-3 has no test or symbol bound by the computed matrix | FR-001-AC-3 |
| FND-004 | high | FR-001-AC-4 has no test or symbol bound by the computed matrix | FR-001-AC-4 |
| FND-005 | high | FR-001-AC-5 has no test or symbol bound by the computed matrix | FR-001-AC-5 |
| FND-006 | high | FR-001-AC-6 has no test or symbol bound by the computed matrix | FR-001-AC-6 |
| FND-007 | high | FR-003-AC-3 has no test or symbol bound by the computed matrix | FR-003-AC-3 |
| FND-008 | high | FR-004-AC-23 has no test or symbol bound by the computed matrix | FR-004-AC-23 |
| FND-009 | high | FR-006-AC-4 has no test or symbol bound by the computed matrix | FR-006-AC-4 |
| FND-010 | high | FR-006-AC-5 has no test or symbol bound by the computed matrix | FR-006-AC-5 |
| FND-011 | high | FR-016-AC-3 has no test or symbol bound by the computed matrix | FR-016-AC-3 |
| FND-012 | high | FR-016-AC-4 has no test or symbol bound by the computed matrix | FR-016-AC-4 |
| FND-013 | high | FR-016-AC-7 has no test or symbol bound by the computed matrix | FR-016-AC-7 |
| FND-014 | high | FR-017-AC-23 has no test or symbol bound by the computed matrix | FR-017-AC-23 |
| FND-015 | high | FR-018-AC-10 has no test or symbol bound by the computed matrix | FR-018-AC-10 |
| FND-016 | high | FR-019-AC-1 has no test or symbol bound by the computed matrix | FR-019-AC-1 |
| FND-017 | high | FR-019-AC-2 has no test or symbol bound by the computed matrix | FR-019-AC-2 |
| FND-018 | high | FR-019-AC-3 has no test or symbol bound by the computed matrix | FR-019-AC-3 |
| FND-019 | high | FR-019-AC-4 has no test or symbol bound by the computed matrix | FR-019-AC-4 |
| FND-020 | high | FR-019-AC-5 has no test or symbol bound by the computed matrix | FR-019-AC-5 |
| FND-021 | high | FR-019-CON-1 has no test or symbol bound by the computed matrix | FR-019-CON-1 |
| FND-022 | high | FR-020-CON-2 has no test or symbol bound by the computed matrix | FR-020-CON-2 |
| FND-023 | high | FR-020-CON-4 has no test or symbol bound by the computed matrix | FR-020-CON-4 |
| FND-024 | high | FR-021-AC-2 has no test or symbol bound by the computed matrix | FR-021-AC-2 |
| FND-025 | high | FR-021-AC-3 has no test or symbol bound by the computed matrix | FR-021-AC-3 |
| FND-026 | high | FR-021-AC-5 has no test or symbol bound by the computed matrix | FR-021-AC-5 |
| FND-027 | high | FR-021-AC-6 has no test or symbol bound by the computed matrix | FR-021-AC-6 |
| FND-028 | high | FR-027-AC-8 has no test or symbol bound by the computed matrix | FR-027-AC-8 |
| FND-029 | high | FR-027-AC-10 has no test or symbol bound by the computed matrix | FR-027-AC-10 |
| FND-030 | high | NFR-001-AC-2 has no test or symbol bound by the computed matrix | NFR-001-AC-2 |
| FND-031 | high | NFR-001-M-1 has no test or symbol bound by the computed matrix | NFR-001-M-1 |
| FND-032 | high | NFR-001-M-2 has no test or symbol bound by the computed matrix | NFR-001-M-2 |
| FND-033 | high | NFR-001-M-3 has no test or symbol bound by the computed matrix | NFR-001-M-3 |
| FND-034 | high | NFR-002-AC-2 has no test or symbol bound by the computed matrix | NFR-002-AC-2 |
| FND-035 | high | NFR-002-M-1 has no test or symbol bound by the computed matrix | NFR-002-M-1 |
| FND-036 | high | NFR-002-M-2 has no test or symbol bound by the computed matrix | NFR-002-M-2 |
| FND-037 | high | NFR-003-M-1 has no test or symbol bound by the computed matrix | NFR-003-M-1 |
| FND-038 | high | NFR-004-M-1 has no test or symbol bound by the computed matrix | NFR-004-M-1 |
| FND-039 | high | NFR-004-M-2 has no test or symbol bound by the computed matrix | NFR-004-M-2 |
| FND-040 | high | NFR-005-M-1 has no test or symbol bound by the computed matrix | NFR-005-M-1 |
| FND-041 | high | NFR-005-M-2 has no test or symbol bound by the computed matrix | NFR-005-M-2 |
| FND-042 | high | NFR-006-M-1 has no test or symbol bound by the computed matrix | NFR-006-M-1 |
| FND-043 | high | NFR-006-M-2 has no test or symbol bound by the computed matrix | NFR-006-M-2 |
| FND-044 | high | NFR-008-M-1 has no test or symbol bound by the computed matrix | NFR-008-M-1 |
| FND-045 | high | NFR-008-M-2 has no test or symbol bound by the computed matrix | NFR-008-M-2 |
| FND-046 | high | NFR-008-M-3 has no test or symbol bound by the computed matrix | NFR-008-M-3 |
| FND-047 | high | NFR-008-M-4 has no test or symbol bound by the computed matrix | NFR-008-M-4 |
| FND-048 | high | NFR-009-M-1 has no test or symbol bound by the computed matrix | NFR-009-M-1 |
| FND-049 | high | NFR-009-M-2 has no test or symbol bound by the computed matrix | NFR-009-M-2 |
| FND-050 | high | NFR-009-M-3 has no test or symbol bound by the computed matrix | NFR-009-M-3 |
| FND-051 | high | NFR-009-M-4 has no test or symbol bound by the computed matrix | NFR-009-M-4 |
| FND-052 | high | No run evidence is bound to 271 criteria; matrix status reflects static traceability only | quoin matrix evidence_status |
| FND-053 | medium | Coverage reports 28 unbacked functional-coverage rows | spec/tests.md |

## Coverage

- Reconciliation: `quoin matrix` (quoin 0.29.2, quire 0.37.1) and `quire coverage` (quire 0.37.1); no run evidence read.
- Plan completion: not assessed
- Criteria: tagged 199 · untagged 51 · tagged-by-ignored-test 0 · method-without-symbol 21
- Evidence: no run evidence: 271
- Coverage reconciliation: unbacked rows 28 · status lies 0 · no-symbol rows 4 · undeclared statuses 0 · untracked symbols 0 · shared trace IDs 16
- Evidence advice: 271 recommendations; mismatch 26 · inconclusive 9 · uncatalogued 41. Recommendations are advisory, not evidence.
- Reverse audit: the source/test TODO/FIXME/XXX scan found no unfinished implementation in the changed path; untracked-symbol findings are summarized above.
- Semantic review: skipped
