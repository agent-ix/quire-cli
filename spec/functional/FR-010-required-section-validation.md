---
id: FR-010
title: "Structural Validation of Rendered Documents"
type: FR
object_type: cli_command
relationships:
  - target: "ix://agent-ix/quire-cli/spec/functional/FR-004"
    type: "extends"
    cardinality: "1:1"
  - target: "ix://agent-ix/quire-rs/spec/functional/FR-032"
    type: "consumes"
    cardinality: "1:1"
---

> **CR note (recast onto FR-032, ADR 0004 in quire-rs):** "Required-section"
> completeness is no longer a separate concept or a `required_sections` manifest
> field. It is subsumed by quire-rs `validate_document` (FR-032) running the
> archetype's `body_extraction` asserts (FR-033) — which check section presence at
> level, non-empty/non-placeholder content, table columns/rows, list items, and id
> patterns. The CLI's default markdown `validate` (FR-004) invokes that path; this
> FR now specifies the CLI's surfacing of those structural diagnostics, not a
> distinct required-sections feature. Prior `--document` flag folded into FR-004's
> default positional document argument.

> **CR-003 note (line number for a fully-absent section):** AC-2 reads "…
> is missing (reason `missing`), **with a line number**." quire-rs
> `validate_document` (FR-032) attributes a 1-based line only when the
> offending element is present in the document; a *fully-absent* required
> section has no document line to point at, so its `missing` diagnostic
> carries `line: None`. The CLI surfaces this verbatim (StR-004 — it cannot
> invent a line). The line-number clause is therefore exercised by the
> `placeholder`/`empty`/`assert` cases where the element is present
> (IT-051, IT-053). Flagging for upstream: if a line is required for absent
> sections, FR-032 must attribute the expected-insertion line — not a CLI
> concern. No silent spec edit made.

## Description

The default markdown `validate` (FR-004) SHALL surface, verbatim from quire-rs
`validate_document`, the structural diagnostics that reject documents violating
their archetype's `body_extraction` asserts (missing/empty/placeholder sections,
malformed tables, under-length lists, id-pattern failures, duplicate headings).
Frontmatter-schema success is necessary but not sufficient. The behavioral
surface is specified below.

## Behavior

The default markdown `validate` (FR-004) SHALL reject documents whose archetype
structure — as declared by `body_extraction` asserts — is violated: a required
section missing, empty, or placeholder-only; a table with wrong columns or too few
rows; a list below `min_items`; an id failing its (possibly `{field}`-interpolated)
pattern; or a per-level duplicate heading.

Frontmatter-schema success SHALL remain necessary but not sufficient. Diagnostics
SHALL be surfaced verbatim from quire-rs `validate_document` (line-numbered,
naming archetype, section/assert, and reason), with the CLI adding no structural
judgement of its own.

## Acceptance Criteria

| ID | Criteria | Verification |
|----|----------|--------------|
| FR-010-AC-1 | `quire validate rendered-fr.md --module $ISO` exits 1 when `## Specification` content is only `TODO`, with reason `placeholder` | Test |
| FR-010-AC-2 | It exits 1 when any FR required section is missing (reason `missing`), with a line number | Test |
| FR-010-AC-3 | It exits 1 when the Acceptance Criteria table has wrong columns or zero data rows (reasons `assert`) | Test |
| FR-010-AC-4 | It exits 0 when frontmatter is valid and all `body_extraction` asserts hold | Test |
| FR-010-AC-5 | Structural failures produce empty stdout and non-empty stderr carrying the quire-rs diagnostics unchanged | Test |

## Dependencies

- **Upstream**: FR-004 validate; quire-rs FR-032 (`validate_document`), FR-033 (`body_extraction` asserts).
- **Downstream**: CI gates relying on structural-validation diagnostics.
