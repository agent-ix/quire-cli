---
id: NFR-011
title: "Same input and template give the same exported document"
type: NFR
quality_attribute: reliability
relationships:
  - target: "ix://agent-ix/quire-cli/spec/non-functional/NFR-010"
    type: "extends"
  - target: "ix://agent-ix/quire-cli/spec/functional/FR-037"
    type: "constrains"
  - target: "ix://agent-ix/quire-cli/spec/functional/FR-035"
    type: "constrains"
  - target: "ix://agent-ix/quire-cli/spec/functional/FR-036"
    type: "constrains"
  - target: "ix://agent-ix/quire-cli/spec/functional/FR-042"
    type: "constrains"
---
# NFR-011: Same input and template give the same exported document

## Statement

For one committed input, one source commit, one template (or the default), the
same approval verdict files and the same converter and `mmdr` versions, `quire export` SHALL
produce documents whose extracted text and extracted structure are equal,
whatever the wall-clock time, time zone, host name string, user, working
directory or capability-root path of the run. Byte equality of the file is not
required, and container details such as compression order are not part of the
extracted text or structure.

## Definitions

- **Extracted text**: for PDF, the output of `pdftotext -layout` page by page;
  for DOCX, the concatenated text of `word/document.xml`, `word/header*.xml`,
  `word/footer*.xml` in part-name order.
- **Extracted structure**: the sequence of heading levels and heading texts, the
  grid of cell texts of every table (rows, columns, text), the page count for PDF,
  the table-of-contents entries, and the set of word-processor fields in DOCX, and for a Mermaid diagram its position in the sequence of blocks, its pixel width and height, and the bytes of its PNG (the `word/media` part for DOCX, the image of `pdfimages -list` for PDF).

## Scope

Applies to Markdown input with the default template and with an organisation
template, and to bundles, on one host with one pair of converter versions and one
set of installed fonts; for a Mermaid diagram, one `mmdr` version. Different converter versions or font sets, including the
system-font fallback of Typst when a template's fonts lack a glyph, may differ and
are outside the property.

## Measurement and Evaluation

| Metric | Target | Threshold | Method |
| --- | --- | --- | --- |
| Differences in extracted text between two exports of the same fixture | 0 | 0 | differential test |
| Differences in extracted structure between two exports of the same fixture | 0 | 0 | differential test |
| Wall-clock-dependent content in the extracted text | 0 | 0 | differential test across a changed clock |

## Acceptance Criteria

| ID | Criteria | Verification |
| --- | --- | --- |
| NFR-011-AC-1 | Exporting one committed fixture twice to each format with real pandoc and typst, once with the process clock and `TZ` changed between runs, yields equal extracted text and equal extracted structure; the test fails naming the missing tool if a converter or extractor is unavailable. | Test |
| NFR-011-AC-2 | The property of NFR-011-AC-1 holds for the default template, a fixture organisation template, a fixture with two approval verdicts, and a bundle of three fixtures. | Test |
| NFR-011-AC-3 | Exporting the same fixture from two different working directories and two different output directories yields equal extracted text and structure. | Test |
| NFR-011-AC-5 | Exporting a fixture with two Mermaid diagrams twice, with the clock and `TZ` changed, yields equal extracted text and structure, with equal image count, pixel dimensions and PNG bytes. | Test |
| NFR-011-AC-4 | A fixture changed by one word yields a different extracted text, so the comparison can fail. | Test |

## Verification

A differential test runs the export twice and compares extracted text and
structure with the extractors above.
