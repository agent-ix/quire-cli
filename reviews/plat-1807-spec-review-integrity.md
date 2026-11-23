---
id: SR-20261116045016
title: "PLAT-1807 / PLAT-1811 quire-cli PR231 integrity analysis"
type: SpecReview
analysis: integrity
scope: "agent-ix/quire-cli PR231 head 569873f; spec/functional/FR-034-document-identity.md, spec/functional/FR-038-approval-block.md, spec/functional/FR-040-export-template-and-identity-errors.md, spec/functional/FR-039-export-bundle.md"
review_set: subset
---

## Summary

Tickets: PLAT-1807, PLAT-1811. This analysis checks id integrity, defined terms and cross-requirement consistency for the PR diff. The PR reuses the existing id FR-034-AC-12, so tests already tagged to the old criterion now back the new release rule. `quire validate` exits 0 on the file. Two stale clauses also remain beside the rewritten text.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | high | Duplicate criterion id. The PR adds `FR-034-AC-12` (release ancestor accepted) while the existing `FR-034-AC-12` (`in/metadata.yaml` double-quoted escaped values) stays at the end of the table. `quire matrix` shows both rows bound to the same tagged tests (`tests/cli_export.rs:1208`, `src/commands/export/template.rs:547`), so the new release rule reads as tagged with no test behind it. `quire validate` exits 0 and does not catch the duplicate. Renumber the new criteria FR-034-AC-13, -14 and -15, and place them after the existing AC-12. | FR-034-AC-12 (both rows); FR-034-AC-13; FR-034-AC-14 |
| FND-002 | low | The term "release ancestor" is used in the FR-034 Release row ("a release ancestor of it (section A)") and the FR-040 Source commit unavailable row, but section A never defines it by that name. Define the term in section A's bullet, or reword both rows to reference the bullet. | FR-034 Release row; FR-040 Source commit unavailable row |
| FND-003 | medium | FR-039 still says "`--approval` verdicts apply to the bundle as a whole". The new FR-038 statement says an approval covers only the record in its Record column, not this document. This is a direct semantic conflict once this PR lands. Reword FR-039 to "are shown once for the bundle, after the last document". | FR-039 B. Assembly; FR-038 Behavior bullet 6 |
| FND-004 | medium | FR-038-AC-16 was rewritten but keeps clauses that no test can pass. The export always launches `engineering-assurance producer-execute` (FR-032; `src/commands/export/producer.rs`), so a recording `engineering-assurance` stub that must be "called none" fails. FR-034 section B forbids any `git` process, so "its `git` calls are the same" compares two empty sets. The new clause "starts no process to compute the Verdict file digest" names no executable to observe and is already implied by FR-034-AC-6. Restate it: no `engineering-assurance` subcommand other than `producer-execute`, no `gpg`, `ssh-keygen` or `git` process, and the producer request is the same as with a standalone verdict. | FR-038-AC-16; FR-034-AC-6; FR-032 |

## Examined scope

```yaml
scope:
- {id: FR-034-AC-12, path: spec/functional/FR-034-document-identity.md, role: examined, excerpt: "A fixture whose tag `v1.2` names commit T, with the source commit S a child of T that adds and modifies files only under the input's directory ..."}
- {id: FR-034-AC-12, path: spec/functional/FR-034-document-identity.md, role: examined, excerpt: "The `in/metadata.yaml` of that export holds each value as a double-quoted YAML string whose Markdown-significant characters are backslash-escaped ..."}
- {id: FR-034-AC-13, path: spec/functional/FR-034-document-identity.md, role: examined, excerpt: "With `--release` naming T, an export is refused naming the tag, the source commit and the offending path or commit ..."}
- {id: FR-034-AC-14, path: spec/functional/FR-034-document-identity.md, role: examined, excerpt: "With `--release` naming a commit that is not on the first-parent chain of the source commit ..."}
- {id: FR-040-AC-3, path: spec/functional/FR-040-export-template-and-identity-errors.md, role: examined, excerpt: "... a `--release` tag on a commit off the first-parent chain of the source commit ... each exit 3 naming the cause ..."}
- {id: FR-038-AC-16, path: spec/functional/FR-038-approval-block.md, role: examined, excerpt: "The export of a release result, run with stub `engineering-assurance`, `gpg` and `ssh-keygen` executables first on `PATH` that record each call, calls none of them ... the export starts no process to compute the Verdict file digest."}
- {id: FR-038-AC-19, path: spec/functional/FR-038-approval-block.md, role: examined, excerpt: "A fabricated verdict file whose `status` is `approved` ..."}
- {id: FR-038-AC-20, path: spec/functional/FR-038-approval-block.md, role: examined, excerpt: "The Verdict file cell of an entry holds `sha256:` ..."}
- {id: FR-038-AC-21, path: spec/functional/FR-038-approval-block.md, role: examined, excerpt: "The text under the `Approval` heading, before the table, is exactly the statement ..."}
- {id: FR-039, path: spec/functional/FR-039-export-bundle.md, role: context_only, excerpt: "`--approval` verdicts apply to the bundle as a whole and appear once, after the last document."}
```

## Verdict

FND-001 blocks merge: a reused criterion id silently transfers test backing. FR-038 ids AC-19 to AC-21 are new and unique, and the FR-040 and US-010 edits keep their ids.
