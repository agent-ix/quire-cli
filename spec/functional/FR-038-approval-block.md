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
([EA FR-041](ix://agent-ix/engineering-assurance/FR-041)), and SHALL show an
approval block with one entry per file. The export verifies no signature and
decides no approval: the block is a presentation of verdicts the toolchain
already computed, in the stance of [FR-031](./FR-031-export-subcommand.md) for
reports. No signature image, typed name or field a person fills in is produced.

```text
quire export <INPUT>... --to pdf|docx --output <PATH> [--approval <PATH>]...
```

## Entry content

| Entry field | Source |
| --- | --- |
| Person | `approver` of the verdict, which is the `principal` of the signer ([EA FR-037](ix://agent-ix/engineering-assurance/FR-037)). For a verdict without `approver`, the `signature.signer.principal` when present, otherwise the text `unmapped`. |
| Time | The tagger date of the tag the verdict names in `tag`, read in-process from the source repository as [FR-034](./FR-034-document-identity.md) reads the commit, in UTC as `YYYY-MM-DD HH:MM:SS UTC`. The verdict has no time member. |
| Meaning | The `meaning` array of the verdict, or `signature.signed_off_meaning.values` when `meaning` is absent, each value as recorded text, in order. When none, the text `not recorded`. |
| Verdict | The `status` of the verdict, verbatim (`approved`, `wrong_record`, `untrusted_signer`, `meaning_missing`, `signed_bad`, `unsigned`, `unverifiable`), and the `reason` when present. |
| Record | The `record_id` and `record_digest` of the verdict. |

## Behavior

- The command SHALL show every supplied verdict, whatever its status, in the order
  given on the command line, so a failed approval is never omitted. It SHALL
  label only the status `approved` as an approval; any other status SHALL show
  as written, with no wording that reads as an approval.
- The block is placed after the body, under the heading `Approval`. In the default
  template ([FR-035](./FR-035-default-export-template.md)) it is a table with
  the columns Person, Time, Meaning, Verdict and Record. For an organisation
  template ([FR-036](./FR-036-organisation-export-template.md)) the entries reach
  the template as the `approvals` metadata list, and the template decides the
  layout.
- The command SHALL NOT infer a person from the commit author, tagger, an email
  or any member other than those in the table, and SHALL NOT default a missing
  meaning.
- The command SHALL read a verdict file as data and never execute it. A file that
  is not an `engineering-assurance.approval-verify-result/v1` document, or whose
  tag cannot be resolved in the source repository, is refused
  ([FR-040](./FR-040-export-template-and-identity-errors.md)).
- The block states what the verdict said at the time it was produced. The command
  does not re-verify and does not state that the verdict is current.

## Acceptance Criteria

| ID | Criteria | Verification |
| --- | --- | --- |
| FR-038-AC-1 | Exporting a fixture with an `approved` verdict whose signer principal is `Ada Example`, whose tag has a known tagger date and whose meaning is `reviewed and approved` yields extracted text holding the person, the UTC time, the meaning and the verdict `approved`, in a block under `Approval` after the body. | Test |
| FR-038-AC-2 | Three fixture verdicts `approved`, `untrusted_signer` and `wrong_record` all appear, in command-line order, and the text of the second and third holds their status and no word `approved` in their entry. | Test |
| FR-038-AC-3 | A verdict with `meaning_missing` shows the text `not recorded` as its meaning and a verdict with two meaning values shows both in order. | Test |
| FR-038-AC-4 | The time shown equals the tagger date of the named tag, and is unchanged when the test process runs under a different `TZ` and clock. | Test |
| FR-038-AC-5 | With an organisation template, a stand-in pandoc sees the `approvals` list in `in/metadata.yaml` with the five entry fields per verdict, in order. | Test |
| FR-038-AC-6 | A file that is not an approval-verify result, a verdict whose tag is not in the source repository, and `--approval` with a JSON input each exit 3 naming the file or flag, with no converter started. | Test |
| FR-038-AC-7 | An export with no `--approval` holds no `Approval` heading and no signature line. | Test |

## Dependencies

- **Upstream**: [FR-034](./FR-034-document-identity.md), [FR-035](./FR-035-default-export-template.md); [EA FR-041](ix://agent-ix/engineering-assurance/FR-041) and [EA FR-037](ix://agent-ix/engineering-assurance/FR-037) own the verdict and the meaning.
- **Downstream**: [FR-040](./FR-040-export-template-and-identity-errors.md).
