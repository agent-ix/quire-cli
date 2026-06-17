---
id: FR-009
title: "Archetype Schema Subcommand"
type: FR
object_type: cli_command
relationships:
  - target: "ix://agent-ix/quire-rs/spec/functional/FR-029"
    type: "consumes"
    cardinality: "1:1"
---

> **CR note (asserts-based contract — render removal, 2026-06-04):** Upstream
> FR-029 was recast by ADR 0004: with templates removed there are no "template
> variables". The input contract is now derived from `frontmatter_schema_ref` + the
> `body_extraction` **asserts** (FR-033) — the structure an author fills and that
> `validate_document` (FR-032) checks. This FR's wording is revised from "template
> variables / required-section-to-variable mapping" to "asserts → required
> structure" (the skeleton/example contract). The contract is still derived from the
> loaded module (manifest + schema), never inferred from rendered markdown.

## Description

The CLI SHALL expose a `schema` subcommand that emits an archetype's input
contract — frontmatter JSON Schema plus the `body_extraction` asserts — as
deterministic JSON on stdout, so LLM authoring agents and CI tools can see the
same structure `validate_document` enforces. The behavioral surface is specified
below.

## Behavior

The CLI SHALL expose a `schema` subcommand:

```bash
quire schema <ARCHETYPE> --module <PATH>
```

The command SHALL load the module registry, resolve `<ARCHETYPE>`, and write the archetype input contract from upstream FR-029 to stdout as deterministic JSON. The output SHALL include the frontmatter JSON Schema and the `body_extraction` asserts (the required headings, table columns, and id-patterns the author must fill, per FR-033) — i.e. the required-structure contract that `validate_document` (FR-032) enforces, **not** any template-variable list.

`schema` SHALL NOT render an artifact and SHALL NOT write files. It is intended for LLM authoring agents and CI tools that need the same input contract quire will enforce.

Unknown archetypes SHALL exit 1 with `UnknownArchetype` diagnostics on stderr. Successful output SHALL follow the stream rules in [FR-006](./FR-006-io-contract.md).

## Acceptance Criteria

| ID | Criteria | Verification |
|----|----------|--------------|
| FR-009-AC-1 | `quire schema FR --module $ISO` exits 0 and emits JSON containing the FR frontmatter schema and the FR `body_extraction` asserts (required headings / columns) | Test |
| FR-009-AC-2 | The emitted JSON describes, per required section, the asserts that `validate_document` will enforce (heading presence/level, table columns, id-patterns) — the asserts-based input contract, with no template-variable list | Test |
| FR-009-AC-3 | `quire schema NONEXISTENT --module $ISO` exits 1 with `UnknownArchetype` on stderr and empty stdout | Test |
| FR-009-AC-4 | Repeated `quire schema FR --module $ISO` calls produce byte-identical stdout | Test |
| FR-009-AC-5 | The command performs module and path-safety checks equivalent to `validate` ([FR-005](./FR-005-path-safety.md)) | Test |

## Dependencies

- **Upstream**: quire-rs [FR-029](ix://agent-ix/quire-rs/FR-029) (archetype input contract); [FR-005](./FR-005-path-safety.md) path-safety.
- **Downstream**: LLM authoring agents and CI tools that consume the contract.
