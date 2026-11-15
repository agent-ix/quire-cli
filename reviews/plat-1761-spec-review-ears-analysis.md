---
id: SR-20261115194619
title: "PLAT-1761 quire-cli PR206 spec-review/ears-analysis"
type: SpecReview
analysis: ears-conformance
scope: "agent-ix/quire-cli PR206 head 42c3183; spec/functional/FR-038-approval-block.md (new and edited statements)"
review_set: subset
---

## Summary

Ticket: PLAT-1761. EARS grammar check of the FR-038 statements this PR adds or edits. `quire validate` was denied, so its `ears:` diagnostics were not collected. The check was done by reading the text.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | low | The refusal bullet is non-singular: one SHALL carries five refusal conditions, in a list that is not in the conditional form. Split it into one `If <condition>, then the command SHALL refuse ...` statement per cause, or move the causes into the FR-040 table and keep one statement here. | spec/functional/FR-038-approval-block.md:85-90 |
| FND-002 | low | Two new statements have an input, not the system, as their subject: "A release result SHALL be one entry ..." and "A standalone `approval-verify` verdict file SHALL produce the entry ...". Restate them as `The command SHALL ...`. | spec/functional/FR-038-approval-block.md:91-93, :135-136 |
| FND-003 | low | The `not_released` bullet joins two obligations in one statement (SHALL NOT ... approval wording, and SHALL prefix ... `claimed:`). Split it. | spec/functional/FR-038-approval-block.md:74-77 |
| FND-004 | low | The opening paragraph of "Release-verify input" says how a release result is interpreted ("the entry takes every member from it") as description, with no SHALL, so its obligation strength is unclear. Make it a SHALL statement or mark it informative. | spec/functional/FR-038-approval-block.md:61-65 |

## Examined scope

```yaml
scope:
- id: FR-038
  path: spec/functional/FR-038-approval-block.md
  role: examined
  excerpt: |-
    `quire export` SHALL accept `--approval <PATH>`, repeatable, each a verdict file of `engineering-assurance approval-verify` ([EA FR-041](ix://agent-ix/engineering-assurance/FR-041)) or a result file of `engineering-assurance release-verify` ([EA FR-047](ix://agent-ix/engineering-assurance/FR-047)), and SHALL generate an approval block (`in/approval.md`, [FR-035](./FR-035-default-export-template.md)) with one entry per file.
- id: FR-038-release-intro
  path: spec/functional/FR-038-approval-block.md
  role: examined
  excerpt: |-
    A file whose `protocol` is `engineering-assurance.release-verify-result/v1` is a release result. Its `status` is `released` or `not_released`, `reasons` lists the failed checks of a `not_released` result, and its `approval` member, when present, is the unchanged FR-041 verdict of the release tag (EA FR-047). The file is read as data like any other verdict file, and the entry takes every member from it.
- id: FR-038-label
  path: spec/functional/FR-038-approval-block.md
  role: examined
  excerpt: |-
    The command SHALL label an entry of a release result as an approval only when the result `status` is `released` and the nested `approval.status` is `approved`.
- id: FR-038-not-released
  path: spec/functional/FR-038-approval-block.md
  role: examined
  excerpt: |-
    If the result `status` is `not_released`, then the command SHALL NOT give the entry a heading, label or Verdict text that reads as an approval, including when the nested `approval.status` is `approved` (for example with reason `decision_owner_mismatch`), and SHALL prefix a recorded Meaning with `claimed:`.
- id: FR-038-reasons
  path: spec/functional/FR-038-approval-block.md
  role: examined
  excerpt: |-
    The command SHALL preserve every entry of `reasons` and SHALL NOT drop, reorder, reinterpret or add a reason.
- id: FR-038-absent
  path: spec/functional/FR-038-approval-block.md
  role: examined
  excerpt: |-
    If `approval` is absent, then the command SHALL NOT supply a signer, a meaning, a signing time or a record from the release result or from anywhere else.
- id: FR-038-no-reverify
  path: spec/functional/FR-038-approval-block.md
  role: examined
  excerpt: |-
    The command SHALL NOT re-verify a signature, a tag or a release check, and SHALL NOT decide a release: it presents the nested verdict and the reasons the result already holds.
- id: FR-038-refuse
  path: spec/functional/FR-038-approval-block.md
  role: examined
  excerpt: |-
    The command SHALL refuse a release result whose `status` is neither `released` nor `not_released`, a `released` result without a nested `approval`, a `released` result whose nested `approval.status` is not `approved`, a `not_released` result whose `reasons` is absent or not a list of strings, and an `approval` member that is present but is not an `engineering-assurance.approval-verify-result/v1` document ([FR-040](./FR-040-export-template-and-identity-errors.md)).
- id: FR-038-one-entry
  path: spec/functional/FR-038-approval-block.md
  role: examined
  excerpt: |-
    A release result SHALL be one entry, in the order given on the command line, and SHALL be handled by the sanitising, escaping, wrapping, sizing and no-inference rules of this requirement exactly as a standalone verdict is.
- id: FR-038-standalone
  path: spec/functional/FR-038-approval-block.md
  role: examined
  excerpt: |-
    A standalone `approval-verify` verdict file SHALL produce the entry this requirement specifies for it, unchanged by the release-verify input.
```

## Verdict

Minor only. The edited Description, the label rule, the `approval`-absent rule and the reasons rule are well-formed ubiquitous or unwanted-behaviour statements with a named subject. The re-verify bullet joins two SHALL NOTs under one theme, which is acceptable in this file's style. The four findings are grammar refinements.

## Dispositions

Round 1, fix commit 9703362.

| FND | outcome | reason |
| --- | --- | --- |
| FND-001 | fixed | The five-cause refusal bullet is replaced by one conditional statement, "If a release result matches a cause of the Release result invalid row of FR-040, then the command SHALL refuse it before any converter starts". The causes are now in the new FR-040 row. |
| FND-002 | fixed | Both statements now have the system as subject: "The command SHALL present a release result as one entry ..." and "The command SHALL produce for a standalone `approval-verify` verdict file the entry ...". |
| FND-003 | fixed | The `not_released` bullet is split into two statements: "SHALL NOT give the entry a heading, label or Verdict text that reads as an approval" and "If the result `status` is `not_released`, then the command SHALL prefix a recorded Meaning with `claimed:`". |
| FND-004 | fixed | The descriptive opening paragraph is replaced by three SHALL bullets: treat the protocol as a release result, read only `protocol`, `status`, `reasons` and `approval`, and SHALL NOT read the top-level members. |
