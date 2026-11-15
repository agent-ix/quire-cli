---
id: SR-20261115194625
title: "PLAT-1761 quire-cli PR208 code-review (with rust-review lane)"
type: SpecReview
analysis: code-review
scope: "agent-ix/quire-cli PR208 head cb23cf4; src/commands/export/approval.rs, tests/cli_export.rs"
review_set: subset
---

## Summary

Ticket: PLAT-1761. Code review, with the rust-review lane folded in, of the PR208 diff (`origin/main...cb23cf4`). The diff adds the `engineering-assurance.release-verify-result/v1` branch to `quire export --approval` (FR-038 Release-verify input, FR-040 Release result invalid). On the PR head, 4 of the 8 new tests fail (`cargo test --locked --test cli_export -- release`: 5 passed, 4 failed). `cargo fmt --check` fails on both changed files, and the build emits a `dead_code` warning that `make lint` (`-D warnings`) rejects. A `released` result whose nested verdict is not `approved` is accepted and shown with the Verdict `approved`.

## Verdict

**FAIL**. There are six high findings: a non-approved nested verdict is labelled an approval; FR-040 invalid inputs are silently accepted; tagged tests fail on the head; and the release row duplicates the standalone row logic. The parts that work: order preservation (AC-13/AC-18 test passes), the `released` approval row matching the standalone row (AC-10 passes), the `not_released` cells taking Person, Signed at and Record as `not available` (AC-12, AC-17 code paths), top-level members not read (only `protocol`, `status`, `reasons` and `approval` are deserialized), and no verification tools started (AC-16 test passes). The review found no vendoring and no ceremony.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | high | A `released` result whose nested `approval.status` is not `approved` (for example `wrong_record`) is accepted, not refused, and its Verdict cell is the hard-coded text `approved`, so a failed approval is labelled an approval. Reproduced: the `released-wrong-nested` case of `invalid_release_results_exit_3_before_converter` exits 0. | src/commands/export/approval.rs:105-114, src/commands/export/approval.rs:313-315 |
| FND-002 | high | A nested verdict that does not deserialize (a member of the wrong JSON type, for example `"meaning": "x"` or `"approver": 1`) is discarded by `.ok()` and treated as absent, when FR-040 Approval invalid requires exit 3. With `released`, the entry then shows the Verdict `approved` with Person, Signed at and Record `not available`. | src/commands/export/approval.rs:162-164 |
| FND-003 | high | A nested `approved` verdict whose `signed_at` is not a valid UTC instant shows `not available`. FR-040 Approval invalid requires exit 3 for this "(standalone or nested in a release result)", and the standalone path does refuse it (approval.rs:196-202), so the two rows diverge. | src/commands/export/approval.rs:260-264 |
| FND-004 | high | Three new tests fail on the head because they expect raw `;` and `:` and no U+200B in the escaped Markdown cell, while `escape_markdown` escapes `;` and `:` and `breakable` inserts U+200B after 16 characters. The failing tests are `a_not_released_result_shows_reasons_and_nested_verdict` (AC-11), `a_not_released_result_without_approval_shows_not_available` (AC-12) and `not_released_with_approved_nested_shows_claimed_meaning_and_not_available` (AC-17). | tests/cli_export.rs:4593, tests/cli_export.rs:4678, tests/cli_export.rs:4967 |
| FND-005 | high | `row_release` reimplements the Person, Signed at, Meaning and Record rules of `row_approval` (approver/principal/unmapped fallback, meaning/signed_off_meaning fallback, record format). There are now two implementations of one rule, and FND-003 is where they have already diverged. The `released`+`approved` row should come from `row_approval` on the nested verdict, with only the `not_released` overrides applied on top. | src/commands/export/approval.rs:244-310, src/commands/export/approval.rs:186-231 |
| FND-006 | medium | `make fmt-check` fails: `cargo fmt --check` reports drift in approval.rs (validate_release, parse, row_release) and throughout the new tests in cli_export.rs. | src/commands/export/approval.rs:101-172, tests/cli_export.rs:4637-4980 |
| FND-007 | medium | `ReleaseResult::protocol` is never read, so the build warns `field protocol is never read` (dead_code), which `make lint` with `-D warnings` rejects. | src/commands/export/approval.rs:66 |
| FND-008 | medium | A `released` result whose `reasons` is a non-empty non-array (for example `"x"` or `{"a":1}`) is accepted, because only `Value::Array` is checked; FR-040 refuses a `released` result with non-empty `reasons`. | src/commands/export/approval.rs:109-113 |
| FND-009 | low | For `released`, the Verdict cell is the constant `approved` rather than "the nested verdict text formed as for a standalone verdict", so a nested `reason` on an approved verdict is dropped (the standalone row shows `approved (reason)`). | src/commands/export/approval.rs:313-315 |
| FND-010 | low | `row_release` takes an unused `_path` and returns an `anyhow::Result` that never errors, and `parse` clones the whole document (`value.clone()`) and the nested object (`approval.clone()`) only to deserialize them; `Verdict::deserialize(&Value)` / `ReleaseResult::deserialize(&value)` borrow instead. | src/commands/export/approval.rs:158, src/commands/export/approval.rs:162-164, src/commands/export/approval.rs:236-240 |

## Examined scope

```yaml
scope:
- id: FR-038-AC-10
  path: spec/functional/FR-038-approval-block.md
  role: examined
  excerpt: "A static `released` release-verify result whose nested `approval` is `approved` with approver `Ada Example`, `signed_at` `2026-01-02T03:04:05Z` and meaning `reviewed and approved` exports successfully; its entry is labelled an approval, its Verdict cell is exactly `approved`, and its Person, Signed at, Meaning and Record cells are byte-identical to the cells of the row of the nested verdict exported as a standalone file."
- id: FR-038-AC-11
  path: spec/functional/FR-038-approval-block.md
  role: examined
  excerpt: "A static `not_released` result with reasons `untrusted_signer` and `release_commit_record_not_added` and a nested `untrusted_signer` approval (meaning `reviewed and approved`, no `reason`) has the Verdict cell exactly `not_released; reasons: [\"untrusted_signer\", \"release_commit_record_not_added\"]; nested: untrusted_signer`, the Meaning `claimed: ` followed by the meaning, Signed at `not available`, and is not labelled an approval."
- id: FR-038-AC-12
  path: spec/functional/FR-038-approval-block.md
  role: examined
  excerpt: "A static `not_released` result with reasons `record_invalid` and no `approval` member has the Verdict cell exactly `not_released; reasons: [\"record_invalid\"]`, Person `not available`, Signed at `not available`, Meaning `not recorded`, Record `not available`, holds no signer, meaning or time, and is not labelled an approval."
- id: FR-038-AC-13
  path: spec/functional/FR-038-approval-block.md
  role: examined
  excerpt: "The `in/approval.md` of an export of a standalone approval-verify fixture A alone, of A with a release result R before it and of A with R after it holds the row of A byte-identical in all three; the exports with R hold exactly one more row than the export without it, the row of R; and the row of A carries the values FR-038-AC-1 specifies."
- id: FR-038-AC-14
  path: spec/functional/FR-038-approval-block.md
  role: examined
  excerpt: "Each of these exits 3 naming the file, with no converter started: a release result with `status` `unknown`; a release result with no `status`; a `released` result without `approval`; a `released` result whose nested status is `wrong_record`; a `released` result with `reasons` `[\"x\"]`; a `not_released` result without `reasons`, one with `reasons` `[1]` and one with `reasons` `\"x\"`; a release result whose `approval` is a string; a release result whose `approval` has another `protocol`."
- id: FR-038-AC-16
  path: spec/functional/FR-038-approval-block.md
  role: examined
  excerpt: "The export of a release result, run with stub `engineering-assurance`, `gpg` and `ssh-keygen` executables first on `PATH` that record each call, calls none of them, and its `git` calls are the same as for the same export with a standalone verdict in its place and none names an `--approval` file or a tag."
- id: FR-038-AC-17
  path: spec/functional/FR-038-approval-block.md
  role: examined
  excerpt: "A static `not_released` result with reasons `decision_owner_mismatch` and a nested `approved` approval (approver `Ada Example`, `signed_at` `2026-01-02T03:04:05Z`, meaning `reviewed and approved`, record fields present) has the Verdict cell exactly `not_released; reasons: [\"decision_owner_mismatch\"]`, Person, Signed at and Record `not available`, the Meaning `claimed: ` followed by the meaning, no heading or label reading as an approval, and neither `Ada Example` nor `2026-01-02 03:04:05 UTC` anywhere in the block."
- id: FR-038-AC-18
  path: spec/functional/FR-038-approval-block.md
  role: examined
  excerpt: "An export with two `--approval` arguments, a standalone verdict and a release result, lists the two entries in command-line order, whichever argument comes first."
- id: FR-040-AC-8
  path: spec/functional/FR-040-export-template-and-identity-errors.md
  role: examined
  excerpt: "Each cause of the Release result invalid row exits 3 naming the file: a release result with `status` `unknown`; a `released` result without `approval`; a `released` result whose `approval.status` is `wrong_record`; a `released` result with `reasons` `[\"x\"]`; a `not_released` result without `reasons`, with `reasons` `[1]` and with `reasons` `\"x\"`; a release result whose `approval` is a string, one whose `approval` has another `protocol`, and one whose `approval` lacks `status`. A `released` result with `reasons` `[]` and a `not_released` result with a nested `approved` approval are not failures."
- id: FR-040-approval-invalid
  path: spec/functional/FR-040-export-template-and-identity-errors.md
  role: examined
  excerpt: "An `--approval` file is missing, not JSON, neither an `engineering-assurance.approval-verify-result/v1` nor an `engineering-assurance.release-verify-result/v1` document, lacks `status`, has a member of the wrong JSON type, is above 16 MiB, is not a regular file or not UTF-8, or is an `approved` verdict (standalone or nested in a release result) whose `signed_at` is not a valid UTC instant `YYYY-MM-DDTHH:MM:SSZ`."
- id: src/commands/export/approval.rs
  path: src/commands/export/approval.rs
  role: examined
  excerpt: "fn validate_release(result: &ReleaseResult, path: &Path) -> anyhow::Result<()>"
- id: tests/cli_export.rs
  path: tests/cli_export.rs
  role: examined
  excerpt: "// ---- Release-verify results (FR-038, FR-040) --------------------------------"
```

## Gates

- `cargo fmt --check -- src/commands/export/approval.rs tests/cli_export.rs`: exit 1 (FND-006).
- `cargo test --locked --test cli_export -- release`: 5 passed, 4 failed (FND-001, FND-004); the build warned `field protocol is never read` (FND-007).
- Clippy, `cargo deny` and the full suite were not run: the brief reserves aggregate gates for the lead's single post-disposition run.

## New findings (disposition pass 1)

Fix round 1 at head 76b7451, diff `cb23cf4..76b7451`.

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-011 | high | The FND-008 fix does not compile: `None \| Some(Value::Array(arr)) if arr.is_empty()` binds `arr` in only one alternative (rustc E0408, E0381), so the `quire` binary and every test fail to build on the head. | src/commands/export/approval.rs:110 |
| FND-012 | high | `release_result_calls_no_verification_tools` calls `which::which("git")`, but `which` is not a dependency in Cargo.toml, so the `cli_export` test target fails to compile (rustc E0433). | tests/cli_export.rs:5112 |
| FND-013 | low | `row_release` passes `Path::new("")` to `row_approval` and its comment says the path is unused for an approved verdict. That is false: `row_approval` puts the path in its `signed_at` error for an approved verdict. The error cannot happen today because `parse` checks `signed_at` first, but if it did it would name `''`. `row` already holds the real path. | src/commands/export/approval.rs:274-275 |
| FND-014 | medium | Fix commit 76b7451 commits `.review-scratch/plat1761-pr208/` to the PR branch: two draft comment bodies plus copies of both SR files from before the dispositions. The review custody rule commits only the latest SR file, at `reviews/<slug>.md`. These copies are already stale and would land in a public repo's history. | .review-scratch/plat1761-pr208/comment-code-review.md, .review-scratch/plat1761-pr208/reviews/plat-1761-code-review.md |

## New findings (disposition pass 3)

Fix round 3 was inspected at head `1be9f39d`; these test hygiene findings were
fixed in the same round before the final gate.

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-015 | medium | `cargo fmt --check -- tests/cli_export.rs` requests multiline formatting for the `md_result` assertion and `page_break_count` assertion. | tests/cli_export.rs:5226, 5258 |
| FND-016 | medium | `md_command` is moved into `with_approvals` and does not need `mut`; the unused-mut warning fails the repository's `-D warnings` lint. | tests/cli_export.rs:5225-5227 |

## Dispositions

| FND | outcome | reason |
| --- | --- | --- |
| FND-001 | fixed | `validate_release` now refuses a `released` result whose nested `status` is not `approved` (approval.rs:143-150), and the released row comes from `row_approval`. |
| FND-002 | fixed | The nested verdict is deserialized with `map_err` to exit 3 ("unusable nested approval"), so it is no longer dropped by `.ok()` (approval.rs:177-181). |
| FND-003 | fixed | `parse` refuses a nested `approved` verdict whose `signed_at` fails `format_instant` (approval.rs:182-191), the same rule as the standalone row. |
| FND-004 | still-open | The three tests now expect escaped `\;` / `\:` and strip U+200B (cli_export.rs:4591, 4676, 5215), but the head does not compile (FND-011, FND-012), so nobody can show they pass. Re-check once the head builds. |
| FND-005 | still-open | Fixed for `released`: Person, Signed at, Record and Verdict now come from `row_approval` (approval.rs:272-277). The not_released branch still copies `row_approval`'s meaning fallback (`meaning` or else `signed_off_meaning.values`, filter non-empty) at approval.rs:287-298 instead of taking the Meaning from `row_approval` and adding the `claimed:` override, so one rule still has two implementations. |
| FND-006 | fixed | `cargo fmt --check -- src/commands/export/approval.rs tests/cli_export.rs` exits 0 on the head. |
| FND-007 | fixed | The unread `protocol` field was removed from `ReleaseResult` (approval.rs:64-69), which was the cause of the dead_code warning. |
| FND-008 | still-open | The attempted fix at approval.rs:109-116 does not compile (FND-011), so the head still accepts no input at all, and nothing shows that a `released` result with `reasons` `"x"` or `{"a":1}` is refused. |
| FND-009 | fixed | A `released` row's Verdict now comes from `row_approval` (approval.rs:275), which gives `approved (reason)` when the nested verdict has a reason. |
| FND-010 | still-open | Partly fixed: `_path` is gone, `value.clone()` is gone, and the `Result` is now justified because `row_approval` can fail. The nested object is still cloned only to deserialize it, `serde_json::from_value::<Verdict>(approval.clone())` (approval.rs:179), where `Verdict::deserialize(approval)` would borrow it. |
| FND-004 | fixed | Round 2 (head c8ca227, diff `76b7451..c8ca227`): the head now compiles (FND-011, FND-012 fixed). The three tests strip U+200B and expect the escaped cells `escape_markdown` produces, for example `not\_released\; reasons\: \[\"record\_invalid\"\]` (cli_export.rs:4676-4682). Judged by reading the code; no test was run in this pass. |
| FND-005 | fixed | Round 2: the meaning fallback is now one helper, `extract_meaning` (approval.rs:220-232), used by both `row_approval` and the not_released branch of `row_release`. |
| FND-008 | fixed | Round 2: the match is split into `None => {}` and `Some(Value::Array(arr)) if arr.is_empty() => {}` (approval.rs:109-117), so a `released` result whose `reasons` is `"x"` or `{"a":1}` reaches `_` and is refused. |
| FND-010 | fixed | Round 2: the nested verdict is read with `Verdict::deserialize(approval)` and borrowed, not cloned (approval.rs:179). |
| FND-011 | fixed | Round 2: the or-pattern binding only one alternative is split into two arms (approval.rs:110-111), so E0408/E0381 no longer apply. |
| FND-012 | fixed | Round 2: `which::which` is removed. The git wrapper now finds the real git by scanning `PATH` and skipping the stub dir (cli_export.rs:5042-5061). |
| FND-013 | fixed | Round 2: `row_release` takes the real `path` and passes it to `row_approval` (approval.rs:274-284), so an error would name the file. |
| FND-014 | fixed | The current PR history lists only the implementation commit and the amended round-3 commit; neither contains `.review-scratch/`, and the PR file list contains only `src/commands/export/approval.rs` and `tests/cli_export.rs`. |
| FND-015 | fixed | At head `889fcea`, the `md_result` and `page_break_count` assertions have rustfmt's requested layout (tests/cli_export.rs:5226-5232, 5256-5262). |
| FND-016 | fixed | At head `889fcea`, `md_command` is immutable before being moved to `with_approvals` (tests/cli_export.rs:5225-5227). |
