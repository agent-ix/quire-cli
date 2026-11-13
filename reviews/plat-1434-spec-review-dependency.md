---
id: SR-20261112234815
title: "PLAT-1434 quire-cli PR181 spec-review/dependency"
type: SpecReview
analysis: dependency
scope: "agent-ix/quire-cli PR181; spec/functional/FR-030..FR-032, spec/non-functional/NFR-010, spec/usecase/US-009"
review_set: subset
---

## Summary

Ticket: PLAT-1434. Dependency analysis of the relationships edges added by US-009, FR-030..FR-032 and NFR-010.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | low | FR-030's Dependencies section names FR-004 (validation), FR-006 and FR-007 as upstream, but its frontmatter `relationships:` holds only StR-004, FR-026, FR-031 and FR-032. It also has no edge to the quire-rs validation FR it reuses (FR-004 `consumes` quire-rs FR-032). Prose-only dependencies are invisible to `quire extract` and to blast-radius queries. | spec/functional/FR-030-export-subcommand.md:5-15 |
| FND-002 | low | Within one PR, US-009 addresses targets as `ix://agent-ix/quire-cli/StR-004` and `.../FR-030`, while FR-030/031/032 and NFR-010 use `ix://agent-ix/quire-cli/spec/<dir>/<ID>`. Both forms already occur in the repo and validate. Use one form within the new set. | spec/usecase/US-009-export-document-to-pdf-docx.md:6 |

## Examined scope

```yaml
scope:
- id: FR-030
  path: spec/functional/FR-030-export-subcommand.md
  role: examined
  excerpt: 'The CLI SHALL provide an `export` subcommand that renders one Markdown artifact or one JSON report as a PDF or DOCX file. `export` lives in quire-cli because quire-cli already owns the Markdown artifact boundary and `quire matrix`; the Quoin and SOUP reports reach it as files, so it needs no dependency on Quoin. It is the only export command: Quoin and Engineering Assurance add none.'
- id: FR-031
  path: spec/functional/FR-031-export-converter-execution.md
  role: examined
  excerpt: '`quire export` SHALL run every converter as one request of the `engineering-assurance.producer-execution-request/v1` protocol, executed by `engineering-assurance producer-execute` (EA FR-029), so a converter is a declared external tool and `quire` launches no converter process itself.'
- id: FR-032
  path: spec/functional/FR-032-export-errors.md
  role: examined
  excerpt: '`quire export` SHALL classify each failure into one of three kinds, name the cause on stderr, use the exit codes of FR-007, and leave no file at `--output`.'
- id: NFR-010
  path: spec/non-functional/NFR-010-export-single-converter-single-path.md
  role: examined
  excerpt: Export SHALL use pandoc and typst as its only converters, launched only through `engineering-assurance producer-execute`, with no converter binary, template or reference document placed in any repository or package.
- id: US-009
  path: spec/usecase/US-009-export-document-to-pdf-docx.md
  role: examined
  excerpt: 'relationships: target "ix://agent-ix/quire-cli/StR-004" (traces_to); FR-030 uses "ix://agent-ix/quire-cli/spec/stakeholder/StR-004"'
```

## Verdict

Minor. Every new `ix://` target resolves, the EA FR-019/029/038 targets exist upstream, and enablement (EA producer-execute, the matrix JSON) is upstream of the feature FRs. Only edge completeness and form are open.

## Dispositions

| FND | outcome | reason |
| --- | --- | --- |
| FND-001 | fixed | - |
| FND-002 | fixed | - |
