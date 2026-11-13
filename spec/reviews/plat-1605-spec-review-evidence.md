---
id: SR-20261112234822
title: "PLAT-1605 quire-cli PR182 spec-review/evidence"
type: SpecReview
analysis: evidence
scope: "agent-ix/quire-cli PR182 head 9194f3a; spec/functional/FR-031,FR-032,FR-034..FR-040, spec/non-functional/NFR-010,NFR-011, spec/stakeholder/StR-004, spec/usecase/US-010"
review_set: subset
---

## Summary

Ticket: PLAT-1605. Verification method and oracle for each new AC.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | medium | FR-038-AC-2 requires 'no word `approved`' in the entries of the `untrusted_signer` and `wrong_record` verdicts. But FR-038 shows Meaning 'as recorded text', and a signer's `Signed-off-meaning` is free text: FR-038-AC-1's own fixture meaning is 'reviewed and approved'. A correct implementation shows that meaning verbatim in a `wrong_record` entry and fails the AC, or must alter the recorded meaning to pass. The same text also breaks the FR's 'no wording that reads as an approval' rule. Decide which wins: show non-approved meanings verbatim but marked (for example 'claimed meaning: ...'), with the AC checking the Verdict cell only, or suppress the meaning for non-approved statuses. | spec/functional/FR-038-approval-block.md:50-53, :74-75 |
| FND-002 | medium | FR-035 sections A and B apply to both formats, but the ACs verify the title page (AC-1) and the header with id and version (AC-3) only for PDF. DOCX gets only the footer fields (AC-4). The DOCX title page, DOCX header and DOCX approval-table placement have no evidence, which is the part base FND-001 shows cannot work as specified. Add DOCX ACs that extract `word/document.xml` and `word/header*.xml`. | spec/functional/FR-035-default-export-template.md:65-68 |

## Examined scope

```yaml
scope:
- id: FR-038-AC-2
  path: spec/functional/FR-038-approval-block.md
  role: examined
  excerpt: Three fixture verdicts `approved`, `untrusted_signer` and `wrong_record` all appear, in command-line order, and the text of the second and third holds their status and no word `approved` in their entry.
- id: FR-035-AC-1
  path: spec/functional/FR-035-default-export-template.md
  role: examined
  excerpt: A PDF export of a committed fixture with no `--template` yields page 1 text with the title and the five identity fields and no body text, and the text of page 2 onward holds the body. Extracted with `pdftotext` per page.
- id: FR-034-AC-6
  path: spec/functional/FR-034-document-identity.md
  role: examined
  excerpt: During an export, `quire` starts no process other than `engineering-assurance`, and the dependency list adds no crate that links a network transport.
- id: NFR-011-AC-4
  path: spec/non-functional/NFR-011-export-reproducibility.md
  role: examined
  excerpt: A fixture changed by one word yields a different extracted text, so the comparison can fail.
- id: FR-034-AC-7
  path: spec/functional/FR-034-document-identity.md
  role: examined
  excerpt: A fixture whose title is `--output=/tmp/x` exports with that text as the title and no change to the arguments of any request; the identity values appear in `in/metadata.yaml` only.
```

## Verdict

Changes required. Test is the right method for the differential and extraction ACs. NFR-011-AC-4 gives the stability comparison a control that can fail. Inspection fits FR-035-AC-5 and FR-037-AC-4. FR-034-AC-6's missing oracle is recorded as dependency FND-001.
