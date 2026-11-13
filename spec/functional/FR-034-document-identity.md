---
id: FR-034
title: "Export document identity from the artifact and its source commit"
type: FR
relationships:
  - target: "ix://agent-ix/quire-cli/spec/stakeholder/StR-004"
    type: "implements"
    cardinality: "1:1"
  - target: "ix://agent-ix/quire-cli/spec/usecase/US-010"
    type: "implements"
  - target: "ix://agent-ix/quire-cli/spec/functional/FR-031"
    type: "extends"
  - target: "ix://agent-ix/quire-cli/spec/functional/FR-033"
    type: "references"
  - target: "ix://agent-ix/quire-cli/spec/functional/FR-040"
    type: "references"
---
# FR-034: Export document identity from the artifact and its source commit

## Description

`quire export` of a Markdown input SHALL assemble a document identity of five
fields, and SHALL hand it to the converter as data for the template
([FR-035](./FR-035-default-export-template.md),
[FR-036](./FR-036-organisation-export-template.md)). The identity ties the
exported file to one committed revision of its source. No field is taken from the
wall clock, the host or the user's environment.

```text
quire export <INPUT>... --to pdf|docx --output <PATH>
             [--source-repo <DIR>] [--source-rev <REV>] [--release <TAG>]
```

## Identity fields

| Field | Source |
| --- | --- |
| Title | The `title` frontmatter member of the input. |
| Document id | The `id` frontmatter member of the input. |
| Version | The `version` frontmatter member of the input, when present. |
| Release | `--release <TAG>`: a tag of the source repository that resolves to the source commit. |
| Source commit hash | The full hash of the commit selected by `--source-rev` (default `HEAD`) in the source repository. |
| Date | The committer date of that commit, in UTC, as `YYYY-MM-DD`. |

At least one of Version and Release SHALL be present; both are shown when both
are present.

## Behavior

- The source repository is `--source-repo`, or the git repository that contains
  the input file when the flag is absent.
- The command SHALL read the commit, its date and the tag in-process from the
  repository's object store. 
- The command SHALL NOT start a `git` process and SHALL NOT
  link a network transport, so [FR-032](./FR-032-export-converter-execution.md)
  stays the only source of child processes.
- The command SHALL NOT read the wall clock, the time zone or a file's
  modification time for any identity field or any text of the output.
- The command SHALL require the input file to be tracked at the source commit and
  byte-equal to the blob committed there, because an identity naming a commit the
  content is not from would be false. An input with uncommitted changes is refused.
- The identity values reach the converter only as members of the data file
  `in/metadata.yaml`, written as JSON string values, never as an argument.
- The identity applies to Markdown input only. A JSON input
  ([FR-031](./FR-031-export-subcommand.md) section B) carries no identity, and the
  options of this FR and of FR-035 through FR-039 are refused with a JSON input.

## Acceptance Criteria

| ID | Criteria | Verification |
| --- | --- | --- |
| FR-034-AC-1 | Exporting a fixture committed with an `id`, a `title` and a `version` to `docx` yields extracted text holding all five fields: the title, the id, the version, the full source commit hash and the committer date as `YYYY-MM-DD`. | Test |
| FR-034-AC-2 | With `--release v1.2` naming a tag on the source commit, the Release field reads `v1.2`; a fixture with no `version` member and no `--release` is refused ([FR-040](./FR-040-export-template-and-identity-errors.md)). | Test |
| FR-034-AC-3 | A fixture committed with a committer date of `2001-02-03T23:30:00-05:00` yields the Date `2001-02-04` (UTC), and the date is the same when the test process runs under a different `TZ` and a different system clock. | Test |
| FR-034-AC-4 | With `--source-rev` naming an older commit at which the input equals the working file, the identity holds that commit's hash and date; naming a commit at which the input differs from the working file is refused. | Test |
| FR-034-AC-5 | An input with uncommitted changes, an untracked input, and an input outside any repository each exit 3 naming the file and writing no output. | Test |
| FR-034-AC-6 | During an export, `quire` starts no process other than `engineering-assurance`, and the dependency list adds no crate that links a network transport. | Test |
| FR-034-AC-7 | A fixture whose title is `--output=/tmp/x` exports with that text as the title and no change to the arguments of any request; the identity values appear in `in/metadata.yaml` only. | Test |
| FR-034-AC-8 | A JSON input with `--source-repo`, with `--release` or with `--source-rev` exits 3 naming the flag. | Test |

## Dependencies

- **Upstream**: [FR-031](./FR-031-export-subcommand.md), [FR-032](./FR-032-export-converter-execution.md).
- **Downstream**: [FR-035](./FR-035-default-export-template.md), [FR-036](./FR-036-organisation-export-template.md), [FR-040](./FR-040-export-template-and-identity-errors.md).
