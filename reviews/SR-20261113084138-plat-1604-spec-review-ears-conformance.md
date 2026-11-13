---
id: "SR-20261113084138"
title: "PLAT-1604 quire-cli PR189 spec-review/ears-conformance"
type: SpecReview
analysis: ears-conformance
scope: "agent-ix/quire-cli PR189 head 1a40cc4; spec/functional/FR-007, FR-031, FR-032, FR-033, FR-042, spec/non-functional/NFR-010, NFR-011, spec/stakeholder/StR-004, spec/usecase/US-009, spec/spec.md"
review_set: subset
---

## Summary

Ticket: PLAT-1604. EARS conformance of the new and edited statements. `quire validate --scope . "spec/**/*.md"` (quire 0.44.0) exits 0 with no EARS warning on FR-042. Its one new-text warning, NFR-010 line 54 `[ac:vague-response] process`, is a false positive on the noun "process group". Two low findings: the scaling rule names no responsible component, and two normative facts are stated without SHALL.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | low | "An image wider than the text area SHALL be scaled down ... SHALL NOT be scaled up" is agentless passive. It names no component that does the scaling (the pandoc DOCX writer's default, the Typst template, or a width attribute set by the filter), so an implementer cannot tell where to make it true. Fix: name the mechanism, for example "the filter SHALL set no width and the template SHALL ...". | spec/functional/FR-042-export-mermaid-diagrams.md:151-155 |
| FND-002 | low | Block identity ("Each block is identified by ...") and the image format ("The image format is PNG ...") are binding behaviour, but they are written as declaratives without SHALL, so readers of the requirement may skip them. Fix: write "quire SHALL identify each block by ..." and "quire SHALL request PNG ...". | spec/functional/FR-042-export-mermaid-diagrams.md:93-98 |

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
- id: FR-042#C-scale
  path: spec/functional/FR-042-export-mermaid-diagrams.md
  role: examined
  excerpt: An image wider than the text area SHALL be scaled down to fit it, and an image SHALL NOT be
    scaled up. The default and organisation templates of [FR-035](./FR-035-default-export-template.md)
    and [FR-036](./FR-036-organisation-export-template.md) apply to the page that holds the image as to
    any other content.
- id: FR-042#A-identity
  path: spec/functional/FR-042-export-mermaid-diagrams.md
  role: examined
  excerpt: Each block is identified by the input path, its ordinal among the `mermaid` blocks of that
    document (1-based, in document order), its first line number in the committed file (1-based, counting
    the frontmatter), and the heading of the section that holds it, when there is one. The line number
    and the section come from the quire-rs record; the upstream requirement is stated in Dependencies.
- id: FR-042#B-png
  path: spec/functional/FR-042-export-mermaid-diagrams.md
  role: examined
  excerpt: 'The image format is PNG for both PDF and DOCX. SVG is not used: Word embeds SVG only with
    a raster fallback, and pandoc makes that fallback by launching a rasterizer of its own, which would
    be a process outside producer-execute. One raster format needs no further tool for either output.'
- id: FR-042#C-count
  path: spec/functional/FR-042-export-mermaid-diagrams.md
  role: examined
  excerpt: If pandoc reads a number of `mermaid` code blocks different from the number `quire` rendered,
    the filter SHALL stop pandoc with an error that states both numbers; the export fails as [FR-033](./FR-033-export-errors.md)
    defines. A block one reader recognises and the other does not, such as a block with attribute syntax,
    therefore fails the export and is never left as source text.
- id: FR-042#D-missing
  path: spec/functional/FR-042-export-mermaid-diagrams.md
  role: examined
  excerpt: If `mmdr` is not found, or is not an executable regular file after resolution, and the input
    holds at least one `mermaid` block, then the export SHALL fail as a missing converter (exit 3, [FR-007](./FR-007-exit-codes.md))
    before any request is made, naming `mmdr` and `--mmdr`.
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
- id: StR-004#bounded-exception
  path: spec/stakeholder/StR-004-thin-boundary-over-quire-rs.md
  role: examined
  excerpt: and the generation of the diagram files and pandoc filter of [FR-042](../functional/FR-042-export-mermaid-diagrams.md),
    which locates Mermaid blocks only through the quire-rs fenced-block query and writes their text as
    renderer input, with no Markdown scan of its own.
```

## Verdict

PASS with low findings. No EARS validator warning on FR-042, and the statements are otherwise well formed.

## Dispositions

| FND | outcome | reason |
| --- | --- | --- |
| FND-001 | fixed | - |
| FND-002 | fixed | - |
