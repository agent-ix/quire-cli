---
id: SR-20261112234819
title: "PLAT-1605 quire-cli PR182 spec-review/failure-domain"
type: SpecReview
analysis: failure-domain
scope: "agent-ix/quire-cli PR182 head 9194f3a; spec/functional/FR-031,FR-032,FR-034..FR-040, spec/non-functional/NFR-010,NFR-011, spec/stakeholder/StR-004, spec/usecase/US-010"
review_set: subset
---

## Summary

Ticket: PLAT-1605. Unstated failure modes in the identity read, approval entries and metadata hand-off. EA FR-037 and FR-041 were read in /home/peter/dev/engineering-assurance at 906aad1.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | high | The claim 'The verdict has no time member' is correct: EA FR-041's result holds status, tag, record_id, record_digest, trust, reason, signature, approver, meaning, signature_kind and binding, with no time. The workaround is unsound. The verdict names the tag only by name, with no tag object id. FR-038 re-reads that name from the source repository at export time, so a tag deleted and re-created, or moved with `-f`, after verification gives the Time of a different tag object, possibly an unsigned one. Nothing ties it to the object that was verified. The tagger date is also the signer's own clock, not a verified time, yet it is shown under 'Time' as if established. The ticket AC asks for the time from the signed-records verdict. StR-004 says to author the upstream FR first when a CLI feature needs it. Fix: file an EA FR adding the tag object id and the tagger date to the approval-verify result. FR-038 then takes Time from the verdict, labelled as the signer's tag date, and drops its own git read for approvals. If the interim read stays, compare the resolved tag object to an id carried in the verdict, and label the column 'Tag date (signer's clock)'. | spec/functional/FR-038-approval-block.md:43, engineering-assurance spec/functional/FR-041-signed-tag-approval-of-a-change-assurance-record.md:64-83 |
| FND-002 | medium | FR-038 requires showing 'every supplied verdict, whatever its status ... so a failed approval is never omitted'. Two statuses EA FR-041 defines break this. (a) An `unverifiable` verdict whose tag does not resolve (reason `revision_unresolved`) names a tag absent from the repository, so FR-040 refuses it with exit 3 instead of showing it. (b) An `unsigned` verdict for a lightweight tag has no tagger, so its Time is undefined. State the Time for these cases (for example `not available`), and keep the refusal only for a verdict file that is not a valid result. | spec/functional/FR-038-approval-block.md:50-53, spec/functional/FR-040-export-template-and-identity-errors.md:38 |
| FND-003 | medium | 'byte-equal to the blob committed there' fails for a correctly committed, unmodified file whenever git applies a conversion between worktree and blob. Examples: `core.autocrlf`/`text=auto` eol conversion on Windows checkouts, a `.gitattributes` `filter=` clean filter, or Git LFS, where the blob is a pointer file. Such inputs are falsely refused as 'not committed'. With LFS, the content committed is not even what pandoc reads. State the comparison precisely: compare the blob to the worktree file after git's clean conversion, or refuse filtered and LFS paths with a distinct diagnostic. Add an AC for a CRLF/autocrlf fixture. | spec/functional/FR-034-document-identity.md:59-61 |
| FND-004 | medium | Pandoc parses every string metadata value as Markdown. Values from a source the exporter does not control enter `in/metadata.yaml`: the title, the signer's `principal`, and the `Signed-off-meaning` trailer text a signer typed. Such a value can carry emphasis, links, or a raw block such as `` `...`{=typst} `` or `{=openxml}` that the converter emits verbatim into the document. A meaning value could therefore rewrite the approval table or add text that reads as an approval. FR-034-AC-7 checks only that no argument changes. Specify that values are escaped so the converter shows them as literal text, and add an AC with a raw-typst and a Markdown-emphasis meaning value that each appear literally. | spec/functional/FR-036-organisation-export-template.md:42-46, spec/functional/FR-034-document-identity.md:62-63 |

## Examined scope

```yaml
scope:
- id: FR-038-time
  path: spec/functional/FR-038-approval-block.md
  role: examined
  excerpt: '| Time | The tagger date of the tag the verdict names in `tag`, read in-process from the source repository as FR-034 reads the commit, in UTC as `YYYY-MM-DD HH:MM:SS UTC`. The verdict has no time member. |'
- id: FR-038-all
  path: spec/functional/FR-038-approval-block.md
  role: examined
  excerpt: The command SHALL show every supplied verdict, whatever its status, in the order given on the command line, so a failed approval is never omitted. It SHALL label only the status `approved` as an approval; any other status SHALL show as written, with no wording that reads as an approval.
- id: FR-034-blob
  path: spec/functional/FR-034-document-identity.md
  role: examined
  excerpt: The command SHALL require the input file to be tracked at the source commit and byte-equal to the blob committed there, because an identity naming a commit the content is not from would be false. An input with uncommitted changes is refused.
- id: FR-036
  path: spec/functional/FR-036-organisation-export-template.md
  role: examined
  excerpt: 'The template receives the identity of FR-034 and the approvals of FR-038 as pandoc metadata variables from `in/metadata.yaml`: `title`, `document-id`, `version`, `release`, `commit`, `date` and `approvals`. A variable that has no value is absent.'
- id: FR-034-AC-7
  path: spec/functional/FR-034-document-identity.md
  role: examined
  excerpt: A fixture whose title is `--output=/tmp/x` exports with that text as the title and no change to the arguments of any request; the identity values appear in `in/metadata.yaml` only.
- id: EA-FR-041
  path: engineering-assurance:spec/functional/FR-041-signed-tag-approval-of-a-change-assurance-record.md
  role: context_only
  excerpt: 'One line of JSON on stdout ... members: status; tag, the tag name the command resolved, and record_id and record_digest as requested; trust; reason (unverifiable only); signature, the FR-036 verdict for the tag; on approved only: approver, meaning, signature_kind; binding.'
```

## Verdict

Changes required (FND-001 high). The JSON-input refusal, the 16 MiB template bound, the no-fallback rule, the 256-input bundle bound and the clock-free date rules have their failure paths specified in FR-040.
