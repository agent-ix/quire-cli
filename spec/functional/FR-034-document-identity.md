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

`quire export` of a Markdown input SHALL assemble the six document identity
fields below and SHALL hand them to the converter as data and as generated
title-page text ([FR-035](./FR-035-default-export-template.md),
[FR-036](./FR-036-organisation-export-template.md)). It SHALL also carry the
current exporter's Quire provenance and preserve an optional declared producer
identity from the committed input frontmatter. The identity ties the exported
file to one committed revision of its source. No field is taken from the wall
clock, the host or the user's environment.

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
| Release | `--release <TAG>`: a tag of the source repository whose commit is the source commit or a release ancestor of it (section A); optional. Shown as `<TAG>` when the tag commit is the source commit, otherwise as `<TAG> (tag commit <full hash of the tag commit>)`; the shown text is the only place the tag commit appears, and the `release` member of `in/metadata.yaml` is the tag name alone. |
| Source commit hash | The full hash of the commit selected by `--source-rev` (default `HEAD`) in the source repository. `--source-rev` is a full 40-digit object hash or a reference name (branch, tag or `HEAD`); abbreviated hashes and revision expressions such as `HEAD~1` are refused, and the diagnostic says so. |
| Date | The committer date of that commit, in UTC, as `YYYY-MM-DD`. |
| Exporter provenance | The current binary's exact `quire provenance` object, including distinct CLI and engine versions. |

At least one of Version and Release SHALL be present; both are shown when both
are present. The `title`, `id` and `version` members are strings or numbers; any other type counts as absent. The frontmatter members are read with `extract_frontmatter` of
[quire-rs FR-006](ix://agent-ix/quire-rs/FR-006).

### Producer identity

The input may carry a direct producer identity when its declared input schema
allows it:

| Member | Meaning |
| --- | --- |
| `producer_identity` | The direct producer's `name` and `version`. |

`producer_identity` is optional. When present, it has exactly the non-empty
string members `name` and `version`; malformed values are refused with the
member name. The command SHALL read it from the committed input blob and
preserve it as data; it SHALL NOT infer a missing identity from `generated_by`,
the export's source commit, `as_of`, or the wall clock. The source commit hash
and date identify the committed input being exported. The exporter SHALL obtain
its own exact Quire CLI and engine provenance from the running binary, without
starting a version-probe process.

## Behavior

### A. Source repository and content

- The source repository SHALL be `--source-repo`, or the git repository that
  contains the input file when the flag is absent.
- The command SHALL read the commit, its date, the tag, the blob and the
  attributes of the input path in process from the repository's object store,
  with read-only access: no index or worktree write, no ref update, no fetch.
- The command SHALL resolve `--release <TAG>` to an object, peeling annotated tags (nested ones included), and SHALL refuse the export naming the cause "tag does not name a commit" when the peeled object is not a commit.
- A commit T is a *release ancestor* of the source commit S when T is S, or T is reached from S by following first parents only and every commit C on that chain after T, up to and including S, satisfies all of: C has exactly one parent; the plain tree difference between C and its parent holds only additions of regular files and modifications of regular files whose mode is unchanged; and each added or modified path lies under the directory of an input. The tree difference is a plain comparison of the two trees with no rename detection, so a rename is a deletion plus an addition and is refused; a deletion, a mode or type change, and any added or changed symbolic link or submodule entry are refused.
- The directory of an input is the repository-relative directory of the input's repository path, the path used to read the committed blob and not the path as typed on the command line, so a symbolic link cannot widen it (an input path with a `..` segment is refused by FR-005). A changed path lies under it when the path's components begin with the directory's components, compared as exact, case-sensitive bytes: `dossier-old/x` is not under `dossier`. An input in the repository root has no directory: if any input of the export (a bundle included) is in the repository root, only a tag on S is accepted.
- The command SHALL accept `--release <TAG>` when its commit is a release ancestor of S, and otherwise SHALL refuse naming the tag, S and the cause: tag commit not on the first-parent chain of S (another branch, history rewritten after the tag, or a commit later than S); a merge commit, with its hash; a changed path that is not an allowed addition or modification, with the commit hash and the path; or history incomplete, naming the missing commit, when a parent object needed for the walk is absent (a shallow clone), in which case the command never accepts the tag. This lets a dossier be committed after the release tag it names. The commit graph and trees are read in process under the read-only rule above, with the gitoxide crates of section B.
- The command SHALL NOT change the Source commit hash or the Date for `--release`: they remain those of the source commit, and the Release field names the tag commit when it differs from the source commit.
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
  `gix-attributes`, `gix-glob`; `gix-traverse` and `gix-diff` may be added for the commit walk and the tree difference of section A, `gix-diff` with `default-features = false`, because its default `blob` feature depends on `gix-command`). The SHA-256 of the verdict file of [FR-038](./FR-038-approval-block.md) is computed in process by a pure-Rust digest crate, which starts no process and links no transport. The `gix` facade is not used because it depends
  on `gix-protocol` and `gix-transport`. The crates `gix`, `gix-protocol`,
  `gix-transport` and `gix-command` are absent from the dependency graph, as
  [NFR-004](../non-functional/NFR-004-no-network.md)-AC-1 checks.

### C. Hand-off to the converter

- The command SHALL write the identity into `in/metadata.yaml` under the one
  reserved key `quire-export`. Its six document identity fields are `title`,
  `document-id`, `version`, `release`, `commit` and `date`; it also contains the
  running exporter's exact `exporter-provenance` object and, when present in the
  committed input, its `producer_identity` object. String identity values are
  JSON string values; provenance objects retain their nested types. The command
  SHALL pass this map with `--metadata-file` and SHALL NOT pass an identity or
  provenance value as an argument.
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
| FR-034-AC-1 | Exporting a fixture committed with an `id`, a `title` and a `version` to `docx` yields extracted text holding all six document identity fields as stated: the title, the id, the version, the full source commit hash and the committer date as `YYYY-MM-DD`, and no Release line. The generated title page also identifies the running Quire CLI and engine versions. | Test |
| FR-034-AC-2 | With `--release v1.2` naming a tag on the source commit, the Release field reads `v1.2`; a fixture with no `version` member and no `--release` is refused ([FR-040](./FR-040-export-template-and-identity-errors.md)). | Test |
| FR-034-AC-3 | A fixture committed with a committer date of `2001-02-03T23:30:00-05:00` yields the Date `2001-02-04` (UTC), and the date is the same when the test process runs under a different `TZ`. The command reads no clock, so no test varies one. | Test |
| FR-034-AC-4 | With `--source-rev` naming an older commit at which the input equals the working file, the identity holds that commit's hash and date; naming a commit at which the input differs from the working file is refused. | Test |
| FR-034-AC-5 | An input with uncommitted changes, an untracked input, and an input outside any repository each exit 3 naming the file and writing no output; a fixture whose working file is the committed blob with CRLF line endings throughout exports without error and yields the text of the committed blob. | Test |
| FR-034-AC-6 | During an export, `quire` starts no process other than `engineering-assurance`, and `cargo deny check bans` passes with the bans of NFR-004-AC-1, and `cargo tree -i` finds none of `gix`, `gix-protocol`, `gix-transport` and `gix-command`. | Test |
| FR-034-AC-7 | A fixture whose title is `--output=/tmp/x` exports with that text as the title and no change to the arguments of any request; the identity values appear in `in/metadata.yaml` and in generated Markdown only. | Test |
| FR-034-AC-8 | A JSON input with `--source-repo`, with `--release` or with `--source-rev` exits 3 naming the flag. | Test |
| FR-034-AC-9 | A fixture whose frontmatter defines `date: 1999-01-01`, `quire-export: {date: 1999-01-01}`, and whose body holds a YAML block defining the same, exports with the commit date on the title page and the YAML block text shown as body text, not as metadata. | Test |
| FR-034-AC-10 | A fixture whose `title` is `*x* ` followed by a backtick span `#pagebreak()` with the raw attribute `{=typst}`, whose id holds a pipe and a bracket, and whose `--release` tag holds backtick, hash, pipe, angle-bracket, brace and dollar characters (git rejects `*` in a ref name, so a tag `*r*` cannot exist; the values git forbids are covered by the escaping unit test), exports to PDF and to DOCX with the values present as written in the extracted text of the title page, of the header and of the footer, with no emphasis applied, no raw block interpreted and no page break added. | Test |
| FR-034-AC-11 | An input path with a `filter=lfs` attribute is refused naming the filter; the working file differing from the blob only in CRLF is not refused. | Test |
| FR-034-AC-12 | The `in/metadata.yaml` of that export holds each value as a double-quoted YAML string whose Markdown-significant characters are backslash-escaped, and a stand-in pandoc reads the same text for `quire-export.title` and `quire-export.release`. | Test |
| FR-034-AC-13 | A fixture whose tag `v1.2` names commit T, with the source commit S a child of T that adds and modifies regular files only under the input's directory, exports with the Release field `v1.2 (tag commit <full hash of T>)`, the Source commit hash equal to S and the Date equal to the committer date of S; the same holds when S is two such commits after T, and when a non-dossier input is edited after the tag, so the Release field names the tag commit T. A tag on S shows `v1.2` alone. A nested annotated tag that peels to T is accepted. In each export the `release` member of `in/metadata.yaml` is `v1.2` alone. | Test |
| FR-034-AC-14 | With `--release` naming T, each of these exits 3 naming the tag, S and the cause with the offending commit hash and path: a commit after T that modifies a file outside every input's directory; one that modifies a file under the sibling directory `dossier-old` for an input under `dossier`; one that modifies a path differing from the input directory only in letter case; a deletion under the input's directory; a rename; a mode change; an added symbolic link; an added submodule entry; and a merge commit. An input given through a symbolic link takes its directory from its repository path. | Test |
| FR-034-AC-15 | With an input in the repository root, and with a bundle of one root input and one input under `dossier`, a tag on a commit before S whose later commits touch only files under `dossier` is refused naming the cause; a tag on S is accepted. | Test |
| FR-034-AC-16 | With `--release` naming a tag on another branch, a tag on a commit that history rewriting removed from the first-parent chain of S, or a tag on a commit later than S, the export exits 3 naming the tag, S and the cause "not on the first-parent chain"; a tag of a tree or of a blob exits 3 naming the cause "tag does not name a commit". | Test |
| FR-034-AC-17 | In a shallow clone whose history lacks the commit between S and T, the export exits 3 naming the tag, S, the cause "history incomplete" and the missing commit hash, and does not accept the tag. | Test |
| FR-034-AC-18 | A committed Markdown fixture with valid `producer_identity` frontmatter preserves its `name` and `version` in `quire-export` metadata and on the generated title page, separately from the export source commit/date and the running exporter's Quire CLI/engine versions. Missing `producer_identity` emits no placeholder; a malformed object is refused naming `producer_identity`. | Test |
| FR-034-AC-19 | Two exports of the same committed source with the same template, Quire build, pandoc, typst, fonts, and inputs yield byte-identical output files, including when the source frontmatter contains `producer_identity`. Changing only the wall clock, time zone, host, working directory, capability-root path, or output path does not change output bytes. | Test |
| FR-034-AC-20 | The `quire-export.exporter-provenance` object identifies the running Quire CLI and engine versions exactly, without probing or starting another process. | Test |

## Dependencies

- **Upstream**: [FR-031](./FR-031-export-subcommand.md), [FR-032](./FR-032-export-converter-execution.md), [NFR-004](../non-functional/NFR-004-no-network.md).
- **Downstream**: [FR-035](./FR-035-default-export-template.md), [FR-036](./FR-036-organisation-export-template.md), [FR-037](./FR-037-export-clock-free-output.md), [FR-040](./FR-040-export-template-and-identity-errors.md), [NFR-011](../non-functional/NFR-011-export-reproducibility.md).
