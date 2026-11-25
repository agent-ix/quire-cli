---
id: SR-20261116045033
title: "Gap analysis — export identity and provenance"
type: SpecReview
analysis: gap-analysis
scope: "agent-ix/quire-cli; spec/functional/FR-034-document-identity.md, spec/functional/FR-035-default-export-template.md, spec/functional/FR-036-organisation-export-template.md, spec/functional/FR-037-export-clock-free-output.md, spec/non-functional/NFR-011-export-reproducibility.md, src/commands/export/identity.rs, src/commands/export/bundle.rs, src/commands/export/template.rs, tests/cli_export.rs"
review_set: subset
---

## Summary

The computed matrix and reverse-gap/stub inspection show unbacked or ignored-only coverage for the new reproducibility and producer identity obligations. Plan completion: not assessed.

## Verdict

**FAIL** — The computed matrix and reverse-gap/stub inspection show unbacked or ignored-only coverage for the new reproducibility and producer identity obligations.

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

## Coverage

Plan completion: not assessed. `quoin matrix --repo . --json` and `quire matrix --scope . --format json` were run. The changed criteria have these matrix states: FR-034-AC-19 `untagged`; FR-037-AC-6 `untagged`; NFR-011-AC-1 and AC-3 `tagged-by-ignored-test`. The focused real-output log records successful explicit executions of the ignored PDF/DOCX reproducibility tests, but there is no bound run evidence in the matrix. Reverse inspection found no stub or unowned behavior in the changed production paths. The audit used the repo-wide computed matrix; pre-existing unrelated gaps were excluded from findings.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | high | FR-034-AC-19 is untagged: the byte-equality test does not bind this producer-identity reproducibility criterion. | spec/functional/FR-034-document-identity.md:163 |
| FND-002 | high | FR-037-AC-6 is untagged: no trace tag binds the new producer identity and source-date byte-stability criterion. | spec/functional/FR-037-export-clock-free-output.md:66 |
| FND-003 | high | NFR-011-AC-1 and AC-3 are bound only by ignored real-converter tests, so the computed matrix does not count them as live test evidence. | tests/cli_export.rs:3798 |
| FND-004 | high | NFR-011-AC-3 is bound only by an ignored real-converter test, so the normal test lane does not exercise its byte-equality oracle. | tests/cli_export.rs:3798 |

## Dispositions

| FND | outcome | reason |
| --- | --- | --- |
| FND-001 | fixed | `after_excerpt`: `    "FR-034-AC-19",` at tests/cli_export.rs:3799; quire matrix now reports `tagged`. |
| FND-002 | fixed | `after_excerpt`: `    "FR-037-AC-6",` at tests/cli_export.rs:3800; quire matrix now reports `tagged`. |
| FND-003 | fixed | `after_excerpt`: `#[test]` followed by the focused function, without `#[ignore]`, at tests/cli_export.rs:3805-3806; the matrix reports NFR-011-AC-1 `tagged`. |
| FND-004 | fixed | `after_excerpt`: the same non-ignored focused test carries `"NFR-011-AC-3"` at tests/cli_export.rs:3802; the matrix reports NFR-011-AC-3 `tagged`. |
