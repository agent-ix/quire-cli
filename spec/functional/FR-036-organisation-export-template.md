---
id: FR-036
title: "Organisation-supplied export template from a local path"
type: FR
relationships:
  - target: "ix://agent-ix/quire-cli/spec/stakeholder/StR-004"
    type: "implements"
    cardinality: "1:1"
  - target: "ix://agent-ix/quire-cli/spec/usecase/US-010"
    type: "implements"
  - target: "ix://agent-ix/quire-cli/spec/functional/FR-031"
    type: "extends"
  - target: "ix://agent-ix/quire-cli/spec/functional/FR-035"
    type: "extends"
  - target: "ix://agent-ix/quire-cli/spec/functional/FR-034"
    type: "consumes"
  - target: "ix://agent-ix/quire-cli/spec/functional/FR-032"
    type: "references"
---
# FR-036: Organisation-supplied export template from a local path

## Description

`quire export` SHALL accept `--template <PATH>`, a template file the user holds
locally, and SHALL use it in place of the default template of
[FR-035](./FR-035-default-export-template.md). The file is the organisation's;
Quire never ships, fetches or stores it.

```text
quire export <INPUT> --to pdf|docx --output <PATH> --template <PATH>
```

## Behavior

- For `--to pdf` the template is a Typst template file with extension `.typ`,
  passed to pandoc as its template. For `--to docx` it is a Word document with
  extension `.docx`, passed to pandoc as its reference document, which supplies
  styles, page setup, headers and footers.
- The command SHALL resolve the path to a canonical regular file, refuse one
  larger than 16 MiB, and copy its bytes unchanged to `in/template.typ` or
  `in/template.docx` in the capability root. The converter reads only that copy.
- The template receives the identity of [FR-034](./FR-034-document-identity.md)
  and the approvals of [FR-038](./FR-038-approval-block.md) as pandoc metadata
  variables from `in/metadata.yaml`: `title`, `document-id`, `version`,
  `release`, `commit`, `date` and `approvals`. A variable that has no value is
  absent. Which of them the template shows is the template's choice; the command
  does not inspect the template.
- A template is one self-contained file. A template that needs another file or a
  downloaded package fails inside the sandboxed converter, and the failure is a
  converter failure ([FR-033](./FR-033-export-errors.md)).
- The command SHALL NOT copy the template, or any part of it, to any location
  other than the capability root, SHALL NOT fall back to the default template when
  the template fails, and SHALL NOT read a template from a URL.
- The path is the user's input and is not resolved against the repository, so a
  template can live outside every repository.

## Acceptance Criteria

| ID | Criteria | Verification |
| --- | --- | --- |
| FR-036-AC-1 | With a fixture `.typ` template that prints a fixed marker line and the `document-id` variable, a PDF export holds the marker and the id and none of the default title page text. With a fixture `.docx` reference document whose footer holds a marker, the DOCX footer holds that marker. | Test |
| FR-036-AC-2 | The template path passed to the request is under the capability root as `in/template.typ` or `in/template.docx`, its bytes equal the user's file, and after the export the capability root is gone and no copy exists beside `--output`, the input or the repository. | Test |
| FR-036-AC-3 | A stand-in pandoc that records `in/metadata.yaml` sees the keys `title`, `document-id`, `commit`, `date` and the key `version` or `release`, and sees `approvals` only when approvals are supplied. | Test |
| FR-036-AC-4 | A template that fails to compile makes the export exit 4 with the converter's message ([FR-033](./FR-033-export-errors.md)), writes no output file and does not retry with the default template. | Test |
| FR-036-AC-5 | A `.docx` template with `--to pdf`, a `.typ` template with `--to docx`, a missing path, a directory, and a file above 16 MiB each exit 3 naming the path ([FR-040](./FR-040-export-template-and-identity-errors.md)) with no converter started. | Test |
| FR-036-AC-6 | `--template` with a JSON input exits 3 naming the flag. | Test |

## Dependencies

- **Upstream**: [FR-031](./FR-031-export-subcommand.md), [FR-032](./FR-032-export-converter-execution.md), [FR-034](./FR-034-document-identity.md), [FR-035](./FR-035-default-export-template.md).
- **Downstream**: [FR-040](./FR-040-export-template-and-identity-errors.md).
