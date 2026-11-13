---
id: "SR-20261113084139"
title: "PLAT-1604 quire-cli PR189 spec-review/evidence"
type: SpecReview
analysis: evidence
scope: "agent-ix/quire-cli PR189 head 1a40cc4; spec/functional/FR-007, FR-031, FR-032, FR-033, FR-042, spec/non-functional/NFR-010, NFR-011, spec/stakeholder/StR-004, spec/usecase/US-009, spec/spec.md"
review_set: subset
---

## Summary

Ticket: PLAT-1604. Verification methods of the new ACs against the obligations they carry. Most ACs are Test with real or stand-in tools, which fits. FR-042-AC-1 with real tools would catch the sandbox defect of the failure-domain review. Three low findings: one Demonstration could be a Test, one AC relies on unstated real-mmdr rejection behaviour, and NFR-010-AC-4's process-group clause names no harness.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | low | FR-042-AC-10 uses Demonstration for a measurable property. For DOCX, `wp:extent cx` can be compared with the `w:sectPr` page width minus margins. For PDF, the image placement box can be compared with the text area. Fix: make it a Test. | spec/functional/FR-042-export-mermaid-diagrams.md:192 |
| FND-002 | low | "A fixture block of Mermaid syntax that real `mmdr` rejects" depends on behaviour that changes between mmdr versions and is not stated. Fix: name the fixture's defect by a case mmdr rejects by design, for example a missing diagram header (mmdr 0.3.1: "unknown or missing Mermaid diagram header"). | spec/functional/FR-042-export-mermaid-diagrams.md:187 |
| FND-003 | low | NFR-010-AC-4 runs real `mmdr` under `strace` but also asserts that "the descendants of the request stay in its process group". The AC does not say whether the run goes through producer-execute, and the process-group assertion means something only there. NFR-004-AC-2 already asserts zero sockets for every subcommand's descendants. Fix: split it into a socket check run directly and a descendant check through producer-execute, or say which harness runs it. | spec/non-functional/NFR-010-export-single-converter-single-path.md:54 |

## Examined scope

```yaml
scope:
- id: FR-042-AC-1
  path: spec/functional/FR-042-export-mermaid-diagrams.md
  role: examined
  excerpt: With real pandoc, typst and mmdr, a committed fixture with two `mermaid` blocks of supported
    diagram types exports to DOCX with exactly two PNG parts under `word/media` referenced by `w:drawing`
    elements, and to PDF with exactly two images by `pdfimages -list`; the fixture's `mermaid` source
    text appears in neither output's extracted text. The test fails naming the missing tool if one is
    unavailable.
- id: FR-042-AC-2
  path: spec/functional/FR-042-export-mermaid-diagrams.md
  role: examined
  excerpt: With stand-in executables, the export makes one `mmdr` request per block in document order,
    then the pandoc request, then (PDF) the typst request, and each `mmdr` request equals the field definitions
    of this FR, validates against `engineering-assurance producer-execute --schema request` and names
    a canonical `producer.executable`.
- id: FR-042-AC-3
  path: spec/functional/FR-042-export-mermaid-diagrams.md
  role: examined
  excerpt: The pandoc request of an export with diagrams carries `--lua-filter in/diagrams.lua` after
    the template argument and before `in/front.md`; the converter input `in/document.md` equals the committed
    body bytes; the working directory, input directory and output directory hold no file other than the
    input and `--output` afterwards.
- id: FR-042-AC-4
  path: spec/functional/FR-042-export-mermaid-diagrams.md
  role: examined
  excerpt: With a stand-in `mmdr` that exits 1 for the second block, the export exits 4, writes no `--output`,
    makes no pandoc request and no `mmdr` request for a third block, and the diagnostic holds the document
    path, ordinal 2, the line number of that block's opening fence in the committed file, its section
    heading and `mmdr`'s stderr.
- id: FR-042-AC-5
  path: spec/functional/FR-042-export-mermaid-diagrams.md
  role: examined
  excerpt: A stand-in `mmdr` that exits 0 without writing the PNG, and one that writes an empty PNG, each
    fail the export as exit 4 naming the block. A fixture block of Mermaid syntax that real `mmdr` rejects
    fails the export the same way.
- id: FR-042-AC-6
  path: spec/functional/FR-042-export-mermaid-diagrams.md
  role: examined
  excerpt: With `mmdr` absent and a fixture holding a `mermaid` block, the export exits 3 naming `mmdr`
    and `--mmdr` and makes no request; with `mmdr` absent and a fixture holding no `mermaid` block, the
    export succeeds and makes no `mmdr` request.
- id: FR-042-AC-7
  path: spec/functional/FR-042-export-mermaid-diagrams.md
  role: examined
  excerpt: A fixture whose `mermaid` block is written in a way quire-rs recognises and pandoc does not
    (or the reverse), such as an attribute-style info string, fails the export with both block counts
    in the diagnostic; the output of no run holds the source text as a code block.
- id: FR-042-AC-8
  path: spec/functional/FR-042-export-mermaid-diagrams.md
  role: examined
  excerpt: A fixture holding a `mermaid` block inside a longer backtick fence of another language, and
    a `~~~mermaid` block, export with images for the second only and the first printed as code.
- id: FR-042-AC-9
  path: spec/functional/FR-042-export-mermaid-diagrams.md
  role: examined
  excerpt: A bundle of two fixtures with one block and two blocks yields three images in input order,
    and a failure of the second document's first block names that document and ordinal 1.
- id: FR-042-AC-10
  path: spec/functional/FR-042-export-mermaid-diagrams.md
  role: examined
  excerpt: The image is no wider than the text area in the PDF and the DOCX, and a diagram narrower than
    the text area is not enlarged, checked on a fixture with one wide and one narrow diagram.
- id: FR-042-AC-11
  path: spec/functional/FR-042-export-mermaid-diagrams.md
  role: examined
  excerpt: A JSON input with `--kind table` whose cell text contains a fenced `mermaid` block starts no
    `mmdr` request.
- id: FR-042-AC-12
  path: spec/functional/FR-042-export-mermaid-diagrams.md
  role: examined
  excerpt: Review of the export code finds the `mermaid` blocks found only through the quire-rs query
    and no Markdown scan for fences in quire-cli.
- id: FR-032-AC-8
  path: spec/functional/FR-032-export-converter-execution.md
  role: examined
  excerpt: The requests of an export with Mermaid blocks are those of FR-042-AC-2 and FR-042-AC-3; an
    export without Mermaid blocks makes no `mmdr` request and its pandoc request carries no `--lua-filter`.
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
- id: NFR-010-AC-4
  path: spec/non-functional/NFR-010-export-single-converter-single-path.md
  role: examined
  excerpt: With real `mmdr` run under `strace -fe network` on a fixture of each supported diagram type,
    `mmdr` makes zero `socket()` calls, and the descendants of the request stay in its process group.
- id: NFR-011-AC-5
  path: spec/non-functional/NFR-011-export-reproducibility.md
  role: examined
  excerpt: Exporting a fixture with two Mermaid diagrams twice, with the clock and `TZ` changed, yields
    equal extracted text and structure, with equal image count, pixel dimensions and PNG bytes.
```

## Verdict

PASS with low findings. The verification methods fit their obligations.

## New findings (disposition pass 1)

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-004 | low | Under quire-rs as it is today (before PLAT-1682), quire-rs counts the first (indented) block and the third, and pandoc counts the third and fourth. The counts agree, so only the text comparison stands between the fixture and an accidental pass. That holds only if the blocks' diagram texts differ. The AC does not require that, and with identical texts "images of the third and fourth blocks" cannot be told apart. Fix: require four distinct diagram texts and check that each image comes from the expected block, for example by its pixel size or by `in/diagrams/<k>.mmd`. | spec/functional/FR-042-export-mermaid-diagrams.md:232 |

## Dispositions

| FND | outcome | reason |
| --- | --- | --- |
| FND-001 | fixed | - |
| FND-002 | fixed | - |
| FND-003 | fixed | - |
