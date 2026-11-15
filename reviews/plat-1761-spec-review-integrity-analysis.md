---
id: SR-20261115194620
title: "PLAT-1761 quire-cli PR206 spec-review/integrity-analysis"
type: SpecReview
analysis: integrity
scope: "agent-ix/quire-cli PR206 head 42c3183; spec/functional/FR-038-approval-block.md"
review_set: subset
---

## Summary

Ticket: PLAT-1761. Checks that the amended FR-038 is complete, consistent and atomic. The FR-040 inconsistency and the Verdict-cell contradiction are recorded in SR-20261115194618 FND-001 and SR-20261115194616 FND-001, and are not repeated here.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | high | FR-038-AC-13 cannot pass as written. AC-1 and AC-2 specify extracted-text contents, not the bytes of an `in/approval.md`, so there is no reference file to compare against. And with a release-verify result also supplied, `in/approval.md` gains a second row, so it cannot be byte-identical to the standalone-only file. Restate it as: the standalone entry's row in `in/approval.md` is byte-identical whether or not a release result is also supplied, and is equal to its row on `main`. | spec/functional/FR-038-approval-block.md:155 |
| FND-002 | medium | FR-038-AC-6 still says "A file that is not an approval-verify result ... exit 3". The amended Description makes a release-verify result valid input, so AC-6 now contradicts it, and a test written from AC-6 would refuse valid input. Reword it to "neither an approval-verify nor a release-verify result". | spec/functional/FR-038-approval-block.md:149, :28-34 |
| FND-003 | low | The release section says "the entry takes every member from it", then two lines later "The top-level `approver`, `meaning` and `signed_at` of a `released` result are not read". It also leaves open whether those top-level members are read for a `not_released` result. Say that the entry takes Person, Signed at, Meaning and Record only from the nested `approval`, whatever the status. | spec/functional/FR-038-approval-block.md:65, :69 |

## Examined scope

```yaml
scope:
- id: FR-038-AC-13
  path: spec/functional/FR-038-approval-block.md
  role: examined
  excerpt: |-
    A standalone approval-verify fixture exports to an `in/approval.md` byte-identical to the one FR-038-AC-1 and FR-038-AC-2 specify, whether or not a release-verify result is also supplied.
- id: FR-038-AC-6
  path: spec/functional/FR-038-approval-block.md
  role: examined
  excerpt: |-
    A file that is not an approval-verify result, a file lacking `status`, and `--approval` with a JSON input each exit 3 naming the file or flag, with no converter started.
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
- id: FR-038-AC-10
  path: spec/functional/FR-038-approval-block.md
  role: examined
  excerpt: |-
    A static `released` release-verify result whose nested `approval` is `approved` with approver `Ada Example`, `signed_at` `2026-01-02T03:04:05Z` and meaning `reviewed and approved` exports successfully, and the entry carries that approver, `2026-01-02 03:04:05 UTC`, that meaning and the verdict `approved`, equal to the entry of the nested verdict exported as a standalone file.
- id: FR-038-AC-14
  path: spec/functional/FR-038-approval-block.md
  role: examined
  excerpt: |-
    A release result with `status` `unknown`, a `released` result without `approval`, a `released` result whose nested status is `wrong_record`, a `not_released` result without `reasons`, and a nested `approval` of another protocol each exit 3 naming the file, with no converter started.
- id: FR-038-AC-15
  path: spec/functional/FR-038-approval-block.md
  role: examined
  excerpt: |-
    A release result whose reason holds `*x* \`#pagebreak()\`{=typst}` and a pipe, and whose nested meaning holds the same, shows both as literal text in PDF and DOCX, adds no table row and applies no emphasis.
- id: FR-038-AC-1
  path: spec/functional/FR-038-approval-block.md
  role: context_only
  excerpt: |-
    Exporting a fixture with an `approved` verdict whose approver is `Ada Example`, whose `signed_at` is `2026-01-02T03:04:05Z` and whose meaning is `reviewed and approved` yields extracted text holding the person, `2026-01-02 03:04:05 UTC`, the meaning and the verdict `approved`, in a block under `Approval` after the document, with the column title naming the signer's claim.
```

## Verdict

Changes required. The new ACs number contiguously (AC-10 to AC-16) with no collisions, the relationships and Dependencies edits match each other, and AC-10, AC-14 and AC-15 are consistent with the statements they test. AC-13 is unsatisfiable (FND-001) and AC-6 is stale (FND-002).

## Dispositions

Round 1, fix commit 9703362.

| FND | outcome | reason |
| --- | --- | --- |
| FND-001 | fixed | FR-038-AC-13 no longer compares against a reference file. It compares the row of A across three exports (A alone, R before A, R after A), requires that row to be byte-identical in all three, requires exactly one extra row for R, and requires the row of A to carry the AC-1 values. |
| FND-002 | fixed | FR-038-AC-6 now reads "A file that is neither an approval-verify nor a release-verify result ...". |
| FND-003 | fixed | The release section now says the command "SHALL read only the members `protocol`, `status`, `reasons` and `approval`" and "SHALL NOT read the top-level `approver`, `meaning`, `signed_at`, `release`, `baseline` or `accepted_anomalies` of a release result, whatever its `status`". It adds "nothing outside [the nested `approval` object] is a source of Person, Signed at, Meaning or Record". |
