---
id: FR-039
title: "Export several documents as one bundle"
type: FR
relationships:
  - target: "ix://agent-ix/quire-cli/spec/stakeholder/StR-004"
    type: "implements"
    cardinality: "1:1"
  - target: "ix://agent-ix/quire-cli/spec/usecase/US-010"
    type: "implements"
  - target: "ix://agent-ix/quire-cli/spec/functional/FR-031"
    type: "extends"
  - target: "ix://agent-ix/quire-cli/spec/functional/FR-034"
    type: "consumes"
  - target: "ix://agent-ix/quire-cli/spec/functional/FR-035"
    type: "extends"
---
# FR-039: Export several documents as one bundle

## Description

`quire export` SHALL accept more than one Markdown `<INPUT>` and write them as one
output file, in command-line order, for a package such as a release dossier.

```text
quire export <INPUT> <INPUT>... --to pdf|docx --output <PATH>
             --bundle-title <TEXT> --bundle-id <ID> [--release <TAG>]
```

## Behavior

- Every input is validated as [FR-031](./FR-031-export-subcommand.md) section A
  requires and is given a document identity as
  [FR-034](./FR-034-document-identity.md) defines, from the one source
  repository and the one `--source-rev`. Inputs from different repositories are
  refused.
- The bundle identity is its title (`--bundle-title`), its id (`--bundle-id`),
  `--release` when given, the source commit hash and the Date. The Date is the
  source commit date, as for one document.
- The default template ([FR-035](./FR-035-default-export-template.md)) gives the
  bundle one title page, followed by a document list table (id, title, version or
  release) in command-line order, one table of contents covering all documents,
  and then each document starting on a new page under its own title as a
  top-level heading, with its headings nested below. Headers and footers use the
  bundle id and the bundle's commit prefix. Page numbers run across the bundle.
- The bundle SHALL NOT merge, reorder, deduplicate or edit documents. A repeated
  input appears twice.
- The sources reach the converter unchanged as `in/document-1.md` through
  `in/document-<n>.md`; with one input the name stays `in/document.md`
  ([FR-032](./FR-032-export-converter-execution.md)).
- An organisation template receives the bundle identity and a `documents` metadata
  list of the per-document identities. `--approval` verdicts apply to the bundle
  as a whole and appear once, after the last document.
- The bundle limit is 256 inputs.

## Acceptance Criteria

| ID | Criteria | Verification |
| --- | --- | --- |
| FR-039-AC-1 | Exporting three committed fixtures to PDF yields one file whose extracted text holds the bundle title page, a document list naming the three ids in command-line order, and the three bodies in that order, each starting on a new page. | Test |
| FR-039-AC-2 | The table of contents holds the headings of all three documents in order, and the footer `Page N of M` runs across the whole bundle with M equal to the page count. | Test |
| FR-039-AC-3 | A repeated input appears twice; swapping the command-line order swaps the document order. | Test |
| FR-039-AC-4 | The stand-in pandoc request lists `in/document-1.md` through `in/document-3.md`, each byte-equal to its source, and a single-input export still lists `in/document.md`. | Test |
| FR-039-AC-5 | A bundle without `--bundle-title` or `--bundle-id`, `--bundle-title` with one input, inputs from two repositories, and 257 inputs each exit 3 naming the cause with no converter started. | Test |
| FR-039-AC-6 | One input in the bundle that fails validation or is uncommitted makes the whole export exit 3 naming that file and writes no output. | Test |

## Dependencies

- **Upstream**: [FR-031](./FR-031-export-subcommand.md), [FR-034](./FR-034-document-identity.md), [FR-035](./FR-035-default-export-template.md).
- **Downstream**: [FR-040](./FR-040-export-template-and-identity-errors.md).
