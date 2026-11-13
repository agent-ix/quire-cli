---
id: SR-20261112234813
title: "PLAT-1434 quire-cli PR181 spec-review/integrity"
type: SpecReview
analysis: integrity
scope: "agent-ix/quire-cli PR181; spec/functional/FR-030..FR-032, spec/non-functional/NFR-010, spec/usecase/US-009"
review_set: subset
---

## Summary

Ticket: PLAT-1434. Integrity analysis: consistency of FR-030..FR-032 and NFR-010 with each other, with FR-007, and with EA FR-019/FR-029/FR-038.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | high | FR-031 sets `producer.version` to 'the version the tool reports for itself', so pandoc and typst must each be run (e.g. `--version`) to learn it. FR-031 §B and FR-031-AC-5 forbid quire to launch any process other than `engineering-assurance`. Running the version probe as a producer-execute request breaks FR-031-AC-1's 'exactly one producer-execute request' for `--to docx`. No conforming implementation can fill the field. Choose one: allow a version-probe request and count it in AC-1, accept a `--pandoc-version`/`--typst-version` input, or record an explicit non-probed value. | spec/functional/FR-031-export-converter-execution.md:54-55, :72, :88, :92 |
| FND-002 | medium | FR-031 says each request 'follows the shape of EA FR-038', but FR-038 is the Syft-specific definition: caller kind `engineering-assurance.sbom-generation`, containment contract `engineering-assurance.syft-containment`, and so on. The export request leaves `producer.name`, `caller`, `containment`, `budget` (including `timeoutMillis`), `cancellation` and `inputs` unstated. FR-032-AC-4 tests 'the request timeout', which no requirement fixes and no flag sets. Give the export its own complete field table, as FR-038 does for Syft. | spec/functional/FR-031-export-converter-execution.md:52, spec/functional/FR-032-export-errors.md:42 |
| FND-003 | medium | FR-032 classifies an 'unreadable' input as Invalid input, exit 3. FR-007, which FR-032 says it follows, states 'Input permission refusals SHALL use 2; missing input SHALL use 3'. A permission-denied input therefore exits 3 under FR-032 and 2 under FR-007. Split unreadable-by-permission (2) from the other invalid-input causes. | spec/functional/FR-032-export-errors.md:27, spec/functional/FR-007-exit-codes.md:29 |
| FND-004 | low | FR-032 maps every converter failure to exit 4. FR-007 defines 4 as 'internal failure: unexpected implementation, serialization or output I/O failure', and an external tool rejecting a document is none of these. The mapping is reasonable, but it changes the meaning of 4 without amending FR-007. State the external-tool class in FR-007, or note it as an FR-007 extension. | spec/functional/FR-032-export-errors.md:26 |
| FND-005 | low | For `--to pdf`, the pandoc request writes `out/document.typ`, and the typst request then reads 'the input under `in/`'. The spec does not say whether the typst request uses a second capability root with the `.typ` copied into its `in/`, or reuses the first root. It also does not say whether the PDF is the 'one required output' of that request alone. Two implementations would differ in cleanup (FR-031-AC-6) and output removal. | spec/functional/FR-031-export-converter-execution.md:48-51 |

## Examined scope

```yaml
scope:
- id: FR-031
  path: spec/functional/FR-031-export-converter-execution.md
  role: examined
  excerpt: '`quire export` SHALL run every converter as one request of the `engineering-assurance.producer-execution-request/v1` protocol, executed by `engineering-assurance producer-execute` (EA FR-029), so a converter is a declared external tool and `quire` launches no converter process itself.'
- id: FR-032
  path: spec/functional/FR-032-export-errors.md
  role: examined
  excerpt: '`quire export` SHALL classify each failure into one of three kinds, name the cause on stderr, use the exit codes of FR-007, and leave no file at `--output`.'
- id: FR-031-AC-1
  path: spec/functional/FR-031-export-converter-execution.md
  role: examined
  excerpt: With fixture executables standing in for pandoc and typst, `--to docx` produces exactly one producer-execute request, and `--to pdf` produces a pandoc request followed by a typst request.
- id: FR-031-AC-5
  path: spec/functional/FR-031-export-converter-execution.md
  role: examined
  excerpt: During a run, the only child process `quire` itself starts is `engineering-assurance`.
- id: FR-032-AC-4
  path: spec/functional/FR-032-export-errors.md
  role: examined
  excerpt: A fixture pandoc that exits non-zero with a stderr message makes the export exit 4 and print that message, and a fixture that outlives the request timeout exits 4 naming `timed_out`.
- id: FR-032-AC-6
  path: spec/functional/FR-032-export-errors.md
  role: examined
  excerpt: Each of a missing input, a `.txt` input, a non-UTF-8 input, malformed JSON, and a JSON input whose shape does not match `--kind matrix` exits 3 naming the file, with no converter started.
- id: FR-007
  path: spec/functional/FR-007-exit-codes.md
  role: context_only
  excerpt: 'Input permission refusals SHALL use 2; missing input SHALL use 3. Output and unclassified I/O failures SHALL use 4. ... | 4 | Internal failure: unexpected implementation, serialization or output I/O failure. |'
```

## Verdict

Changes required. FND-001 is a contradiction no implementation can satisfy. The remaining findings are gaps in the request definition and the exit-code mapping. The converter decision, request ordering (FR-031-AC-4) and output-only-on-completed rule are internally consistent.

## New findings (disposition pass 1)

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-006 | low | The fix amended FR-007 row 4 for external-tool failures. FR-007-AC-9 now also asserts that a missing converter exits 3, but row 3 still lists only input paths, argv, malformed input, archetypes, selection and module/validation failures. A missing external executable is not in the table that defines code 3. Add 'a missing external tool' to row 3. | spec/functional/FR-007-exit-codes.md:24, :47 |
| FND-007 | low | EA FR-019 opens the capability root and the executable 'without following symbolic links in any path component'. FR-031 asks only for an 'absolute resolved path' and a 'fresh absolute temporary directory'. It does not say these must be canonical, symlink-free paths. A PATH entry that is a symlink (a common install layout for pandoc and typst) or a TMPDIR under a symlink then yields `unavailable` or `refused`. FR-032 reports that as 'missing converter' or 'converter failure', and the diagnostic is wrong. State that both paths are canonicalized before the request is built. | spec/functional/FR-031-export-converter-execution.md:55, :58 |

## Dispositions

| FND | outcome | reason |
| --- | --- | --- |
| FND-001 | fixed | - |
| FND-002 | fixed | - |
| FND-003 | fixed | - |
| FND-004 | fixed | - |
| FND-005 | fixed | - |
| FND-006 | fixed | - |
| FND-007 | fixed | - |
