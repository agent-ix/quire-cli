---
id: SR-20261115194623
title: "PLAT-1761 quire-cli PR206 spec-review/evidence-analysis"
type: SpecReview
analysis: evidence
scope: "agent-ix/quire-cli PR206 head 42c3183; spec/functional/FR-038-approval-block.md (FR-038-AC-10..AC-16)"
review_set: subset
---

## Summary

Ticket: PLAT-1761. Checks the verification method and the oracle strength of the new ACs. `quoin advise` was denied, so the declared method catalog was not queried. The repository's established method for FR-038 is Test, and every new AC is marked Test, which is the right method for each.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | medium | FR-038-AC-11's second sentence covers the riskiest case, a `not_released` result whose nested approval is `approved`, but asserts only "not labelled an approval". It does not assert the Verdict text, the `claimed:` Meaning prefix or the Signed at cell, which are the parts that can read as an approval (see SR-20261115194616 FND-001 and FND-002). Give it explicit expected Verdict, Meaning and Signed at values. | spec/functional/FR-038-approval-block.md:153 |
| FND-002 | medium | FR-038-AC-12 asserts only that `record_invalid` appears in the Verdict cell. The Verdict row requires `unavailable`, then `not_released`, then the reasons, so a test written from AC-12 passes when `not_released` or the absent-approval marker is missing. State the full expected Verdict text. | spec/functional/FR-038-approval-block.md:154, :70 |
| FND-003 | low | FR-038-AC-16's "makes no signature check" cannot be observed as worded. Follow the FR-040-AC-6 pattern and name what the test observes: no `git`, `engineering-assurance`, `gpg` or `ssh-keygen` process is started. | spec/functional/FR-038-approval-block.md:158 |
| FND-004 | low | FR-038-AC-14 does not cover the refusal case "`reasons` ... not a list of strings" (for example `reasons: [1]` or `reasons: "x"`), which the refusal bullet states. | spec/functional/FR-038-approval-block.md:156, :87-88 |

## Examined scope

```yaml
scope:
- id: FR-038-AC-10
  path: spec/functional/FR-038-approval-block.md
  role: examined
  excerpt: |-
    A static `released` release-verify result whose nested `approval` is `approved` with approver `Ada Example`, `signed_at` `2026-01-02T03:04:05Z` and meaning `reviewed and approved` exports successfully, and the entry carries that approver, `2026-01-02 03:04:05 UTC`, that meaning and the verdict `approved`, equal to the entry of the nested verdict exported as a standalone file.
- id: FR-038-AC-11
  path: spec/functional/FR-038-approval-block.md
  role: examined
  excerpt: |-
    A static `not_released` result with a nested `untrusted_signer` approval and reasons `decision_owner_mismatch` and `release_commit_record_not_added` shows the nested status, `not_released` and both reasons in order in the Verdict cell, the meaning prefixed `claimed:`, and is not labelled an approval. A `not_released` result whose nested approval is `approved` is likewise not labelled an approval.
- id: FR-038-AC-12
  path: spec/functional/FR-038-approval-block.md
  role: examined
  excerpt: |-
    A static `not_released` result with reasons `record_invalid` and no `approval` member shows Person and Signed at `not available`, Meaning `not recorded`, Record `not available` and the reason `record_invalid` in the Verdict cell, and holds no signer, meaning or time.
- id: FR-038-AC-13
  path: spec/functional/FR-038-approval-block.md
  role: examined
  excerpt: |-
    A standalone approval-verify fixture exports to an `in/approval.md` byte-identical to the one FR-038-AC-1 and FR-038-AC-2 specify, whether or not a release-verify result is also supplied.
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
- id: FR-038-AC-16
  path: spec/functional/FR-038-approval-block.md
  role: examined
  excerpt: |-
    The release-verify fixtures are static documents; the export of one starts no git process, reads no repository and makes no signature check.
- id: FR-040-AC-6
  path: spec/functional/FR-040-export-template-and-identity-errors.md
  role: context_only
  excerpt: |-
    In every case above no converter or `engineering-assurance` process is started, `--output` is absent, and a pre-existing `--output` is unchanged.
```

## Verdict

Minor changes. Test is the right method for every new AC, and AC-10 and AC-15 have strong oracles: equality with the standalone entry, and literal-text checks in both formats. The AC-13 oracle problem is recorded in SR-20261115194620 FND-001. FND-001 and FND-002 here are weak oracles for the `not_released` paths.

## Dispositions

Round 1, fix commit 9703362.

| FND | outcome | reason |
| --- | --- | --- |
| FND-001 | fixed | The nested-`approved` `not_released` case moved from AC-11's second sentence into the new FR-038-AC-17. AC-17 states the exact Verdict cell, Person, Signed at and Record `not available`, the `claimed: ` Meaning, and the absence of the signer and the time from the block. |
| FND-002 | fixed | FR-038-AC-12 now asserts "the Verdict cell exactly `not_released; reasons: ["record_invalid"]`" and every other cell value. |
| FND-003 | fixed | FR-038-AC-16 now names what the test observes: stub `engineering-assurance`, `gpg` and `ssh-keygen` executables first on `PATH` record no call, and the `git` calls equal those of a standalone-verdict export and name no `--approval` file or tag. |
| FND-004 | fixed | FR-038-AC-14 now lists "a `not_released` result without `reasons`, one with `reasons` `[1]` and one with `reasons` `"x"`". |
