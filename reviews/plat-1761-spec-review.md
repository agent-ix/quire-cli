---
id: SR-20261115194618
title: "PLAT-1761 quire-cli PR206 spec-review/base"
type: SpecReview
analysis: base
scope: "agent-ix/quire-cli PR206 head 42c3183; spec/functional/FR-038-approval-block.md, FR-040-export-template-and-identity-errors.md"
review_set: subset
---

## Summary

Ticket: PLAT-1761. Base checklist over the FR-038 change, read against its downstream FR-040. The scope-matched sub-analyses are EARS, integrity, object, dependency, evidence and risk-complexity, each in its own artifact. `quire validate` was denied by the permission layer and was not run. FR-038's statements about EA FR-047 were checked against the EA FR-047 text; the one mismatch is recorded in SR-20261115194624 FND-001.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | high | FR-038 defers its refusals to FR-040, but FR-040 was not updated. The "Approval invalid" row still names "not an `engineering-assurance.approval-verify-result/v1` document" as a cause, so every release-verify result FR-038 now accepts is an Approval invalid failure (exit 3) under FR-040. The new refusal causes (unknown release `status`, `released` without `approval`, `released` with a non-approved nested status, missing or non-string `reasons`, a nested `approval` of another protocol) have no row, and FR-040-AC-5 ("each approval failure of the table") cannot cover them. Amend the FR-040 row in this PR. | spec/functional/FR-040-export-template-and-identity-errors.md:39, :59, spec/functional/FR-038-approval-block.md:85-90 |
| FND-002 | low | The new AC rows FR-038-AC-10 to AC-16 are placed before FR-038-AC-8, so the table reads AC-7, AC-9, AC-10 ... AC-16, AC-8. AC-9 was already out of order. Order the rows by number. | spec/functional/FR-038-approval-block.md:150-159 |

## Examined scope

```yaml
scope:
- id: FR-038
  path: spec/functional/FR-038-approval-block.md
  role: examined
  excerpt: |-
    `quire export` SHALL accept `--approval <PATH>`, repeatable, each a verdict file of `engineering-assurance approval-verify` ([EA FR-041](ix://agent-ix/engineering-assurance/FR-041)) or a result file of `engineering-assurance release-verify` ([EA FR-047](ix://agent-ix/engineering-assurance/FR-047)), and SHALL generate an approval block (`in/approval.md`, [FR-035](./FR-035-default-export-template.md)) with one entry per file.
- id: FR-038-refuse
  path: spec/functional/FR-038-approval-block.md
  role: examined
  excerpt: |-
    The command SHALL refuse a release result whose `status` is neither `released` nor `not_released`, a `released` result without a nested `approval`, a `released` result whose nested `approval.status` is not `approved`, a `not_released` result whose `reasons` is absent or not a list of strings, and an `approval` member that is present but is not an `engineering-assurance.approval-verify-result/v1` document ([FR-040](./FR-040-export-template-and-identity-errors.md)).
- id: FR-038-data
  path: spec/functional/FR-038-approval-block.md
  role: examined
  excerpt: |-
    The command SHALL read a verdict file as data and never execute it. A file that is neither an `engineering-assurance.approval-verify-result/v1` nor an `engineering-assurance.release-verify-result/v1` document, or that lacks `status`, is refused before any converter starts, as is a file above 16 MiB, one that is not a regular file, and an `approved` verdict whose `signed_at` is not a valid UTC instant ([FR-040](./FR-040-export-template-and-identity-errors.md)).
- id: FR-038-standalone
  path: spec/functional/FR-038-approval-block.md
  role: examined
  excerpt: |-
    A standalone `approval-verify` verdict file SHALL produce the entry this requirement specifies for it, unchanged by the release-verify input.
- id: FR-040-approval-invalid
  path: spec/functional/FR-040-export-template-and-identity-errors.md
  role: examined
  excerpt: |-
    | Approval invalid | An `--approval` file is missing, not JSON, not an `engineering-assurance.approval-verify-result/v1` document, lacks `status`, has a member of the wrong JSON type, is above 16 MiB, is not a regular file or not UTF-8, or is an `approved` verdict whose `signed_at` is not a valid UTC instant `YYYY-MM-DDTHH:MM:SSZ`. | 3 | Names the file and the first cause. |
- id: FR-040-AC-5
  path: spec/functional/FR-040-export-template-and-identity-errors.md
  role: examined
  excerpt: |-
    Each approval failure of the table other than an unreadable file exits 3 naming the file, an unreadable approval file exits 2, and an `unverifiable` verdict for an unresolved tag is not a failure; the export names the first failing file when two fail.
```

## Verdict

Changes required. The change is spec-only, fits the PR body's stated scope and keeps the FR-031 presentation stance. Public-repo hygiene is clean: no roadmap or delivery-order content, no pins or digests, and no compatibility layer (the standalone path is unchanged, not shimmed). Blocking: FR-040 still makes the new input an error (FND-001), and the code-review artifact SR-20261115194616 records the Verdict-cell contradiction (its FND-001). Sub-analysis artifacts: SR-20261115194619 (EARS), SR-20261115194620 (integrity), SR-20261115194621 (object), SR-20261115194622 (dependency), SR-20261115194623 (evidence), SR-20261115194624 (risk-complexity).

## Dispositions

Round 1, fix commit 9703362.

| FND | outcome | reason |
| --- | --- | --- |
| FND-001 | fixed | FR-040's Approval invalid row now names "neither an `engineering-assurance.approval-verify-result/v1` nor an `engineering-assurance.release-verify-result/v1` document". A new Release result invalid row lists every release refusal cause (exit 3), and the new FR-040-AC-8 covers each cause along with two cases that are not failures. FR-040-AC-6 and AC-7 now say "every failure of the table". |
| FND-002 | fixed | The AC rows now run FR-038-AC-1 to AC-18 in numeric order (AC-8 and AC-9 have moved before AC-10). The duplicated AC-17 and AC-18 rows that follow are recorded as FND-003. |

## New findings (disposition pass 1)

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-003 | medium | Fix round 1 added the FR-038-AC-17 and FR-038-AC-18 rows twice. The Acceptance Criteria table lists AC-17 and AC-18 at lines 183-184 and again, with identical text, at lines 185-186. Two rows carry the same criterion id, so the computed Test Matrix and any id-keyed tooling see a duplicate AC. The table also no longer ends in numeric order. Delete lines 185-186. | spec/functional/FR-038-approval-block.md:183-186 |

Round 2, fix commit 2b5f321.

| FND | outcome | reason |
| --- | --- | --- |
| FND-003 | fixed | The second copies of FR-038-AC-17 and AC-18 were removed. The Acceptance Criteria table now lists the ids once each in numeric order, ending with AC-17 and AC-18. |
