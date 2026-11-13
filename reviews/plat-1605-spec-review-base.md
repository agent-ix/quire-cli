---
id: SR-20261112234817
title: "PLAT-1605 quire-cli PR182 spec-review/base"
type: SpecReview
analysis: base
scope: "agent-ix/quire-cli PR182 head 9194f3a; spec/functional/FR-031,FR-032,FR-034..FR-040, spec/non-functional/NFR-010,NFR-011, spec/stakeholder/StR-004, spec/usecase/US-010"
review_set: subset
---

## Summary

Ticket: PLAT-1605. Base checklist over the PR's FR-034..FR-040, NFR-011, US-010 and the FR-031, FR-032, NFR-010 and StR-004 edits. Checked against the PLAT-1605 acceptance criteria and the merged PLAT-1434 export spec.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | high | For DOCX the default template cannot produce the parts FR-035 section A requires. FR-032 passes the DOCX template only as `--reference-doc`. Pandoc ignores the body content of a reference document and takes only its styles, page setup, headers and footers. Pandoc's DOCX title block is limited to title, subtitle, author, date and abstract, so a title page with labelled Document id, Version, Release and Source commit lines cannot be made. The same applies to the FR-038 approval table after the body and the FR-039 document list table. FR-032 fixes the pandoc arguments (no filter) and FR-039 says the sources reach pandoc unchanged, so no other route is allowed. Fix: state how DOCX gets these parts. One option is for quire to generate an extra Markdown input (for example `in/front.md` and `in/approval.md`) built from the metadata, and for FR-032 to list it. Then add DOCX ACs for the title page and the approval block; today only PDF is tested. | spec/functional/FR-035-default-export-template.md:32-44, spec/functional/FR-032-export-converter-execution.md:62 |
| FND-002 | high | The identity reaches pandoc only through `--metadata-file in/metadata.yaml`. Pandoc's documented precedence is that metadata in the document's own YAML block overrides `--metadata-file` values. The input Markdown reaches pandoc with its frontmatter. So an artifact whose frontmatter has `date:` (SpecReview and other archetypes carry one), `commit:`, `release:`, `approvals:` or `documents:` replaces the commit-derived identity the template prints. The title page would then show a date not taken from the source commit, which breaks the ticket's 'never the wall clock / from the source commit' rule. FR-034-AC-1 and AC-3 would pass on a fixture without those keys. Fix: put the identity under one reserved key (for example `quire-export:`) and refuse an input whose frontmatter defines that key, or strip the frontmatter before conversion. Add an AC with a fixture whose frontmatter carries a conflicting `date`. | spec/functional/FR-034-document-identity.md:62-63, spec/functional/FR-036-organisation-export-template.md:42-46 |
| FND-003 | high | With the FR-032 argument list and unchanged sources, FR-039's bundle layout cannot be produced. Pandoc concatenates several inputs into one document and merges their YAML blocks into one metadata map. Each input's `title` and `id` therefore override one another and the `--metadata-file` bundle title and id (see FND-002). No input's title becomes a top-level heading. No page break is inserted between documents. Heading levels are not shifted to 'nest below' (that needs `--shift-heading-level-by`, which is not in FR-032). Fix: specify the mechanism. One option is for quire to generate a per-document separator input holding the document's title heading and a page break, add `--shift-heading-level-by=1`, and keep the per-document identities only in the `documents` metadata list under a reserved key. | spec/functional/FR-039-export-bundle.md:40-50, spec/functional/FR-032-export-converter-execution.md:62 |
| FND-004 | low | FR-034 says the identity has 'five fields', but its table lists six: Title, Document id, Version, Release, Source commit hash and Date. FR-034-AC-1 and FR-035-AC-1 also say 'five'. State the count consistently, for example 'six fields, of which Version and Release are each optional but not both absent'. | spec/functional/FR-034-document-identity.md:21, :33-41, :72 |
| FND-005 | low | US-010's title, 'Approver receives an identified, templated, signed export', says the export is signed. FR-038 says the export signs nothing and verifies nothing; it presents verdicts. A reader filing a controlled copy could take the title as a claim that the PDF is signed. Rename it, for example to '... export with its approval verdicts'. | spec/usecase/US-010-approver-gets-identified-templated-document.md:3, :25 |

## Examined scope

```yaml
scope:
- id: FR-034
  path: spec/functional/FR-034-document-identity.md
  role: examined
  excerpt: '`quire export` of a Markdown input SHALL assemble a document identity of five fields, and SHALL hand it to the converter as data for the template (FR-035, FR-036). ... The identity values reach the converter only as members of the data file `in/metadata.yaml`, written as JSON string values, never as an argument.'
- id: FR-035
  path: spec/functional/FR-035-default-export-template.md
  role: examined
  excerpt: 'The default template SHALL produce, in this order: 1. Title page: the title, then the identity fields of FR-034 as labelled lines (Document id, Version, Release when present, Source commit, Date), and nothing from the body. ... 2. Table of contents ... 3. Body ...'
- id: FR-035-AC-1
  path: spec/functional/FR-035-default-export-template.md
  role: examined
  excerpt: A PDF export of a committed fixture with no `--template` yields page 1 text with the title and the five identity fields and no body text, and the text of page 2 onward holds the body. Extracted with `pdftotext` per page.
- id: FR-036
  path: spec/functional/FR-036-organisation-export-template.md
  role: examined
  excerpt: 'The template receives the identity of FR-034 and the approvals of FR-038 as pandoc metadata variables from `in/metadata.yaml`: `title`, `document-id`, `version`, `release`, `commit`, `date` and `approvals`. A variable that has no value is absent.'
- id: FR-038
  path: spec/functional/FR-038-approval-block.md
  role: examined
  excerpt: The block is placed after the body, under the heading `Approval`. In the default template (FR-035) it is a table with the columns Person, Time, Meaning, Verdict and Record.
- id: FR-039
  path: spec/functional/FR-039-export-bundle.md
  role: examined
  excerpt: The default template (FR-035) gives the bundle one title page, followed by a document list table (id, title, version or release) in command-line order, one table of contents covering all documents, and then each document starting on a new page under its own title as a top-level heading, with its headings nested below. ... The sources reach the converter unchanged as `in/document-1.md` through `in/document-<n>.md`.
- id: FR-032
  path: spec/functional/FR-032-export-converter-execution.md
  role: examined
  excerpt: 'Pandoc also receives, in this order before the input, `--metadata-file`, `in/metadata.yaml`; `--table-of-contents` and `--toc-depth`, `3`; and `--template`, `in/template.typ` (PDF) or `--reference-doc`, `in/template.docx` (DOCX); the inputs of a bundle are `in/document-1.md` through `in/document-<n>.md` (FR-039). These entries are written by FR-034 through FR-036 and hold no input text. Typst: `compile`, `typ/document.typ`, `out/document.pdf`.'
- id: US-010
  path: spec/usecase/US-010-approver-gets-identified-templated-document.md
  role: examined
  excerpt: As an approver or auditor who keeps controlled documents outside the repository I want every exported document to carry its identity, follow my organisation's layout, show who approved it and when, and come out the same each time So that I can file the PDF or DOCX as a controlled copy and prove which source revision it came from.
- id: FR-031
  path: spec/functional/FR-031-export-subcommand.md
  role: examined
  excerpt: The last two lines of options, and the identity, title page and layout a Markdown input receives, are specified by FR-034 through FR-039; a Markdown `<INPUT>` may repeat for a bundle. ... A Markdown input exported by the commands of those FRs needs a committed source and an identity; JSON inputs carry none.
- id: NFR-010
  path: spec/non-functional/NFR-010-export-single-converter-single-path.md
  role: examined
  excerpt: Export SHALL use pandoc and typst as its only converters ... with no converter binary, template file or reference document placed in this repository or its package. The default template is generated by the export code at export time and an organisation template is the user's local file; neither is stored here.
```

## Verdict

Changes required. Ticket ACs met on paper: there is a default template, an organisation template from a local path, all identity fields (date from the committer date, no wall clock), stability as a differential property over extracted text and structure (NFR-011, with a fail-able control in AC-4), an approval block, bundles, and `quire validate` exits 0 (one new grammar warning on FR-034:54). ID numbering continues FR-033 -> FR-034..FR-040, NFR-010 -> NFR-011 and US-009 -> US-010 with no gaps or collisions. The FR-031, FR-032 and NFR-010 edits do not contradict PLAT-1434 FR-031..FR-033. Public-repo hygiene is clean: no roadmap or unreleased design, no standard text, no compatibility layer (fallback to the default template is forbidden), no pins or digests. NFR-010 is not broken by a generated default template: it is authored export code, not a copied file. Blocking: the specified pandoc mechanism cannot produce the DOCX parts (FND-001); the document's own frontmatter overrides the identity (FND-002); the bundle layout cannot be produced (FND-003).

## New findings (disposition pass 1)

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-006 | medium | The fix commit put the six SR files in spec/reviews/. This repo's SpecReview root is reviews/ at the repository root: merged PR #181 (PLAT-1434) committed reviews/plat-1434-spec-review-*.md, and gap-analysis writes there too. spec/reviews/ holds only three legacy files. Move the six files to reviews/ under the same names, using the updated versions from this disposition pass. | spec/reviews/plat-1605-spec-review-*.md |

## Dispositions

| FND | outcome | reason |
| --- | --- | --- |
| FND-001 | fixed | - |
| FND-002 | fixed | - |
| FND-003 | fixed | - |
| FND-004 | fixed | - |
| FND-005 | fixed | - |
