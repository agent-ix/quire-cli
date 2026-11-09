---
id: SR-20261102086812
title: "PLAT-847 quire-cli PR160 spec-review"
type: SpecReview
analysis: base
scope: "agent-ix/quire-cli PR160 head b05771b; spec/functional/FR-026-matrix-subcommand.md"
review_set: subset
---

## Summary

Ticket: PLAT-847. This PR changes one spec line, the wording of FR-026-AC-16. It replaces the comma-list guidance for `Trace:` with guidance to write the id in the declared marker form, and renames the correct-form control to "Correct marker controls". No sub-analysis was selected: the edit changes only wording, with no new statement, relationship or domain object.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | low | No findings (placeholder) | - |

## Examined scope

```yaml
scope:
- id: FR-026-AC-16
  path: spec/functional/FR-026-matrix-subcommand.md
  role: examined
  excerpt: "... an ID not read by the module's declared trace grammar is named with the symbol and guidance to write it in the declared marker form. These advisories do not alter the engine's bindings, stdout matrix payload, or strict verdict. Correct marker controls emit neither range nor unmatched-tag warnings."
```

## Verdict

Clean. The amended AC matches the new stderr message at src/commands/matrix.rs:90 and the CR-215 marker-only binding model. Its verification method is still Test. That test does not assert the guidance clause, which is recorded as gap-analysis FND-001.
