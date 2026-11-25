---
id: SR-967175026302197768900474181935084795
title: "Base spec review — quire-cli selftest matrix criterion"
type: SpecReview
analysis: base
scope: "agent-ix/quire-cli; spec/functional/FR-047-quire-selftest.md"
review_set: subset
relationships:
  - target: "ix://agent-ix/quire-cli/FR-047"
    type: references
---

## Summary

Reviewed the changed FR-047-AC-3 against the base requirement checklist. The criterion is observable and verifiable, but the code/test evidence depends on a marker path not present in the active module configuration.

## Verdict

**CONDITIONAL** — the requirement is understandable and testable; the companion module configuration must be present for the stated binding behavior to hold.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | medium | AC evidence uses a fixture-only entry path, so it does not prove the active module binds selftest markers. | FR-047-AC-3 |

## Validation

- The changed criterion has a unique ID and Test verification method.
- Its new obligations are externally observable: strict failure and exact script-path reporting.
- The focused test covers a valid marker, removal, and an unknown criterion, but its fixture-local module rule does not cover the active module configuration.
- No other FR-047 requirement or acceptance criterion changed.
- No applicable AssuranceProfile was found under spec/.
- quire validate --scope . spec/functional/FR-047-quire-selftest.md --strict — exit 0.

