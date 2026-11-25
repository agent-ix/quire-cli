---
id: SR-20261116045034
title: "Spec review base — export identity and provenance"
type: SpecReview
analysis: base
scope: "agent-ix/quire-cli; spec/functional/FR-034-document-identity.md, spec/functional/FR-035-default-export-template.md, spec/functional/FR-036-organisation-export-template.md, spec/functional/FR-037-export-clock-free-output.md, spec/non-functional/NFR-011-export-reproducibility.md, src/commands/export/identity.rs, src/commands/export/bundle.rs, src/commands/export/template.rs, tests/cli_export.rs"
review_set: subset
---

## Summary

The edited requirements are traceable and testable overall. One sentence broadens the source provenance contract to generic toolchain provenance, which conflicts with the narrowed producer identity boundary and the implementation.

## Verdict

**CONDITIONAL** — The edited requirements are traceable and testable overall.

## Scope

| Unit | Role | Excerpt |
| --- | --- | --- |
| FR-034 Description and producer identity section | examined | Export identity carries producer and exporter provenance; source commit/date remain the subject revision identity. |
| FR-034-AC-18..20 | examined | Preserve optional producer identity; reproduce output bytes; report exact current CLI/engine provenance. |
| FR-035 Behavior and AC-1,2,4,5 | examined | Default title page and page furniture show exporter versions and present producer name/version. |
| FR-036 Behavior and AC-3 | examined | Converter metadata carries current exporter provenance and optional producer identity. |
| FR-037 Description and AC-6 | examined | Source commit date remains the Date; producer identity and output bytes remain stable across time-zone/clock changes. |
| NFR-011 Statement, measurement, AC-1,3 | examined | Same Quire/converter/font/input produces byte-identical outputs despite environment changes. |
| src/commands/export/{identity,bundle,template}.rs | examined | Parse source producer identity, attach running Quire provenance, and render metadata/title/page furniture. |
| tests/cli_export.rs | examined | Focused producer identity tests and real converter byte/reproducibility tests. |

## Review Notes

The Test Matrix was computed; the specific untagged and ignored-only changed criteria are recorded in the separate gap-analysis artifact. Criterion-strength analysis was skipped because the installed SpecReview analysis vocabulary has no `criterion-strength` value and no Jev client is available. No AssuranceProfile applies.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | medium | FR-034 Description requires preserving “producer and toolchain provenance” from input, but its field contract names only producer_identity and the approved boundary excludes generic toolchain_provenance. | spec/functional/FR-034-document-identity.md:17 |

## Dispositions

| FND | outcome | reason |
| --- | --- | --- |
| FND-001 | fixed | `after_excerpt`: “It SHALL also carry the current exporter's Quire provenance and preserve an optional declared producer identity from the committed input frontmatter. The identity ties the exported file to one committed revision of its source.” in spec/functional/FR-034-document-identity.md. The generic toolchain provenance promise is gone; the narrowed contract matches the ticket ruling. |
