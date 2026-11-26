---
id: SR-967175026302197768900474181941935084800
title: Specification object review — Quire CLI proposed ratings
type: SpecReview
analysis: object-review
scope: agent-ix/quire-cli; PLAT-1449; FeatureRiskRating objects FRR-001 and FRR-002
review_set: subset
relationships:
  - { target: "ix://agent-ix/quire-cli/FRR-001", type: references }
  - { target: "ix://agent-ix/quire-cli/FRR-002", type: references }
---

## Summary

Both FeatureRiskRating objects preserve the expected object type, supplier scope, rating values, proposal state, and intended-use link. Their relationship sets cover the 46 FRs without duplicate targets.

## Verdict

**PASS** — the object shape and target population are complete; acceptance remains pending a named person.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | low | No findings (placeholder) | - |

## Scope

- FRR-001 `FeatureRiskRating` identity, supplier scope, high rating, proposed status, IU-001 link, and 11 FR targets: examined.
- FRR-002 `FeatureRiskRating` identity, supplier scope, not-high rating, proposed status, IU-001 link, and 35 FR targets: examined.
- Prohibited acceptance and private-data fields: examined.

## Notes

No `rated_by`, score, failure-mode, detection, or private repository reference is present in the records. The two objects remain unsigned and unaccepted in the proposed state. A person must still accept the proposed assessment through the named-person workflow.
