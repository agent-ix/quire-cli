---
id: StR-004
title: "Thin process boundary, no parallel implementations"
type: StR
---

## Stakeholder Need

`quire-rs` is the canonical engine. Any Markdown parser/renderer/validator behavior in this CLI that diverges from `quire-rs` creates the exact drift problem `quire-rs` was built to solve.

For artifact operations, the CLI SHALL be a **thin process boundary** over `quire-rs`. Every observable
artifact behavior in `parse`/`extract`/`validate`/`assurance` SHALL trace to a
`quire-rs` FR. If a feature request cannot be satisfied by adding flags or
composing existing `quire-rs` APIs, the upstream FR is authored first and this
CLI is updated to expose the new surface.

The distributed developer gate [FR-027](../functional/FR-027-check-lock-subcommand.md)
is a narrow exception: quire-cli owns Cargo.lock TOML intake and agent-ix Git
repository coherence, which are not Markdown artifact or engine semantics.
The export subcommand [FR-031](../functional/FR-031-export-subcommand.md) is a
second narrow exception: quire-cli owns the presentation transform from an
already-computed JSON report to Markdown tables,
which are output formatting for an external converter and not Markdown artifact
or engine semantics. Markdown artifacts are still parsed and validated only by
`quire-rs` (the frontmatter and body boundary come from its `extract_frontmatter`, [quire-rs FR-006](ix://agent-ix/quire-rs/FR-006)), and the converter reads the Markdown body itself.
The export subcommand's further owned logic is bounded to four things, each
output data or layout for the external converter and none a Markdown or engine
semantic: the read-only git lookups of
[FR-034](../functional/FR-034-document-identity.md) (one commit, its committer
date, one tag's target, one blob and the attributes of one path, with no index
or worktree write, no ref update and no fetch); the generation of the default
template and of the title page, contents, separator and approval Markdown of
[FR-035](../functional/FR-035-default-export-template.md); and the presentation
of an already-computed approval verdict by
[FR-038](../functional/FR-038-approval-block.md), which reads its members and
verifies nothing; and the generation of the diagram files and pandoc filter of
[FR-042](../functional/FR-042-export-mermaid-diagrams.md), which locates Mermaid
blocks only through the quire-rs fenced-block query and writes their text as
renderer input, with no Markdown scan of its own. This exception authorizes no document parser, engine model, graph, assurance,
source-analysis, or general qualification-tool duplication. The existing
private `quire-qualify` package remains repository-local under
[FR-025](../functional/FR-025-rust-qualification-tooling.md).

## Priority

Must-Have

## Rationale

`quire-rs` is the canonical engine; any parser/renderer/validator behavior
duplicated in the CLI recreates the exact drift problem the engine was built to
eliminate. Keeping the CLI a thin process boundary — argv parsing, path-safety,
stream wiring, and calls into `quire-rs` — guarantees observable artifact behavior traces to
an upstream FR and that new features are authored upstream first rather than
forked into the CLI.

## Validation Criteria


This need is considered satisfied when every CLI FR that wraps an engine API
declares the upstream `quire-rs` FR in its relationships, the `src/` tree carries
no parsing/rendering/validation, graph, or output-schema logic of its own, and the review checklist
enforces the thin-boundary stance:

| ID | Criteria | Validation |
|----|----------|------------|
| StR-004-VC-1 | **StR-004-AC-1**: Each FR in `quire-cli/spec/functional/` that wraps a `quire-rs` API declares the upstream `quire-rs` FR ID in its frontmatter `relationships:` array (`type: implements`). | Demonstration |
| StR-004-VC-2 | **StR-004-AC-2**: `src/` contains no markdown parsing, template rendering, graph construction, assurance projection, or JSON Schema definition/validation logic — only argv/premise parsing, path-safety checks, stdin/stdout wiring, calls into `quire-rs`, and the explicitly owned FR-027 Cargo.lock developer gate and the export logic bounded above (FR-031 JSON-to-table transform, FR-034 read-only git lookups, FR-035 template and front-matter generation, FR-038 verdict presentation, FR-042 diagram files and filter generation from the quire-rs fenced-block query). | Inspection |
| StR-004-VC-3 | **StR-004-AC-3**: Code review checklist for this repo includes: "Does any new logic belong upstream in `quire-rs`?". | Demonstration |
