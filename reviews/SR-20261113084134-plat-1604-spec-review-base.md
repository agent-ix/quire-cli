---
id: "SR-20261113084134"
title: "PLAT-1604 quire-cli PR189 spec-review/base"
type: SpecReview
analysis: base
scope: "agent-ix/quire-cli PR189 head 1a40cc4; spec/functional/FR-007, FR-031, FR-032, FR-033, FR-042, spec/non-functional/NFR-010, NFR-011, spec/stakeholder/StR-004, spec/usecase/US-009, spec/spec.md"
review_set: subset
---

## Summary

Ticket: PLAT-1604. Base checklist review of the spec-only PR that makes `mmdr` (mermaid-rs-renderer) the Mermaid renderer for `quire export`, adds FR-042 and amends FR-007/031/032/033, NFR-010/011, StR-004, US-009 and spec.md. The renderer facts were checked against the published crate mermaid-rs-renderer 0.3.1 (crates.io, MIT, README states "under active early development") and its source: it starts no child process (no `std::process::Command`), has no network client, and `save_png` creates no parent directory. Four findings: the trade-off record says unsupported Mermaid syntax is a render failure, but mmdr's own validator documents a deliberately narrow check, so unsupported constructs can render incompletely with exit 0; the supported diagram set is never named; the renderer choice is left pending owner confirmation; and the options table has smaller factual gaps. Public-repo hygiene is clean: no roadmap, no paywalled text, no compatibility layer, no pins or SHAs.

## Findings

| ID | Severity | Summary | Refs | Escape Cause |
| --- | --- | --- | --- | --- |
| FND-001 | high | The trade-off record states that a diagram using syntax mmdr does not support is a render failure and an export error. mmdr 0.3.1 does not behave that way: src/validator.rs says its coverage is "deliberately narrow" (five detection paths) and that "the full per-diagram-type validation surface is follow-up work", and the parser skips lines it does not handle. An unsupported construct can therefore render an incomplete diagram with exit 0, and quire would embed it. That is silent content loss, which the ticket AC forbids ("never silently dropped"). FR-042-AC-5 tests only syntax that mmdr rejects. Fix: state plainly that mmdr detects only some invalid or unsupported syntax, and that what it accepts but does not draw is out of quire's detection. Whether to accept that residual risk is an OWNER decision. | spec/functional/FR-042-export-mermaid-diagrams.md:63-69 | wrong-requirement |
| FND-002 | medium | "Supported diagram types" (FR-042-AC-1, NFR-010-AC-4) is never defined. No list exists in FR-042, and mmdr's own README claims 23 types while warning that output "may not yet match mermaid-cli in all cases". Two implementers would pick different fixtures, and NFR-010-AC-4 ("each supported diagram type") has no bounded set. Fix: name the diagram types this export guarantees, at least the architecture and flow diagrams the ticket names (flowchart and one of sequence, C4 or architecture), and use exactly those in the fixtures. | spec/functional/FR-042-export-mermaid-diagrams.md:183 | wrong-requirement |
| FND-003 | medium | The ticket's outcome requires the renderer to be chosen in the spec, but the record ends "This choice is the owner's to confirm." That makes the normative decision conditional. If the owner confirms, replace the sentence with the confirmed decision. If not, the PR is not ready. This is an OWNER decision, not fixable text. | spec/functional/FR-042-export-mermaid-diagrams.md:63-69 | wrong-requirement |
| FND-004 | low | The options table has three factual gaps. (1) The Typst Universe package `mmdr` (0.2.2) is a WebAssembly build of this same renderer, with no JavaScript engine, and it can be installed locally for offline compiles. The real reason to reject it is that Typst serves PDF only, not DOCX. (2) mmdc's Chromium download happens when puppeteer is installed, not on the first run. (3) "Its output is a pure function of the diagram text, its version and the installed fonts" is stated as fact but is not verified: mmdr loads system fonts through fontdb and writes a font cache under $HOME/.cache/mmdr. Fix: correct (1) and (2), and state (3) as an assumption that NFR-011-AC-5 checks. | spec/functional/FR-042-export-mermaid-diagrams.md:58-60 | wrong-requirement |

## Examined scope

```yaml
scope:
- id: FR-042
  path: spec/functional/FR-042-export-mermaid-diagrams.md
  role: examined
  excerpt: '`quire export` SHALL render every fenced code block whose language tag is `mermaid` in a Markdown
    input as an image in the PDF or DOCX output, in place of the block''s source text. The renderer is
    `mmdr`, run as one request of the `engineering-assurance.producer-execution-request/v1` protocol through
    `engineering-assurance producer-execute` for each block, as [FR-032](./FR-032-export-converter-execution.md)
    runs pandoc and typst. A block that cannot be rendered is an export error that names the block; it
    is never dropped and never left as source text.'
- id: FR-042#renderer-decision
  path: spec/functional/FR-042-export-mermaid-diagrams.md
  role: examined
  excerpt: 'Trade-off recorded as the decision: `mmdr` is young. It supports a subset of the Mermaid diagram
    types, and its layout is not identical to the Mermaid reference renderer''s. A diagram that uses syntax
    `mmdr` does not support is a render failure and an export error ([FR-033](./FR-033-export-errors.md));
    there is no second renderer to fall back to. Choosing `mmdc` instead would trade those limits for
    a browser and a Node runtime, with the containment and network exposure above. This choice is the
    owner''s to confirm.'
- id: FR-042#options-table
  path: spec/functional/FR-042-export-mermaid-diagrams.md
  role: examined
  excerpt: '| `mmdr` (native Rust) | Chosen | One executable with no runtime dependency. It spawns no
    descendant, so containment holds trivially. It has no network need. Its output is a pure function
    of the diagram text, its version and the installed fonts, which is the carve-out NFR-011 already makes
    for fonts. |

    | A pandoc filter or a Typst package | Rejected | The known Mermaid pandoc filters call `mmdc`, so
    they add its runtime and a process launched outside producer-execute. Typst Mermaid packages are fetched
    from a package registry at compile time, which needs the network, or embed a JavaScript engi'
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
- id: FR-042-AC-9
  path: spec/functional/FR-042-export-mermaid-diagrams.md
  role: examined
  excerpt: A bundle of two fixtures with one block and two blocks yields three images in input order,
    and a failure of the second document's first block names that document and ordinal 1.
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
- id: FR-007-AC-9
  path: spec/functional/FR-007-exit-codes.md
  role: examined
  excerpt: A converter failure of `quire export` (including a failed Mermaid diagram render) exits 4 and
    a missing converter (including `mmdr`) exits 3, as FR-033 defines.
- id: NFR-010
  path: spec/non-functional/NFR-010-export-single-converter-single-path.md
  role: examined
  excerpt: Export SHALL use pandoc and typst as its only document converters and `mmdr` ([FR-042](../functional/FR-042-export-mermaid-diagrams.md))
    as its only diagram renderer, each launched only through `engineering-assurance producer-execute`,
    with no converter or renderer binary, template file or reference document placed in this repository
    or its package.
- id: NFR-010-AC-4
  path: spec/non-functional/NFR-010-export-single-converter-single-path.md
  role: examined
  excerpt: With real `mmdr` run under `strace -fe network` on a fixture of each supported diagram type,
    `mmdr` makes zero `socket()` calls, and the descendants of the request stay in its process group.
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

FAIL. This is not mergeable as written. FND-001 is a wrong claim about the renderer that matters to the ticket's never-silently-dropped AC. FND-003 leaves the renderer decision unconfirmed. Both need the owner. FND-002 and FND-004 are text fixes. Clean on public-repo hygiene, exit-code consistency with FR-007, the US-009 examples and FR-031 section C.

## Dispositions

| FND | outcome | reason |
| --- | --- | --- |
| FND-001 | fixed | - |
| FND-002 | fixed | - |
| FND-003 | fixed | - |
| FND-004 | fixed | - |
