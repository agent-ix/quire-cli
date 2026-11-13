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
  - target: "ix://agent-ix/quire-rs/FR-006"
    type: "implements"
  - target: "ix://agent-ix/quire-cli/spec/functional/FR-033"
    type: "references"
  - target: "ix://agent-ix/quire-cli/spec/functional/FR-040"
    type: "references"
---
# FR-034: Export document identity from the artifact and its source commit

## Description

`quire export` of a Markdown input SHALL assemble a document identity of six
fields, and SHALL hand it to the converter as data and as generated title-page
text ([FR-035](./FR-035-default-export-template.md),
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
| Version | The `version` frontmatter member of the input; optional. |
| Release | `--release <TAG>`: a tag of the source repository that resolves to the source commit; optional. |
| Source commit hash | The full hash of the commit selected by `--source-rev` (default `HEAD`) in the source repository. `--source-rev` is a full 40-digit object hash or a reference name (branch, tag or `HEAD`); abbreviated hashes and revision expressions such as `HEAD~1` are refused, and the diagnostic says so. |
| Date | The committer date of that commit, in UTC, as `YYYY-MM-DD`. |

At least one of Version and Release SHALL be present; both are shown when both
are present. The `title`, `id` and `version` members are strings or numbers; any other type counts as absent. The frontmatter members are read with `extract_frontmatter` of
[quire-rs FR-006](ix://agent-ix/quire-rs/FR-006).

## Behavior

### A. Source repository and content

- The source repository SHALL be `--source-repo`, or the git repository that
  contains the input file when the flag is absent.
- The command SHALL read the commit, its date, the tag, the blob and the
  attributes of the input path in process from the repository's object store,
  with read-only access: no index or worktree write, no ref update, no fetch.
- The command SHALL use the blob committed at the source commit for the input
  path as the content to validate and convert, so the identity and the content
  come from one revision.
- The command SHALL compare the working file with that blob after treating each
  CRLF as LF on both sides, and SHALL refuse the export when they differ, because
  the file has changes that are not committed. A working file whose line endings
  differ from the blob only by CRLF against LF is not a difference.
- The command SHALL refuse an input path to which git applies a `filter`
  attribute (which covers Git LFS), because the committed bytes are then not the
  bytes a reader of the file sees; the diagnostic names the filter.
- The command SHALL NOT read the wall clock, the time zone or a file's
  modification time for any identity field or any text of the output.

### B. No process, no network

- The command SHALL NOT start a `git` process.
- The command SHALL link no git network transport. The git library is the set
  of pure-Rust gitoxide component crates that read the object store, refs and
  attributes (`gix-discover`, `gix-odb`, `gix-ref`, `gix-object`, `gix-hash`,
  `gix-attributes`, `gix-glob`). The `gix` facade is not used because it depends
  on `gix-protocol` and `gix-transport`. The crates `gix`, `gix-protocol`,
  `gix-transport` and `gix-command` are absent from the dependency graph, as
  [NFR-004](../non-functional/NFR-004-no-network.md)-AC-1 checks.

### C. Hand-off to the converter

- The command SHALL write the identity into `in/metadata.yaml` under the one
  reserved key `quire-export` (members `title`, `document-id`, `version`,
  `release`, `commit`, `date`), as JSON string values, and SHALL pass it with
  `--metadata-file`. It SHALL NOT pass an identity value as an argument.
- The command SHALL escape each identity value (`title`, `document-id`,
  `version`, `release`, `commit`, `date`) before writing it to
  `in/metadata.yaml`, because pandoc parses a string metadata value as Markdown
  and a template prints it. The value is first Markdown-escaped as the next
  bullet states, then written as a double-quoted YAML string, so pandoc reads the
  Markdown-literal text and a Typst template prints it as literal text in any
  header, footer or title.
- The command SHALL escape every identity value that it writes into generated
  Markdown ([FR-035](./FR-035-default-export-template.md)) so that the converter
  shows it as literal text: each ASCII punctuation character is backslash-escaped
  and each line break is replaced by a space.
- The converter input SHALL be the body of the committed blob without its
  frontmatter, and the command SHALL invoke pandoc with the Markdown reader
  extension `yaml_metadata_block` disabled, so no YAML block in the source can
  define, override or add a metadata member. The frontmatter is never converter
  input.
- The identity applies to Markdown input only. A JSON input
  ([FR-031](./FR-031-export-subcommand.md) section B) carries no identity, and the
  options of this FR and of FR-035 through FR-039 SHALL be refused with a JSON
  input.

## Acceptance Criteria

| ID | Criteria | Verification |
| --- | --- | --- |
| FR-034-AC-1 | Exporting a fixture committed with an `id`, a `title` and a `version` to `docx` yields extracted text holding all six fields as stated: the title, the id, the version, the full source commit hash and the committer date as `YYYY-MM-DD`, and no Release line. | Test |
| FR-034-AC-2 | With `--release v1.2` naming a tag on the source commit, the Release field reads `v1.2`; a fixture with no `version` member and no `--release` is refused ([FR-040](./FR-040-export-template-and-identity-errors.md)). | Test |
| FR-034-AC-3 | A fixture committed with a committer date of `2001-02-03T23:30:00-05:00` yields the Date `2001-02-04` (UTC), and the date is the same when the test process runs under a different `TZ` and a different system clock. | Test |
| FR-034-AC-4 | With `--source-rev` naming an older commit at which the input equals the working file, the identity holds that commit's hash and date; naming a commit at which the input differs from the working file is refused. | Test |
| FR-034-AC-5 | An input with uncommitted changes, an untracked input, and an input outside any repository each exit 3 naming the file and writing no output; a fixture whose working file is the committed blob with CRLF line endings throughout exports without error and yields the text of the committed blob. | Test |
| FR-034-AC-6 | During an export, `quire` starts no process other than `engineering-assurance`, and `cargo deny check bans` passes with the bans of NFR-004-AC-1, and `cargo tree -i` finds none of `gix`, `gix-protocol`, `gix-transport` and `gix-command`. | Test |
| FR-034-AC-7 | A fixture whose title is `--output=/tmp/x` exports with that text as the title and no change to the arguments of any request; the identity values appear in `in/metadata.yaml` and in generated Markdown only. | Test |
| FR-034-AC-8 | A JSON input with `--source-repo`, with `--release` or with `--source-rev` exits 3 naming the flag. | Test |
| FR-034-AC-9 | A fixture whose frontmatter defines `date: 1999-01-01`, `quire-export: {date: 1999-01-01}`, and whose body holds a YAML block defining the same, exports with the commit date on the title page and the YAML block text shown as body text, not as metadata. | Test |
| FR-034-AC-10 | A fixture whose `title` is `*x* ` followed by a backtick span `#pagebreak()` with the raw attribute `{=typst}`, whose id holds a pipe and a bracket, and whose `--release` tag holds backtick, hash, pipe, angle-bracket, brace and dollar characters (git rejects `*` in a ref name, so a tag `*r*` cannot exist; the values git forbids are covered by the escaping unit test), exports to PDF and to DOCX with the values present as written in the extracted text of the title page, of the header and of the footer, with no emphasis applied, no raw block interpreted and no page break added. | Test |
| FR-034-AC-11 | An input path with a `filter=lfs` attribute is refused naming the filter; the working file differing from the blob only in CRLF is not refused. | Test |
| FR-034-AC-12 | The `in/metadata.yaml` of that export holds each value as a double-quoted YAML string whose Markdown-significant characters are backslash-escaped, and a stand-in pandoc reads the same text for `quire-export.title` and `quire-export.release`. | Test |

## Dependencies

- **Upstream**: [FR-031](./FR-031-export-subcommand.md), [FR-032](./FR-032-export-converter-execution.md), [NFR-004](../non-functional/NFR-004-no-network.md).
- **Downstream**: [FR-035](./FR-035-default-export-template.md), [FR-036](./FR-036-organisation-export-template.md), [FR-040](./FR-040-export-template-and-identity-errors.md).
