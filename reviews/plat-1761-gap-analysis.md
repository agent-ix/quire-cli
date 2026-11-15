---
id: SR-20261115194617
title: "PLAT-1761 quire-cli PR206 gap-analysis"
type: SpecReview
analysis: gap-analysis
scope: "agent-ix/quire-cli PR206 head 42c3183; spec/functional/FR-038-approval-block.md; src/commands/export/approval.rs; tests/cli_export.rs"
review_set: subset
---

## Summary

Ticket: PLAT-1761. A planless gap pass. Plan completion: not assessed. `quoin matrix` and `quire matrix` were denied by the permission layer, so the AC-to-test check was done by hand: a grep of `src/` and `tests/` for the new AC ids, `release-verify`, `release_verify` and `not_released`. The grep returned no matches.

This PR is spec-only by design (the PR body says so). The finding below records the resulting spec-to-code gap so it is tracked. It does not object to splitting the spec from the implementation.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | high | FR-038 now requires `--approval` to accept an `engineering-assurance.release-verify-result/v1` file. approval.rs refuses every protocol other than `engineering-assurance.approval-verify-result/v1` with exit 3, so the code on this branch breaks the amended Description. FR-038-AC-10 to AC-16 have no implementation and no `#[trace]`-tagged test. FR-038 must not be treated as met until the PLAT-1761 implementation lands with tests for AC-10 to AC-16. | spec/functional/FR-038-approval-block.md:28-34, :152-158, src/commands/export/approval.rs:13, :81-83 |

## Examined scope

```yaml
scope:
- id: FR-038
  path: spec/functional/FR-038-approval-block.md
  role: examined
  excerpt: |-
    `quire export` SHALL accept `--approval <PATH>`, repeatable, each a verdict file of `engineering-assurance approval-verify` ([EA FR-041](ix://agent-ix/engineering-assurance/FR-041)) or a result file of `engineering-assurance release-verify` ([EA FR-047](ix://agent-ix/engineering-assurance/FR-047)), and SHALL generate an approval block (`in/approval.md`, [FR-035](./FR-035-default-export-template.md)) with one entry per file.
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
- id: src/commands/export/approval.rs
  path: src/commands/export/approval.rs
  role: examined
  excerpt: |-
    if value.get("protocol").and_then(Value::as_str) != Some(RESULT_PROTOCOL) {
        invalid_request!("--approval '{name}': not an {RESULT_PROTOCOL} document");
    }
- id: FR-038-AC-6
  path: spec/functional/FR-038-approval-block.md
  role: context_only
  excerpt: |-
    A file that is not an approval-verify result, a file lacking `status`, and `--approval` with a JSON input each exit 3 naming the file or flag, with no converter started.
```

## Verdict

Expected gap for a spec-first PR: AC-10 to AC-16 are new and have no code and no tests. FR-038-AC-1 to AC-9 keep their existing tagged tests (for example `an_unusable_approval_file_exits_3_naming_it_before_any_converter` at tests/cli_export.rs:4271 for AC-6). None of the new text changes the standalone path that those tests cover. No stubs or coverage inflation were introduced, because no code changed.

## Dispositions

Round 1, fix commit 9703362.

| FND | outcome | reason |
| --- | --- | --- |
| FND-001 | deferred | PR #206 is spec-only by design. The implementation portion of the same ticket, PLAT-1761, adds release-result parsing to src/commands/export/approval.rs and `#[trace]`-tagged tests for FR-038-AC-10 to AC-18 and FR-040-AC-8 (round 1 added AC-17, AC-18 and FR-040-AC-8 to the scope). The coder has already recorded that scope on the ticket. FR-038 is not met until that implementation lands. |
