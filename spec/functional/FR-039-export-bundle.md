---
id: FR-039
title: "Export several documents as one bundle"
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
---
# FR-039: Export several documents as one bundle

## Description

`quire export` SHALL accept more than one Markdown `<INPUT>` and write them as one
output file, in command-line order, for a package such as a release dossier.

```text
quire export <INPUT> <INPUT>... --to pdf|docx --output <PATH>
             --bundle-title <TEXT> --bundle-id <ID> [--release <TAG>]
```

## Behavior

### A. Identity and validation

- The command SHALL validate every input as [FR-031](./FR-031-export-subcommand.md)
  section A requires and SHALL give it a document identity as
  [FR-034](./FR-034-document-identity.md) defines, from the one source repository
  and the one `--source-rev`, and SHALL refuse inputs from different repositories.
- The bundle identity SHALL be its title (`--bundle-title`), its id
  (`--bundle-id`), `--release` when given, the source commit hash and the Date,
  which is the source commit date as for one document. The bundle has no Version.
- The command SHALL limit a bundle to 256 inputs.

### B. Assembly

The sources are separate pandoc inputs, and pandoc merges the inputs into one
document; the command therefore generates the structure:

- The command SHALL generate `in/front.md` ([FR-035](./FR-035-default-export-template.md))
  as a bundle title page (title, id, Release when given, Source commit, Date),
  a page break, a table listing the documents in command-line order (id, title,
  version or release), a page break, and one table of contents.
- The command SHALL generate `in/separator-<n>.md` for document n, holding a page
  break and a level-1 heading `<id>: <title>` of that document, and SHALL pass
  the pandoc inputs in the order front matter, then each separator followed by
  its `in/document-<n>.md`, then the approval block.
- The command SHALL NOT re-level the headings of a document, SHALL NOT pass
  `--shift-heading-level-by`, and SHALL NOT merge, reorder, deduplicate or edit
  documents. A repeated input appears twice.
- Because the frontmatter of every document is removed and YAML blocks are
  disabled ([FR-034](./FR-034-document-identity.md) section C), no document
  defines metadata, and the per-document identities appear only in the generated
  separators and document list.
- Headers and footers use the bundle id and the bundle's commit prefix, and page
  numbers run across the bundle.
- `--approval` verdicts apply to the bundle as a whole and appear once, after the
  last document.
- The sources SHALL reach the converter as `in/document-1.md` through
  `in/document-<n>.md`; with one input the name stays `in/document.md`
  ([FR-032](./FR-032-export-converter-execution.md)).

## Acceptance Criteria

| ID | Criteria | Verification |
| --- | --- | --- |
| FR-039-AC-1 | Exporting three committed fixtures to PDF yields one file whose extracted text holds the bundle title page, a document list naming the three ids in command-line order, and the three documents in that order, each starting on a new page under a heading `<id>: <title>`. The same export to DOCX yields the three headings in `word/document.xml` in order, each after a page break. | Test |
| FR-039-AC-2 | The table of contents holds the separator headings and the headings of all three documents in order, and the footer `Page N of M` runs across the whole bundle with M equal to the page count. | Test |
| FR-039-AC-3 | A repeated input appears twice; swapping the command-line order swaps the document order. | Test |
| FR-039-AC-4 | The stand-in pandoc request lists `in/front.md`, then `in/separator-1.md`, `in/document-1.md` and so on in that order, each document input equal to its committed body without frontmatter, and a single-input export lists `in/front.md` and `in/document.md`. | Test |
| FR-039-AC-5 | The request holds no `--shift-heading-level-by`, and the heading levels of each document in the output equal those of its source. | Test |
| FR-039-AC-6 | Three fixtures whose frontmatter each define `title` and `id` produce a bundle whose title page shows the `--bundle-title` and `--bundle-id` and whose separators show each fixture's own id and title. | Test |
| FR-039-AC-7 | A bundle without `--bundle-title` or `--bundle-id`, `--bundle-title` with one input, inputs from two repositories, and 257 inputs each exit 3 naming the cause with no converter started. | Test |
| FR-039-AC-8 | One input in the bundle that fails validation or is uncommitted makes the whole export exit 3 naming that file and writes no output. | Test |

## Dependencies

- **Upstream**: [FR-031](./FR-031-export-subcommand.md), [FR-034](./FR-034-document-identity.md), [FR-035](./FR-035-default-export-template.md).
- **Downstream**: [FR-040](./FR-040-export-template-and-identity-errors.md).
