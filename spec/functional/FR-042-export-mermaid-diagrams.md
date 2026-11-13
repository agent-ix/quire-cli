---
id: FR-042
title: "Render Mermaid diagrams as images in exported documents"
type: FR
relationships:
  - target: "ix://agent-ix/quire-cli/spec/stakeholder/StR-004"
    type: "implements"
    cardinality: "1:1"
  - target: "ix://agent-ix/quire-cli/spec/usecase/US-009"
    type: "implements"
  - target: "ix://agent-ix/quire-cli/spec/functional/FR-031"
    type: "extends"
  - target: "ix://agent-ix/quire-cli/spec/functional/FR-032"
    type: "extends"
  - target: "ix://agent-ix/quire-cli/spec/functional/FR-033"
    type: "references"
  - target: "ix://agent-ix/quire-cli/spec/functional/FR-007"
    type: "references"
  - target: "ix://agent-ix/quire-rs/FR-010"
    type: "consumes"
  - target: "ix://agent-ix/engineering-assurance/FR-019"
    type: "references"
  - target: "ix://agent-ix/engineering-assurance/FR-029"
    type: "references"
---
# FR-042: Render Mermaid diagrams as images in exported documents

## Description

`quire export` SHALL render every fenced code block whose language tag is
`mermaid` in a Markdown input as an image in the PDF or DOCX output, in place of
the block's source text. The renderer is `mmdr`, run as one request of the
`engineering-assurance.producer-execution-request/v1` protocol through
`engineering-assurance producer-execute` for each block, as
[FR-032](./FR-032-export-converter-execution.md) runs pandoc and typst. A block
that cannot be rendered is an export error that names the block; it is never
dropped and never left as source text.

This FR replaces the former rule of FR-031 section C that a `mermaid` block is
an ordinary code block.

## Renderer decision

`mmdr` is the renderer: the `mermaid-rs-renderer` project, a native Rust
Mermaid renderer under the MIT licence that reads Mermaid text and writes SVG
or PNG without a browser or a Node runtime (invoked as
`mmdr -i <file> -o <file> -e png`). The user installs it, as pandoc and typst.

Options weighed, against the constraints that bind export: no network
([NFR-004](../non-functional/NFR-004-no-network.md)), reproducible output
([NFR-011](../non-functional/NFR-011-export-reproducibility.md)), and a
descendant tree that stays inside the `process-group-v1` containment of
[EA FR-019](ix://agent-ix/engineering-assurance/FR-019), which is cooperative
and not adversarial.

| Option | Verdict | Reason |
| --- | --- | --- |
| `mmdr` (native Rust) | Chosen | One executable with no runtime dependency. It spawns no descendant, so containment holds trivially. It has no network need. Its output is a pure function of the diagram text, its version and the installed fonts, which is the carve-out NFR-011 already makes for fonts. |
| `mmdc` (Mermaid CLI) | Rejected | It needs Node and a headless Chromium. Chromium spawns a process tree (zygote, GPU and renderer processes) and uses its own sandbox, which conflicts with a process-group-only contract; the first run downloads a browser, which needs the network; it executes the diagram's text in a browser engine; and its output depends on the browser build and its fonts. |
| A pandoc filter or a Typst package | Rejected | The known Mermaid pandoc filters call `mmdc`, so they add its runtime and a process launched outside producer-execute. Typst Mermaid packages are fetched from a package registry at compile time, which needs the network, or embed a JavaScript engine of their own. Neither is a renderer; each moves the problem. |
| Graphviz or D2 | Rejected | They do not read Mermaid syntax. |

Trade-off recorded as the decision: `mmdr` is young. It supports a subset of
the Mermaid diagram types, and its layout is not identical to the Mermaid
reference renderer's. A diagram that uses syntax `mmdr` does not support is a
render failure and an export error ([FR-033](./FR-033-export-errors.md)); there
is no second renderer to fall back to. Choosing `mmdc` instead would trade
those limits for a browser and a Node runtime, with the containment and
network exposure above. This choice is the owner's to confirm.

`mmdr` is installed by the user and found on PATH or through `--mmdr`. No
renderer binary, font, theme or template is added to this repository or its
package ([NFR-010](../non-functional/NFR-010-export-single-converter-single-path.md)).

## Inputs

- `--mmdr <PATH>`: overrides the PATH lookup of the `mmdr` executable, as
  `--pandoc` does for pandoc. The flag is accepted for every input and used
  only when a Markdown input holds at least one `mermaid` block.
- The Markdown input or inputs of [FR-031](./FR-031-export-subcommand.md). A
  JSON input holds no Mermaid block and starts no renderer.

## Behavior

### A. Finding the blocks

- `quire` SHALL find the blocks with the fenced-block query of quire-rs
  ([quire-rs FR-010](ix://agent-ix/quire-rs/FR-010) `extract_diagrams`, filtered
  to the language `mermaid`), over the committed content that
  [FR-034](./FR-034-document-identity.md) section A validates and converts. It
  SHALL NOT scan the Markdown text for fences itself, because Markdown parsing
  stays in quire-rs ([StR-004](../stakeholder/StR-004-thin-boundary-over-quire-rs.md)).
- Each block is identified by the input path, its ordinal among the `mermaid`
  blocks of that document (1-based, in document order), its first line number
  in the committed file (1-based, counting the frontmatter), and the heading of
  the section that holds it, when there is one. The line number and the
  section come from the quire-rs record; the upstream requirement is stated in
  Dependencies.
- A document with no `mermaid` block starts no renderer and needs no `mmdr`. Its
  requests and output are those of [FR-031](./FR-031-export-subcommand.md)
  and [FR-032](./FR-032-export-converter-execution.md) without this FR.

### B. Rendering

- For each block, in document order, `quire` SHALL write the block source,
  byte for byte as the quire-rs record carries it, to `in/diagrams/<k>.mmd`,
  where `<k>` is the 1-based ordinal over all blocks of the export (the
  document ordinal for a single input, the running ordinal across the inputs of
  a bundle, [FR-039](./FR-039-export-bundle.md)), and SHALL make one `mmdr`
  request that writes the PNG of that block.
- Each `mmdr` request SHALL be the complete field set of the request table in
  [FR-032](./FR-032-export-converter-execution.md) with these definitions, and
  the fields of that table not named here apply unchanged: `producer.name`
  `mmdr`; `arguments` `-i`, `in/diagrams/<k>.mmd`, `-o`, the image path, `-e`,
  `png`, in that order, where the image path is `img/<k>.png` for DOCX and
  `typ/img/<k>.png` for PDF; `outputs` one entry, role `diagram`, that same
  path, `required` true; `environment` as FR-032; `budget`
  `timeoutMillis` 60000, `maxOutputArtifacts` 1, `maxConcurrency` 1 and the
  other bounds of FR-032; `containment` and `response` as FR-032. The requests
  share the one capability root of the export, and run before the pandoc
  request, one at a time.
- A run SHALL stop at the first block whose request does not complete. No later
  block is rendered, no pandoc or typst request is made, and the error is the
  one of section D.
- The image format is PNG for both PDF and DOCX. SVG is not used: Word embeds
  SVG only with a raster fallback, and pandoc makes that fallback by launching
  a rasterizer of its own, which would be a process outside producer-execute.
  One raster format needs no further tool for either output.

### C. Placing the images

- The Markdown handed to the converter stays the committed body, byte for byte
  ([FR-031](./FR-031-export-subcommand.md) section A); `quire` does not edit it.
  `quire` SHALL instead write a pandoc Lua filter, `in/diagrams.lua`, generated
  at export time and stored nowhere, and SHALL add `--lua-filter in/diagrams.lua`
  to the pandoc request, after the metadata and template arguments and before
  the input files.
- The filter replaces the k-th code block that pandoc reads with the class
  `mermaid` by one image of `<k>.png` whose alternative text is
  `Mermaid diagram <ordinal>` followed by the section heading when there is one.
  Pandoc, not `quire`, recognises the blocks in the Markdown.
- If pandoc reads a number of `mermaid` code blocks different from the number
  `quire` rendered, the filter SHALL stop pandoc with an error that states both
  numbers; the export fails as [FR-033](./FR-033-export-errors.md) defines. A block
  one reader recognises and the other does not, such as a block with attribute
  syntax, therefore fails the export and is never left as source text.
- The image path in the filter is the path from the capability root for DOCX
  (`img/<k>.png`) and the path from the Typst source directory for PDF, where
  the images are written under `typ/img/`. Each request names the path that its
  tool resolves.
- An image wider than the text area SHALL be scaled down to fit it, and an image
  SHALL NOT be scaled up. The default and organisation templates of
  [FR-035](./FR-035-default-export-template.md) and
  [FR-036](./FR-036-organisation-export-template.md) apply to the page that holds
  the image as to any other content.
- The `--sandbox` argument of FR-032 stays in effect for every export. If the
  installed pandoc cannot read the image files under it, the export fails as
  a converter failure with pandoc's message; `quire` SHALL NOT remove
  `--sandbox` to make it pass.

### D. Failure

- A block whose request ends in any state other than `completed`, or whose
  output PNG is absent or empty, SHALL fail the export. The diagnostic names the
  document path, the ordinal, the line number, the section heading when there
  is one, the state, the exit code if any, and `mmdr`'s stderr as the executor
  retains it, so the author can find the block. The errors, exit codes and the
  rule that no partial `--output` is left are those of
  [FR-033](./FR-033-export-errors.md).
- If `mmdr` is not found, or is not an executable regular file after
  resolution, and the input holds at least one `mermaid` block, then the
  export SHALL fail as a missing converter (exit 3,
  [FR-007](./FR-007-exit-codes.md)) before any request is made, naming `mmdr`
  and `--mmdr`. The check is made with the other tools, so a PDF export with
  Typst or `mmdr` absent fails before pandoc runs.
- The command SHALL NOT drop a block, print its source text in its place,
  continue after a failed block, substitute a placeholder image or retry.

## Acceptance Criteria

| ID | Criteria | Verification |
| --- | --- | --- |
| FR-042-AC-1 | With real pandoc, typst and mmdr, a committed fixture with two `mermaid` blocks of supported diagram types exports to DOCX with exactly two PNG parts under `word/media` referenced by `w:drawing` elements, and to PDF with exactly two images by `pdfimages -list`; the fixture's `mermaid` source text appears in neither output's extracted text. The test fails naming the missing tool if one is unavailable. | Test |
| FR-042-AC-2 | With stand-in executables, the export makes one `mmdr` request per block in document order, then the pandoc request, then (PDF) the typst request, and each `mmdr` request equals the field definitions of this FR, validates against `engineering-assurance producer-execute --schema request` and names a canonical `producer.executable`. | Test |
| FR-042-AC-3 | The pandoc request of an export with diagrams carries `--lua-filter in/diagrams.lua` after the template argument and before `in/front.md`; the converter input `in/document.md` equals the committed body bytes; the working directory, input directory and output directory hold no file other than the input and `--output` afterwards. | Test |
| FR-042-AC-4 | With a stand-in `mmdr` that exits 1 for the second block, the export exits 4, writes no `--output`, makes no pandoc request and no `mmdr` request for a third block, and the diagnostic holds the document path, ordinal 2, the line number of that block's opening fence in the committed file, its section heading and `mmdr`'s stderr. | Test |
| FR-042-AC-5 | A stand-in `mmdr` that exits 0 without writing the PNG, and one that writes an empty PNG, each fail the export as exit 4 naming the block. A fixture block of Mermaid syntax that real `mmdr` rejects fails the export the same way. | Test |
| FR-042-AC-6 | With `mmdr` absent and a fixture holding a `mermaid` block, the export exits 3 naming `mmdr` and `--mmdr` and makes no request; with `mmdr` absent and a fixture holding no `mermaid` block, the export succeeds and makes no `mmdr` request. | Test |
| FR-042-AC-7 | A fixture whose `mermaid` block is written in a way quire-rs recognises and pandoc does not (or the reverse), such as an attribute-style info string, fails the export with both block counts in the diagnostic; the output of no run holds the source text as a code block. | Test |
| FR-042-AC-8 | A fixture holding a `mermaid` block inside a longer backtick fence of another language, and a `~~~mermaid` block, export with images for the second only and the first printed as code. | Test |
| FR-042-AC-9 | A bundle of two fixtures with one block and two blocks yields three images in input order, and a failure of the second document's first block names that document and ordinal 1. | Test |
| FR-042-AC-10 | The image is no wider than the text area in the PDF and the DOCX, and a diagram narrower than the text area is not enlarged, checked on a fixture with one wide and one narrow diagram. | Demonstration |
| FR-042-AC-11 | A JSON input with `--kind table` whose cell text contains a fenced `mermaid` block starts no `mmdr` request. | Test |
| FR-042-AC-12 | Review of the export code finds the `mermaid` blocks found only through the quire-rs query and no Markdown scan for fences in quire-cli. | Inspection |

## Dependencies

- **Upstream**: [FR-031](./FR-031-export-subcommand.md), [FR-032](./FR-032-export-converter-execution.md), [FR-033](./FR-033-export-errors.md), [FR-007](./FR-007-exit-codes.md).
- **Upstream, quire-rs requirement needed**: `extract_diagrams` of [quire-rs FR-010](ix://agent-ix/quire-rs/FR-010) returns a 0-based index among all fenced blocks, the language, the source and the section heading, but no line position. quire-rs must add to each fenced-block record the 1-based line of the opening fence in the full file (frontmatter included) and the language-filtered ordinal, with the fence recognition of its FR-007 and FR-011 CR-004 (backtick and tilde, matching close). This FR depends on that addition; quire-cli SHALL NOT compute positions itself.
- **Upstream, engineering-assurance**: none. Neither network access nor filesystem confinement is a producer-execute guarantee ([EA FR-019](ix://agent-ix/engineering-assurance/FR-019) `process-group-v1`); the no-network property of `mmdr` rests on the renderer, and is checked by [NFR-010](../non-functional/NFR-010-export-single-converter-single-path.md).
- **Downstream**: [NFR-011](../non-functional/NFR-011-export-reproducibility.md).
