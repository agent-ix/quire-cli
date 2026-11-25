---
id: SR-20261116045037
title: "Failure-domain analysis — export identity and provenance"
type: SpecReview
analysis: failure-domain
scope: "agent-ix/quire-cli; spec/functional/FR-034-document-identity.md, spec/functional/FR-035-default-export-template.md, spec/functional/FR-036-organisation-export-template.md, spec/functional/FR-037-export-clock-free-output.md, spec/non-functional/NFR-011-export-reproducibility.md, src/commands/export/identity.rs, src/commands/export/bundle.rs, src/commands/export/template.rs, tests/cli_export.rs"
review_set: subset
---

## Summary

The changed identity fields state the direct producer uniqueness and malformed-value refusal behavior, and the implementation rejects malformed producer_identity before conversion. No new extension, evaluation, or graph failure domain was introduced.

## Verdict

**PASS** — The changed identity fields state the direct producer uniqueness and malformed-value refusal behavior, and the implementation rejects malformed producer_identity before conversion.

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

## Review Evidence

Checked identity confusion, malformed producer fields, source commit/date versus producer identity, and time/environment purity. Producer identity remains distinct from the running exporter provenance and source revision.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | low | No findings (placeholder) | - |
