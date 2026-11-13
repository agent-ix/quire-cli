---
id: FR-033
title: "Export errors: missing converter, converter failure, invalid input"
type: FR
relationships:
  - target: "ix://agent-ix/quire-cli/spec/functional/FR-031"
    type: "extends"
  - target: "ix://agent-ix/quire-cli/spec/functional/FR-032"
    type: "extends"
  - target: "ix://agent-ix/quire-cli/spec/functional/FR-007"
    type: "references"
---
# FR-033: Export errors: missing converter, converter failure, invalid input

## Description

`quire export` SHALL classify each failure into one of the kinds below, name the
cause on stderr, use the exit codes of [FR-007](./FR-007-exit-codes.md), and
leave no file at `--output`.

## Behavior

| Kind | Cause | Exit | Diagnostic |
| --- | --- | --- | --- |
| Missing converter | `pandoc` (or `typst` for `--to pdf`) or `engineering-assurance` is not found on PATH or at the flag path or is not executable, or the result state is `unavailable`. | 3 | Names the tool, the flag that overrides it and where it looked. |
| Converter failure | The result state is `failed`, `timed_out`, `refused`, `malformed_response` or `cancelled`, or the required output is absent. | 4 | Names the tool, the state, the exit code if any, and the producer's stderr as the executor retains it. |
| Unsupported host | The result state is `containment_failure` on a host other than Linux. | 4 | States that export is unsupported on this host because producer-execute containment requires Linux. |
| Containment failure | The result state is `containment_failure` on Linux. | 4 | Names the state and the executor's reason. |
| Executor error | `producer-execute` exits 2 with an `engineering-assurance.error/v1` machine error, for example `producer_execute_request_invalid`. | 4 | Names the machine-error code and message, and says the installed `engineering-assurance` may not match this `quire`. |
| Invalid input | Input file missing, wrong extension, not valid UTF-8, a Markdown file failing validation, JSON that does not parse or does not match its `--kind`, or a flag combination that FR-031 refuses. | 3 | Names the file or flag and the first cause. |
| Unreadable input | The input exists but a permission refusal prevents reading it. | 2 | Names the file. |
| Output refused | `--output` names the same file as `<INPUT>`. | 2 | Names the path; checked before any conversion. |
| Output unwritable | The `--output` directory is missing or unwritable, or `--output` is a directory. | 4 | Names the path; checked before any conversion and again at the copy. |

- Failure SHALL never produce a partial `--output`. A pre-existing file at
  `--output` SHALL be left unchanged on failure.
- The command SHALL detect a missing converter before any conversion starts, so
  a PDF export with Typst absent fails before pandoc runs.
- The command SHALL NOT retry, substitute another converter or print a
  successful message on failure.
- The command SHALL NOT pass input text as a converter argument, path or option;
  input text reaches the converter only as the content of `in/document.md`.

## Acceptance Criteria

| ID | Criteria | Verification |
| --- | --- | --- |
| FR-033-AC-1 | With `pandoc` absent, `--to docx` exits 3, names `pandoc` and `--pandoc`, and writes no file. | Test |
| FR-033-AC-2 | With `typst` absent and `pandoc` present, `--to pdf` exits 3 naming `typst`, and no pandoc request was made. | Test |
| FR-033-AC-3 | With `engineering-assurance` absent, the export exits 3 naming `engineering-assurance`. | Test |
| FR-033-AC-4 | A fixture pandoc that exits non-zero with a stderr message makes the export exit 4 and print that message, and a fixture that outlives the request timeout exits 4 naming `timed_out`. | Test |
| FR-033-AC-5 | A fixture pandoc that exits 0 without writing the output exits 4. | Test |
| FR-033-AC-6 | Each of a missing input, a `.txt` input, a non-UTF-8 input, malformed JSON, and a JSON input whose shape does not match `--kind matrix` exits 3 naming the file, with no converter started. | Test |
| FR-033-AC-7 | After a failed export over a pre-existing `--output`, that file's bytes are unchanged. | Test |
| FR-033-AC-8 | A stand-in `engineering-assurance` that exits 2 with a machine error makes the export exit 4 naming the error code; a stand-in result `containment_failure` on a non-Linux host makes the export exit 4 with the unsupported-host message. | Test |
| FR-033-AC-9 | An input without read permission exits 2; `--output` equal to the input exits 2 and leaves the input unchanged; a missing output directory and an `--output` that is a directory each exit 4; none starts a converter. | Test |
| FR-033-AC-10 | A fixture whose first heading reads `--output=/tmp/x` exports with that text in the document and no change to the arguments of any request. | Test |

## Dependencies

- **Upstream**: [FR-031](./FR-031-export-subcommand.md), [FR-032](./FR-032-export-converter-execution.md) and [FR-007](./FR-007-exit-codes.md).
