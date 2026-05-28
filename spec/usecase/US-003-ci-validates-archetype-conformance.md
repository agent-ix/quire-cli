---
id: US-003
title: "CI validates that all committed artifacts conform to their archetype"
artifact_type: US
relationships:
  - target: "ix://agent-ix/quire-cli/spec/stakeholder/StR-001"
    type: "implements"
    cardinality: "1:1"
  - target: "ix://agent-ix/quire-cli/spec/stakeholder/StR-004"
    type: "implements"
    cardinality: "1:1"
---

## Story

As **CI for a spec repo**, I want to run `quire validate FR --module $ISO --data path/to/fr.md.frontmatter.json` (or `--data -` with extracted frontmatter on stdin) for every committed FR and fail the build on schema violations, so that drift between committed artifacts and the archetype's schema is caught at push time, not at next agent invocation.

## Context

Wraps `quire-rs::validate` (upstream FR-002). Exit code drives CI gating.

## Acceptance

- **US-003-AC-1**: `quire validate FR --module $ISO --data valid.json` exits 0, no stdout.
- **US-003-AC-2**: `quire validate FR --module $ISO --data invalid.json` exits 1, structured violation list on stderr (per upstream FR-017).
- **US-003-AC-3**: `quire validate NONEXISTENT --module $ISO --data x.json` exits 1 with an `UnknownArchetype` diagnostic on stderr.
