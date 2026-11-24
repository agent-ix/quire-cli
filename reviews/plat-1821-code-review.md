---
id: SR-20261116045030
title: "Code review — PLAT-1821 quire-cli anomaly record"
type: SpecReview
analysis: code-review
scope: "agent-ix/quire-cli; spec/anomalies/AN-001-dispatch-only-verification.md, spec/anomalies/index.md, spec/functional/FR-022-npm-distribution.md, .github/workflows/ci.yml, .github/workflows/release.yml, .github/workflows/tag-attestation.yml"
review_set: subset
relationships:
  - target: "ix://agent-ix/quire-cli/FR-022"
    type: references
---

## Summary

Ticket PLAT-1821 adds one first-party Anomaly record for quire-cli's dispatch-only CI and manual release workflow. The record is parseable and keeps assessment fields unset, but its broad title and opening claim omit the existing tag-push attestation workflow.

## Verdict

**CONDITIONAL** — one medium finding asks the record to distinguish manual CI/publication from tag-triggered release attestation.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | medium | The record describes repository verification and release verification as dispatch-only without distinguishing the `v*` tag-push attestation workflow, which runs release verification on tags. Narrow the title and description to CI and manual publication, and state that tag attestation is tag-triggered and does not run on main pushes. | spec/anomalies/AN-001-dispatch-only-verification.md:3,12-16; .github/workflows/tag-attestation.yml:4-7 |

## Review Notes

- No applicable AssuranceProfile was found under `spec/`.
- The Anomaly instance uses the shared `spec-artifacts-process@v0.36.0` type; its `identified` status permits omitted severity and its explicit `safety_impact: unassessed` does not claim an assessment.
- The anomaly index links the record. Focused `quoin anomaly report spec/anomalies --json` read it with `unreadable: []` and reported `AN-001` as unassessed.
- The diff contains no production source or tests, so gap-analysis and language-specific reviews were not applicable. I checked for duplicate local records; neither the type nor workflow behavior is copied into a second implementation.

