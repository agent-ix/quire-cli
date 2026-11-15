---
id: FR-038
title: "Approval block from signed-approval verdicts"
type: FR
relationships:
  - target: "ix://agent-ix/quire-cli/spec/stakeholder/StR-004"
    type: "implements"
    cardinality: "1:1"
  - target: "ix://agent-ix/quire-cli/spec/usecase/US-010"
    type: "implements"
  - target: "ix://agent-ix/quire-cli/spec/functional/FR-031"
    type: "extends"
  - target: "ix://agent-ix/quire-cli/spec/functional/FR-034"
    type: "consumes"
  - target: "ix://agent-ix/quire-cli/spec/functional/FR-035"
    type: "extends"
  - target: "ix://agent-ix/engineering-assurance/FR-041"
    type: "consumes"
  - target: "ix://agent-ix/engineering-assurance/FR-037"
    type: "references"
  - target: "ix://agent-ix/engineering-assurance/FR-047"
    type: "consumes"
---
# FR-038: Approval block from signed-approval verdicts

## Description

`quire export` SHALL accept `--approval <PATH>`, repeatable, each a verdict file
of `engineering-assurance approval-verify`
([EA FR-041](ix://agent-ix/engineering-assurance/FR-041)) or a result file of
`engineering-assurance release-verify`
([EA FR-047](ix://agent-ix/engineering-assurance/FR-047)), and SHALL generate an
approval block (`in/approval.md`, [FR-035](./FR-035-default-export-template.md))
with one entry per file. The export verifies no signature, decides no approval and
reads no repository for it: the block is a presentation of verdicts the toolchain
already computed, in the stance of [FR-031](./FR-031-export-subcommand.md) for
reports. No signature image, typed name or field a person fills in is produced.

```text
quire export <INPUT>... --to pdf|docx --output <PATH> [--approval <PATH>]...
```

## Entry content

Every member is taken from the verdict file; nothing is read from git.

| Entry field | Source |
| --- | --- |
| Person | `approver` of the verdict, which is the `principal` of the signer ([EA FR-037](ix://agent-ix/engineering-assurance/FR-037)). For a verdict without `approver`, the `signature.signer.principal` when present, otherwise the text `unmapped`. |
| Signed at | `signed_at` of the verdict, a UTC instant, shown as `YYYY-MM-DD HH:MM:SS UTC` under the column title `Signed at (signer's claim)`. For a verdict without `signed_at` (every status other than `approved`, or `signed_at_missing`) the text `not available`. |
| Meaning | The `meaning` array of the verdict, or `signature.signed_off_meaning.values` when `meaning` is absent, each value as recorded text, in order, written as a JSON string and the values joined by `, `, so that the boundary between two values is visible (`["a, b"]` and `["a", "b"]` differ). When none, the text `not recorded`. |
| Verdict | The `status` of the verdict, verbatim (`approved`, `wrong_record`, `untrusted_signer`, `meaning_missing`, `signed_bad`, `unsigned`, `unverifiable`), and the `reason` when present. |
| Record | The `record_id` and `record_digest` of the verdict, and the `tag`. |

The time is what the signer's tag claims: the verification does not attest it
([EA FR-041](ix://agent-ix/engineering-assurance/FR-041)), and the block says so
in the column title.

## Release-verify input

A file whose `protocol` is `engineering-assurance.release-verify-result/v1` is a
release result. Its `status` is `released` or `not_released`, `reasons` lists the
failed checks of a `not_released` result, and its `approval` member, when present,
is the unchanged FR-041 verdict of the release tag (EA FR-047). The file is read as
data like any other verdict file, and the entry takes every member from it.

| Entry field | Source for a release result |
| --- | --- |
| Person, Signed at, Meaning, Record | When `approval` is present, the nested `approval` verdict, read exactly as the table above reads a verdict file. The top-level `approver`, `meaning` and `signed_at` of a `released` result are not read. When `approval` is absent: Person `not available`, Signed at `not available`, Meaning `not recorded`, Record `not available`. |
| Verdict | For `released`, the nested `status`, which is `approved`. Otherwise the nested `status` (`unavailable` when `approval` is absent), then `not_released`, then each entry of `reasons` verbatim in order, as recorded text joined by `, `. |

- The command SHALL label an entry of a release result as an approval only when the
  result `status` is `released` and the nested `approval.status` is `approved`.
- If the result `status` is `not_released`, then the command SHALL NOT give the
  entry a heading, label or Verdict text that reads as an approval, including when
  the nested `approval.status` is `approved` (for example with reason
  `decision_owner_mismatch`), and SHALL prefix a recorded Meaning with `claimed:`.
- The command SHALL preserve every entry of `reasons` and SHALL NOT drop, reorder,
  reinterpret or add a reason.
- If `approval` is absent, then the command SHALL NOT supply a signer, a meaning, a
  signing time or a record from the release result or from anywhere else.
- The command SHALL NOT re-verify a signature, a tag or a release check, and SHALL
  NOT decide a release: it presents the nested verdict and the reasons the result
  already holds.
- The command SHALL refuse a release result whose `status` is neither `released`
  nor `not_released`, a `released` result without a nested `approval`, a `released`
  result whose nested `approval.status` is not `approved`, a `not_released` result
  whose `reasons` is absent or not a list of strings, and an `approval` member that
  is present but is not an `engineering-assurance.approval-verify-result/v1`
  document ([FR-040](./FR-040-export-template-and-identity-errors.md)).
- A release result SHALL be one entry, in the order given on the command line, and
  SHALL be handled by the sanitising, escaping, wrapping, sizing and
  no-inference rules of this requirement exactly as a standalone verdict is.

## Behavior

- The command SHALL show every supplied verdict, whatever its status, in the order
  given on the command line, so a failed approval is never omitted. This includes
  a verdict whose tag did not resolve (`unverifiable`) and a lightweight tag
  (`unsigned`); these show with `not available` where a member is absent.
- The command SHALL label only an entry whose `status` is `approved` as an
  approval.
- For an entry of any other status the command SHALL prefix a recorded Meaning
  with `claimed:` (the text `not recorded` is not prefixed) and set the Verdict
  cell to the status.
- The command SHALL NOT give an entry of any other status a heading, label or
  Verdict text that reads as an approval. The recorded meaning
  text itself is shown verbatim.
- The command SHALL replace each line break, control character and Unicode
  bidirectional control (U+061C, U+200E, U+200F, U+202A to U+202E, U+2066 to
  U+2069) in a verdict value by a space, a Meaning value before it is written as a
  JSON string.
- The command SHALL escape every verdict value it writes into `in/approval.md`
  as [FR-034](./FR-034-document-identity.md) section C states, so a principal or
  a meaning typed by a signer shows as literal text and cannot add a raw block,
  a link, emphasis or a table row.
- The block is placed after the document, under the level-1 heading `Approval`.
  In the default template it is a table with the columns Person, Signed at
  (signer's claim), Meaning, Verdict and Record; an organisation template styles
  the same table.
- The command SHALL allow a line break (U+200B) after every 16 characters of an
  unbroken run in any cell of the table, such as a 71-character record digest or
  a URL in a meaning. Text extraction of such a cell may therefore contain
  U+200B; the verified record, not the extracted text, is the identity of the
  signed record.
- The command SHALL size every column of the table to at least its longest
  unbreakable run, so no cell content overflows into a neighbouring column.
- The command SHALL NOT infer a person from the commit author, tagger, an email
  or any member other than those in the table, and SHALL NOT default a missing
  meaning.
- The command SHALL read a verdict file as data and never execute it. A file that
  is neither an `engineering-assurance.approval-verify-result/v1` nor an
  `engineering-assurance.release-verify-result/v1` document, or that lacks
  `status`, is refused before any converter starts, as is a file above 16 MiB, one that is not a regular file, and an `approved` verdict whose `signed_at` is not a valid UTC instant ([FR-040](./FR-040-export-template-and-identity-errors.md)).
- A standalone `approval-verify` verdict file SHALL produce the entry this
  requirement specifies for it, unchanged by the release-verify input.
- The block states what the verdict said when it was produced. The command does
  not re-verify and does not state that the verdict is current.

## Acceptance Criteria

| ID | Criteria | Verification |
| --- | --- | --- |
| FR-038-AC-1 | Exporting a fixture with an `approved` verdict whose approver is `Ada Example`, whose `signed_at` is `2026-01-02T03:04:05Z` and whose meaning is `reviewed and approved` yields extracted text holding the person, `2026-01-02 03:04:05 UTC`, the meaning and the verdict `approved`, in a block under `Approval` after the document, with the column title naming the signer's claim. | Test |
| FR-038-AC-2 | Three fixture verdicts `approved`, `untrusted_signer` and `wrong_record`, the last two with meaning `reviewed and approved`, all appear in command-line order; the second and third show the meaning prefixed `claimed:`, the verdict cell equal to their status, and `not available` as signed time; only the first entry is labelled an approval. | Test |
| FR-038-AC-3 | A `meaning_missing` verdict shows the text `not recorded` as its meaning, an `unverifiable` verdict for a tag that did not resolve and an `unsigned` verdict for a lightweight tag are shown with `not available` where absent, and a verdict with two meaning values shows both in order. | Test |
| FR-038-AC-4 | The time shown equals the verdict's `signed_at` and is unchanged when the test process runs under a different `TZ` and when the source commit has a different date; an `approved` verdict with `signed_at_missing` shows `not available`. | Test |
| FR-038-AC-5 | A verdict whose meaning is `*x* \`#pagebreak()\`{=typst}` and whose principal holds a pipe shows both as literal text in PDF and DOCX, adds no table row and applies no emphasis. | Test |
| FR-038-AC-6 | A file that is not an approval-verify result, a file lacking `status`, and `--approval` with a JSON input each exit 3 naming the file or flag, with no converter started. | Test |
| FR-038-AC-7 | An export with no `--approval` holds no `Approval` heading and no `in/approval.md`. | Test |
| FR-038-AC-9 | An approval whose record digest is `sha256:` followed by 64 hexadecimal digits and an `untrusted_signer` entry whose meaning is a 73-character URL, exports to PDF with every cell's content inside its own column: no word of one column's text overlaps another column's in the page geometry. | Test |
| FR-038-AC-10 | A static `released` release-verify result whose nested `approval` is `approved` with approver `Ada Example`, `signed_at` `2026-01-02T03:04:05Z` and meaning `reviewed and approved` exports successfully, and the entry carries that approver, `2026-01-02 03:04:05 UTC`, that meaning and the verdict `approved`, equal to the entry of the nested verdict exported as a standalone file. | Test |
| FR-038-AC-11 | A static `not_released` result with a nested `untrusted_signer` approval and reasons `decision_owner_mismatch` and `release_commit_record_not_added` shows the nested status, `not_released` and both reasons in order in the Verdict cell, the meaning prefixed `claimed:`, and is not labelled an approval. A `not_released` result whose nested approval is `approved` is likewise not labelled an approval. | Test |
| FR-038-AC-12 | A static `not_released` result with reasons `record_invalid` and no `approval` member shows Person and Signed at `not available`, Meaning `not recorded`, Record `not available` and the reason `record_invalid` in the Verdict cell, and holds no signer, meaning or time. | Test |
| FR-038-AC-13 | A standalone approval-verify fixture exports to an `in/approval.md` byte-identical to the one FR-038-AC-1 and FR-038-AC-2 specify, whether or not a release-verify result is also supplied. | Test |
| FR-038-AC-14 | A release result with `status` `unknown`, a `released` result without `approval`, a `released` result whose nested status is `wrong_record`, a `not_released` result without `reasons`, and a nested `approval` of another protocol each exit 3 naming the file, with no converter started. | Test |
| FR-038-AC-15 | A release result whose reason holds `*x* \`#pagebreak()\`{=typst}` and a pipe, and whose nested meaning holds the same, shows both as literal text in PDF and DOCX, adds no table row and applies no emphasis. | Test |
| FR-038-AC-16 | The release-verify fixtures are static documents; the export of one starts no git process, reads no repository and makes no signature check. | Test |
| FR-038-AC-8 | The same verdict files exported from two different source repositories yield a byte-identical `in/approval.md`; the approval is built from the verdict files alone. | Test |

## Dependencies

- **Upstream**: [FR-034](./FR-034-document-identity.md), [FR-035](./FR-035-default-export-template.md); [EA FR-041](ix://agent-ix/engineering-assurance/FR-041) (including `signed_at`, PLAT-1592) and [EA FR-037](ix://agent-ix/engineering-assurance/FR-037) own the verdict and the meaning; [EA FR-047](ix://agent-ix/engineering-assurance/FR-047) owns the release result (PLAT-1761).
- **Downstream**: [FR-040](./FR-040-export-template-and-identity-errors.md).
