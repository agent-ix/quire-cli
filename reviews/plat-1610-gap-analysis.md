---
id: SR-14574
title: "PLAT-1610 quire-cli PR193 gap-analysis"
type: SpecReview
analysis: gap-analysis
scope: "agent-ix/quire-cli PR193 head 5a05572; spec/functional/FR-038-approval-block.md, FR-039-export-bundle.md, FR-040-export-template-and-identity-errors.md, FR-034-document-identity.md (section C), FR-032-export-converter-execution.md, FR-007-exit-codes.md; src/commands/export/**; tests/cli_export.rs"
review_set: subset
---

## Summary

Ticket: PLAT-1610. This is a planless gap pass over the PR diff. Plan completion: not assessed.

The `quoin matrix` run (quoin 0.30.1) reports all 384 criteria in the repository as `untagged`, including criteria on main that have long-standing `#[trace(...)]` tags. The computed matrix therefore cannot see this repository's trace tags. That is not caused by this PR, and it should be reported separately to the quoin owner. The AC-to-test mapping below was checked by hand from the `#[trace]` attributes.

The ticket ACs were checked against measured behaviour, not against the ticket's own claims:

- **Two approvals with person, time and meaning:** met (FR-038-AC-1).
- **A non-valid verdict rendered as not verified, never signed:** met. Only `approved` gets a time and an unprefixed meaning, and every other status shows its own status text. This was confirmed on real verdicts.
- **The block built only from verifier output, with no verification by export:** met by construction. approval.rs reads only the named files and makes no git, network or process call. `load` runs before `identity::resolve`.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | low | FR-038-AC-8 ("with `--source-repo` unavailable after the identity is read, the block is unchanged") is not exercised as written. Its test only asserts that the block lacks the commit author name `Fixture`. That covers the no-inference-from-commit-author bullet, not the no-repository-read AC. Structurally the code is adequate: `approval::load` takes only paths and runs before identity resolution. But the AC is not testable as worded, so either reclassify it as Inspection or test that two exports from different source repositories or commits produce a byte-identical `in/approval.md`. | spec/functional/FR-038-approval-block.md:96, tests/cli_export.rs:3256-3302 |
| FND-002 | low | FR-038 Behavior says "For an entry of any other status the command SHALL prefix the Meaning cell with `claimed:`", while the table and AC-3 require the text `not recorded` when there is no meaning. The code writes `not recorded` with no prefix on non-approved rows. That reading is sensible, because `claimed: not recorded` would read as a signer's claim, but the spec text is contradictory. Scope the prefix to a recorded meaning. | spec/functional/FR-038-approval-block.md:63-64, src/commands/export/approval.rs:120-124 |
| FND-003 | low | FR-038 Meaning says "each value as recorded text, in order" and names no separator. The code joins with `; `, so the values `["a; b"]` and `["a", "b"]` render identically and the rendered block loses the value boundary. The spec should name the separator, or give one value per line. | spec/functional/FR-038-approval-block.md:47, src/commands/export/approval.rs:121-122 |
| FND-004 | low | The code refuses with exit 3 a malformed `signed_at` on an `approved` verdict, and a member whose JSON type is wrong ("unusable verdict"). The test asserts the first case under FR-038-AC-6. The FR-040 "Approval invalid" row lists only a missing file, invalid JSON, a wrong protocol and a missing `status`, so this behaviour exceeds the spec text. The refusal is right: it never guesses or reprints a time, which matches FR-040's no-substitute rule and EA FR-041-CON-3. Add the cause to the FR-040 table. | spec/functional/FR-040-export-template-and-identity-errors.md:39, src/commands/export/approval.rs:100-107, tests/cli_export.rs:3453 |
| FND-005 | low | FR-038-AC-4 requires the time to be unchanged "under a different `TZ` and clock". The test varies only `TZ`. The code reads no clock, so the risk is low, but the clock half is not exercised (SOURCE_DATE_EPOCH or a fake clock would do it). | spec/functional/FR-038-approval-block.md:92, tests/cli_export.rs:3383-3399 |
| FND-006 | low | FR-038-AC-3's "a verdict with two meaning values shows both in order" is asserted only in `two_approvals_show_person_time_and_meaning_after_the_document`, which is tagged FR-038-AC-1, FR-038-AC-8 and FR-032-AC-3 but not FR-038-AC-3. | tests/cli_export.rs:3256 |

## Examined scope

```yaml
scope:
- {id: FR-038-AC-1, path: spec/functional/FR-038-approval-block.md, role: examined, excerpt: "Exporting a fixture with an approved verdict whose approver is Ada Example, whose signed_at is 2026-01-02T03:04:05Z and whose meaning is reviewed and approved yields extracted text holding the person, 2026-01-02 03:04:05 UTC, the meaning and the verdict approved, in a block under Approval after the document, with the column title naming the signer's claim."}
- {id: FR-038-AC-2, path: spec/functional/FR-038-approval-block.md, role: examined, excerpt: "Three fixture verdicts approved, untrusted_signer and wrong_record ... all appear in command-line order; the second and third show the meaning prefixed claimed:, the verdict cell equal to their status, and not available as signed time; only the first entry is labelled an approval."}
- {id: FR-038-AC-3, path: spec/functional/FR-038-approval-block.md, role: examined, excerpt: "A meaning_missing verdict shows the text not recorded as its meaning, an unverifiable verdict ... and an unsigned verdict ... are shown with not available where absent, and a verdict with two meaning values shows both in order."}
- {id: FR-038-AC-4, path: spec/functional/FR-038-approval-block.md, role: examined, excerpt: "The time shown equals the verdict's signed_at and is unchanged when the test process runs under a different TZ and clock; an approved verdict with signed_at_missing shows not available."}
- {id: FR-038-AC-5, path: spec/functional/FR-038-approval-block.md, role: examined, excerpt: "A verdict whose meaning is *x* `#pagebreak()`{=typst} and whose principal holds a pipe shows both as literal text in PDF and DOCX, adds no table row and applies no emphasis."}
- {id: FR-038-AC-6, path: spec/functional/FR-038-approval-block.md, role: examined, excerpt: "A file that is not an approval-verify result, a file lacking status, and --approval with a JSON input each exit 3 naming the file or flag, with no converter started."}
- {id: FR-038-AC-7, path: spec/functional/FR-038-approval-block.md, role: examined, excerpt: "An export with no --approval holds no Approval heading and no in/approval.md."}
- {id: FR-038-AC-8, path: spec/functional/FR-038-approval-block.md, role: examined, excerpt: "The export with --approval makes no repository read for the approval: with --source-repo unavailable after the identity is read, the block is unchanged."}
- {id: FR-039-AC-4, path: spec/functional/FR-039-export-bundle.md, role: examined, excerpt: "--approval verdicts apply to the bundle as a whole and appear once, after the last document."}
- {id: FR-040-AC-5, path: spec/functional/FR-040-export-template-and-identity-errors.md, role: examined, excerpt: "Each approval failure of the table exits 3 naming the file, and an unverifiable verdict for an unresolved tag is not a failure; the export names the first failing file when two fail."}
- {id: FR-040-AC-6, path: spec/functional/FR-040-export-template-and-identity-errors.md, role: examined, excerpt: "In every case above no converter or engineering-assurance process is started, --output is absent, and a pre-existing --output is unchanged."}
- {id: FR-032-AC-3, path: spec/functional/FR-032-export-converter-execution.md, role: examined, excerpt: "The pandoc request carries ... the input files in/front.md and in/document.md (plus in/approval.md with approvals)"}
- {id: FR-034-AC-12, path: spec/functional/FR-034-document-identity.md, role: context_only, excerpt: "The command SHALL escape every identity value that it writes into generated Markdown so that the converter shows it as literal text: each ASCII punctuation character is backslash-escaped and each line break is replaced by a space."}
- {id: FR-007, path: spec/functional/FR-007-exit-codes.md, role: context_only, excerpt: "exit code classes; 3 invalid input, 2 unreadable"}
```

## Verdict

There are no medium or high gaps. Every FR-038 AC has a tagged test, and FR-038-AC-5 at the converter level is covered only by the ignored real-converter test, which is acceptable because pandoc and typst are not installed. The coder's four spec-gap choices are each consistent with FR-038's intent and with EA FR-041's rule that "a time beside an unaccepted signature would read as an attested one":

1. A non-approved entry never shows a time. The real verifier emits no time on non-approved verdicts, and AC-2 requires `not available`.
2. `not recorded` has no `claimed:` prefix (FND-002 is a spec wording issue only).
3. The `; ` joiner (FND-003 is a spec gap).
4. A malformed `signed_at` is refused with exit 3 (FND-004 is a spec gap). The refusal is the right behaviour.

FND-001 through FND-006 are spec and trace refinements that can be addressed in the spec or accepted. The medium code-review finding (SR-14573 FND-001) is the one blocker.
