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
---
# FR-038: Approval block from signed-approval verdicts

## Description

`quire export` SHALL accept `--approval <PATH>`, repeatable, each a verdict file
of `engineering-assurance approval-verify`
([EA FR-041](ix://agent-ix/engineering-assurance/FR-041)), and SHALL generate an
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
  unbroken run in the Record cell, such as a 71-character record digest.
- The command SHALL size every column of the table to at least its longest
  unbreakable run, so no cell content overflows into a neighbouring column.
- The command SHALL NOT infer a person from the commit author, tagger, an email
  or any member other than those in the table, and SHALL NOT default a missing
  meaning.
- The command SHALL read a verdict file as data and never execute it. A file that
  is not an `engineering-assurance.approval-verify-result/v1` document, or that
  lacks `status`, is refused, as is a file above 16 MiB, one that is not a regular file, and an `approved` verdict whose `signed_at` is not a valid UTC instant ([FR-040](./FR-040-export-template-and-identity-errors.md)).
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
| FR-038-AC-9 | An approval whose record digest is `sha256:` followed by 64 hexadecimal digits, with an `untrusted_signer` entry, exports to PDF with every cell's content inside its own column: no word of one column's text overlaps another column's in the page geometry. | Test |
| FR-038-AC-8 | The same verdict files exported from two different source repositories yield a byte-identical `in/approval.md`; the approval is built from the verdict files alone. | Test |

## Dependencies

- **Upstream**: [FR-034](./FR-034-document-identity.md), [FR-035](./FR-035-default-export-template.md); [EA FR-041](ix://agent-ix/engineering-assurance/FR-041) (including `signed_at`, PLAT-1592) and [EA FR-037](ix://agent-ix/engineering-assurance/FR-037) own the verdict and the meaning.
- **Downstream**: [FR-040](./FR-040-export-template-and-identity-errors.md).
