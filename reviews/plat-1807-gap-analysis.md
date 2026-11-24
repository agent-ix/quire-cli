---
id: SR-20261116045019
title: "PLAT-1807 / PLAT-1811 quire-cli PR232 gap-analysis"
type: SpecReview
analysis: gap-analysis
scope: "agent-ix/quire-cli PR232 head 89a0b81; src/commands/export/{approval,identity,bundle,template}.rs, src/commands/export.rs, tests/cli_export.rs, spec/functional/FR-034, FR-038, FR-040, FR-005"
review_set: subset
---

## Summary

Tickets: PLAT-1807 (FR-038) and PLAT-1811 (FR-034 / FR-040). Planless gap pass scoped to the diff. Plan completion: not assessed. `quire coverage --scope . --json` (quire 0.52.0, engine 0.62.2) over this head lists none of FR-034-AC-13 to AC-17, FR-038-AC-1, 2, 9 to 13, 16, 17, 19 to 21 or FR-040-AC-3 among `unbacked_rows`, `status_lies` or `no_symbol_rows`: every in-scope criterion has a tagged test. Test strength was checked by mutation in the code-review artifact (SR-20261116045018), whose FND-001 and FND-002 are the two surviving mutations. No stubs, `todo!()` or placeholder returns were found in the changed code. Reverse gap: the default PDF template's `#show table: set text(size: 9pt)` has no owning AC; it falls under FR-035 section C (fonts are fixed in the generated template) and serves FR-038-AC-9, so it is not recorded as a gap.

The one spec gap is the coder's: the `..` example of FR-034-AC-14 cannot be met.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | medium | FR-034-AC-14 ends "An input given as `dossier/../dossier/x.md` or through a symbolic link takes its directory from its repository path", and section A says "`..` and symbolic links cannot widen it". FR-005 refuses every CLI path argument with a literal `..` segment. Measured on this head: `quire export <repo>/dossier/../dossier/doc.md --release v1.2` exits 2 with `PathTraversal: argument 'document' rejected ... (contains .. segment)` from validation, after the release rule has already accepted the tag, both for a tag on S and for a tag before S. (The coder's note says exit 3; it is 2.) The `..` half of the AC can never pass, and changing the guard would weaken FR-005's sandbox contract for one example. Minimal spec fix: in FR-034-AC-14 replace "An input given as `dossier/../dossier/x.md` or through a symbolic link takes its directory from its repository path." with "An input given through a symbolic link takes its directory from its repository path."; in FR-034 section A replace "so `..` and symbolic links cannot widen it" with "so a symbolic link cannot widen it (an input path with a `..` segment is refused by FR-005)". The symlink half is already tested by `the_input_directory_comes_from_its_repository_path`. | spec/functional/FR-034-document-identity.md:62; spec/functional/FR-034-document-identity.md:133; src/safety.rs:51; spec/functional/FR-005-path-safety.md:32 |

## Examined scope

```yaml
scope:
- {id: FR-034-AC-13, path: spec/functional/FR-034-document-identity.md, role: examined, excerpt: "A fixture whose tag `v1.2` names commit T, with the source commit S a child of T that adds and modifies regular files only under the input's directory, exports with the Release field `v1.2 (tag commit <full hash of T>)` ..."}
- {id: FR-034-AC-14, path: spec/functional/FR-034-document-identity.md, role: examined, excerpt: "... An input given as `dossier/../dossier/x.md` or through a symbolic link takes its directory from its repository path."}
- {id: FR-034-AC-15, path: spec/functional/FR-034-document-identity.md, role: examined, excerpt: "With an input in the repository root, and with a bundle of one root input and one input under `dossier`, a tag on a commit before S whose later commits touch only files under `dossier` is refused naming the cause; a tag on S is accepted."}
- {id: FR-034-AC-16, path: spec/functional/FR-034-document-identity.md, role: examined, excerpt: "With `--release` naming a tag on another branch, a tag on a commit that history rewriting removed from the first-parent chain of S, or a tag on a commit later than S, the export exits 3 ..."}
- {id: FR-034-AC-17, path: spec/functional/FR-034-document-identity.md, role: examined, excerpt: "In a shallow clone whose history lacks the commit between S and T, the export exits 3 naming the tag, S, the cause \"history incomplete\" and the missing commit hash, and does not accept the tag."}
- {id: FR-034, path: spec/functional/FR-034-document-identity.md, role: examined, excerpt: "The directory of an input is the repository-relative directory of the input's repository path, the path used to read the committed blob and not the path as typed on the command line, so `..` and symbolic links cannot widen it."}
- {id: FR-038-AC-1, path: spec/functional/FR-038-approval-block.md, role: examined, excerpt: "... the verdict `unverified claim: approved`, in a block under `Approval` after the document ..."}
- {id: FR-038-AC-2, path: spec/functional/FR-038-approval-block.md, role: examined, excerpt: "... every Verdict cell begins with `unverified claim: ` and none equals a bare status token."}
- {id: FR-038-AC-9, path: spec/functional/FR-038-approval-block.md, role: examined, excerpt: "... exports to PDF with every cell's content inside its own column, the Verdict file column included ..."}
- {id: FR-038-AC-10, path: spec/functional/FR-038-approval-block.md, role: examined, excerpt: "A static `released` release-verify result ... its Verdict cell is exactly `unverified claim: approved` ..."}
- {id: FR-038-AC-11, path: spec/functional/FR-038-approval-block.md, role: examined, excerpt: "A static `not_released` result with reasons `untrusted_signer` and `release_commit_record_not_added` ..."}
- {id: FR-038-AC-12, path: spec/functional/FR-038-approval-block.md, role: examined, excerpt: "A static `not_released` result with reasons `record_invalid` and no `approval` member ..."}
- {id: FR-038-AC-13, path: spec/functional/FR-038-approval-block.md, role: examined, excerpt: "... holds the row of A, Verdict file cell included, byte-identical in all three ..."}
- {id: FR-038-AC-16, path: spec/functional/FR-038-approval-block.md, role: examined, excerpt: "... starts no other process for the approval block or the Verdict file digest ..."}
- {id: FR-038-AC-17, path: spec/functional/FR-038-approval-block.md, role: examined, excerpt: "A static `not_released` result with reasons `decision_owner_mismatch` and a nested `approved` approval ..."}
- {id: FR-038-AC-19, path: spec/functional/FR-038-approval-block.md, role: examined, excerpt: "A fabricated verdict file whose `status` is `approved` ..."}
- {id: FR-038-AC-20, path: spec/functional/FR-038-approval-block.md, role: examined, excerpt: "The Verdict file cell of an entry equals `sha256:` followed by the SHA-256 of the verdict file's bytes ..."}
- {id: FR-038-AC-21, path: spec/functional/FR-038-approval-block.md, role: examined, excerpt: "The text under the `Approval` heading, before the table, is exactly the statement of this requirement ..."}
- {id: FR-040-AC-3, path: spec/functional/FR-040-export-template-and-identity-errors.md, role: examined, excerpt: "... each exit 3 naming the tag, the source commit and the cause; a `--release` tag on a one-parent ancestor that only adds or modifies files under the input directory is not a failure."}
- {id: FR-005, path: spec/functional/FR-005-path-safety.md, role: context_only, excerpt: "Reject `..` literal segments in any user-supplied path argument (`--module`, positional `<DOC>`, and `edit`'s `--out`/`--in-place` target)."}
- {id: FR-035, path: spec/functional/FR-035-default-export-template.md, role: context_only, excerpt: "The default template SHALL fix the page size (A4), margins, fonts and heading styles inside the generated template, so they do not depend on the host."}
```

## Verdict

One medium spec gap (FND-001), to be fixed in this PR by dropping the `..` example as stated; no code change is needed for it. Every other in-scope criterion is implemented and backed by a tagged CLI test. The quoin composed-chain test (quoin FR-191-AC-27/28) is a follow-up and was not assessed.

## Dispositions

Disposition pass 1, against fix commit 84d4148. FR-034 section A and FR-034-AC-14 carry the wording this review recommended, and `quire validate` on the amended FR-034 exits 0.

| FND | outcome | reason |
| --- | --- | --- |
| FND-001 | fixed | - |
