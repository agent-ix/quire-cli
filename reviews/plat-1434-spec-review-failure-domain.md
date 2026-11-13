---
id: SR-20261112234814
title: "PLAT-1434 quire-cli PR181 spec-review/failure-domain"
type: SpecReview
analysis: failure-domain
scope: "agent-ix/quire-cli PR181; spec/functional/FR-030..FR-032, spec/non-functional/NFR-010, spec/usecase/US-009"
review_set: subset
---

## Summary

Ticket: PLAT-1434. Failure-domain analysis of the export error model (FR-032) and execution path (FR-031) against EA producer-execute's real outcomes.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | medium | EA FR-029 states that a request yielding no result writes an `engineering-assurance.error/v1` machine error and exits 2 (`producer_execute_request_unreadable`, `_request_invalid`, `_request_structure_invalid`, `_result_encoding_failed`). An engineering-assurance version whose request schema differs from what quire builds hits this path. FR-031 says quire 'reads its single result line', and FR-032's three kinds classify only result states and absent tools. This outcome has no exit code and no diagnostic. Add it, for example as a converter failure naming the machine-error code. | spec/functional/FR-032-export-errors.md:23-29, spec/functional/FR-031-export-converter-execution.md:72 |
| FND-002 | low | On any non-Linux host, EA FR-019/FR-029 always return `containment_failure`. Every export therefore exits 4 on macOS and Windows, which is plausibly the platform of US-009's office-tools reviewer. FR-031 §B notes this, but FR-032's diagnostic for that state is only 'names the tool, the state'. The user gets no message that export is unsupported on the host. | spec/functional/FR-031-export-converter-execution.md:74, spec/functional/FR-032-export-errors.md:26 |
| FND-003 | low | Output-side failures are unclassified: `--output` naming the input file itself, an output directory that is missing or unwritable, or an `--output` that is a directory. FR-032 says 'each failure' falls into one of three kinds, but none of them fits. FR-007 would give 4 for output I/O. State it, and state whether `--output` equal to `<INPUT>` is refused before conversion. | spec/functional/FR-032-export-errors.md:21 |

## Examined scope

```yaml
scope:
- id: FR-032
  path: spec/functional/FR-032-export-errors.md
  role: examined
  excerpt: '`quire export` SHALL classify each failure into one of three kinds, name the cause on stderr, use the exit codes of FR-007, and leave no file at `--output`.'
- id: FR-031
  path: spec/functional/FR-031-export-converter-execution.md
  role: examined
  excerpt: '`quire` SHALL launch only `engineering-assurance producer-execute`, with the request on stdin (`--request -`), and read its single result line. It SHALL NOT call a converter, a shell or any other executable.'
- id: FR-032-AC-1
  path: spec/functional/FR-032-export-errors.md
  role: examined
  excerpt: With `pandoc` absent, `--to docx` exits 3, names `pandoc` and `--pandoc`, and writes no file.
- id: FR-032-AC-2
  path: spec/functional/FR-032-export-errors.md
  role: examined
  excerpt: With `typst` absent and `pandoc` present, `--to pdf` exits 3 naming `typst`, and no pandoc request was made.
- id: FR-032-AC-3
  path: spec/functional/FR-032-export-errors.md
  role: examined
  excerpt: With `engineering-assurance` absent, the export exits 3 naming `engineering-assurance`.
- id: FR-032-AC-4
  path: spec/functional/FR-032-export-errors.md
  role: examined
  excerpt: A fixture pandoc that exits non-zero with a stderr message makes the export exit 4 and print that message, and a fixture that outlives the request timeout exits 4 naming `timed_out`.
- id: FR-032-AC-5
  path: spec/functional/FR-032-export-errors.md
  role: examined
  excerpt: A fixture pandoc that exits 0 without writing the output exits 4.
- id: FR-032-AC-6
  path: spec/functional/FR-032-export-errors.md
  role: examined
  excerpt: Each of a missing input, a `.txt` input, a non-UTF-8 input, malformed JSON, and a JSON input whose shape does not match `--kind matrix` exits 3 naming the file, with no converter started.
- id: FR-032-AC-7
  path: spec/functional/FR-032-export-errors.md
  role: examined
  excerpt: After a failed export over a pre-existing `--output`, that file's bytes are unchanged.
- id: FR-030-AC-8
  path: spec/functional/FR-030-export-subcommand.md
  role: examined
  excerpt: Exporting the same input twice never writes a file other than `--output`, and the working directory and input directory hold no new file.
- id: FR-031-AC-6
  path: spec/functional/FR-031-export-converter-execution.md
  role: examined
  excerpt: After success and after each failure of FR-032, the temporary capability root no longer exists.
- id: US-009
  path: spec/usecase/US-009-export-document-to-pdf-docx.md
  role: context_only
  excerpt: As a reviewer or approver who works in office tools rather than a repository I want a Quire Markdown artifact, or a JSON report the toolchain wrote, as a PDF or DOCX file So that I can read, circulate and sign it without a Markdown viewer or a JSON reader.
```

## Verdict

Changes required (FND-001). The no-partial-output rule, the pre-existing-output rule (FR-032-AC-7), the missing-converter-before-conversion rule (FR-032-AC-2) and the capability-root cleanup (FR-031-AC-6) are sound and testable.

## Dispositions

| FND | outcome | reason |
| --- | --- | --- |
| FND-001 | fixed | - |
| FND-002 | fixed | - |
| FND-003 | fixed | - |
