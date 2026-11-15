---
id: SR-20261115194616
title: "PLAT-1761 quire-cli PR206 code-review"
type: SpecReview
analysis: code-review
scope: "agent-ix/quire-cli PR206 head 42c3183; spec/functional/FR-038-approval-block.md (git diff origin/main...HEAD)"
review_set: subset
---

## Summary

Ticket: PLAT-1761. Correctness review of the spec-only FR-038 change that lets `--approval` take an `engineering-assurance.release-verify-result/v1` file. The diff touches one file. No Rust changed, so no rust-review lane ran. The review was done by reading the files. `quire validate` and `quoin` were denied by the permission layer in this session and were not run.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | high | The Verdict row for a release result requires the nested `status` first, then `not_released`, then the reasons. For a `not_released` result whose nested approval is `approved`, the Verdict cell therefore begins with `approved` (`approved, not_released, decision_owner_mismatch`). The bullet two lines below forbids Verdict text that reads as an approval in exactly this case. The two statements contradict each other. One fix is to lead with `not_released` and show the nested status as a labelled sub-part, for example `not_released (approval: approved); decision_owner_mismatch`. | spec/functional/FR-038-approval-block.md:70, :74-77 |
| FND-002 | medium | For a `not_released` result with a nested `approved` verdict, Signed at is read "exactly as the table above reads a verdict file", so it shows the signer's time. The standalone rule (and approval.rs:100-102) shows a time only beside an entry labelled an approval, because a time beside an unaccepted approval reads as attested (EA FR-041). The release section gives an entry that is not an approval a signing time. State that Signed at is `not available` unless the entry is labelled an approval. | spec/functional/FR-038-approval-block.md:69, :50, src/commands/export/approval.rs:100-102 |
| FND-003 | medium | Reasons are joined "as recorded text joined by `, `", and the nested status, `not_released` and the reasons all share that one separator. A reason that contains `, ` cannot be told apart from two reasons, so a reader cannot check the "never drop, reorder or add a reason" rule from the block. This is the boundary defect FR-038 already fixed for Meaning by writing each value as a JSON string. Quote each reason the same way. | spec/functional/FR-038-approval-block.md:70, :78-79, :51 |
| FND-004 | medium | A `released` result that carries a non-empty `reasons` is neither refused (the refusal list covers only `not_released` reasons) nor shown (the Verdict row shows reasons only for the "Otherwise" branch). The reasons are silently dropped, which breaks "SHALL preserve every entry of `reasons`". Either refuse a `released` result with non-empty `reasons` or show them. | spec/functional/FR-038-approval-block.md:70, :78-79, :85-90 |

## Examined scope

```yaml
scope:
- id: FR-038
  path: spec/functional/FR-038-approval-block.md
  role: examined
  excerpt: |-
    `quire export` SHALL accept `--approval <PATH>`, repeatable, each a verdict file of `engineering-assurance approval-verify` ([EA FR-041](ix://agent-ix/engineering-assurance/FR-041)) or a result file of `engineering-assurance release-verify` ([EA FR-047](ix://agent-ix/engineering-assurance/FR-047)), and SHALL generate an approval block (`in/approval.md`, [FR-035](./FR-035-default-export-template.md)) with one entry per file.
- id: FR-038-release-verdict-row
  path: spec/functional/FR-038-approval-block.md
  role: examined
  excerpt: |-
    | Verdict | For `released`, the nested `status`, which is `approved`. Otherwise the nested `status` (`unavailable` when `approval` is absent), then `not_released`, then each entry of `reasons` verbatim in order, as recorded text joined by `, `. |
- id: FR-038-release-member-row
  path: spec/functional/FR-038-approval-block.md
  role: examined
  excerpt: |-
    | Person, Signed at, Meaning, Record | When `approval` is present, the nested `approval` verdict, read exactly as the table above reads a verdict file. The top-level `approver`, `meaning` and `signed_at` of a `released` result are not read. When `approval` is absent: Person `not available`, Signed at `not available`, Meaning `not recorded`, Record `not available`. |
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
- id: FR-038-refuse
  path: spec/functional/FR-038-approval-block.md
  role: examined
  excerpt: |-
    The command SHALL refuse a release result whose `status` is neither `released` nor `not_released`, a `released` result without a nested `approval`, a `released` result whose nested `approval.status` is not `approved`, a `not_released` result whose `reasons` is absent or not a list of strings, and an `approval` member that is present but is not an `engineering-assurance.approval-verify-result/v1` document ([FR-040](./FR-040-export-template-and-identity-errors.md)).
- id: FR-038-label
  path: spec/functional/FR-038-approval-block.md
  role: examined
  excerpt: |-
    The command SHALL label an entry of a release result as an approval only when the result `status` is `released` and the nested `approval.status` is `approved`.
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
- id: FR-038-one-entry
  path: spec/functional/FR-038-approval-block.md
  role: examined
  excerpt: |-
    A release result SHALL be one entry, in the order given on the command line, and SHALL be handled by the sanitising, escaping, wrapping, sizing and no-inference rules of this requirement exactly as a standalone verdict is.
- id: FR-038-signed-at-row
  path: spec/functional/FR-038-approval-block.md
  role: context_only
  excerpt: |-
    | Signed at | `signed_at` of the verdict, a UTC instant, shown as `YYYY-MM-DD HH:MM:SS UTC` under the column title `Signed at (signer's claim)`. For a verdict without `signed_at` (every status other than `approved`, or `signed_at_missing`) the text `not available`. |
- id: src/commands/export/approval.rs
  path: src/commands/export/approval.rs
  role: context_only
  excerpt: |-
    // A time beside a signature the verifier did not accept as an approval
    // would read as attested, so only an `approved` entry shows one.
    let signed_at = match verdict.signed_at.as_deref().filter(|_| approved) {
```

## Verdict

Changes required. The extension is narrow and keeps the FR-031 presentation stance: no re-verification, no release decision, no invented signer, meaning, time or record when `approval` is absent, and refusal before the converter starts. The `released` path is consistent. The `not_released` path with a nested `approved` verdict is where it breaks: the Verdict cell text it requires contradicts its own no-approval-wording rule (FND-001), and it shows a signing time on an entry that is not an approval (FND-002). FND-003 and FND-004 are gaps in how the reasons are kept. Public-repo hygiene is clean: no roadmap content, no pins or digests, no compatibility layer.

## Dispositions

Round 1, fix commit 9703362.

| FND | outcome | reason |
| --- | --- | --- |
| FND-001 | fixed | The release Verdict row now reads "For `not_released`, up to three segments joined by `; `: first the text `not_released`; ... third, only when `approval` is present and its `status` is not `approved`, `nested: ` ...", and "A nested `approved` status of a `not_released` result is not shown." FR-038-AC-17 asserts the exact cell `not_released; reasons: ["decision_owner_mismatch"]`. |
| FND-002 | fixed | The release member row now says "For a `not_released` result whose nested `approval.status` is `approved`: Person `not available`, Signed at `not available`, Record `not available`". FR-038-AC-17 asserts neither `Ada Example` nor `2026-01-02 03:04:05 UTC` appears in the block. The matching change to the status-keyed filter in approval.rs belongs to the implementation portion of PLAT-1761. |
| FND-003 | fixed | Reasons are now written as "a JSON array, each reason one array element written as a JSON string (`["a", "b"]`, so a reason containing `, ` stays one value)", and the segments are joined by `; `. FR-038-AC-11 asserts a reason `a, b` shows as the one element `"a, b"`. |
| FND-004 | fixed | FR-040's new Release result invalid row refuses a result that "is `released` with a non-empty `reasons`". FR-038-AC-14 and FR-040-AC-8 test `reasons` `["x"]` on a `released` result, and FR-040-AC-8 states that `reasons` `[]` is not a failure. |
