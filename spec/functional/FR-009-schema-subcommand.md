---
id: FR-009
title: "Archetype Schema Subcommand"
artifact_type: FR
object_type: cli_command
relationships:
  - target: "ix://agent-ix/quire-rs/spec/functional/FR-029"
    type: "consumes"
    cardinality: "1:1"
---

## Behavior

The CLI SHALL expose a `schema` subcommand:

```bash
quire schema <ARCHETYPE> --module <PATH>
```

The command SHALL load the module registry, resolve `<ARCHETYPE>`, and write the archetype input contract from upstream FR-029 to stdout as deterministic JSON. The output SHALL include the frontmatter JSON Schema, manifest `required_sections`, template variables, and required-section-to-variable mapping diagnostics.

`schema` SHALL NOT render an artifact and SHALL NOT write files. It is intended for LLM render agents and CI tools that need the same input contract quire will enforce.

Unknown archetypes SHALL exit 1 with `UnknownArchetype` diagnostics on stderr. Successful output SHALL follow the stream rules in FR-006.

## Acceptance

- **FR-009-AC-1**: `quire schema FR --module $ISO` exits 0 and emits JSON containing the FR frontmatter schema and required sections.
- **FR-009-AC-2**: The emitted JSON includes a template-variable list for FR and maps the variables that populate required sections when statically knowable.
- **FR-009-AC-3**: `quire schema NONEXISTENT --module $ISO` exits 1 with `UnknownArchetype` on stderr and empty stdout.
- **FR-009-AC-4**: Repeated `quire schema FR --module $ISO` calls produce byte-identical stdout.
- **FR-009-AC-5**: The command performs module and path safety checks equivalent to `render` and `validate`.
