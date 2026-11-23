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
reads no repository for it: the block is a presentation of verdicts computed
elsewhere, in the stance of [FR-031](./FR-031-export-subcommand.md) for
reports. Because the export does not verify, it presents every entry, whatever its
status, as an unverified claim and never as a verified approval. No signature
image, typed name or field a person fills in is produced.

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
| Verdict | The text `unverified claim: ` followed by the `status` of the verdict, verbatim (`approved`, `wrong_record`, `untrusted_signer`, `meaning_missing`, `signed_bad`, `unsigned`, `unverifiable`), and the `reason` when present. The prefix is present for every status, `approved` included, so an `approved` verdict reads `unverified claim: approved`. |
| Verdict file | The text `sha256:` followed by the 64 lowercase hexadecimal digits of the SHA-256 of the verdict file's bytes as read, computed in process; and, when the verdict holds a `tag_object_id`, a single space and `tag object: ` followed by it as recorded text, in the same cell. With no `tag_object_id` the cell is the digest alone. |
| Record | The `record_id` and `record_digest` of the verdict, and the `tag`. |

The time is what the signer's tag claims: the verification does not attest it
([EA FR-041](ix://agent-ix/engineering-assurance/FR-041)), and the block says so
in the column title. The Verdict file column lets a reader tie the entry to the
exact file it was read from and, through the tag object, to the tag that file
names; the export does not check either.

## Release-verify input

- The command SHALL treat a file whose `protocol` is
  `engineering-assurance.release-verify-result/v1` as a release result and SHALL
  read it as data, like any other verdict file.
- The command SHALL read only the members `protocol`, `status`, `reasons` and
  `approval` of a release result. Its `status` is `released` or `not_released`;
  `reasons` lists the failed checks of a `not_released` result, an FR-041 status
  other than `approved` being itself one of them
  ([EA FR-047](ix://agent-ix/engineering-assurance/FR-047)); `approval`, when
  present, is the unchanged FR-041 verdict of the release tag.
- The command SHALL NOT read the top-level `approver`, `meaning`, `signed_at`,
  `release`, `baseline` or `accepted_anomalies` of a release result, whatever its
  `status`.

For a standalone verdict file the sources are the members of that file, as the
table above states. For a release result the nested `approval` object stands for
the verdict file in that table, and nothing outside it is a source of Person,
Signed at, Meaning or Record.

| Entry field | Source for a release result |
| --- | --- |
| Person, Signed at, Meaning, Record | The nested `approval` object, read as the table above reads a verdict file. When `approval` is absent: Person `not available`, Signed at `not available`, Meaning `not recorded`, Record `not available`. For a `not_released` result whose nested `approval.status` is `approved`: Person `not available`, Signed at `not available`, Record `not available`, and the Meaning read as above and prefixed `claimed:`. |
| Verdict file | Formed as for a standalone verdict file, over the bytes of the release result file. The ` tag object: ` part is present only when the Record cell is read from the nested `approval` (not in the `claimed:` case above) and that object holds a `tag_object_id`. |
| Verdict | The text `unverified claim: ` followed by: for `released`, the nested verdict text formed as for a standalone verdict (`approved`); for `not_released`, up to three segments joined by `; `: first the text `not_released`; second `reasons: ` and the `reasons` as a JSON array, each reason one array element written as a JSON string (`["a", "b"]`, so a reason containing `, ` stays one value); third, only when `approval` is present and its `status` is not `approved`, `nested: ` and the nested verdict text formed as for a standalone verdict (`status`, or `status (reason)` when the nested `reason` is present). |

A nested `status` that is not `approved` is therefore shown twice, once as a
reason (EA FR-047) and once in the `nested:` segment, which also carries the nested
`reason`. A nested `approved` status of a `not_released` result is not shown.

- If the result `status` is `not_released`, then the command SHALL take no Person,
  Signed at or Record value from the nested `approval`, including when the nested
  `approval.status` is `approved` (for example with reason
  `decision_owner_mismatch`), and the Verdict cell SHALL start with
  `unverified claim: not_released`.
- If the result `status` is `not_released`, then the command SHALL prefix a
  recorded Meaning with `claimed:`.
- The command SHALL preserve every entry of `reasons` and SHALL NOT drop, reorder,
  reinterpret or add a reason.
- If `approval` is absent, then the command SHALL NOT supply a signer, a meaning, a
  signing time or a record from the release result or from anywhere else.
- The command SHALL NOT re-verify a signature, a tag or a release check, and SHALL
  NOT decide a release: it presents the nested verdict and the reasons the result
  already holds.
- If a release result matches a cause of the Release result invalid row of
  [FR-040](./FR-040-export-template-and-identity-errors.md), then the command SHALL
  refuse it before any converter starts.
- The command SHALL present a release result as one entry, in the order given on the
  command line, and SHALL handle it by the sanitising, escaping, wrapping, sizing
  and no-inference rules of this requirement exactly as it handles a standalone
  verdict.
- The command SHALL NOT order or group entries by kind. A release result that a
  caller adds (for example a dossier export adding the result of
  `engineering-assurance release-verify`) takes the position the caller gives it
  among the `--approval` arguments, and `--approval` inputs a user supplied
  ([quoin FR-191](ix://agent-ix/quoin/FR-191) passes them unchanged) keep their
  relative order.

## Behavior

- The command SHALL show every supplied verdict, whatever its status, in the order
  given on the command line, so a failed approval is never omitted. This includes
  a verdict whose tag did not resolve (`unverifiable`) and a lightweight tag
  (`unsigned`); these show with `not available` where a member is absent.
- The command SHALL begin the Verdict cell of every entry, whatever its status and
  whether it is a standalone verdict or a release result, with the text
  `unverified claim: `.
- The command SHALL NOT write a Verdict cell that consists of a bare status token
  (`approved`, `released` or any other status): the cell always holds the prefix
  `unverified claim: ` before the status text. The block adds no column, heading
  or text that states or implies verification beyond the Verdict prefix and the
  fixed statement below. The recorded meaning text itself is shown verbatim.
- For a standalone verdict entry whose `status` is not `approved` the command SHALL
  prefix a recorded Meaning with `claimed:` (the text `not recorded` is not
  prefixed). The Meaning of a release-result entry follows Release-verify input.
- The command SHALL show, in the Verdict file cell of every entry, the SHA-256 of
  the verdict file's bytes and, when present, its `tag_object_id`, and SHALL
  compute the digest in process without starting a process.
- Under the level-1 heading `Approval` and before the table, the command SHALL
  write, as fixed text and for every export that has an approval block, the
  statement: `The verdicts below were computed elsewhere and are not re-verified by
  this export. An approval covers the record named in its Record column, not this
  document.` The command SHALL NOT compare any digest of the document with a record
  digest, and the statement is unchanged by the verdicts supplied.
- The command SHALL replace each line break, control character and Unicode
  bidirectional control (U+061C, U+200E, U+200F, U+202A to U+202E, U+2066 to
  U+2069) in a verdict value by a space, a Meaning value before it is written as a
  JSON string.
- The command SHALL escape every verdict value it writes into `in/approval.md`
  as [FR-034](./FR-034-document-identity.md) section C states, so a principal or
  a meaning typed by a signer shows as literal text and cannot add a raw block,
  a link, emphasis or a table row.
- The block is placed after the document, under the level-1 heading `Approval`.
  After the statement, in the default template, it is a table with the columns Person, Signed at
  (signer's claim), Meaning, Verdict, Verdict file and Record; an organisation template styles
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
- The command SHALL produce for a standalone `approval-verify` verdict file the
  entry this requirement specifies for it, unchanged by whether a release result is
  also supplied.
- The block states what the verdict file said when it was supplied. The command
  does not re-verify and does not state that the verdict is true, current or
  that it covers this document.

## Acceptance Criteria

| ID | Criteria | Verification |
| --- | --- | --- |
| FR-038-AC-1 | Exporting a fixture with an `approved` verdict whose approver is `Ada Example`, whose `signed_at` is `2026-01-02T03:04:05Z` and whose meaning is `reviewed and approved` yields extracted text holding the person, `2026-01-02 03:04:05 UTC`, the meaning and the verdict `unverified claim: approved`, in a block under `Approval` after the document, with the column title naming the signer's claim. | Test |
| FR-038-AC-2 | Three fixture verdicts `approved`, `untrusted_signer` and `wrong_record`, the last two with meaning `reviewed and approved`, all appear in command-line order; the second and third show the meaning prefixed `claimed:`, the verdict cell equal to `unverified claim: ` followed by their status, and `not available` as signed time; the first shows the verdict `unverified claim: approved`; every Verdict cell begins with `unverified claim: ` and none equals a bare status token. | Test |
| FR-038-AC-3 | A `meaning_missing` verdict shows the text `not recorded` as its meaning, an `unverifiable` verdict for a tag that did not resolve and an `unsigned` verdict for a lightweight tag are shown with `not available` where absent, and a verdict with two meaning values shows both in order. | Test |
| FR-038-AC-4 | The time shown equals the verdict's `signed_at` and is unchanged when the test process runs under a different `TZ` and when the source commit has a different date; an `approved` verdict with `signed_at_missing` shows `not available`. | Test |
| FR-038-AC-5 | A verdict whose meaning is `*x* \`#pagebreak()\`{=typst}` and whose principal holds a pipe shows both as literal text in PDF and DOCX, adds no table row and applies no emphasis. | Test |
| FR-038-AC-6 | A file that is neither an approval-verify nor a release-verify result, a file lacking `status`, and `--approval` with a JSON input each exit 3 naming the file or flag, with no converter started. | Test |
| FR-038-AC-7 | An export with no `--approval` holds no `Approval` heading and no `in/approval.md`. | Test |
| FR-038-AC-8 | The same verdict files exported from two different source repositories yield a byte-identical `in/approval.md`; the approval is built from the verdict files alone. | Test |
| FR-038-AC-9 | An approval whose record digest is `sha256:` followed by 64 hexadecimal digits and an `untrusted_signer` entry whose meaning is a 73-character URL, and a `tag_object_id` of 40 hexadecimal digits, exports to PDF with every cell's content inside its own column, the Verdict file column included: no word of one column's text overlaps another column's in the page geometry. | Test |
| FR-038-AC-10 | A static `released` release-verify result whose nested `approval` is `approved` with approver `Ada Example`, `signed_at` `2026-01-02T03:04:05Z` and meaning `reviewed and approved` exports successfully; its Verdict cell is exactly `unverified claim: approved`, and its Person, Signed at, Meaning and Record cells are byte-identical to the cells of the row of the nested verdict exported as a standalone file. | Test |
| FR-038-AC-11 | A static `not_released` result with reasons `untrusted_signer` and `release_commit_record_not_added` and a nested `untrusted_signer` approval (meaning `reviewed and approved`, no `reason`) has the Verdict cell exactly `unverified claim: not_released; reasons: ["untrusted_signer", "release_commit_record_not_added"]; nested: untrusted_signer`, the Meaning `claimed: ` followed by the meaning, Signed at `not available`, and is not labelled an approval. With a nested `reason` of `x, y` the third segment is `nested: untrusted_signer (x, y)` after the same prefix. A reason `a, b` shows as the one array element `"a, b"`. | Test |
| FR-038-AC-12 | A static `not_released` result with reasons `record_invalid` and no `approval` member has the Verdict cell exactly `unverified claim: not_released; reasons: ["record_invalid"]`, Person `not available`, Signed at `not available`, Meaning `not recorded`, Record `not available`, holds no signer, meaning or time, and is not labelled an approval. | Test |
| FR-038-AC-13 | The `in/approval.md` of an export of a standalone approval-verify fixture A alone, of A with a release result R before it and of A with R after it holds the row of A, Verdict file cell included, byte-identical in all three; the exports with R hold exactly one more row than the export without it, the row of R; and the row of A carries the values FR-038-AC-1 specifies. | Test |
| FR-038-AC-14 | Each of these exits 3 naming the file, with no converter started: a release result with `status` `unknown`; a release result with no `status`; a `released` result without `approval`; a `released` result whose nested status is `wrong_record`; a `released` result with `reasons` `["x"]`; a `not_released` result without `reasons`, one with `reasons` `[1]` and one with `reasons` `"x"`; a release result whose `approval` is a string; a release result whose `approval` has another `protocol`. | Test |
| FR-038-AC-15 | A `not_released` release result whose reason holds `*x* \`#pagebreak()\`{=typst}` and a pipe, and whose nested non-approved verdict has a meaning and a `reason` holding the same, shows each as literal text in PDF and DOCX, adds no table row and applies no emphasis. | Test |
| FR-038-AC-16 | The export of a release result, run with stub `gpg` and `ssh-keygen` executables and a stub `engineering-assurance` first on `PATH` that record each call, never invokes `gpg`, `ssh-keygen` or any `engineering-assurance` subcommand other than `producer-execute` as [FR-032](./FR-032-export-converter-execution.md) requires it, starts no other process for the approval block or the Verdict file digest, and names no `--approval` file or tag in any call. | Test |
| FR-038-AC-17 | A static `not_released` result with reasons `decision_owner_mismatch` and a nested `approved` approval (approver `Ada Example`, `signed_at` `2026-01-02T03:04:05Z`, meaning `reviewed and approved`, record fields present) has the Verdict cell exactly `unverified claim: not_released; reasons: ["decision_owner_mismatch"]`, Person, Signed at and Record `not available`, the Meaning `claimed: ` followed by the meaning, and neither `Ada Example` nor `2026-01-02 03:04:05 UTC` anywhere in the block. | Test |
| FR-038-AC-18 | An export with two `--approval` arguments, a standalone verdict and a release result, lists the two entries in command-line order, whichever argument comes first. | Test |
| FR-038-AC-19 | A fabricated verdict file whose `status` is `approved` and whose approver, record, signature and meaning are invented, with no real signed tag behind it, exports with the Verdict cell exactly `unverified claim: approved`; no Verdict cell in the table equals a bare status token. The same holds for a `released` release result with a nested `approved` approval. | Test |
| FR-038-AC-20 | The Verdict file cell of an entry equals `sha256:` followed by the SHA-256 of the verdict file's bytes, as a test computes it independently over the file, followed, when the verdict holds a `tag_object_id`, by a single space and `tag object: ` and that id; a verdict without `tag_object_id` has the digest alone. For a `not_released` result whose nested approval is `approved` the cell holds the digest alone. | Test |
| FR-038-AC-21 | The text under the `Approval` heading, before the table, is exactly the statement of this requirement and contains no digest, byte-identical for an export with one `approved` verdict, one `wrong_record` verdict and one release result, and appears once for an export with several entries. | Test |

## Dependencies

- **Upstream**: [FR-034](./FR-034-document-identity.md), [FR-035](./FR-035-default-export-template.md); [EA FR-041](ix://agent-ix/engineering-assurance/FR-041) (including `signed_at`, PLAT-1592) and [EA FR-037](ix://agent-ix/engineering-assurance/FR-037) own the verdict and the meaning; [EA FR-047](ix://agent-ix/engineering-assurance/FR-047) owns the release result (PLAT-1761).
- **Downstream**: [FR-040](./FR-040-export-template-and-identity-errors.md).
