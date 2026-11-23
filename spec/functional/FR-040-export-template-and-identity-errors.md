---
id: FR-040
title: "Export errors: template, identity, approval and bundle input"
type: FR
relationships:
  - target: "ix://agent-ix/quire-cli/spec/functional/FR-033"
    type: "extends"
  - target: "ix://agent-ix/quire-cli/spec/functional/FR-034"
    type: "references"
  - target: "ix://agent-ix/quire-cli/spec/functional/FR-036"
    type: "references"
  - target: "ix://agent-ix/quire-cli/spec/functional/FR-038"
    type: "references"
  - target: "ix://agent-ix/quire-cli/spec/functional/FR-039"
    type: "references"
  - target: "ix://agent-ix/quire-cli/spec/functional/FR-007"
    type: "references"
  - target: "ix://agent-ix/engineering-assurance/FR-047"
    type: "consumes"
---
# FR-040: Export errors: template, identity, approval and bundle input

## Description

[FR-033](./FR-033-export-errors.md) classifies export failures. This FR adds the
failures of the options of [FR-034](./FR-034-document-identity.md) through
[FR-039](./FR-039-export-bundle.md). Each is checked before any converter starts,
leaves no file at `--output` and leaves a pre-existing file there unchanged, as
FR-033 requires.

## Behavior

| Kind | Cause | Exit | Diagnostic |
| --- | --- | --- | --- |
| Template invalid | `--template` path missing, not a regular file, above 16 MiB, or with an extension not matching `--to` (`.typ` for `pdf`, `.docx` for `docx`). | 3 | Names the path and the first cause. |
| Template unreadable | `--template` exists but a permission refusal prevents reading it. | 2 | Names the path. |
| Identity data missing | The input lacks `title`, `id`, or both `version` and `--release`; a bundle lacks `--bundle-title` or `--bundle-id`. | 3 | Names the file and every missing field. |
| Source commit unavailable | No repository contains the input, `--source-repo` is not a repository, `--source-rev` or `--release` does not resolve, `--release` does not resolve to a commit that is the source commit or a release ancestor of it ([FR-034](./FR-034-document-identity.md) section A: first parents only, each later commit one-parent, adding or modifying files only under an input's directory), or the repository's object store is unreadable. | 3 | Names the repository, the revision or tag, and the first cause: tag unresolved, tag commit not on the first-parent chain, merge commit, deleted or renamed path, or path outside the input directory. |
| Source not committed | The input is untracked at the source commit or differs from the committed blob (compared with CRLF read as LF). | 3 | Names the file and says to commit it or pass an older `--source-rev`. |
| Source filtered | The input path carries a git `filter` attribute, including Git LFS. | 3 | Names the file and the filter. |
| Approval invalid | An `--approval` file is missing, not JSON, neither an `engineering-assurance.approval-verify-result/v1` nor an `engineering-assurance.release-verify-result/v1` document, lacks `status`, has a member of the wrong JSON type, is above 16 MiB, is not a regular file or not UTF-8, or is an `approved` verdict (standalone or nested in a release result) whose `signed_at` is not a valid UTC instant `YYYY-MM-DDTHH:MM:SSZ`. | 3 | Names the file and the first cause. |
| Release result invalid | An `--approval` file of protocol `engineering-assurance.release-verify-result/v1` has a `status` that is neither `released` nor `not_released`; is `released` without an `approval` member; is `released` with an `approval.status` other than `approved`; is `released` with a non-empty `reasons`; is `not_released` with `reasons` absent or not a list of strings; or has an `approval` member that is not an object of protocol `engineering-assurance.approval-verify-result/v1` with a `status` ([EA FR-047](ix://agent-ix/engineering-assurance/FR-047)). | 3 | Names the file and the first cause. |
| Approval unreadable | An `--approval` file exists but a permission refusal prevents reading it. | 2 | Names the file. |
| Bundle invalid | Inputs from different repositories or at different source commits, more than 256 inputs, or a bundle option with one input. | 3 | Names the cause. |
| Option on JSON input | Any option of FR-034 through FR-039 with a JSON input. | 3 | Names the flag. |

- The command SHALL name the first failing cause.
- The command SHALL NOT substitute the
  default template, a default identity value, the current date or `HEAD` for a
  missing or invalid one.
- The command SHALL report an identity field missing at the time the export
  starts, so no partial output exists.

## Acceptance Criteria

| ID | Criteria | Verification |
| --- | --- | --- |
| FR-040-AC-1 | A fixture without `id`, a fixture without `title`, and a fixture with neither `version` nor `--release` each exit 3 naming the missing field, and one fixture missing two fields names both. | Test |
| FR-040-AC-2 | A missing template path, a directory, an extension that does not match `--to`, and a template above 16 MiB each exit 3 naming the path; an unreadable template exits 2. | Test |
| FR-040-AC-3 | A `--source-repo` that is not a repository, an unresolvable `--source-rev`, an unresolvable `--release`, a `--release` tag on a commit off the first-parent chain of the source commit, and a `--release` tag whose chain to the source commit holds a merge commit, a deletion or a path outside the input directory each exit 3 naming the cause; a `--release` tag on a one-parent ancestor that only adds or modifies files under the input directory is not a failure. | Test |
| FR-040-AC-4 | An untracked input, an input differing from its committed blob and an input with a `filter` attribute each exit 3 naming the file, the last also the filter. | Test |
| FR-040-AC-5 | Each approval failure of the table other than an unreadable file exits 3 naming the file, an unreadable approval file exits 2, and an `unverifiable` verdict for an unresolved tag is not a failure; the export names the first failing file when two fail. | Test |
| FR-040-AC-6 | In every failure of the table no converter or `engineering-assurance` process is started, `--output` is absent, and a pre-existing `--output` is unchanged. | Test |
| FR-040-AC-7 | No failure of the table falls back to the default template, the current date or `HEAD`: the diagnostic is the only result. | Test |
| FR-040-AC-8 | Each cause of the Release result invalid row exits 3 naming the file: a release result with `status` `unknown`; a `released` result without `approval`; a `released` result whose `approval.status` is `wrong_record`; a `released` result with `reasons` `["x"]`; a `not_released` result without `reasons`, with `reasons` `[1]` and with `reasons` `"x"`; a release result whose `approval` is a string, one whose `approval` has another `protocol`, and one whose `approval` lacks `status`. A `released` result with `reasons` `[]` and a `not_released` result with a nested `approved` approval are not failures. | Test |

## Dependencies

- **Upstream**: [FR-033](./FR-033-export-errors.md), [FR-034](./FR-034-document-identity.md), [FR-036](./FR-036-organisation-export-template.md), [FR-038](./FR-038-approval-block.md), [FR-039](./FR-039-export-bundle.md), [FR-007](./FR-007-exit-codes.md); [EA FR-047](ix://agent-ix/engineering-assurance/FR-047) owns the release result whose refusal causes the Release result invalid row lists.
