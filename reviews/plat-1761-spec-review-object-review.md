---
id: SR-20261115194621
title: "PLAT-1761 quire-cli PR206 spec-review/object-review"
type: SpecReview
analysis: base
scope: "agent-ix/quire-cli PR206 head 42c3183; spec/functional/FR-038-approval-block.md (release-verify result object and entry mapping)"
review_set: subset
---

## Summary

Ticket: PLAT-1761. Reviews the new domain object (the release-verify result, with its `status`, `reasons` and nested `approval`) and how it maps onto the five entry fields. FR-038's description of the object was checked against the EA FR-047 Outputs and matches it: `approval` is the unchanged FR-041 result and is absent only when the reasons include `record_unreadable` or `record_invalid`.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | medium | The nested verdict's `reason` member is not mapped. A standalone verdict's Verdict cell is "the `status` ... and the `reason` when present" (approval.rs renders it as `status (reason)`). The release Verdict row names only "the nested `status`", and the "read exactly as the table above" rule covers only Person, Signed at, Meaning and Record. Whether a nested `reason` (for example on `untrusted_signer`) is shown is therefore undefined, and two implementers would differ. Say the nested Verdict text is formed as for a standalone verdict. | spec/functional/FR-038-approval-block.md:70, :52, src/commands/export/approval.rs:127-130 |
| FND-002 | low | The Verdict row adds `unavailable` as a status value for a release result with no `approval`. It is not one of the FR-041 statuses listed in the standalone Verdict row, and it is close to the real status `unverifiable`, so a reader can take it for a verifier status. Use wording that cannot be read as a status, for example `no approval verdict`. | spec/functional/FR-038-approval-block.md:70, :52 |

## Examined scope

```yaml
scope:
- id: FR-038-release-intro
  path: spec/functional/FR-038-approval-block.md
  role: examined
  excerpt: |-
    A file whose `protocol` is `engineering-assurance.release-verify-result/v1` is a release result. Its `status` is `released` or `not_released`, `reasons` lists the failed checks of a `not_released` result, and its `approval` member, when present, is the unchanged FR-041 verdict of the release tag (EA FR-047). The file is read as data like any other verdict file, and the entry takes every member from it.
- id: FR-038-release-member-row
  path: spec/functional/FR-038-approval-block.md
  role: examined
  excerpt: |-
    | Person, Signed at, Meaning, Record | When `approval` is present, the nested `approval` verdict, read exactly as the table above reads a verdict file. The top-level `approver`, `meaning` and `signed_at` of a `released` result are not read. When `approval` is absent: Person `not available`, Signed at `not available`, Meaning `not recorded`, Record `not available`. |
- id: FR-038-release-verdict-row
  path: spec/functional/FR-038-approval-block.md
  role: examined
  excerpt: |-
    | Verdict | For `released`, the nested `status`, which is `approved`. Otherwise the nested `status` (`unavailable` when `approval` is absent), then `not_released`, then each entry of `reasons` verbatim in order, as recorded text joined by `, `. |
- id: FR-038-verdict-row
  path: spec/functional/FR-038-approval-block.md
  role: context_only
  excerpt: |-
    | Verdict | The `status` of the verdict, verbatim (`approved`, `wrong_record`, `untrusted_signer`, `meaning_missing`, `signed_bad`, `unsigned`, `unverifiable`), and the `reason` when present. |
- id: src/commands/export/approval.rs
  path: src/commands/export/approval.rs
  role: context_only
  excerpt: |-
    let verdict_text = match &verdict.reason {
        Some(reason) => format!("{} ({reason})", verdict.status),
        None => verdict.status.clone(),
    };
```

## Verdict

Minor changes. The object boundary is right: the nested `approval` is the unchanged FR-041 verdict, the top-level release members are not used as substitutes, and the refusal rule rejects a nested member of any other protocol. The two findings are mapping gaps in the Verdict field.

## Dispositions

Round 1, fix commit 9703362.

| FND | outcome | reason |
| --- | --- | --- |
| FND-001 | fixed | The release Verdict row now forms the nested text "as for a standalone verdict (`status`, or `status (reason)` when the nested `reason` is present)" in both the `released` and `nested:` cases. FR-038-AC-11 asserts `nested: untrusted_signer (x, y)`. |
| FND-002 | fixed | `unavailable` is removed. When `approval` is absent there is no `nested:` segment, and FR-038-AC-12 asserts the cell is exactly `not_released; reasons: ["record_invalid"]`. |
