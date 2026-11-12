---
id: SR-20261112032003
title: "EARS conformance of FR-028 (quire-cli PR 166)"
type: SpecReview
analysis: ears-conformance
scope: "agent-ix/quire-cli PR166 head 0ad25b1; spec/functional/FR-028-matrix-criticality-column.md (Description statements)"
review_set: subset
relationships:
  - target: ix://agent-ix/quire-cli/FR-028
    type: references
---

# EARS conformance of FR-028 (quire-cli PR 166)

## Summary

Ticket: PLAT-1412. Checked the normative statements in FR-028's Description against EARS patterns. The three SHALL clauses (append the column, render absent as `unassigned`, SHALL NOT default/infer/inherit) are well-formed ubiquitous and unwanted-behaviour statements, one subject each.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | low | The JSON rule, which FR-028-AC-4 tests, has no SHALL: "In `--format json` the CLI passes the engine's value through unmodified". Fix: "When `--format json` is given, the CLI SHALL pass the engine's `criticality` through unmodified." | spec/functional/FR-028-matrix-criticality-column.md:28-30 |

## Verdict

Low only.
