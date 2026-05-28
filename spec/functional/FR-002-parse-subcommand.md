---
id: FR-002
title: "quire parse subcommand"
artifact_type: FR
object_type: cli_command
relationships:
  - target: "ix://agent-ix/quire-cli/spec/usecase/US-002"
    type: "implements"
    cardinality: "1:1"
  - target: "ix://agent-ix/quire-rs/spec/functional/FR-005"
    type: "consumes"
    cardinality: "1:1"
  - target: "ix://agent-ix/quire-rs/spec/functional/FR-006"
    type: "consumes"
    cardinality: "1:1"
  - target: "ix://agent-ix/quire-rs/spec/functional/FR-008"
    type: "consumes"
    cardinality: "1:1"
---

## Behavior

The CLI SHALL expose a `parse` subcommand with the following surface:

```
quire parse <DOC|->
```

Required arguments:
- `<DOC|->` — positional, path to a `.md` file or `-` to read from stdin.

Behavior:
1. Path-safety check on `<DOC>` (per **FR-005**) when not `-`.
2. Read full document into memory.
3. Dispatch to `quire_rs::parse_document(text)`.
4. Serialize the resulting `QuireDocument` as compact JSON (one line) on stdout.
5. Non-fatal parse diagnostics (e.g. malformed frontmatter, fallback applied per upstream FR-006) are written to stderr; stdout remains valid JSON.

Output schema MUST be stable across patch releases and SHOULD mirror the public `QuireDocument` Rust type field-for-field.

## Acceptance

- **FR-002-AC-1**: `quire parse some-doc.md | jq '.frontmatter.id'` returns the document's `id` field for an artifact with `id: FR-035` in frontmatter.
- **FR-002-AC-2**: `cat some-doc.md | quire parse -` produces identical output to `quire parse some-doc.md`.
- **FR-002-AC-3**: A document with malformed frontmatter still produces parseable JSON on stdout; stderr contains a non-fatal diagnostic naming the frontmatter line.
- **FR-002-AC-4**: Stdout for an empty document is a valid `QuireDocument` JSON with empty `sections[]`.
- **FR-002-AC-5**: The byte offsets in the JSON output match the byte slices `quire_rs::parse_document` produces in-process (round-trip fixture test).
