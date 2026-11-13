---
id: SR-20261113103140
title: "PLAT-1604 quire-cli PR194 gap-analysis"
type: SpecReview
analysis: gap-analysis
scope: "agent-ix/quire-cli PR194 head cd1ce4b; spec/functional/FR-042-export-mermaid-diagrams.md (AC-1..AC-15), FR-031, FR-032, FR-033, FR-034, FR-037, FR-040, spec/non-functional/NFR-004, NFR-010, NFR-011; src/commands/export/**; tests/cli_export.rs"
review_set: subset
---

## Summary

Ticket: PLAT-1604. This is a planless gap pass over the PR diff. Plan completion: not assessed. The AC-to-test mapping was checked by hand from the `#[trace]` attributes, because earlier runs of the computed matrix showed it cannot see this repository's tags (reported on PLAT-1610).

Ticket ACs, measured:

- **A fixture with a Mermaid block exports with the diagram as an image in both formats.** Met with stand-ins: one request per block, the PNG under `img/` or `typ/img/`, and the filter wired in. The real-converter proof (FR-042-AC-1) is written but ignored and unrun, because the tools are not installed, and it is incomplete against the AC (FND-004).
- **A diagram that fails to render is an export error naming the block, never dropped or left as source.** The render-failure half is met: exit 4 with document, ordinal, line, section and stderr (AC-4, AC-5). The never-left-as-source half fails for a document whose only Mermaid block is one quire-rs misses (SR-20261113103139 FND-001).
- **The renderer runs through producer-execute.** Met (AC-2). The test validates each `mmdr` request against the real `producer-execute --schema request` and compares it field for field.

FR-031, FR-032, FR-033, FR-034, FR-037, FR-040 and NFR-004, NFR-010, NFR-011 as touched by this change:

- **FR-031-AC-6.** Covered with stand-ins.
- **FR-032.** The `--lua-filter` position and the request table extension match. FR-032-AC-8 is covered.
- **FR-033 and FR-040.** Exit 4 for a failed render and exit 3 for a missing `mmdr`. No partial `--output` is left.
- **FR-034.** Blocks come from the committed text.
- **FR-037 and NFR-011.** The `mmdr` request carries `SOURCE_DATE_EPOCH` and an empty `HOME`.
- **NFR-004 and NFR-010.** No network client and no binary, font or theme is added.
- **NFR-011-AC-5.** Untested (FND-005).

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | medium | FR-042-AC-8 has no test, neither stand-in nor real. It is the AC that proves quire-rs and pandoc agree on which blocks are Mermaid: a four-space-indented fence, a four-backtick `text` fence holding ```` ```mermaid ````, a `~~~mermaid` block and a four-backtick `mermaid` fence. A stand-in version can run now. It would assert that exactly two `mmdr` requests are made, that `in/diagrams/1.mmd` and `2.mmd` hold the third and fourth texts, and that the diagnostics and alt text carry ordinals 1 and 2. | spec/functional/FR-042-export-mermaid-diagrams.md:244, tests/cli_export.rs |
| FND-002 | medium | FR-042-AC-14 has no test: the generated filter run by real pandoc over a capability root with `img/2.png` missing or `in/diagrams/2.mmd` changed. The Lua filter has never executed anywhere, and this AC is the only one that exercises it in isolation. Write it as an ignored real-pandoc test, as the repo does for its other real-converter ACs. | spec/functional/FR-042-export-mermaid-diagrams.md:250, src/commands/export/diagrams.rs:87-149 |
| FND-003 | medium | FR-042-AC-15 is only half covered. `real_pandoc_blocks_that_quire_rs_does_not_find_fail_the_export` builds the list-item fixture but no block-quote fixture. The block-quote case is also the one that SR-20261113103139 FND-001 makes succeed when it is the document's only block. The same test's AC-7 assertion checks only the substring `mermaid block`, which matches both the per-block and the count message. AC-7 and AC-15 require the count message ("both block counts"), so the assertion cannot tell them apart. | tests/cli_export.rs:4818-4861 |
| FND-004 | medium | `real_converters_embed_the_diagrams_as_images_in_both_formats` (FR-042-AC-1) leaves out four parts of the AC: the `w:drawing` references to the two `word/media` PNGs, the `img/1.png` and `img/2.png` paths in the pandoc-written `typ/document.typ`, the absence of the flowchart source (only `sequenceDiagram` is checked), and that the DOCX does not hold the alt texts as run text in place of the images. | tests/cli_export.rs:4775-4816 |
| FND-005 | medium | FR-042-AC-10 (the `wp:extent` width of a wide and a narrow diagram in DOCX) and NFR-011-AC-5 (two Mermaid diagrams exported twice under a changed clock and TZ give equal image bytes) have no test. Both are real-converter ACs. They should be written as ignored tests like the others, not omitted. | spec/functional/FR-042-export-mermaid-diagrams.md:246, spec/non-functional/NFR-011-export-reproducibility.md:69 |
| FND-006 | low | FR-042-AC-5's third sentence has no test: a block with no diagram header line fails with real `mmdr`, which rejects it as a missing header. | spec/functional/FR-042-export-mermaid-diagrams.md:241 |
| FND-007 | low | The second half of FR-042-AC-6 (no `mmdr` and no mermaid block: the export succeeds with no `mmdr` request) is asserted by `a_document_without_a_mermaid_block_makes_no_mmdr_request_and_carries_no_filter`. That test is tagged FR-042-AC-2 and FR-032-AC-8, not FR-042-AC-6, and FR-042-AC-2 is the wrong binding for it. | tests/cli_export.rs:4582-4598 |
| FND-008 | low | FR-042 section C says the alt text is `Mermaid diagram <n>` "followed by the section heading" and names no separator. The coder chose `Mermaid diagram <n>: <heading>`, which reads well, is test-pinned and is safe. The spec should state the `: ` separator so the test and the spec agree. | spec/functional/FR-042-export-mermaid-diagrams.md:169-171, src/commands/export/diagrams.rs:58-61 |

## Examined scope

```yaml
scope:
- {id: FR-042-AC-1, path: spec/functional/FR-042-export-mermaid-diagrams.md, role: examined, excerpt: "With real pandoc, typst and mmdr, a committed fixture with one flowchart and one sequence diagram exports to DOCX with exactly two PNG parts under word/media referenced by w:drawing elements, and to PDF with exactly two images by pdfimages -list ..."}
- {id: FR-042-AC-2, path: spec/functional/FR-042-export-mermaid-diagrams.md, role: examined, excerpt: "With stand-in executables, the export makes one mmdr request per block in document order, then the pandoc request, then (PDF) the typst request, and each mmdr request equals the field definitions of this FR ..."}
- {id: FR-042-AC-3, path: spec/functional/FR-042-export-mermaid-diagrams.md, role: examined, excerpt: "The pandoc request of an export with diagrams carries --sandbox and --lua-filter in/diagrams.lua after the template argument and before in/front.md ..."}
- {id: FR-042-AC-4, path: spec/functional/FR-042-export-mermaid-diagrams.md, role: examined, excerpt: "With a stand-in mmdr that exits 1 for the second block, the export exits 4, writes no --output, makes no pandoc request and no mmdr request for a third block ..."}
- {id: FR-042-AC-5, path: spec/functional/FR-042-export-mermaid-diagrams.md, role: examined, excerpt: "A stand-in mmdr that exits 0 without writing the PNG, and one that writes an empty PNG, each fail the export as exit 4 naming the block. A fixture whose block has no diagram header line fails the export the same way with real mmdr ..."}
- {id: FR-042-AC-6, path: spec/functional/FR-042-export-mermaid-diagrams.md, role: examined, excerpt: "With mmdr absent and a fixture holding a mermaid block, the export exits 3 naming mmdr and --mmdr and makes no request; with mmdr absent and a fixture holding no mermaid block, the export succeeds and makes no mmdr request."}
- {id: FR-042-AC-7, path: spec/functional/FR-042-export-mermaid-diagrams.md, role: examined, excerpt: "A fixture with a block pandoc reads as mermaid and quire-rs does not (an attribute-style info string) fails the export as exit 4 ... whose stderr carries the filter's message with both block counts; the output of no run holds the source text as a code block."}
- {id: FR-042-AC-8, path: spec/functional/FR-042-export-mermaid-diagrams.md, role: examined, excerpt: "A fixture with ... a four-space-indented fence tagged mermaid, a four-backtick fence tagged text that holds a line reading three backticks and mermaid, a ~~~mermaid block, and a four-backtick fence tagged mermaid ... exports with exactly two images ..."}
- {id: FR-042-AC-9, path: spec/functional/FR-042-export-mermaid-diagrams.md, role: examined, excerpt: "A bundle of two fixtures with one block and two blocks yields three images in input order, and a failure of the second document's first block names that document and ordinal 1; the alternative text of the third image names ordinal 2."}
- {id: FR-042-AC-10, path: spec/functional/FR-042-export-mermaid-diagrams.md, role: examined, excerpt: "In a DOCX of a fixture with one diagram wider and one narrower than the text area, the wp:extent width of the first does not exceed the page width minus margins ..."}
- {id: FR-042-AC-11, path: spec/functional/FR-042-export-mermaid-diagrams.md, role: examined, excerpt: "A JSON input with --kind table whose cell text contains a fenced mermaid block starts no mmdr request."}
- {id: FR-042-AC-12, path: spec/functional/FR-042-export-mermaid-diagrams.md, role: examined, excerpt: "Review of the export code finds the mermaid blocks found only through the quire-rs query and no Markdown scan for fences in quire-cli."}
- {id: FR-042-AC-13, path: spec/functional/FR-042-export-mermaid-diagrams.md, role: examined, excerpt: "Two fixtures that differ only in the section heading above a mermaid block, one hostile and one plain, yield byte-equal in/diagrams.lua; exporting the hostile one creates no file pwned, and the heading text appears literally in the image's alternative text."}
- {id: FR-042-AC-14, path: spec/functional/FR-042-export-mermaid-diagrams.md, role: examined, excerpt: "With real pandoc, the generated filter run over a capability root in which img/2.png is absent, and over one in which in/diagrams/2.mmd differs from the second block, each makes pandoc exit non-zero with stderr naming block 2 and write no output."}
- {id: FR-042-AC-15, path: spec/functional/FR-042-export-mermaid-diagrams.md, role: examined, excerpt: "A fixture with a mermaid block inside a list item, and one with a mermaid block inside a block quote, each fail the export as exit 4 ... and no output file."}
- {id: FR-032-AC-8, path: spec/functional/FR-032-export-converter-execution.md, role: examined, excerpt: "The requests of an export with Mermaid blocks are those of FR-042-AC-2 and FR-042-AC-3; an export without Mermaid blocks makes no mmdr request and its pandoc request carries no --lua-filter."}
- {id: FR-031-AC-6, path: spec/functional/FR-031-export-subcommand.md, role: examined, excerpt: "A fixture with a mermaid block exports with an image in the output and the block's source text absent, as FR-042-AC-1 defines; the converter input in/document.md equals the bytes of the committed body ..."}
- {id: NFR-011-AC-5, path: spec/non-functional/NFR-011-export-reproducibility.md, role: examined, excerpt: "Exporting a fixture with two Mermaid diagrams twice, with the clock and TZ changed, yields equal extracted text and structure, with equal image count, pixel dimensions and image bytes ..."}
- {id: NFR-010-AC-2, path: spec/non-functional/NFR-010-export-single-converter-single-path.md, role: examined, excerpt: "The published npm package file list and the repository tree hold no pandoc, typst or mmdr binary, no font or Mermaid theme file ..."}
- {id: NFR-004, path: spec/non-functional/NFR-004-no-network.md, role: context_only, excerpt: "no network"}
- {id: FR-033, path: spec/functional/FR-033-export-errors.md, role: context_only, excerpt: "export errors, exit codes, no partial --output"}
- {id: FR-034, path: spec/functional/FR-034-document-identity.md, role: context_only, excerpt: "committed content validated and converted"}
- {id: FR-037, path: spec/functional/FR-037-export-clock-free-output.md, role: context_only, excerpt: "clock-free output"}
- {id: FR-040, path: spec/functional/FR-040-export-template-and-identity-errors.md, role: context_only, excerpt: "template and identity errors"}
```

## Verdict

**CONDITIONAL** for this method, with medium gaps and no high ones. The blocking defect is SR-20261113103139 FND-001, which this analysis also sees as the reason the AC-7 and AC-15 tests cannot pass.

Met with tagged stand-in tests: AC-2, AC-3, AC-4, AC-5 (stand-in half), AC-6, AC-9, AC-11 and AC-13. AC-12 is met by inspection: `diagrams::find` is the only finder and calls `extract_diagrams`, and no fence scan exists in `src/`.

AC-1, AC-7 and AC-15 are written as ignored real-converter tests. Leaving them unrun is acceptable, because the tools are not installed.

AC-8, AC-10, AC-14, NFR-011-AC-5 and the real-mmdr part of AC-5 are not written at all. They should be (FND-001, FND-002, FND-005, FND-006). AC-8 can run now with stand-ins.
