---
id: StR-004
title: "Thin process boundary, no parallel implementations"
type: StR
---

## Stakeholder Need

`quire-rs` is the canonical engine. Any parser/renderer/validator behavior in this CLI that diverges from `quire-rs` creates the exact drift problem `quire-rs` was built to solve.

The CLI SHALL be a **thin process boundary** over `quire-rs`. Every observable behavior in `render`/`parse`/`extract`/`validate` SHALL trace to a `quire-rs` FR. If a feature request cannot be satisfied by adding flags or composing existing `quire-rs` APIs, the upstream FR is authored first and this CLI is updated to expose the new surface.

## Priority

Must-Have

## Rationale

`quire-rs` is the canonical engine; any parser/renderer/validator behavior
duplicated in the CLI recreates the exact drift problem the engine was built to
eliminate. Keeping the CLI a thin process boundary — argv parsing, path-safety,
stream wiring, and calls into `quire-rs` — guarantees observable behavior traces to
an upstream FR and that new features are authored upstream first rather than
forked into the CLI.

## Validation Criteria

This need is considered satisfied when every CLI FR that wraps an engine API
declares the upstream `quire-rs` FR in its relationships, the `src/` tree carries
no parsing/rendering/validation logic of its own, and the review checklist
enforces the thin-boundary stance:

- **StR-004-AC-1**: Each FR in `quire-cli/spec/functional/` that wraps a `quire-rs` API declares the upstream `quire-rs` FR ID in its frontmatter `relationships:` array (`type: implements`).
- **StR-004-AC-2**: `src/` contains no markdown parsing, no template rendering, no JSON Schema validation logic — only argv parsing, path-safety checks, stdin/stdout wiring, and calls into `quire-rs`.
- **StR-004-AC-3**: Code review checklist for this repo includes: "Does any new logic belong upstream in `quire-rs`?"
