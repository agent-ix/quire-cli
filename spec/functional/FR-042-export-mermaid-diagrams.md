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
    type: "implements"
  - target: "ix://agent-ix/quire-cli/spec/functional/FR-035"
    type: "references"
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

Decision: `mmdr` is the renderer. It is the `mermaid-rs-renderer` project, a
native Rust Mermaid renderer under the MIT licence that reads Mermaid text and
writes SVG or PNG without a browser or a Node runtime (invoked as
`mmdr -i <file> -o <file> -e png`). The user installs it, as pandoc and typst.

Options weighed, against the constraints that bind export: no network
([NFR-004](../non-functional/NFR-004-no-network.md)), reproducible output
([NFR-011](../non-functional/NFR-011-export-reproducibility.md)), and a
descendant tree that stays inside the `process-group-v1` containment of
[EA FR-019](ix://agent-ix/engineering-assurance/FR-019), which is cooperative
and not adversarial.

| Option | Verdict | Reason |
| --- | --- | --- |
| `mmdr` (native Rust) | Chosen | One executable with no runtime dependency. It starts no child process and has no network client, so containment holds trivially. |
| `mmdc` (Mermaid CLI) | Rejected | It needs Node and a headless Chromium. Installing it downloads a browser, which needs the network; Chromium spawns a process tree (zygote, GPU and renderer processes) and uses its own sandbox, which conflicts with a process-group-only contract; it executes the diagram's text in a browser engine; and its output depends on the browser build and its fonts. |
| A Typst package for Mermaid | Rejected | The Typst Universe `mmdr` package is a WebAssembly build of the same renderer and needs no network once installed locally, but Typst produces PDF only, so DOCX would still need another route. |
| A pandoc filter for Mermaid | Rejected | The known Mermaid pandoc filters call `mmdc`, so they add its runtime and a process launched outside producer-execute. |
| Graphviz or D2 | Rejected | They do not read Mermaid syntax. |

Assumption, not verified: the output of `mmdr` is a function of the diagram
text, the `mmdr` version and the installed fonts. `mmdr` loads system fonts and
writes a font cache under `$HOME`, which is empty for every request
([FR-032](./FR-032-export-converter-execution.md)). [NFR-011-AC-5](../non-functional/NFR-011-export-reproducibility.md)
checks the assumption.

Known risks, accepted with this decision:

- `mmdr` is in early development; its project says its output may not yet match
  the Mermaid reference renderer's in all cases, and its layout differs.
- Its syntax validation is deliberately narrow, and its parser skips lines it does
  not handle. A construct it does not support can therefore render an
  incomplete diagram and exit 0. `quire` detects only what `mmdr` reports as a
  failure; it cannot detect a diagram that `mmdr` accepts but draws incompletely.
  The export is not a proof that a diagram is complete, and the author checks the
  rendered document.
- There is no second renderer to fall back to. A diagram `mmdr` rejects is an
  export error ([FR-033](./FR-033-export-errors.md)).

Guaranteed diagram types: the export guarantees the flowchart (`flowchart` and
`graph`), sequence (`sequenceDiagram`), class (`classDiagram`) and state
(`stateDiagram-v2`) diagrams. The fixtures of the acceptance criteria use these
types and no others. Another type that `mmdr` supports is rendered on the same
terms with no guarantee that it is complete.

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
- `quire` SHALL identify each block by the input path, its ordinal among the
  `mermaid` blocks of that document (1-based, in document order, the `index` of
  the quire-rs record plus one), the line number of its opening fence in the
  committed file (1-based, counting the frontmatter), and the heading of the
  section that holds it, when there is one. The line number is the one the
  quire-rs ticket PLAT-1682 adds (Dependencies); every diagnostic of this FR
  carries it.
- A document with no `mermaid` block starts no renderer and needs no `mmdr`. Its
  requests and output are those of [FR-031](./FR-031-export-subcommand.md)
  and [FR-032](./FR-032-export-converter-execution.md) without this FR, except
  that every Markdown input, with or without a block, gets the generated filter
  of section C with a block count of 0 or more. The filter is what fails the
  export when pandoc reads a `mermaid` block that quire-rs did not find, so a
  block is never left as source text because quire-rs missed it. A JSON input
  gets no filter and starts no renderer.

### B. Rendering

- Before the first `mmdr` request, `quire` SHALL create `in/diagrams/` and the
  image directory in the capability root: `img/` for DOCX and `typ/img/` for
  PDF. The producer does not create output parents
  ([EA FR-019](ix://agent-ix/engineering-assurance/FR-019)).
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
- `quire` SHALL request PNG for both PDF and DOCX. SVG is not used: Word embeds
  SVG only with a raster fallback, and pandoc makes that fallback by launching
  a rasterizer of its own, which would be a process outside producer-execute.
  One raster format needs no further tool for either output.

### C. Placing the images

- The Markdown handed to the converter stays the committed body, byte for byte
  ([FR-031](./FR-031-export-subcommand.md) section A); `quire` does not edit it.
  `quire` SHALL instead write a pandoc Lua filter, `in/diagrams.lua`, generated
  at export time and stored nowhere, for every Markdown input (with a block
  count of 0 when quire-rs found no block), and SHALL add
  `--lua-filter in/diagrams.lua` to the pandoc request, after the metadata and
  template arguments and before the input files. The filter source SHALL be fixed text plus the integer count
  of blocks and the image directory name, and SHALL contain no text taken from a
  document. Document text reaches the filter only as data files: the block
  source `in/diagrams/<k>.mmd` and the alternative text `in/diagrams/<k>.alt`,
  which `quire` writes and the filter reads as data. The filter runs outside
  pandoc's sandbox, so splicing document text into its source would be code
  execution taken from the document.
- The alternative text is `Mermaid diagram <n>` with `<n>` the ordinal of the
  block within its own document (the ordinal of the diagnostics), followed by
  the separator `: ` and the section heading when there is one, for example
  `Mermaid diagram 2: Data flow`.
- For the k-th code block that pandoc reads with the class `mermaid`, the filter
  SHALL: compare the block text with the content of `in/diagrams/<k>.mmd`, after
  converting line endings to LF in both, and fail pandoc on any difference,
  naming `k`; read the image file `<k>.png` from the image directory (`img/`
  for DOCX, `typ/img/` for PDF) and fail pandoc, naming `k`, if it cannot be
  read or is empty; for DOCX insert the bytes into the media bag under the name
  `img/<k>.png` with `pandoc.mediabag.insert`; and replace the block by one
  paragraph holding one image named `img/<k>.png`, with the alternative text.
  The image is named `img/<k>.png` for both formats: for DOCX that is the
  media-bag name and the path from the capability root; for PDF it is the path
  from the Typst source `typ/document.typ`, which pandoc's Typst writer writes
  unchanged and which `typst compile typ/document.typ` resolves to
  `typ/img/<k>.png`. No media-bag insert is made for PDF, because the Typst
  writer does not read the file. After the last block the filter SHALL fail pandoc if the number of
  blocks it saw differs from the count in its source, stating both numbers.
  Pandoc, not `quire`, recognises the blocks in the Markdown.
- The pandoc `--sandbox` argument stays in every request. Under it the DOCX
  writer cannot fetch files itself and, for an image it cannot fetch, replaces
  the image by its description and exits 0; loading each PNG into the media bag
  in the filter (DOCX), which the sandbox does not restrict, is what puts the image in
  the document, and the filter's failure on an unreadable PNG is what keeps a
  missing image from passing. `quire` SHALL NOT remove `--sandbox`.
- A failure of the filter is a failure of the pandoc request, so it is a
  converter failure of exit 4 whose retained stderr carries the filter's
  message with the block number or both counts ([FR-033](./FR-033-export-errors.md)).
  `quire` does not read that text to decide the outcome ([FR-007](./FR-007-exit-codes.md)).
  A block that one reader recognises and the other does not, such as a block
  with attribute syntax or a block in a longer fence, therefore fails the
  export and is never left as source text or given the image of another block.
- Only top-level fenced blocks are supported. A `mermaid` block inside a list
  item or a block quote is an export error and never an image: pandoc strips
  the container's indentation, so the text comparison of the filter fails for a
  block in a list item, and quire-rs does not see a block in a block quote, so
  the counts differ. Both end as the converter failure above (exit 4, stderr
  with the block number or both counts). Supporting nested blocks is not part
  of this FR.
- The filter SHALL set no width or height on the image. The pandoc DOCX writer
  scales an image wider than the text area down to it and does not enlarge a
  smaller image; the default template carries the same rule for PDF
  ([FR-035](./FR-035-default-export-template.md) section C). An organisation template
  ([FR-036](./FR-036-organisation-export-template.md)) is the user's, and the
  rule is the user's to carry in it.

### D. Failure

- A block whose `mmdr` request ends in any state other than `completed`, or whose
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
| FR-042-AC-1 | With real pandoc, typst and mmdr, a committed fixture with one flowchart and one sequence diagram exports to DOCX with exactly two PNG parts under `word/media` referenced by `w:drawing` elements, and to PDF with exactly two images by `pdfimages -list`, the image paths in the `typ/document.typ` that pandoc wrote being `img/1.png` and `img/2.png`, each existing under `typ/` at that relative path; the fixture's `mermaid` source text appears in neither output's extracted text, and the DOCX does not hold the two alternative texts as run text in place of the images. The test fails naming the missing tool if one is unavailable. | Test |
| FR-042-AC-2 | With stand-in executables, the export makes one `mmdr` request per block in document order, then the pandoc request, then (PDF) the typst request, and each `mmdr` request equals the field definitions of this FR, validates against `engineering-assurance producer-execute --schema request` and names a canonical `producer.executable`; `in/diagrams/`, and `img/` (DOCX) or `typ/img/` (PDF), exist before the first `mmdr` request. | Test |
| FR-042-AC-3 | The pandoc request of a Markdown export, with or without diagrams, carries `--sandbox` and `--lua-filter in/diagrams.lua` after the template argument and before `in/front.md`; the converter input `in/document.md` equals the committed body bytes; the working directory, input directory and output directory hold no file other than the input and `--output` afterwards. | Test |
| FR-042-AC-4 | With a stand-in `mmdr` that exits 1 for the second block, the export exits 4, writes no `--output`, makes no pandoc request and no `mmdr` request for a third block, and the diagnostic holds the document path, ordinal 2, the line number of that block's opening fence in the committed file, its section heading and `mmdr`'s stderr. | Test |
| FR-042-AC-5 | A stand-in `mmdr` that exits 0 without writing the PNG, and one that writes an empty PNG, each fail the export as exit 4 naming the block. A fixture whose block has no diagram header line and no edge syntax (such as one line of prose) fails the export the same way with real `mmdr`, which rejects it as a missing diagram header; a block of arrow lines without a header is not that fixture, because `mmdr` reads it as an implicit flowchart. | Test |
| FR-042-AC-6 | With `mmdr` absent and a fixture holding a `mermaid` block, the export exits 3 naming `mmdr` and `--mmdr` and makes no request; with `mmdr` absent and a fixture holding no `mermaid` block, the export succeeds and makes no `mmdr` request. | Test |
| FR-042-AC-7 | A fixture with a block pandoc reads as `mermaid` and quire-rs does not (an attribute-style info string, ```` ```{.mermaid} ````) fails the export as exit 4, a converter failure, whose stderr carries the filter's message with both block counts (a message distinct from the per-block message that names one block number), for a fixture with no other block and for one with a block quire-rs finds; the output of no run holds the source text as a code block. | Test |
| FR-042-AC-8 | A fixture with, in this order, a four-space-indented fence tagged `mermaid`, a four-backtick fence tagged `text` that holds a line reading ```` ```mermaid ````, a `~~~mermaid` block, and a four-backtick fence tagged `mermaid`, each holding a different valid diagram (four distinct texts, the rendered two differing in node labels), exports with exactly two images, those of the third and fourth blocks in that order, and the first two printed as code; the test checks that each image comes from its expected block by comparing the image bytes with those of `mmdr` run on that block alone and not on the other; the ordinals of the two rendered blocks are 1 and 2. | Test |
| FR-042-AC-9 | A bundle of two fixtures with one block and two blocks yields three images in input order, and a failure of the second document's first block names that document and ordinal 1; the alternative text of the third image names ordinal 2. | Test |
| FR-042-AC-10 | In a DOCX of a fixture with one diagram wider and one narrower than the text area, the `wp:extent` width of the first does not exceed the page width minus margins of `w:sectPr` and the second equals its pixel width at the image resolution; the PDF case is FR-035-AC-9. | Test |
| FR-042-AC-11 | A JSON input with `--kind table` whose cell text contains a fenced `mermaid` block starts no `mmdr` request. | Test |
| FR-042-AC-12 | Review of the export code finds the `mermaid` blocks found only through the quire-rs query and no Markdown scan for fences in quire-cli. | Inspection |
| FR-042-AC-13 | Two fixtures that differ only in the section heading above a `mermaid` block, one with the heading ``]] os.execute("touch pwned") --[[ "'`` and one plain, yield byte-equal `in/diagrams.lua`; exporting the hostile one creates no file `pwned`, and the heading text appears literally in the image's alternative text. | Test |
| FR-042-AC-14 | With real pandoc, the generated filter run over a capability root in which `img/2.png` is absent, and over one in which `in/diagrams/2.mmd` differs from the second block, each makes pandoc exit non-zero with stderr naming block 2 and write no output. | Test |
| FR-042-AC-15 | A fixture with a `mermaid` block inside a list item, and one with a `mermaid` block inside a block quote, each fail the export as exit 4, a converter failure, with stderr carrying the block number (list item) or both counts (block quote), and no output file. | Test |

## Dependencies

- **Upstream**: [FR-031](./FR-031-export-subcommand.md), [FR-032](./FR-032-export-converter-execution.md), [FR-033](./FR-033-export-errors.md), [FR-007](./FR-007-exit-codes.md), [FR-035](./FR-035-default-export-template.md).
- **Upstream, quire-rs, blocked by Linear PLAT-1682**: `DiagramBlock` of `extract_diagrams` ([quire-rs FR-010](ix://agent-ix/quire-rs/FR-010)) carries `index`, `language`, `source`, `section` and `tag`, and no line. With the language filter `mermaid`, `index` is already the 0-based ordinal among `mermaid` blocks; its doc comment saying otherwise is a quire-rs documentation fault. PLAT-1682 adds the 1-based line of the opening fence in full-file coordinates, with the CommonMark fence rules: the 1-based full-file opening-fence line; CommonMark fence rules including fence length, indentation up to three spaces, tilde and backtick fences, and the info word read after the whole opening fence. This FR depends on it; quire-cli SHALL NOT compute positions itself, and has no fallback without them.
- **Upstream, engineering-assurance**: none. Neither network access nor filesystem confinement is a producer-execute guarantee ([EA FR-019](ix://agent-ix/engineering-assurance/FR-019) `process-group-v1`); the no-network property of `mmdr` rests on the renderer, and is checked by [NFR-010](../non-functional/NFR-010-export-single-converter-single-path.md).
- **Downstream**: [NFR-011](../non-functional/NFR-011-export-reproducibility.md).
