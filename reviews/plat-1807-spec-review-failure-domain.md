---
id: SR-20261116045017
title: "PLAT-1811 quire-cli PR231 failure-domain analysis of the release-ancestor rule"
type: SpecReview
analysis: failure-domain
scope: "agent-ix/quire-cli PR231 head 569873f; spec/functional/FR-034-document-identity.md section A, spec/functional/FR-040-export-template-and-identity-errors.md"
review_set: subset
---

## Summary

Ticket: PLAT-1811. This analysis covers the failure modes of the new `--release` descendant rule in FR-034 section A: the first-parent walk, change classification, path matching, multiple inputs and incomplete history.

The core rule is sound. With every later commit single-parent, the first-parent walk equals the full ancestry. Merges are refused. A rename is refused even without rename detection, because a raw tree diff shows it as a deletion. Peeling is stated, a tag on S is accepted, Source commit hash and Date stay those of S, and refusals reuse exit 3. The gaps are in edges that the ACs leave open, so two implementations can both pass every AC and still differ.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | medium | Path matching is not pinned by an AC. "In that directory or any directory below it" is component-wise, but a string prefix test (`dossier-old/x` under `dossier`) passes every AC. Add an AC with a sibling-prefix directory (`dossier` vs `dossier-old`) that is refused. State that the comparison is on the repository path bytes, case-sensitive, and that the input directory comes from the input's repository path (the one used for the blob lookup), not the argv path, so a symlinked or `..` argument cannot widen it. | FR-034 section A bullet 3; FR-034-AC-13 (new) |
| FND-002 | medium | A bundle that mixes a root input with a non-root input gets two conflicting answers. "Each added or modified path lies under the directory of an input" accepts a later commit under the non-root input's directory. "For [a root input] only a tag on the source commit is accepted" refuses it. Pick one, for example: any root input among the inputs restricts `--release` to a tag on S. Add it to FR-034-AC-13. | FR-034 section A bullet 3; FR-039 |
| FND-003 | low | Change classification is unspecified beyond add, modify, delete and rename. Rename detection should be stated as none: a raw tree diff, where a rename is a deletion plus an addition, refused and named by the deleted path, with no git similarity heuristic. Also say how a mode-only change, a file-to-symlink type change, an added symlink and an added gitlink (submodule) are classed. | FR-034 section A bullet 3; FR-040 Source commit unavailable row |
| FND-004 | medium | Incomplete history is unhandled. In a shallow clone, or a store missing a parent object, the walk from S stops at a missing commit before reaching T. The spec does not say whether that is "not on the first-parent chain", "object store unreadable", or its own cause. Add the cause (history incomplete, naming the missing commit, exit 3) and an AC with a shallow fixture. | FR-034 section A bullet 3; FR-034-AC-14 (new); FR-040-AC-3 |
| FND-005 | low | FR-034-AC-14 covers a tag on another branch and on rewritten history, but not a tag on a descendant of S (tag after the source commit), or a tag that peels to a non-commit (a tag of a tree or blob, or a nested annotated tag). Add these as AC cases. | FR-034-AC-14 (new); FR-040-AC-3 |

## Examined scope

```yaml
scope:
- {id: FR-034, path: spec/functional/FR-034-document-identity.md, role: examined, excerpt: "... every commit on that chain after the tag commit, up to and including the source commit, satisfies all of: it has exactly one parent; its change against that parent only adds or modifies files; and each added or modified path lies under the directory of an input ..."}
- {id: FR-034-AC-12, path: spec/functional/FR-034-document-identity.md, role: examined, excerpt: "A fixture whose tag `v1.2` names commit T, with the source commit S a child of T ..."}
- {id: FR-034-AC-13, path: spec/functional/FR-034-document-identity.md, role: examined, excerpt: "... modifies a file outside every input's directory, deletes a file (even under the input's directory), renames a file, is a merge commit, or, for an input in the repository root, exists at all."}
- {id: FR-034-AC-14, path: spec/functional/FR-034-document-identity.md, role: examined, excerpt: "... (a tag on another branch, and a tag on a commit that history rewriting removed from that chain) the export is refused ..."}
- {id: FR-040-AC-3, path: spec/functional/FR-040-export-template-and-identity-errors.md, role: examined, excerpt: "... a `--release` tag on a one-parent ancestor that only adds or modifies files under the input directory is not a failure."}
- {id: FR-007, path: spec/functional/FR-007-exit-codes.md, role: context_only, excerpt: "3 | Invalid request ... 4 | Internal failure ..."}
```

## Verdict

The rule is implementable with the readers already linked (FR-034 section B) and testable with git fixtures. Close FND-001, FND-002 and FND-004 with ACs before coding. FND-003 and FND-005 are wording or AC additions.

## New findings (disposition pass 1)

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-006 | low | FR-034 section B now allows adding `gix-diff`. gix-diff 0.67's default features (`blob`) pull `gix-command`, which the same section bans and FR-034-AC-6 checks with `cargo tree -i`. Say that `gix-diff` is added with `default-features = false` (tree diff only), so the permission cannot be read as allowing the banned crate. | FR-034 section B; FR-034-AC-6 |

## Dispositions

| FND | outcome | reason |
| --- | --- | --- |
| FND-001 | fixed | - |
| FND-002 | fixed | - |
| FND-003 | fixed | - |
| FND-004 | fixed | - |
| FND-005 | fixed | - |
| FND-006 | fixed | - |
