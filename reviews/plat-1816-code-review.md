---
id: SR-20261116045032
title: "Code review — export identity and provenance"
type: SpecReview
analysis: code-review
scope: "agent-ix/quire-cli; spec/functional/FR-034-document-identity.md, spec/functional/FR-035-default-export-template.md, spec/functional/FR-036-organisation-export-template.md, spec/functional/FR-037-export-clock-free-output.md, spec/non-functional/NFR-011-export-reproducibility.md, src/commands/export/identity.rs, src/commands/export/bundle.rs, src/commands/export/template.rs, tests/cli_export.rs"
review_set: subset
---

## Summary

Reviewed all nine changed files and the source/test paths needed to establish identity, provenance, and reproducibility behavior. The implementation matches the bounded producer/exporter identity model; two edited criteria lack direct test trace tags.

## Verdict

**CONDITIONAL** — Reviewed all nine changed files and the source/test paths needed to establish identity, provenance, and reproducibility behavior.

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

Rust lane: `cargo fmt --check`, focused unit/CLI checks, real PDF/DOCX metadata and reproducibility checks, focused Clippy, and `git diff --check` were reported exit 0 by the coder. Their recorded commands were run through `locked-build`; this review did not rerun them. No CI workflow files changed. The source and tests were inspected for panic paths, external input handling, serialization, determinism, and assertion strength. No repo AssuranceProfile applies.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | medium | FR-034-AC-19 has no trace-tagged test; the byte-comparison real-converter test is tagged only to NFR-011 criteria. | tests/cli_export.rs:3798 |
| FND-002 | medium | FR-037-AC-6 has no trace-tagged test despite the producer identity and byte-stability criterion. | tests/cli_export.rs:3798 |

## New findings (disposition pass 1)

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-003 | medium | FR-034-AC-19's byte-equality test compares outputs only on one host, so it cannot detect host-dependent output bytes. | tests/cli_export.rs:3838 |

The new real-output test varies time zone, waits for the next wall-clock second, and changes working/output/capability paths, but both exports run on the same host. A host-specific value embedded in both outputs would therefore pass the byte-equality assertion. The code review did not opt into semantic gap analysis; this is a test-oracle finding from the requested focused oracle check.

## Dispositions

| FND | outcome | reason |
| --- | --- | --- |
| FND-001 | fixed | `after_excerpt`: `    "FR-034-AC-19",` at tests/cli_export.rs:3799. The real-output test now binds the criterion directly. |
| FND-002 | fixed | `after_excerpt`: `    "FR-037-AC-6",` at tests/cli_export.rs:3800. The real-output test now binds the criterion directly. |
