---
id: FR-031
title: "quire export subcommand"
type: FR
relationships:
  - target: "ix://agent-ix/quire-cli/spec/stakeholder/StR-004"
    type: "implements"
    cardinality: "1:1"
  - target: "ix://agent-ix/quire-cli/spec/usecase/US-009"
    type: "implements"
  - target: "ix://agent-ix/quire-cli/spec/functional/FR-004"
    type: "consumes"
  - target: "ix://agent-ix/quire-rs/FR-032"
    type: "consumes"
  - target: "ix://agent-ix/quire-rs/FR-006"
    type: "implements"
  - target: "ix://agent-ix/quire-cli/spec/functional/FR-006"
    type: "consumes"
  - target: "ix://agent-ix/quire-cli/spec/functional/FR-007"
    type: "consumes"
  - target: "ix://agent-ix/quire-cli/spec/functional/FR-026"
    type: "references"
  - target: "ix://agent-ix/quire-cli/spec/functional/FR-032"
    type: "references"
  - target: "ix://agent-ix/quire-cli/spec/functional/FR-033"
    type: "references"
  - target: "ix://agent-ix/quire-cli/spec/functional/FR-034"
    type: "references"
---
# FR-031: quire export subcommand

## Description

The CLI SHALL provide an `export` subcommand that renders one Markdown artifact
or one JSON report as a PDF or DOCX file. `export` lives in quire-cli because
quire-cli already owns the Markdown artifact boundary and `quire matrix`; the
JSON reports reach it as files, so it needs no dependency on Quoin.

Boundary: Markdown parsing and validation stay in `quire-rs` (consumed through
[FR-004](./FR-004-validate-subcommand.md)). The converter reads the Markdown body
itself, so `export` parses no Markdown and edits none. The only logic quire-cli
owns is the JSON-to-table text below, which is output formatting for the
converter; [StR-004](../stakeholder/StR-004-thin-boundary-over-quire-rs.md)
records this as a bounded exception, as it does for FR-027. It is not an engine
behavior, so no upstream FR owns it.

```text
quire export <INPUT> --to pdf|docx --output <PATH>
             [--kind matrix|rmm|soup|table] [--module <PATH>]...
             [--pandoc <PATH>] [--typst <PATH>]
             [--engineering-assurance <PATH>]
             [--template <PATH>] [--source-repo <DIR>] [--source-rev <REV>]
             [--release <TAG>] [--approval <PATH>]...
             [--bundle-title <TEXT>] [--bundle-id <ID>]
```

The last two lines of options, and the identity, title page and layout a Markdown
input receives, are specified by [FR-034](./FR-034-document-identity.md) through
[FR-039](./FR-039-export-bundle.md); a Markdown `<INPUT>` may repeat for a
bundle. This FR's section A states the content the converter takes from the
source; the template of FR-035 or FR-036 frames it.

The conversion is the work of external tools launched by Engineering
Assurance; [FR-032](./FR-032-export-converter-execution.md) owns that execution
and [FR-033](./FR-033-export-errors.md) owns the failures.

## Inputs

- `<INPUT>`: a path to a Markdown file (extension `.md`) or a JSON file
  (extension `.json`). The extension selects the input class; `-` is not
  accepted.
- `--to`: the output format, `pdf` or `docx`. Required.
- `--output`: the output file path. Required. It is written only on success.
- `--kind`: required for a JSON input and refused for a Markdown input.
- `--module`: repeatable module directory used to validate a Markdown input. It
  resolves exactly as `--module` of [FR-004](./FR-004-validate-subcommand.md),
  including ambient discovery when omitted. It is refused for a JSON input.
- `--pandoc`, `--typst`, `--engineering-assurance`: paths that override the PATH
  lookup of the `pandoc`, `typst` and `engineering-assurance` executables.

## Outputs

- One file at `--output` in the requested format. Nothing is written to stdout
  on success. Diagnostics go to stderr ([FR-006](./FR-006-io-contract.md)).
- The output is a view and never an artifact: `export` writes no file other
  than `--output`.

## Behavior

### A. Markdown input

- The command SHALL validate the file as `quire validate` does, against the
  `--module` set, before conversion, so an invalid artifact is never exported.
  The content validated and converted is the one committed at the source commit
  ([FR-034](./FR-034-document-identity.md) section A).
- The command SHALL pass the body of the artifact, the bytes after its YAML
  frontmatter block unchanged, to the converter as the Markdown source
  (`in/document.md`); the frontmatter is not converter input, and its `title` is
  shown through the identity of FR-034. The body boundary is the one `extract_frontmatter` of
  [quire-rs FR-006](ix://agent-ix/quire-rs/FR-006) reports, so `export` finds it
  with no Markdown parsing of its own. The converter takes headings,
  ordered and unordered lists, tables, fenced code blocks, emphasis, links and
  block quotes from the body; those SHALL appear in the output.

### B. JSON input

`--kind` selects a table rendering. The JSON is read as data and never executed.
The command renders each table as a Markdown pipe table in the one Markdown text
it hands to the converter.

| Kind | Accepted shape | Table content |
| --- | --- | --- |
| `matrix` | The output of `quire matrix --format json`: a `coverage_matrix` array of `{document, criteria[]}` entries; or no `coverage_matrix` member (zero population). | One table per entry, headed by `document`, one row per criterion with every member of the criterion object as a column. For zero population, the single line `No obligations matched this scope.` |
| `rmm` | The output of `quoin discharge --rmm --json`: a `requirements-mapping-v1` document with a `rows` array (one object per clause: `clauseId`, `force`, `applicability`, `compliance`, `reason`, `evidenceRefs`, `tailoringDescription`, `tailoringRationale`, and no others). Or the pack form of `quoin discharge --rmm --pack --json`: a `requirements-mapping-pack-v1` document with an `index` array and a `matrices` array of `requirements-mapping-v1` documents. | Single form: the line `As of: <asOf>`, one table headed by `clauseSet`, one row per element of `rows`, then `unusedFacts`, when non-empty, as one further table. Pack form: the line `As of: <asOf>`, the `index` as a table, then one such single-form rendering per matrix, in order. |
| `soup` | The output of `quoin soup report --json`: a top-level object with a `rows` array of objects with the members `purl`, `id`, `path`, `version`, `serves`, `known_anomalies`, `support_status`, `end_of_support`, `unresolved_advisory`, `past_end_of_support`, `support_disagreement`, `disposition_clause`, `advisories` and `no_advisory_reported_by`; the object also carries `as_of`, `unmatched`, `scans` and `unreadable_scans`. | The line `As of: <as_of>`, the `rows` as one table, one row per object, with every member as a column; `unmatched` and `scans`, when non-empty, as one further table each, headed by the member name; and `unreadable_scans`, when non-empty, as a bulleted list headed `unreadable_scans`. |
| `table` | A top-level object with one or more array members, each holding objects. | One table per such member, headed by the member name. |

- The `rmm` and `soup` shapes are those the Quoin reports emit today (the
  `RmmRow` row of the requirements mapping matrix and the row of the SOUP
  report); `export` reads the named members and refuses a document that lacks
  `rows`, `index` or `matrices` as its form requires.
- Columns are the union of member names in first-seen order. Every member of
  every row appears; a missing member renders as an empty cell, a scalar as its
  text and a nested value as its compact JSON text. Pipe characters and line
  breaks in a cell are escaped so a cell never splits a row.
- If a JSON input does not match the shape its `--kind` requires, then the
  command SHALL refuse it as invalid input ([FR-033](./FR-033-export-errors.md)).
- Column ordering, units, grouping and captions beyond this are specified by
  PLAT-1603.

### C. Mermaid diagrams

- A fenced code block whose info string is `mermaid` is an ordinary fenced code
  block: the converter prints its source verbatim.
- The command SHALL NOT render a Mermaid block to an image, because no diagram
  renderer is a declared tool.
- The command SHALL NOT edit the Markdown source to mark Mermaid blocks, because
  finding them is Markdown parsing, which stays in `quire-rs`.

### D. Scope exclusions

Document identity, title page, headers and footers, organisation templates,
approval and signature blocks, bundles and stable styling are specified by
[FR-034](./FR-034-document-identity.md) through
[FR-040](./FR-040-export-template-and-identity-errors.md) and
[NFR-011](../non-functional/NFR-011-export-reproducibility.md), not by this FR.
A Markdown input exported by the commands of those FRs needs a committed source
and an identity; JSON inputs carry none.

## Acceptance Criteria

| ID | Criteria | Verification |
| --- | --- | --- |
| FR-031-AC-1 | Exporting a committed fixture FR artifact containing headings, a list, a table and a code block to `pdf` and to `docx` writes both files, and the text extracted from each contains every fixture heading and the code block text. The extractors are `pdftotext` for PDF and an unzip of `word/document.xml` for DOCX, and the test fails, naming the missing tool, if real pandoc, typst or the extractor is unavailable. | Test |
| FR-031-AC-2 | Exporting a committed Markdown file that carries an `id`, a `title` and a `version` and fails validation against the `--module` set exits 3, writes no output file and starts no converter; the same file passes against a module set that admits it. | Test |
| FR-031-AC-3 | A JSON input without `--kind`, a Markdown input with `--kind`, and a JSON input with `--module` each exit 3 with a diagnostic naming the flag. | Test |
| FR-031-AC-4 | Exporting a `quire matrix --format json` fixture with `--kind matrix` to `docx` yields tables whose rows hold every criterion id of the fixture, extracted as in FR-031-AC-1; a fixture without `coverage_matrix` yields the zero-population line. | Test |
| FR-031-AC-5 | A `--kind table` fixture whose rows carry a nested object yields a cell holding that object as compact JSON, and the assembled Markdown text holds every member of every row. | Test |
| FR-031-AC-6 | A fixture with a `mermaid` block exports with the block's source text present and no image in the output; the converter input `in/document.md` equals the bytes of the committed body after the frontmatter. | Test |
| FR-031-AC-7 | A fixture with frontmatter exports with the `title` on the title page and no `type:` frontmatter line in the extracted text. | Test |
| FR-031-AC-8 | After an export, the working directory, the input directory and the output directory hold no file other than the input and `--output`. | Test |
| FR-031-AC-9 | A `--kind rmm` fixture in the single form and one in the pack form, and a `--kind soup` fixture, each export with one table row per element of `rows`; the pack form also yields its index table and one table per matrix. | Test |
| FR-031-AC-10 | A `--kind soup` document lacking `rows`, a `--kind rmm` document that is neither a `requirements-mapping-v1` nor a `requirements-mapping-pack-v1` document, and a pack document lacking `matrices` are each refused as invalid input, exit 3, with no converter started. | Test |
| FR-031-AC-11 | A `--kind rmm` fixture with a non-empty `unusedFacts` and an `asOf`, and a `--kind soup` fixture with a non-empty `unreadable_scans`, `unmatched` and `scans` and an `as_of`, export with the `As of` line, every `unusedFacts` row, and every `unreadable_scans` entry present in the extracted text. | Test |

## Dependencies

- **Upstream**: [FR-026](./FR-026-matrix-subcommand.md) writes the matrix JSON; [FR-004](./FR-004-validate-subcommand.md) validates Markdown; [FR-006](./FR-006-io-contract.md) and [FR-007](./FR-007-exit-codes.md) own streams and exit codes.
- **Downstream**: [FR-032](./FR-032-export-converter-execution.md) executes the conversion; [FR-033](./FR-033-export-errors.md) owns the failures.
