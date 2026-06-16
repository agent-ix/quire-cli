---
id: US-003
title: "CI validates that all committed artifacts conform to their archetype"
type: US
relationships:
  - target: "ix://agent-ix/quire-cli/spec/stakeholder/StR-001"
    type: "implements"
    cardinality: "1:1"
  - target: "ix://agent-ix/quire-cli/spec/stakeholder/StR-004"
    type: "implements"
    cardinality: "1:1"
---

> **CR note (markdown-only — render removal, 2026-06-04):** `validate` is
> markdown-only; the `--data <frontmatter.json>` / `--json` context mode is removed
> (§2bis). CI now validates the committed markdown document directly.

## Story

As **CI for a spec repo**, I want to run `quire validate path/to/fr.md --module $ISO` for every committed FR and fail the build on structural-validation failures, so that drift between committed artifacts and the archetype's required structure is caught at push time, not at next agent invocation.

## Context

Wraps `quire-rs::validate_document` (upstream FR-032). Exit code drives CI gating.

## Acceptance

- **US-003-AC-1**: `quire validate valid-fr.md --module $ISO` exits 0, no stdout.
- **US-003-AC-2**: `quire validate invalid-fr.md --module $ISO` exits 1, structured violation list on stderr (per upstream FR-017).
- **US-003-AC-3**: `quire validate fr.md --module $ISO --archetype NONEXISTENT` exits 1 with an `UnknownArchetype` diagnostic on stderr.
