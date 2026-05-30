---
id: FR-010
title: "Validate Required Sections"
artifact_type: FR
object_type: cli_command
relationships:
  - target: "ix://agent-ix/quire-cli/spec/functional/FR-004"
    type: "extends"
    cardinality: "1:1"
  - target: "ix://agent-ix/quire-rs/spec/functional/FR-030"
    type: "consumes"
    cardinality: "1:1"
---

## Behavior

The `validate` subcommand SHALL reject rendered documents whose manifest `required_sections` are missing, empty, or populated only by placeholder/default text.

The CLI validation path SHALL remain compatible with context JSON validation, but document validation MUST be able to run against rendered markdown so required-section completeness can be checked after template expansion. Frontmatter schema success SHALL remain necessary but SHALL NOT be sufficient for a rendered document to validate.

Diagnostics SHALL name the archetype, section, and reason (`missing`, `empty`, or `placeholder`) using the structured diagnostic format from upstream quire-rs.

## Acceptance

- **FR-010-AC-1**: `quire validate FR --module $ISO --data valid-context.json` continues to validate context JSON against the frontmatter schema.
- **FR-010-AC-2**: `quire validate FR --module $ISO --document rendered-fr.md` exits 1 when `rendered-fr.md` has a `## Specification` section containing only `TODO`.
- **FR-010-AC-3**: `quire validate FR --module $ISO --document rendered-fr.md` exits 1 when any FR required section is missing.
- **FR-010-AC-4**: `quire validate FR --module $ISO --document rendered-fr.md` exits 0 when frontmatter is valid and all required sections contain substantive content.
- **FR-010-AC-5**: Required-section failures produce empty stdout, non-empty stderr, and include the failed section names.
