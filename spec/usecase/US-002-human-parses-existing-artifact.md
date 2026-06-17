---
id: US-002
title: "Human inspects an existing artifact's structure via quire parse"
type: US
relationships:
  - target: "ix://agent-ix/quire-cli/spec/stakeholder/StR-001"
    type: "implements"
    cardinality: "1:1"
---

## Story

As a **human spec author debugging a malformed artifact**, I want to run `quire parse FR-035-module-manifest-schema.md` and receive a JSON tree showing frontmatter, headings, and byte ranges, so that I can see exactly how the parser interpreted my file without booting a Python notebook.

## Context

Wraps `quire-rs::parse_document` (upstream [FR-005](../functional/FR-005-path-safety.md) / [FR-006](../functional/FR-006-io-contract.md) / [FR-007](../functional/FR-007-exit-codes.md) / [FR-008](../functional/FR-008-json-output-encoding.md)). Output is the `QuireDocument` structure serialized as JSON. Useful for ad-hoc debugging, CI link-validation tooling, and `jq` pipelines.

## Acceptance

- **US-002-AC-1**: `quire parse some-doc.md` writes a JSON document to stdout with `frontmatter`, `sections[]`, and byte offsets, exits 0.
- **US-002-AC-2**: The JSON deserializes into a Rust value structurally equivalent to `quire_rs::QuireDocument`.
- **US-002-AC-3**: A document with malformed frontmatter still parses (frontmatter-with-fallback per upstream [FR-006](../functional/FR-006-io-contract.md)), reporting the frontmatter parse failure as a non-fatal diagnostic on stderr.
- **US-002-AC-4**: `quire parse -` reads the document from stdin.
