---
id: FR-032
title: "Run the export converter through producer-execute"
type: FR
relationships:
  - target: "ix://agent-ix/quire-cli/spec/functional/FR-031"
    type: "extends"
  - target: "ix://agent-ix/engineering-assurance/FR-019"
    type: "references"
  - target: "ix://agent-ix/engineering-assurance/FR-029"
    type: "references"
  - target: "ix://agent-ix/engineering-assurance/FR-038"
    type: "references"
  - target: "ix://agent-ix/quire-cli/spec/functional/FR-042"
    type: "references"
---
# FR-032: Run the export converter through producer-execute

## Description

`quire export` SHALL run every converter as one request of the
`engineering-assurance.producer-execution-request/v1` protocol, executed by
`engineering-assurance producer-execute`
([EA FR-029](ix://agent-ix/engineering-assurance/FR-029)), so a converter is a
declared external tool and `quire` launches no converter process itself. A
Markdown input with Mermaid blocks adds one `mmdr` request per block, defined by
[FR-042](./FR-042-export-mermaid-diagrams.md) with the field table below.

## Converter decision

Pandoc is the converter. Typst is a declared second tool used only to typeset
PDF. `mmdr` is a declared third tool used only to render Mermaid blocks to
images ([FR-042](./FR-042-export-mermaid-diagrams.md)); it converts no document.

- DOCX needs a document-format writer. Pandoc writes DOCX natively from
  Markdown. Typst writes only PDF, so Typst alone cannot serve both formats.
- Pandoc cannot typeset PDF on its own: it hands the document to a PDF engine,
  and a pandoc-launched engine would be a child process outside the
  producer-execute boundary. Pandoc therefore writes Typst source
  (`-t typst`), and Typst compiles that source to PDF as its own request.
  Typst is chosen over a LaTeX engine because it is one self-contained
  executable, where a LaTeX distribution is many packages.
- Pandoc and Typst are installed by the user and found on PATH or through the
  flags of [FR-031](./FR-031-export-subcommand.md). No converter binary,
  template file or reference document is added to this repository or its package;
  the default template is generated at export time
  ([FR-035](./FR-035-default-export-template.md)) and an organisation template is
  the user's local file ([FR-036](./FR-036-organisation-export-template.md)).

## Behavior

### A. Requests

- A Markdown input with Mermaid blocks SHALL first make the `mmdr` requests of
  [FR-042](./FR-042-export-mermaid-diagrams.md), which add the
  diagram files to the pandoc request there defined; every Markdown input's pandoc request carries `--lua-filter`. The pandoc request is
  made only if all of them completed.
- `--to docx` SHALL make one pandoc request. `--to pdf` SHALL make a pandoc
  request that writes `typ/document.typ`, then a typst request that compiles it
  to `out/document.pdf`. The typst request is made only if the pandoc request
  completed. All requests of one export, including the `mmdr` requests of FR-042, SHALL share one capability root.
- Each request is the complete field set below. Fields not listed are not set.

| Request field | Export definition |
| --- | --- |
| `protocol` | `engineering-assurance.producer-execution-request/v1`. |
| `producer` | `name` `pandoc`, `typst` or `mmdr` ([FR-042](./FR-042-export-mermaid-diagrams.md)); `executable` the canonical absolute path (below); `version` the literal `unreported`. `quire` does not probe a converter for its version, because a probe would be a second launch. |
| `caller` | `{"kind": "quire.export", "version": "1"}`. |
| `procedure` | `direct`. |
| `capabilityRoot` | A fresh absolute temporary directory holding `in/` (the Markdown, `in/document.md`, with `in/metadata.yaml` and the template copy), `typ/` (the Typst source, PDF only), and for a Markdown input with Mermaid blocks `in/diagrams/`, `in/diagrams.lua` and `img/` (DOCX) or `typ/img/` (PDF), and `out/` (the output, and the tool's home and temporary directories). |
| `arguments` | Literal entries in order. Pandoc: `--sandbox`, `--standalone`, `--from`, `markdown-yaml_metadata_block`, `--to`, then `docx` or `typst`, `--output`, `out/document.docx` or `typ/document.typ`, `--metadata-file`, `in/metadata.yaml`, then for a Markdown input `--preserve-tabs` (a tab in a code block stays a tab, so the Mermaid filter compares the block with the bytes `quire` wrote), then `--template`, `in/template.typ` (PDF) or `--reference-doc`, `in/template.docx` (DOCX), then, for a Markdown input (with or without Mermaid blocks), `--lua-filter`, `in/diagrams.lua`, then the input files in order: `in/front.md`, then `in/document.md` (one input) or, for each document n of a bundle, `in/separator-<n>.md` and `in/document-<n>.md` ([FR-039](./FR-039-export-bundle.md)), then `in/approval.md` when `--approval` is given ([FR-038](./FR-038-approval-block.md)). The generated files are written by [FR-034](./FR-034-document-identity.md) through [FR-039](./FR-039-export-bundle.md); no argument holds input text. Typst: `compile`, `typ/document.typ`, `out/document.pdf`. |
| `environment` | `HOME=<capabilityRoot>/out/home` and `TMPDIR=<capabilityRoot>/out/tmp`, both created empty before the run, and `SOURCE_DATE_EPOCH` ([FR-037](./FR-037-export-clock-free-output.md)). No other variable. |
| `inputs` | Empty. The tools read their input from the capability root. |
| `stdin` | `{"kind": "null"}`. |
| `outputs` | One entry: role `document`, path the `--output` target above, `required` true; for an `mmdr` request one entry, role `diagram`, the image path ([FR-042](./FR-042-export-mermaid-diagrams.md)). |
| `containment` | `{"profile": "process-group-v1", "contract": {"kind": "quire.export-containment", "version": "1"}}`. |
| `cancellation` | `{"kind": "disabled"}`. |
| `budget` | `timeoutMillis` 120000, `maxStdoutBytes` 1048576, `maxStderrBytes` 1048576, `maxInputBytes` 268435456, `maxOutputArtifacts` 1, `maxOutputBytes` 268435456, `maxDescendants` 64, `maxConcurrency` 1. |
| `response` | The built-in `engineering-assurance.process-evidence` version `1` protocol and `engineering-assurance.cli-process-evidence-adapter` version `1` adapter, with an exit policy admitting exit code 0 only. |

- `quire` SHALL resolve each converter and `engineering-assurance` executable
  to its canonical path (every symbolic link in every component resolved) and
  SHALL create the capability root under a canonical temporary directory, because
  the executor opens the executable and capability root without following
  symbolic links in any component
  ([EA FR-019](ix://agent-ix/engineering-assurance/FR-019)). A tool that is not
  a regular executable file after resolution is a missing converter
  ([FR-033](./FR-033-export-errors.md)). If the executor still refuses the
  request, the diagnostic names the canonical path it passed.
- `quire` SHALL write `in/document.md` before the run, copy the retained output
  artifact to `--output` only on a `completed` result, and remove the capability
  root on every exit path.

### B. Execution path

- `quire` SHALL launch only `engineering-assurance producer-execute`, with the
  request on stdin (`--request -`), and read its single result line. It SHALL
  NOT call a converter, a shell or any other executable.
- If the result state is `containment_failure` on a host other than Linux, the
  export fails as [FR-033](./FR-033-export-errors.md) defines.
- The command SHALL NOT embed a converter, add a converter crate dependency or
  fall back to a second converter when the first is absent.

## Acceptance Criteria

| ID | Criteria | Verification |
| --- | --- | --- |
| FR-032-AC-1 | With fixture executables standing in for pandoc and typst, `--to docx` produces exactly one producer-execute request, and `--to pdf` produces a pandoc request followed by a typst request; no other process is started by `quire` and no version probe is made. | Test |
| FR-032-AC-2 | Every request used by the tests validates against the schema published by `engineering-assurance producer-execute --schema request`, names a canonical `producer.executable`, and equals the field table above. A converter reached through a symbolic link is passed as its resolved target, and a link to a non-file is reported as a missing converter. | Test |
| FR-032-AC-3 | The pandoc request carries `--sandbox`, `--standalone`, `--from markdown-yaml_metadata_block`, `--metadata-file in/metadata.yaml`, `--preserve-tabs` (Markdown input), one template argument under `in/`, one `--output` under `out/` (DOCX) or `typ/` (PDF) and the input files `in/front.md` and `in/document.md` (plus `in/approval.md` with approvals); the typst request carries exactly `compile`, one input `typ/document.typ` and one output `out/document.pdf`. | Inspection |
| FR-032-AC-4 | When the pandoc request does not complete, no typst request is made. | Test |
| FR-032-AC-5 | During a run, the only child process `quire` itself starts is `engineering-assurance`. | Test |
| FR-032-AC-6 | After success and after each failure of [FR-033](./FR-033-export-errors.md), the one temporary capability root no longer exists. | Test |
| FR-032-AC-7 | With real pandoc and typst, a fixture exports to a DOCX that opens as a zip and a PDF that starts with `%PDF`. The test fails naming the missing tool if either is unavailable. | Test |
| FR-032-AC-8 | The requests of an export with Mermaid blocks are those of FR-042-AC-2 and FR-042-AC-3; an export of a Markdown input without Mermaid blocks makes no `mmdr` request and its pandoc request carries `--lua-filter in/diagrams.lua` whose block count is 0; a JSON input makes no `mmdr` request and its pandoc request carries no `--lua-filter` and no `--preserve-tabs`. | Test |

## Dependencies

- **Upstream**: [FR-031](./FR-031-export-subcommand.md); [EA FR-019](ix://agent-ix/engineering-assurance/FR-019) and [EA FR-029](ix://agent-ix/engineering-assurance/FR-029) own the executor and command.
- **Downstream**: [FR-033](./FR-033-export-errors.md), [FR-042](./FR-042-export-mermaid-diagrams.md).
