---
id: FR-037
title: "Export output carries no wall-clock or host-dependent content"
type: FR
relationships:
  - target: "ix://agent-ix/quire-cli/spec/stakeholder/StR-004"
    type: "implements"
    cardinality: "1:1"
  - target: "ix://agent-ix/quire-cli/spec/usecase/US-010"
    type: "implements"
  - target: "ix://agent-ix/quire-cli/spec/functional/FR-032"
    type: "extends"
  - target: "ix://agent-ix/quire-cli/spec/functional/FR-034"
    type: "consumes"
  - target: "ix://agent-ix/quire-cli/spec/non-functional/NFR-011"
    type: "references"
---
# FR-037: Export output carries no wall-clock or host-dependent content

## Description

`quire export` SHALL make the conversion independent of the moment, the host and
the process environment, so that the stability property of
[NFR-011](../non-functional/NFR-011-export-reproducibility.md) holds. The date of
a document is the source commit date of [FR-034](./FR-034-document-identity.md)
and nothing else.

## Behavior

- The converter requests of [FR-032](./FR-032-export-converter-execution.md) SHALL
  set the environment variable `SOURCE_DATE_EPOCH` to the Unix time of the source
  commit date, in addition to `HOME` and `TMPDIR`, so a converter that stamps a
  creation or modification time stamps the commit time. The environment field of
  FR-032 is read with this one added variable.
- Document properties that name a time (DOCX core properties `created` and
  `modified`, PDF creation and modification dates) SHALL hold the source commit
  time, or be absent. They SHALL NOT hold the export time.
- The command SHALL pass the converter no host name, user name, absolute path of
  the repository or of the capability root as document text or property.
- Where a converter would record a path or a random identifier in the file, the
  command SHALL set the option or metadata that suppresses it; a property it
  cannot suppress is excluded from the stability property by NFR-011 and named there.
- The command SHALL NOT add a generation timestamp, a page of "exported on", or
  a random document identifier to any output.
- Dates inside the body are the author's text and are not rewritten.

## Acceptance Criteria

| ID | Criteria | Verification |
| --- | --- | --- |
| FR-037-AC-1 | Every request made by the tests holds exactly the environment variables `HOME`, `TMPDIR` and `SOURCE_DATE_EPOCH`, and `SOURCE_DATE_EPOCH` equals the Unix time of the fixture's commit. | Test |
| FR-037-AC-2 | With real pandoc and typst, the DOCX `docProps/core.xml` `created` and `modified` values and the PDF creation date each equal the fixture's commit time or are absent, and none is within a day of the test's wall-clock time when the commit is dated years earlier. | Test |
| FR-037-AC-3 | The extracted text of both formats holds no host name, user name, capability-root path or repository path of the test run. | Test |
| FR-037-AC-4 | A search of the export code finds no read of the wall clock, local time zone or file modification time on the path from input to request. | Inspection |

## Dependencies

- **Upstream**: [FR-032](./FR-032-export-converter-execution.md), [FR-034](./FR-034-document-identity.md).
- **Downstream**: [NFR-011](../non-functional/NFR-011-export-reproducibility.md).
