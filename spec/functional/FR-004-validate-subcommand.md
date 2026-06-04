---
id: FR-004
title: "quire validate subcommand"
artifact_type: FR
object_type: cli_command
relationships:
  - target: "ix://agent-ix/quire-cli/spec/usecase/US-003"
    type: "implements"
    cardinality: "1:1"
  - target: "ix://agent-ix/quire-rs/spec/functional/FR-032"
    type: "consumes"
    cardinality: "1:1"
  - target: "ix://agent-ix/quire-rs/spec/functional/FR-002"
    type: "consumes"
    cardinality: "1:1"
---

> **CR note (flipped default, ADR 0004 in quire-rs):** The default `validate`
> input is now a **markdown document**, not a context JSON object. Markdown
> validation dispatches to quire-rs `validate_document` (FR-032); the prior
> context/data validation (quire-rs FR-002) is preserved behind `--json`. The CLI
> remains a thin wrapper (StR-004) — no validation logic lives here.

## Behavior

The CLI SHALL expose a `validate` subcommand with two modes:

```
quire validate <DOC.md|-> [--module <PATH>] [--archetype <NAME>]      # default: markdown
quire validate <ARCHETYPE> --module <PATH> --json <FILE|->            # context/data
```

**Default (markdown) mode** — when the positional argument is a document path or
`-`:
1. Path-safety on the document path and `--module`.
2. Read the markdown; resolve the archetype from frontmatter `artifact_type` unless `--archetype` overrides.
3. Dispatch to quire-rs `validate_document(archetype, doc_text)` (FR-032): structural validation over `body_extraction` asserts + frontmatter-schema + per-level heading uniqueness.
4. On success: exit 0, no stdout. On failure: write the line-numbered structured diagnostics to stderr, exit 1.

**Context mode** — when `--json` is given with a positional `ARCHETYPE`: dispatch
to quire-rs `validate(&compiled_archetype, &data)` (FR-002) over a context JSON
object (the legacy behavior). Success → exit 0; schema violation → structured
list on stderr, exit 1.

`validate` SHALL NOT render or write any artifact body. It is a fast CI / authoring gate.

## Acceptance

- **FR-004-AC-1**: `quire validate valid-fr.md --module $ISO` exits 0 with no output (frontmatter valid, all required structure present).
- **FR-004-AC-2**: `quire validate broken-fr.md --module $ISO` exits 1; stderr contains a line-numbered diagnostic naming the failing section/assert.
- **FR-004-AC-3**: `quire validate fr.md --module $ISO --archetype FR` overrides frontmatter-derived archetype resolution.
- **FR-004-AC-4**: `quire validate FR --module $ISO --json valid.json` exits 0; `--json invalid.json` (missing required `id`) exits 1 with a `/id` `required` violation on stderr (context mode preserved).
- **FR-004-AC-5**: `quire validate NONEXISTENT --module $ISO --json x.json` exits 1 with `UnknownArchetype` on stderr.
- **FR-004-AC-6**: All validation logic is delegated to quire-rs; an audit confirms the CLI crate contains no structural-validation logic of its own (StR-004 thin boundary).
