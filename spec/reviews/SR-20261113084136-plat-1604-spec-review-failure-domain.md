---
id: "SR-20261113084136"
title: "PLAT-1604 quire-cli PR189 spec-review/failure-domain"
type: SpecReview
analysis: failure-domain
scope: "agent-ix/quire-cli PR189 head 1a40cc4; spec/functional/FR-007, FR-031, FR-032, FR-033, FR-042, spec/non-functional/NFR-010, NFR-011, spec/stakeholder/StR-004, spec/usecase/US-009, spec/spec.md"
review_set: subset
---

## Summary

Ticket: PLAT-1604. Failure modes of block discovery, mmdr requests, the generated Lua filter and image placement. Checked against pandoc's documented `--sandbox` semantics, EA FR-019 output rules (the producer creates its own output parents) and the mmdr 0.3.1 source. Four findings: under `--sandbox` the DOCX writer cannot read the PNGs, so pandoc warns and replaces each image with its alt text while exiting 0, and the failure path the spec relies on never fires; document text (the section heading) goes into generated Lua code with no encoding rule; a count-only check can place the wrong diagram; and nobody creates the image output directory, which mmdr does not create.

## Findings

| ID | Severity | Summary | Refs | Escape Cause |
| --- | --- | --- | --- | --- |
| FND-001 | high | Under `--sandbox`, pandoc limits reader and writer IO to the files named on the command line (pandoc manual). The DOCX writer therefore cannot fetch `img/<k>.png`. For an image it cannot fetch, pandoc emits the warning "Could not fetch resource ...: replacing image with description" and exits 0. The diagram becomes its alt text and the export reports success, so the spec's contingency ("the export fails as a converter failure with pandoc's message") never happens. This is a near-certain outcome of the design as written, not an edge case. Fix: specify that the filter reads each PNG with Lua io and calls `pandoc.mediabag.insert` (the manual says filters are not limited by the sandbox), and that the filter fails pandoc if an image cannot be loaded. Confidence is medium because no local pandoc was available to reproduce; FR-042-AC-1 will catch it. | spec/functional/FR-042-export-mermaid-diagrams.md:156-159 | wrong-requirement |
| FND-002 | high | The generated Lua filter builds alt text from the section heading, which is document text. The spec gives no rule for how that text enters the filter. A Lua filter runs with full io/os access (it is not sandboxed), so splicing a heading such as `]] os.execute(...) --[[` into the source is code execution taken from document content. FR-033-AC-10 already guards the same class of problem for request arguments. Fix: the filter source SHALL be fixed text plus integer ordinals only, with any document text read as data (for example a JSON file decoded by `pandoc.json.decode`) or taken from pandoc's own AST. Add an AC with a heading that contains Lua quote and long-bracket characters. | spec/functional/FR-042-export-mermaid-diagrams.md:138-141 | missing-requirement |
| FND-003 | medium | Comparing only the total count cannot detect two opposite disagreements in one document. quire-rs trims leading whitespace, so a 4-space-indented ```` ```mermaid ```` counts as a mermaid block while pandoc reads it as an indented code block. A ```` ````mermaid ```` four-backtick fence is language "" to quire-rs (its regex `^(?:```\|~~~)(\w*)` stops at the 4th backtick) but class mermaid to pandoc. Both counts are then 1, and pandoc replaces the four-backtick block with the indented block's image: the wrong diagram, silently. Fix: have the filter compare each code block's text with `in/diagrams/<k>.mmd` and fail on any difference, which also covers count mismatch. | spec/functional/FR-042-export-mermaid-diagrams.md:142-146 | wrong-requirement |
| FND-004 | medium | The mmdr output path is `img/<k>.png` or `typ/img/<k>.png`, but no requirement creates `img/` or `typ/img/`. EA FR-019 says "The producer creates its own declared output parents", and mmdr 0.3.1 calls `pixmap.save_png(output)` without creating directories. As written, every mmdr request would fail. Fix: quire SHALL create `img/` (DOCX) or `typ/img/` (PDF) in the capability root before the first mmdr request. | spec/functional/FR-042-export-mermaid-diagrams.md:111-121 | missing-requirement |

## Examined scope

```yaml
scope:
- id: FR-042#C-sandbox
  path: spec/functional/FR-042-export-mermaid-diagrams.md
  role: examined
  excerpt: The `--sandbox` argument of FR-032 stays in effect for every export. If the installed pandoc
    cannot read the image files under it, the export fails as a converter failure with pandoc's message;
    `quire` SHALL NOT remove `--sandbox` to make it pass.
- id: FR-042#C-lua
  path: spec/functional/FR-042-export-mermaid-diagrams.md
  role: examined
  excerpt: '`quire` SHALL instead write a pandoc Lua filter, `in/diagrams.lua`, generated at export time
    and stored nowhere, and SHALL add `--lua-filter in/diagrams.lua` to the pandoc request, after the
    metadata and template arguments and before the input files.'
- id: FR-042#C-filter
  path: spec/functional/FR-042-export-mermaid-diagrams.md
  role: examined
  excerpt: The filter replaces the k-th code block that pandoc reads with the class `mermaid` by one image
    of `<k>.png` whose alternative text is `Mermaid diagram <ordinal>` followed by the section heading
    when there is one. Pandoc, not `quire`, recognises the blocks in the Markdown.
- id: FR-042#C-count
  path: spec/functional/FR-042-export-mermaid-diagrams.md
  role: examined
  excerpt: If pandoc reads a number of `mermaid` code blocks different from the number `quire` rendered,
    the filter SHALL stop pandoc with an error that states both numbers; the export fails as [FR-033](./FR-033-export-errors.md)
    defines. A block one reader recognises and the other does not, such as a block with attribute syntax,
    therefore fails the export and is never left as source text.
- id: FR-042#B-request
  path: spec/functional/FR-042-export-mermaid-diagrams.md
  role: examined
  excerpt: 'Each `mmdr` request SHALL be the complete field set of the request table in [FR-032](./FR-032-export-converter-execution.md)
    with these definitions, and the fields of that table not named here apply unchanged: `producer.name`
    `mmdr`; `arguments` `-i`, `in/diagrams/<k>.mmd`, `-o`, the image path, `-e`, `png`, in that order,
    where the image path is `img/<k>.png` for DOCX and `typ/img/<k>.png` for PDF; `outputs` one entry,
    role `diagram`, that same path, `required` true; `environment` as FR-032; `budget` `timeoutMillis`
    60000, `maxOutputArtifacts` 1, `maxConcurrency` 1 and the other bounds of FR-'
- id: FR-042#A-identity
  path: spec/functional/FR-042-export-mermaid-diagrams.md
  role: examined
  excerpt: Each block is identified by the input path, its ordinal among the `mermaid` blocks of that
    document (1-based, in document order), its first line number in the committed file (1-based, counting
    the frontmatter), and the heading of the section that holds it, when there is one. The line number
    and the section come from the quire-rs record; the upstream requirement is stated in Dependencies.
- id: FR-042#D-failure
  path: spec/functional/FR-042-export-mermaid-diagrams.md
  role: examined
  excerpt: A block whose request ends in any state other than `completed`, or whose output PNG is absent
    or empty, SHALL fail the export. The diagnostic names the document path, the ordinal, the line number,
    the section heading when there is one, the state, the exit code if any, and `mmdr`'s stderr as the
    executor retains it, so the author can find the block.
- id: FR-042-AC-1
  path: spec/functional/FR-042-export-mermaid-diagrams.md
  role: examined
  excerpt: With real pandoc, typst and mmdr, a committed fixture with two `mermaid` blocks of supported
    diagram types exports to DOCX with exactly two PNG parts under `word/media` referenced by `w:drawing`
    elements, and to PDF with exactly two images by `pdfimages -list`; the fixture's `mermaid` source
    text appears in neither output's extracted text. The test fails naming the missing tool if one is
    unavailable.
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
- id: StR-004#bounded-exception
  path: spec/stakeholder/StR-004-thin-boundary-over-quire-rs.md
  role: examined
  excerpt: and the generation of the diagram files and pandoc filter of [FR-042](../functional/FR-042-export-mermaid-diagrams.md),
    which locates Mermaid blocks only through the quire-rs fenced-block query and writes their text as
    renderer input, with no Markdown scan of its own.
```

## Verdict

FAIL. FND-001 (sandbox: images silently replaced by alt text) and FND-002 (document text spliced into executable Lua) are high and must be fixed in the spec before implementation. FND-003 and FND-004 are medium text fixes. The fail-closed design otherwise holds: no partial output, no retry, no placeholder, and the run stops at the first failed block.
