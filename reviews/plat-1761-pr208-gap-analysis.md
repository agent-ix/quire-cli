---
id: SR-20261115194626
title: "PLAT-1761 quire-cli PR208 gap-analysis"
type: SpecReview
analysis: gap-analysis
scope: "agent-ix/quire-cli PR208 head cb23cf4; spec/functional/FR-038-approval-block.md (FR-038-AC-10..AC-18), spec/functional/FR-040-export-template-and-identity-errors.md (FR-040-AC-5, FR-040-AC-8); src/commands/export/approval.rs; tests/cli_export.rs"
review_set: subset
relationships:
  - target: "ix://agent-ix/quire-cli/spec/functional/FR-038"
    type: "references"
  - target: "ix://agent-ix/quire-cli/spec/functional/FR-040"
    type: "references"
---

## Summary

Ticket: PLAT-1761. This is a planless gap analysis of the PR208 change against the merged FR-038 and FR-040 criteria. Every criterion in scope carries a `#[trace]` tag on at least one non-ignored test. Several of those tests do not assert what their criterion states, and the FR-040 Approval invalid causes for nested verdicts have no test. Test failures and code defects are recorded in the code-review artifact (SR-20261115194625) and are not repeated here.

## Verdict

**CONDITIONAL**. No criterion is untagged. The evidence is hollow for FR-038-AC-13, AC-15, AC-16 and AC-17, and the new nested causes of FR-040 Approval invalid are not tested. The tagged evidence that holds: AC-10 (byte-identity with the standalone row), AC-14/FR-040-AC-8 (except the case recorded under code-review FND-001) and AC-18. No unspecified behavior was added, and nothing is a stub.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | medium | FR-038-AC-15 requires literal text "in PDF and DOCX", but `a_not_released_result_escapes_markup_in_reasons_and_nested_reason` checks only the stub converter's `approval.md` input. No real-converter test exports a release result (the ignored `real_converters_show_the_approval_block_after_the_document` covers standalone verdicts only). | tests/cli_export.rs:4835-4878 |
| FND-002 | medium | FR-038-AC-16 requires the `git` calls to equal those of the standalone export and that no call names an `--approval` file or a tag. `release_result_calls_no_verification_tools` records only the engineering-assurance, gpg and ssh-keygen stubs, and never records or compares `git` calls. | tests/cli_export.rs:4880-4940 |
| FND-003 | medium | FR-038-AC-17's assertion `!block.contains("2026-01-02 03:04:05 UTC")` cannot fail: every cell is escaped, so a leaked time would appear as `2026\-01\-02 03\:04\:05 UTC` and the raw string never matches. | tests/cli_export.rs:4979 |
| FND-004 | medium | FR-038-AC-13's test never exports A alone, so it cannot check that the row of A is byte-identical in all three exports or that the exports with R hold exactly one more row. It checks only the Person cell of A, not the AC-1 values (time, meaning, verdict). | tests/cli_export.rs:4697-4745 |
| FND-005 | medium | No test covers the two causes FR-040 Approval invalid adds for release results, both counted by FR-040-AC-5 ("each approval failure of the table"): a nested `approved` verdict with an invalid `signed_at`, and a nested verdict with a member of the wrong JSON type. Both are currently accepted (code-review FND-002, FND-003). | spec/functional/FR-040-export-template-and-identity-errors.md:41, :62; tests/cli_export.rs:4747-4833 |

## Examined scope

```yaml
scope:
- id: FR-038-AC-13
  path: spec/functional/FR-038-approval-block.md
  role: examined
  excerpt: "The `in/approval.md` of an export of a standalone approval-verify fixture A alone, of A with a release result R before it and of A with R after it holds the row of A byte-identical in all three; the exports with R hold exactly one more row than the export without it, the row of R; and the row of A carries the values FR-038-AC-1 specifies."
- id: FR-038-AC-15
  path: spec/functional/FR-038-approval-block.md
  role: examined
  excerpt: "A `not_released` release result whose reason holds `*x* \\`#pagebreak()\\`{=typst}` and a pipe, and whose nested non-approved verdict has a meaning and a `reason` holding the same, shows each as literal text in PDF and DOCX, adds no table row and applies no emphasis."
- id: FR-038-AC-16
  path: spec/functional/FR-038-approval-block.md
  role: examined
  excerpt: "The export of a release result, run with stub `engineering-assurance`, `gpg` and `ssh-keygen` executables first on `PATH` that record each call, calls none of them, and its `git` calls are the same as for the same export with a standalone verdict in its place and none names an `--approval` file or a tag."
- id: FR-038-AC-17
  path: spec/functional/FR-038-approval-block.md
  role: examined
  excerpt: "... no heading or label reading as an approval, and neither `Ada Example` nor `2026-01-02 03:04:05 UTC` anywhere in the block."
- id: FR-040-AC-5
  path: spec/functional/FR-040-export-template-and-identity-errors.md
  role: examined
  excerpt: "Each approval failure of the table other than an unreadable file exits 3 naming the file, an unreadable approval file exits 2, and an `unverifiable` verdict for an unresolved tag is not a failure; the export names the first failing file when two fail."
- id: FR-038-AC-10
  path: spec/functional/FR-038-approval-block.md
  role: examined
  excerpt: "... its Person, Signed at, Meaning and Record cells are byte-identical to the cells of the row of the nested verdict exported as a standalone file."
- id: FR-038-AC-18
  path: spec/functional/FR-038-approval-block.md
  role: examined
  excerpt: "An export with two `--approval` arguments, a standalone verdict and a release result, lists the two entries in command-line order, whichever argument comes first."
- id: FR-040-AC-8
  path: spec/functional/FR-040-export-template-and-identity-errors.md
  role: examined
  excerpt: "Each cause of the Release result invalid row exits 3 naming the file ... A `released` result with `reasons` `[]` and a `not_released` result with a nested `approved` approval are not failures."
- id: FR-038-AC-11
  path: spec/functional/FR-038-approval-block.md
  role: context_only
  excerpt: "A static `not_released` result with reasons `untrusted_signer` and `release_commit_record_not_added` and a nested `untrusted_signer` approval ..."
- id: FR-038-AC-12
  path: spec/functional/FR-038-approval-block.md
  role: context_only
  excerpt: "A static `not_released` result with reasons `record_invalid` and no `approval` member ..."
- id: FR-038-AC-14
  path: spec/functional/FR-038-approval-block.md
  role: context_only
  excerpt: "Each of these exits 3 naming the file, with no converter started: ..."
```

## Coverage

- Matrix: `quire matrix` / `quoin matrix` were denied by the harness permission layer in this session. The fallback is a grep index of `#[trace(...)]` tags in `tests/cli_export.rs`, declared as such. FR-038-AC-10 through AC-18 and FR-040-AC-8 are each tagged by a non-ignored test; FR-040-AC-5 is tagged by the pre-existing approval-failure test, which has no nested cases.
- Tagged tests failing on the head: AC-11, AC-12, AC-14/FR-040-AC-8 and AC-17 (see SR-20261115194625 FND-001, FND-004).
- Reverse gap: none; every new code path is owned by FR-038 Release-verify input or FR-040 Release result invalid.
- Stubs and coverage inflation: none in source; the hollow assertions are FND-001 to FND-004 above.
- Semantic review: skipped; it is opt-in and this non-interactive dispatch has no user to ask.
- Plan completion: not assessed

## New findings (disposition pass 1)

Fix round 1 at head 76b7451, diff `cb23cf4..76b7451`.

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-006 | medium | The new ignored AC-15 test `release_results_render_literal_text_in_pdf_and_docx` can never pass. It asserts the block contains ``*x* `#pagebreak()`{=typst}`` and then that the same block does not contain `pagebreak()`. Its `<em>` check runs after `strip_tags`, so that check cannot fail. It also duplicates `release_approval_text_reaches_real_converters` (cli_export.rs:5249), which is tagged to the same criterion. | tests/cli_export.rs:5004-5067 |

## New findings (disposition pass 2)

Fix round 2 at head c8ca227, diff `76b7451..c8ca227`.

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-007 | medium | `release_approval_text_reaches_real_converters` (the AC-15 test, ignored) can never pass. It asserts the visible block contains ``*x*`#pagebreak()`{=typst}``, which includes `pagebreak()`, and then asserts the same block does not contain `pagebreak()`. This is the FND-006 contradiction moved into the remaining test. | tests/cli_export.rs:5257-5265 |
| FND-008 | medium | The same test calls `rig.converter_input("approval.md")`, which reads `log/in/approval.md`. Only the stand-in pandoc writes that file, and this test runs the real converters (`quire()` with no `--pandoc`/`--typst`). So the file is never written and `.expect("converter input")` panics before the row-count check runs. | tests/cli_export.rs:5226, tests/cli_export.rs:174-176 |
| FND-009 | medium | The added DOCX emphasis check `docx_raw.matches("<em>").count() == 0` cannot fail. `docx_text` returns `word/document.xml`, where pandoc marks emphasis as `<w:i/>` run properties and never writes `<em>`. AC-15's "applies no emphasis" is still not checked for DOCX or PDF. | tests/cli_export.rs:5236-5241, tests/cli_export.rs:1579-1589 |

## New findings (disposition pass 3)

Fix round 3 at head eccdfe1, diff `c8ca227..eccdfe1` (tests/cli_export.rs only).

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-010 | medium | The new no-page-break checks in `release_approval_text_reaches_real_converters` can never pass. `pdf_page_count(&pdf) == 1` fails because every single-document export has a title page, a contents page and a body (see `real_pdf_has_a_title_page_contents_header_footer_and_page_numbers`, which reads page 3). `!docx_raw.contains("w:type=\"page\"")` fails because `compose_front` always writes `PAGE_BREAK_OPENXML` twice into the DOCX. Neither assertion can detect a page break caused by the hostile text. | tests/cli_export.rs:5240-5249; src/commands/export/template.rs:130, :138, :144 |
| FND-011 | medium | AC-15 requires each of the reason, the nested meaning and the nested `reason` to show as literal text, with the pipe. The test checks `block.contains(expected)` and `block.contains("*x*")` once each. One literal occurrence of the three passes, and the pipe in the nested reason is never checked, so a nested field that lost its escaping or was dropped would still pass. | tests/cli_export.rs:5265-5274 |

## Dispositions

| FND | outcome | reason |
| --- | --- | --- |
| FND-001 | still-open | There are now two ignored real-converter tests. The first can never pass (FND-006). The second checks only that the hostile text is literal in PDF and DOCX, not AC-15's "adds no table row and applies no emphasis". Neither compiles on the head (SR-20261115194625 FND-012). |
| FND-002 | still-open | The test now wraps `git`, compares its calls between the release and standalone exports, and checks that no call names the file, `--approval` or `approvals/` (cli_export.rs:5111-5188). But the wrapper calls `which::which`, which does not compile (SR-20261115194625 FND-012), so this evidence does not run. |
| FND-003 | fixed | The AC-17 test now checks the escaped time and `!block.contains("2026")` together with the Person and Signed at cells, so a leaked time fails the test (cli_export.rs:5232-5244). |
| FND-004 | fixed | The AC-13 test now exports A alone, asserts one row against two, asserts that the row of A is byte-identical in all three exports, and checks that its Person, Signed at, Meaning, Verdict and Record carry the AC-1 values (cli_export.rs:4710-4769). |
| FND-005 | fixed | `invalid_release_results_exit_3_before_converter` adds three cases: a nested approved verdict with an invalid `signed_at`, and nested verdicts with a wrong-typed `approver` and a wrong-typed `meaning` (cli_export.rs:4860-4912). |
| FND-001 | still-open | Round 2 (head c8ca227): the duplicate test is gone, and `release_approval_text_reaches_real_converters` now attempts AC-15's "adds no table row" and "applies no emphasis". It still cannot give evidence. It contradicts itself on `pagebreak()` (FND-007), it panics reading a stub-only converter log (FND-008), and its emphasis check cannot fail (FND-009). |
| FND-002 | fixed | Round 2: the git wrapper no longer uses `which` and finds the real git on `PATH`. The test records git calls for the release and standalone exports, asserts they are equal, and asserts no call names the file, `--approval` or `approvals/` (cli_export.rs:5042-5126). |
| FND-006 | fixed | Round 2: `release_results_render_literal_text_in_pdf_and_docx` is deleted (cli_export.rs diff `76b7451..c8ca227`), so the duplicate test that could never pass is gone. Its contradiction lives on in the remaining test (FND-007). |
| FND-007 | still-open | New in round 2: no fix yet. Drop the `!block.contains("pagebreak()")` assertion, or assert that no page break happened some other way, for example the page count or the text after the block. |
| FND-008 | still-open | New in round 2: no fix yet. Count rows in the real converter output (or in an export run with the stand-in converters) instead of reading `log/in/approval.md`. |
| FND-009 | still-open | New in round 2: no fix yet. Check for `<w:i/>` / `<w:b/>` run properties in `document.xml`, or for missing `*` characters in the visible text. |
| FND-001 | fixed | Round 3, updated at head `889fcea`: the AC-15 real-converter test checks literal hostile text in all three fields with the pipe, one data row in both stand-in markdown and the DOCX table, literal `*x*`, and the template page-break baseline (cli_export.rs:5226-5308). The real converter test remains ignored and is run in the final gate. |
| FND-007 | fixed | Round 3: the `!block.contains("pagebreak()")` assertion is removed, so the test no longer contradicts its own `block.contains(&expected)` (cli_export.rs:5265-5269). The check that replaced it is FND-010. |
| FND-008 | fixed | Round 3: the row count now comes from a second export run with the stand-in converters, `rig.export(&rig.fixture(), "pdf", &md_out)`, which passes `--pandoc`/`--typst` stubs that write `log/in/approval.md` (cli_export.rs:5225-5237). |
| FND-009 | fixed | Round 3: the `<em>` check is gone. The test now asserts that the visible PDF and DOCX text still holds `*x*` (cli_export.rs:5270-5274), and that check fails if emphasis eats the asterisks. |
| FND-010 | fixed | Round 3, updated at head `889fcea`: the page checks assert the template baseline, 3 PDF pages (title, contents, body) and exactly 2 DOCX page breaks, so an injected break fails the test (cli_export.rs:5252-5263). |
| FND-011 | fixed | Round 3, updated at head `889fcea`: the test requires at least three occurrences of hostile text and checks the pipe in the nested reason (cli_export.rs:5280-5295). |
