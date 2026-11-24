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

## Dispositions

| FND | Outcome | Reason |
| --- | --- | --- |
| FND-001 | still-open | The anomaly record now distinguishes main-push CI, manual publication, and `v*` tag attestation, but `spec/anomalies/index.md:10` still labels it “CI and release verification are dispatch-only,” preserving the overbroad claim in the public index. Update the index label before merge. |
| FND-001 | fixed | The record and index now describe main CI as dispatch-only and publication as manual, while explicitly stating that tag attestation runs on `v*` pushes. |

FND-001 after_excerpt (spec/anomalies/AN-001-dispatch-only-verification.md:12-22):

```text
The 2026-11-23 regulated-standards review found that this repository does not
run its CI validation automatically for changes on `main`. The exact workflow
triggers are:

- `.github/workflows/ci.yml` runs only on `workflow_dispatch`; it does not run
  on pushes or pull requests.
- `.github/workflows/release.yml` runs only on `workflow_dispatch`; publication
  is manually started, as specified by FR-022-AC-5.
- `.github/workflows/tag-attestation.yml` runs on pushes of tags matching
  `v*`. This tag-triggered attestation is separate from CI validation of
  changes on `main`.
```
