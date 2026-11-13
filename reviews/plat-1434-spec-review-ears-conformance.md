---
id: SR-20261112234816
title: "PLAT-1434 quire-cli PR181 spec-review/ears-conformance"
type: SpecReview
analysis: ears-conformance
scope: "agent-ix/quire-cli PR181; spec/functional/FR-030..FR-032, spec/non-functional/NFR-010, spec/usecase/US-009"
review_set: subset
---

## Summary

Ticket: PLAT-1434. EARS conformance of the new FR/NFR statements; `quire validate` grammar lint reports no warnings on the new files.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | low | NFR-010 says no converter binary, template or reference document is placed 'in any repository or package', but its ACs inspect only this repository and its npm package. Other repositories are outside quire-cli's control and cannot be verified from it. FR-030's 'Quoin and Engineering Assurance add none' has the same problem. Scope both statements to this repository and package. | spec/non-functional/NFR-010-export-single-converter-single-path.md:18, spec/functional/FR-030-export-subcommand.md:24 |
| FND-002 | low | 'Input is data: its text is never used as a command, path or option.' has no SHALL and no subject, and no AC checks it. Restate it as 'The command SHALL NOT pass input text as a converter argument, path or option' and cover it with an AC (for example, a heading that looks like a pandoc flag is rendered as text). | spec/functional/FR-032-export-errors.md:35 |

## Examined scope

```yaml
scope:
- id: FR-030
  path: spec/functional/FR-030-export-subcommand.md
  role: examined
  excerpt: 'The CLI SHALL provide an `export` subcommand that renders one Markdown artifact or one JSON report as a PDF or DOCX file. `export` lives in quire-cli because quire-cli already owns the Markdown artifact boundary and `quire matrix`; the Quoin and SOUP reports reach it as files, so it needs no dependency on Quoin. It is the only export command: Quoin and Engineering Assurance add none.'
- id: FR-030 (Mermaid)
  path: spec/functional/FR-030-export-subcommand.md
  role: examined
  excerpt: A fenced code block whose info string is `mermaid` SHALL be rendered as a verbatim code block preceded by the caption `Diagram source (Mermaid)`. The command SHALL NOT render a Mermaid block to an image, because no diagram renderer is a declared tool, and SHALL print a stderr warning naming each Mermaid block it carried as source.
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
```

## Verdict

Minor. The unwanted-behaviour statements ('If a JSON input does not match ... then the command SHALL refuse') are canonical. Two statements are unverifiable or unmodal as written.

## Dispositions

| FND | outcome | reason |
| --- | --- | --- |
| FND-001 | fixed | - |
| FND-002 | fixed | - |
