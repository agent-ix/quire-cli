---
id: US-004
title: "Cross-reference extraction for graph ingestion"
type: US
relationships:
  - target: "ix://agent-ix/quire-cli/spec/stakeholder/StR-001"
    type: "implements"
    cardinality: "1:1"
  - target: "ix://agent-ix/quire-cli/spec/stakeholder/StR-004"
    type: "implements"
    cardinality: "1:1"
---

## Story

As a **batch indexer ingesting a spec corpus into Filament**, I want to run `quire extract FR-035.md --module $FILAMENT_CORE` and receive a JSON object with both the body-extraction results (`extraction`) and the harvested relationships (`edges`), so that I can feed one tool's output directly to Filament's graph upsert API without writing intermediate AST traversal code.

## Context

Wraps `quire-rs::extract` + `quire-rs::harvest_edges` (upstream FR-011, FR-015). The module supplies the body-extraction DSL declared per-archetype; the harvested edges include both frontmatter sugar fields (`dependencies`, `supersedes`, …) and structured `relationships:` blocks.

## Acceptance

- **US-004-AC-1**: `quire extract some-doc.md --module $MOD` writes `{"extraction": {...}, "edges": [...]}` JSON to stdout, exits 0.
- **US-004-AC-2**: Edges are deduped by `(source, type, target)` per upstream FR-015.
- **US-004-AC-3**: A document referencing an unknown archetype kind in `--module` exits 1 with an `UnknownArchetype` diagnostic; partial extraction is not emitted.
