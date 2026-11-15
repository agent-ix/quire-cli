---
id: SR-20261115194624
title: "PLAT-1761 quire-cli PR206 spec-review/risk-complexity-analysis"
type: SpecReview
analysis: risk-complexity
scope: "agent-ix/quire-cli PR206 head 42c3183; spec/functional/FR-038-approval-block.md (Release-verify input, FR-038-AC-10..AC-16)"
review_set: subset
---

## Summary

Ticket: PLAT-1761. Technical risk and volatility of the FR-038 extension before it is turned into tasks. The implementation complexity is moderate: approval.rs needs a second protocol branch that unwraps the nested verdict, reuses `row()` for the four member cells, and builds a composite Verdict cell. The status-keyed `signed_at` filter at approval.rs:102 will then need to key on the entry's label, not on the nested status (see SR-20261115194616 FND-002).

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | medium | Every fact FR-038 states about the release-verify result is a claim about another repository's contract: the `status` values, `reasons` being a list of strings, the nested `approval` being the unchanged FR-041 verdict, and the top-level `approver`, `meaning` and `signed_at` on `released`. The strict refusal rule exits 3 on any difference, and AC-16 allows only hand-written static fixtures. If EA FR-047 differs, the implementation passes every AC and fails on real output. Checked against EA FR-047 Outputs: the `status` values, the nested `approval` as the unchanged FR-041 result (absent exactly when the reasons include `record_unreadable` or `record_invalid`) and the top-level `approver`, `meaning` and `signed_at` on `released` match. FR-047 does not state the element type of `reasons`; its entries are the enumerated check codes. One fixture shape does not match: FR-047 lists "an FR-041 status other than `approved`" as itself one of the `reasons`, so a real `not_released` result with a nested `untrusted_signer` carries `untrusted_signer` in `reasons`. FR-038-AC-11's fixture (nested `untrusted_signer`, reasons `decision_owner_mismatch` and `release_commit_record_not_added`) is a shape `release-verify` does not produce, and on real output the Verdict row shows the nested status twice. Align the AC-11 fixture with FR-047 and state whether the Verdict cell repeats the nested status. | spec/functional/FR-038-approval-block.md:61-65, :70, :85-90, :153, :158; EA spec/functional/FR-047-release-record-and-signed-release-tag.md:80-92 |

## Examined scope

```yaml
scope:
- id: FR-038-release-intro
  path: spec/functional/FR-038-approval-block.md
  role: examined
  excerpt: |-
    A file whose `protocol` is `engineering-assurance.release-verify-result/v1` is a release result. Its `status` is `released` or `not_released`, `reasons` lists the failed checks of a `not_released` result, and its `approval` member, when present, is the unchanged FR-041 verdict of the release tag (EA FR-047). The file is read as data like any other verdict file, and the entry takes every member from it.
- id: FR-038-refuse
  path: spec/functional/FR-038-approval-block.md
  role: examined
  excerpt: |-
    The command SHALL refuse a release result whose `status` is neither `released` nor `not_released`, a `released` result without a nested `approval`, a `released` result whose nested `approval.status` is not `approved`, a `not_released` result whose `reasons` is absent or not a list of strings, and an `approval` member that is present but is not an `engineering-assurance.approval-verify-result/v1` document ([FR-040](./FR-040-export-template-and-identity-errors.md)).
- id: FR-038-AC-16
  path: spec/functional/FR-038-approval-block.md
  role: examined
  excerpt: |-
    The release-verify fixtures are static documents; the export of one starts no git process, reads no repository and makes no signature check.
- id: FR-038-no-reverify
  path: spec/functional/FR-038-approval-block.md
  role: examined
  excerpt: |-
    The command SHALL NOT re-verify a signature, a tag or a release check, and SHALL NOT decide a release: it presents the nested verdict and the reasons the result already holds.
- id: src/commands/export/approval.rs
  path: src/commands/export/approval.rs
  role: context_only
  excerpt: |-
    const RESULT_PROTOCOL: &str = "engineering-assurance.approval-verify-result/v1";
```

## Verdict

Low to moderate risk. The change is additive, the standalone path is explicitly kept unchanged, and the no-re-verify stance keeps the export free of git, network and child-process dependencies. The main volatility is the cross-repository result shape (FND-001). It is a single external contract; checked against EA FR-047, it matches except for the FR-041 status being repeated in `reasons`, which the AC-11 fixture omits.

## Dispositions

Round 1, fix commit 9703362.

| FND | outcome | reason |
| --- | --- | --- |
| FND-001 | fixed | FR-038 now states the FR-047 shape: `reasons` lists the failed checks, "an FR-041 status other than `approved` being itself one of them". The AC-11 fixture carries reasons `untrusted_signer` and `release_commit_record_not_added` with a nested `untrusted_signer`. The spec states the repetition explicitly: "A nested `status` that is not `approved` is therefore shown twice, once as a reason (EA FR-047) and once in the `nested:` segment". |
