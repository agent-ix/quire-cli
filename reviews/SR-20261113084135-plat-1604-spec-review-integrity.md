---
id: "SR-20261113084135"
title: "PLAT-1604 quire-cli PR189 spec-review/integrity"
type: SpecReview
analysis: integrity
scope: "agent-ix/quire-cli PR189 head 1a40cc4; spec/functional/FR-007, FR-031, FR-032, FR-033, FR-042, spec/non-functional/NFR-010, NFR-011, spec/stakeholder/StR-004, spec/usecase/US-009, spec/spec.md"
review_set: subset
---

## Summary

Ticket: PLAT-1604. Integrity and consistency of FR-042 against FR-007, FR-031..FR-040, NFR-010 and NFR-011. Exit codes agree with FR-007: a missing mmdr exits 3 and a diagram failure exits 4. FR-031 section C and FR-031-AC-6 are cleanly replaced and do not contradict merged FR-034..FR-040. Five findings: the count-mismatch row of FR-033 can only be classified by reading pandoc's stderr text, which FR-007 forbids; FR-032's field table was not updated for the third tool; NFR-011's PDF image-bytes measure names a tool that reports no bytes; the alt-text ordinal is ambiguous in a bundle; and new ACs are numbered out of order.

## Findings

| ID | Severity | Summary | Refs | Escape Cause |
| --- | --- | --- | --- | --- |
| FND-001 | medium | FR-033 classifies a count mismatch as a Diagram failure, with a diagnostic that names both counts. The mismatch is raised by the Lua filter inside pandoc, so producer-execute reports only a pandoc request in state `failed` with a nonzero exit, the same as any other pandoc failure. quire can tell a mismatch apart only by parsing pandoc's stderr, and FR-007 requires outcomes derived from typed errors, "never diagnostic text". Fix: classify a mismatch as a Converter failure (exit 4, unchanged) whose retained stderr carries the filter's message with both counts. Alternatively, have the filter write a declared output that quire reads. | spec/functional/FR-033-export-errors.md:29 | wrong-requirement |
| FND-002 | low | FR-032's request table still says `producer` is `pandoc` or `typst`, `outputs` is role `document`, and the capability root holds only `in/`, `typ/` and `out/`. Its section A still says "Both requests of one export SHALL share one capability root". FR-042 adds `in/diagrams/`, `in/diagrams.lua`, `img/` and `typ/img/`, a `diagram` role and N more requests. A reader of FR-032 alone sees a closed table that FR-042 contradicts. Fix: widen those rows and the sentence, or make each row point to FR-042. | spec/functional/FR-032-export-converter-execution.md:67-70 | wrong-requirement |
| FND-003 | low | For PDF, the extracted structure takes "the bytes of its PNG (... the image of `pdfimages -list` for PDF)". `pdfimages -list` prints only metadata, never bytes. Typst also re-encodes the raster into the PDF, so the mmdr PNG bytes do not exist in the PDF at all. Fix: for PDF, compare the bytes extracted with `pdfimages -png`, or the image stream, and say so. | spec/non-functional/NFR-011-export-reproducibility.md:37 | wrong-requirement |
| FND-004 | low | The alt text is `Mermaid diagram <ordinal>`, but a bundle has two ordinals: the document ordinal and the running export ordinal `<k>`. The text does not say which one the alt text uses. Fix: name it (the document ordinal, to match the diagnostic). | spec/functional/FR-042-export-mermaid-diagrams.md:138-141 | wrong-requirement |
| FND-005 | low | New ACs are inserted out of numeric order: FR-032-AC-8 comes before AC-7, FR-033-AC-11 before AC-10, and NFR-011-AC-5 before AC-4. Fix: append them in order. | spec/functional/FR-032-export-converter-execution.md:114 | wrong-requirement |

## Examined scope

```yaml
scope:
- id: FR-033#diagram-failure-row
  path: spec/functional/FR-033-export-errors.md
  role: examined
  excerpt: '| Diagram failure | An `mmdr` request for a Mermaid block ends in a state other than `completed`,
    its PNG is absent or empty, or pandoc and `quire` count different numbers of Mermaid blocks ([FR-042](./FR-042-export-mermaid-diagrams.md)).
    | 4 | Names the block by document path, ordinal, line number and section heading, then the state,
    the exit code if any and the stderr; for a count mismatch, both counts. |'
- id: FR-033-AC-11
  path: spec/functional/FR-033-export-errors.md
  role: examined
  excerpt: A failing `mmdr` for one block is a diagram failure (exit 4) naming that block, and an absent
    `mmdr` for a document with a Mermaid block is a missing converter (exit 3), as FR-042-AC-4 and FR-042-AC-6
    define; neither leaves a file at `--output`.
- id: FR-007-AC-9
  path: spec/functional/FR-007-exit-codes.md
  role: examined
  excerpt: A converter failure of `quire export` (including a failed Mermaid diagram render) exits 4 and
    a missing converter (including `mmdr`) exits 3, as FR-033 defines.
- id: FR-032#field-table
  path: spec/functional/FR-032-export-converter-execution.md
  role: examined
  excerpt: '| `producer` | `name` `pandoc` or `typst`; `executable` the canonical absolute path (below);
    `version` the literal `unreported`. ... |

    | `capabilityRoot` | A fresh absolute temporary directory holding `in/` (the Markdown, `in/document.md`,
    with `in/metadata.yaml` and the template copy), `typ/` (the Typst source, PDF only) and `out/` (the
    output, and the tool''s home and temporary directories). |'
- id: FR-032#A-mmdr
  path: spec/functional/FR-032-export-converter-execution.md
  role: examined
  excerpt: A Markdown input with Mermaid blocks SHALL first make the `mmdr` requests of [FR-042](./FR-042-export-mermaid-diagrams.md),
    which add `--lua-filter` and the diagram files to the pandoc request there defined. The pandoc request
    is made only if all of them completed.
- id: FR-032-AC-8
  path: spec/functional/FR-032-export-converter-execution.md
  role: examined
  excerpt: The requests of an export with Mermaid blocks are those of FR-042-AC-2 and FR-042-AC-3; an
    export without Mermaid blocks makes no `mmdr` request and its pandoc request carries no `--lua-filter`.
- id: FR-031#C
  path: spec/functional/FR-031-export-subcommand.md
  role: examined
  excerpt: A fenced code block whose language tag is `mermaid` SHALL appear in the PDF or DOCX output
    as a rendered image, not as source text. The rendering, the block identification and the failures
    are specified by [FR-042](./FR-042-export-mermaid-diagrams.md).
- id: FR-031-AC-6
  path: spec/functional/FR-031-export-subcommand.md
  role: examined
  excerpt: A fixture with a `mermaid` block exports with an image in the output and the block's source
    text absent, as FR-042-AC-1 defines; the converter input `in/document.md` equals the bytes of the
    committed body after the frontmatter. A fixture with a code block of another language exports with
    that block's text present and no image.
- id: NFR-011#extracted-structure
  path: spec/non-functional/NFR-011-export-reproducibility.md
  role: examined
  excerpt: the set of word-processor fields in DOCX, and for a Mermaid diagram its position in the sequence
    of blocks, its pixel width and height, and the bytes of its PNG (the `word/media` part for DOCX, the
    image of `pdfimages -list` for PDF).
- id: NFR-011-AC-5
  path: spec/non-functional/NFR-011-export-reproducibility.md
  role: examined
  excerpt: Exporting a fixture with two Mermaid diagrams twice, with the clock and `TZ` changed, yields
    equal extracted text and structure, with equal image count, pixel dimensions and PNG bytes.
- id: FR-042#C-filter
  path: spec/functional/FR-042-export-mermaid-diagrams.md
  role: examined
  excerpt: The filter replaces the k-th code block that pandoc reads with the class `mermaid` by one image
    of `<k>.png` whose alternative text is `Mermaid diagram <ordinal>` followed by the section heading
    when there is one. Pandoc, not `quire`, recognises the blocks in the Markdown.
- id: FR-042-AC-9
  path: spec/functional/FR-042-export-mermaid-diagrams.md
  role: examined
  excerpt: A bundle of two fixtures with one block and two blocks yields three images in input order,
    and a failure of the second document's first block names that document and ordinal 1.
- id: FR-042#D-missing
  path: spec/functional/FR-042-export-mermaid-diagrams.md
  role: examined
  excerpt: If `mmdr` is not found, or is not an executable regular file after resolution, and the input
    holds at least one `mermaid` block, then the export SHALL fail as a missing converter (exit 3, [FR-007](./FR-007-exit-codes.md))
    before any request is made, naming `mmdr` and `--mmdr`.
- id: US-009-EX-4
  path: spec/usecase/US-009-export-document-to-pdf-docx.md
  role: examined
  excerpt: '**Given** an artifact with a Mermaid architecture diagram, **When** the reviewer exports it
    to `pdf` and to `docx`, **Then** each file shows the diagram as an image, not its source text, as
    specified by FR-042-AC-1.'
- id: US-009-EX-5
  path: spec/usecase/US-009-export-document-to-pdf-docx.md
  role: examined
  excerpt: '**Given** an artifact whose second Mermaid block has a syntax error, **When** the reviewer
    runs the export, **Then** the command names the block by document, ordinal and line, and writes no
    output file, as specified by FR-042-AC-4.'
```

## Verdict

FAIL (medium). FND-001 needs a text change to FR-033 so classification does not depend on diagnostic text. The rest are low text fixes. No contradiction with the merged FR-034..FR-040 was found.

## New findings (disposition pass 1)

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-006 | low | This was introduced by the fix. FR-042 now requires that "the default template of FR-035 SHALL carry a Typst rule that scales an image wider than the text area down". FR-035, which owns the default template's content (its section C Layout), is not amended. Someone implementing the template from FR-035 alone would miss the rule. Fix: add the image-scaling rule to FR-035 section C, with an AC there, and keep only a reference in FR-042. | spec/functional/FR-042-export-mermaid-diagrams.md:195-201; spec/functional/FR-035-default-export-template.md:76-81 |

## Dispositions

| FND | outcome | reason |
| --- | --- | --- |
| FND-001 | fixed | - |
| FND-002 | fixed | - |
| FND-003 | fixed | - |
| FND-004 | fixed | - |
| FND-005 | fixed | - |
