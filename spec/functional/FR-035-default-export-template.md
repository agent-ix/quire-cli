---
id: FR-035
title: "Default export template: title page, contents, headers, footers, page numbers"
type: FR
relationships:
  - target: "ix://agent-ix/quire-cli/spec/functional/FR-042"
    type: "references"
  - target: "ix://agent-ix/quire-cli/spec/stakeholder/StR-004"
    type: "implements"
    cardinality: "1:1"
  - target: "ix://agent-ix/quire-cli/spec/usecase/US-010"
    type: "implements"
  - target: "ix://agent-ix/quire-cli/spec/functional/FR-031"
    type: "extends"
  - target: "ix://agent-ix/quire-cli/spec/functional/FR-034"
    type: "consumes"
  - target: "ix://agent-ix/quire-cli/spec/functional/FR-032"
    type: "references"
---
# FR-035: Default export template: title page, contents, headers, footers, page numbers

## Description

`quire export` of a Markdown input SHALL generate the standard parts of a
controlled copy as Markdown input to pandoc, in the same way for PDF and DOCX.

When it gets no `--template`, `quire export` SHALL apply a default template that
supplies page setup, styles, headers, footers and page numbers. Neither the
generated parts nor the default template is stored in this repository or its
package: the export code generates both at export time into the capability root
([NFR-010](../non-functional/NFR-010-export-single-converter-single-path.md)).

This split exists because pandoc takes only styles, page setup, headers and footers
from a DOCX reference document and a template only wraps the body; content such as
a title page, a contents list or a table is document body, so it is input text.

## Behavior

### A. Generated parts

The command SHALL generate these parts, in these places, for every Markdown export
whatever the template:

| Part | File | Position | Content |
| --- | --- | --- | --- |
| Front matter | `in/front.md` | Before the document | The title page, a page break, the table of contents, a page break. |
| Separators | `in/separator-<n>.md` | Before document n of a bundle | See [FR-039](./FR-039-export-bundle.md). |
| Approval | `in/approval.md` | After the document | See [FR-038](./FR-038-approval-block.md); present only with `--approval`. |

- The title page SHALL hold the title, then the identity fields of
  [FR-034](./FR-034-document-identity.md) as labelled lines (Document id, Version
  when present, Release when present, Source commit, Date), and no text of the
  document.
- The table of contents SHALL list the headings of level 1 to 3 of the document
  and the generated headings, with page numbers in PDF.
- The command SHALL write each page break as a raw block of the output format,
  `{=typst}` for PDF and `{=openxml}` for DOCX, never taken from input text.
- The command SHALL write the table of contents likewise as a raw block: a Typst
  `outline` of depth 3 for PDF and a TOC field for DOCX.
- The command SHALL pass the pandoc inputs in the order front matter, document
  (or separator and document pairs), approval, and SHALL NOT pass
  `--table-of-contents`.

### B. Headers, footers and page numbers

- The default template SHALL give every page after the title page a header holding
  the document id and the Version (or Release when no Version) and a footer
  holding the first 12 characters of the source commit hash and the page number
  in the form `Page N of M`. The title page carries no header, footer or page
  number.
- For PDF the default template SHALL be a Typst template that reads the
  `quire-export` metadata. For DOCX the command SHALL generate the reference
  document per export, with the identity text in its header and footer written
  as XML-escaped literal text, a different first page without header or footer,
  and the page number and page count as the word-processor fields `PAGE` and
  `NUMPAGES`, so they stay correct when the document is edited.

### C. Layout

- The default template SHALL fix the page size (A4), margins, fonts and heading
  styles inside the generated template, so they do not depend on the host.
- The default template SHALL use only fonts that the converter bundles or declares
  as generic families, and SHALL add no image of its own.
- The default template SHALL carry a Typst rule that scales an image wider than
  the text area down to it and does not enlarge a smaller image, so that a
  rendered diagram ([FR-042](./FR-042-export-mermaid-diagrams.md)) fits the page.

## Acceptance Criteria

| ID | Criteria | Verification |
| --- | --- | --- |
| FR-035-AC-1 | A PDF export of a committed fixture with no `--template` yields page 1 text with the title and the six identity fields as FR-034 states and no document text, and the text of page 2 onward holds the document. Extracted with `pdftotext` per page. | Test |
| FR-035-AC-2 | The same DOCX export yields `word/document.xml` text in which the title and the identity lines precede the first page-break element and no document text precedes that break. | Test |
| FR-035-AC-3 | The PDF holds a table of contents whose entries equal the fixture's level 1 to 3 headings in order, and the DOCX holds a TOC field. | Test |
| FR-035-AC-4 | PDF pages after the first hold a header with the document id and version, and a footer with the 12-character commit prefix and `Page N of M` for the page's own N and the total M; page 1 holds no `Page 1 of`. | Test |
| FR-035-AC-5 | The DOCX has a `word/header*.xml` part holding the document id and version and a `word/footer*.xml` part holding the `PAGE` and `NUMPAGES` fields and the 12-character commit prefix, a first-page-different setting in `word/document.xml`, and no header or footer text on that first page. | Test |
| FR-035-AC-6 | The repository tree and the package file list hold no `.typ`, `.docx` or `.dotx` file for the default template or the generated parts, and these reach the converter only through the capability root. | Inspection |
| FR-035-AC-7 | A fixture with no headings still exports, with an empty table of contents, and the title page is unchanged. | Test |
| FR-035-AC-8 | The pandoc request lists `in/front.md` before the document input and holds no `--table-of-contents`; the page-break and contents raw blocks in `in/front.md` equal the output format and no raw block elsewhere in the generated Markdown comes from input text. | Test |
| FR-035-AC-9 | In a PDF of a fixture holding one diagram wider and one narrower than the text area, the pixel width over the horizontal resolution of `pdfimages -list` does not exceed the text area for the first and equals the natural size for the second. | Test |

## Dependencies

- **Upstream**: [FR-031](./FR-031-export-subcommand.md), [FR-032](./FR-032-export-converter-execution.md), [FR-034](./FR-034-document-identity.md).
- **Downstream**: [FR-036](./FR-036-organisation-export-template.md), [FR-038](./FR-038-approval-block.md), [FR-039](./FR-039-export-bundle.md).
