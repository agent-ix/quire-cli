---
id: SR-20261116045018
title: "PLAT-1807 / PLAT-1811 quire-cli PR232 code-review"
type: SpecReview
analysis: code-review
scope: "agent-ix/quire-cli PR232 head 89a0b81; Cargo.toml, Cargo.lock, CHANGELOG.md, src/commands/export.rs, src/commands/export/approval.rs, src/commands/export/bundle.rs, src/commands/export/identity.rs, src/commands/export/template.rs, tests/cli_export.rs"
review_set: subset
---

## Summary

Tickets: PLAT-1807 (FR-038 approval block) and PLAT-1811 (FR-034 / FR-040 release-ancestor rule). Code review with the rust-review lane folded in, scoped to `git diff origin/main...HEAD`, judged against the specs merged in quire-cli #231 and against origin/main.

Gates run on this head: `cargo fmt --check` clean; `cargo clippy --locked --all-features -- -D warnings` clean; `cargo deny check bans` ok; `cargo tree -e normal -i` finds none of `gix`, `gix-protocol`, `gix-transport`, `gix-command`; the lockfile adds only `gix-diff` 0.69.0, `sha2` 0.10.9 and the RustCrypto support crates (`digest`, `block-buffer`, `crypto-common`, `generic-array`, `typenum`, `cpufeatures`), with one `gix-object` (0.66.1) and one `gix-hash` (0.28.1). Focused tests: 27 export tests on approval, verdict, release, tag and dossier filters pass, and the two ignored real-converter tests (FR-038-AC-9 geometry, release text) pass with real pandoc and typst.

Mutation check, one source mutation at a time, each reverted: 16 of 18 killed. Killed: dropping the `unverified claim: ` prefix; dropping the tag object; disabling the directory rule; allowing deletion (a rename is a deletion plus an addition here, since `gix_diff::tree::Recorder` does no rename tracking, so this also covers "rename via similarity"); a string-prefix component match (`dossier-old`); a case-insensitive match; accepting a merge commit; accepting a shallow history; accepting a tag off the first-parent chain or after the source commit; dropping the root-input rule; showing the tag object in the `not_released` + nested `approved` case; dropping `(tag commit …)` from the Release line; writing the shown text into `metadata.yaml`; dropping `claimed:` for a non-approved standalone verdict; dropping the fixed statement; allowing a mode change. Survived: computing the digest by spawning `sha256sum` (FND-001) and dropping the offending commit hash from a refusal (FND-002).

Probes through the CLI (temporary tests, reverted): an empty commit after the tag is accepted (vacuously allowed by FR-034 section A); an added executable file and a content edit of an executable file with mode unchanged are accepted (gix `EntryMode::is_blob` covers 100644 and 100755, so "regular file" means both); an octopus merge is refused naming the merge commit; a tag of a tag of a tree is refused with "tag does not name a commit".

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | medium | FR-038-AC-16 says the export "starts no other process for the approval block or the Verdict file digest". No test can fail if the digest is computed by a process: replacing `Sha256::digest` with a spawned `sha256sum` survives `release_result_calls_no_verification_tools` (it stubs only `gpg`, `ssh-keygen` and `engineering-assurance`) and `quire_starts_no_process_but_the_executor` (strace, but run without `--approval`). Add `--approval` to the strace test, or assert in the AC-16 test that the execve trace holds only the executor. | src/commands/export/approval.rs:101; tests/cli_export.rs:5079; tests/cli_export.rs:1360 |
| FND-002 | low | FR-034-AC-14 requires the refusal to name "the offending commit hash and path". In every scenario of `a_later_commit_that_is_not_an_allowed_change_refuses_the_tag` and in the merge test the offending commit is S itself, so the S needle also satisfies the commit-hash check; removing `commit {commit}` from the refusal survives. Put one allowed commit after the offending one and assert the offending hash. | src/commands/export/identity.rs:459; tests/cli_export.rs:5656; tests/cli_export.rs:5765 |
| FND-003 | low | FR-040-AC-3 requires an unresolvable `--release` to exit 3 "naming the tag, the source commit and the cause". `unresolvable_sources_are_refused_naming_the_cause` asserts only the tag name; the code does name S and "the release tag does not resolve", but no assertion checks it. | tests/cli_export.rs:1062; src/commands/export/identity.rs:385 |
| FND-004 | low | `identity::resolve` now builds `Release { tag, tag_commit: None }` from `SourceOptions.release` without checking the tag; the check moved to `attach_release`, which each caller must remember to run. Both callers do, but an unchecked `Release` is representable and a third caller would show an unverified release. Let `attach_release` (or a constructor that runs `check_release`) be the only producer of `Release`, and drop `release` from `SourceOptions`. | src/commands/export/identity.rs:107; src/commands/export/identity.rs:154 |
| FND-005 | low | `a_bundle_accepts_a_later_commit_under_any_input_directory` asserts only that the Release line contains `tag commit`, not the hash of the v1.1 commit, and binds an unused `tag_commit` (`let _ = tag_commit;`, also in `a_shallow_history_never_accepts_the_tag`). Assert the exact `Release: v1\.1 \(tag commit <hash>\)` line and drop the dead bindings. | tests/cli_export.rs:5608; tests/cli_export.rs:5912 |

## Examined scope

```yaml
scope:
- {id: FR-038-AC-1, path: spec/functional/FR-038-approval-block.md, role: examined, excerpt: "... yields extracted text holding the person, `2026-01-02 03:04:05 UTC`, the meaning and the verdict `unverified claim: approved` ..."}
- {id: FR-038-AC-2, path: spec/functional/FR-038-approval-block.md, role: examined, excerpt: "... every Verdict cell begins with `unverified claim: ` and none equals a bare status token."}
- {id: FR-038-AC-9, path: spec/functional/FR-038-approval-block.md, role: examined, excerpt: "... and a `tag_object_id` of 40 hexadecimal digits, exports to PDF with every cell's content inside its own column, the Verdict file column included ..."}
- {id: FR-038-AC-10, path: spec/functional/FR-038-approval-block.md, role: examined, excerpt: "... its Verdict cell is exactly `unverified claim: approved`, and its Person, Signed at, Meaning and Record cells are byte-identical ..."}
- {id: FR-038-AC-11, path: spec/functional/FR-038-approval-block.md, role: examined, excerpt: "... Verdict cell exactly `unverified claim: not_released; reasons: [...]; nested: untrusted_signer` ..."}
- {id: FR-038-AC-12, path: spec/functional/FR-038-approval-block.md, role: examined, excerpt: "... Verdict cell exactly `unverified claim: not_released; reasons: [\"record_invalid\"]` ..."}
- {id: FR-038-AC-13, path: spec/functional/FR-038-approval-block.md, role: examined, excerpt: "... holds the row of A, Verdict file cell included, byte-identical in all three ..."}
- {id: FR-038-AC-16, path: spec/functional/FR-038-approval-block.md, role: examined, excerpt: "... starts no other process for the approval block or the Verdict file digest, and names no `--approval` file or tag in any call."}
- {id: FR-038-AC-17, path: spec/functional/FR-038-approval-block.md, role: examined, excerpt: "... Verdict cell exactly `unverified claim: not_released; reasons: [\"decision_owner_mismatch\"]`, Person, Signed at and Record `not available` ..."}
- {id: FR-038-AC-19, path: spec/functional/FR-038-approval-block.md, role: examined, excerpt: "A fabricated verdict file whose `status` is `approved` ... exports with the Verdict cell exactly `unverified claim: approved` ..."}
- {id: FR-038-AC-20, path: spec/functional/FR-038-approval-block.md, role: examined, excerpt: "The Verdict file cell of an entry equals `sha256:` followed by the SHA-256 of the verdict file's bytes ... For a `not_released` result whose nested approval is `approved` the cell holds the digest alone."}
- {id: FR-038-AC-21, path: spec/functional/FR-038-approval-block.md, role: examined, excerpt: "The text under the `Approval` heading, before the table, is exactly the statement of this requirement and contains no digest ..."}
- {id: FR-034-AC-6, path: spec/functional/FR-034-document-identity.md, role: examined, excerpt: "... `cargo deny check bans` passes with the bans of NFR-004-AC-1, and `cargo tree -i` finds none of `gix`, `gix-protocol`, `gix-transport` and `gix-command`."}
- {id: FR-034-AC-13, path: spec/functional/FR-034-document-identity.md, role: examined, excerpt: "A fixture whose tag `v1.2` names commit T ... exports with the Release field `v1.2 (tag commit <full hash of T>)` ... In each export the `release` member of `in/metadata.yaml` is `v1.2` alone."}
- {id: FR-034-AC-14, path: spec/functional/FR-034-document-identity.md, role: examined, excerpt: "With `--release` naming T, each of these exits 3 naming the tag, S and the cause with the offending commit hash and path ..."}
- {id: FR-034-AC-15, path: spec/functional/FR-034-document-identity.md, role: examined, excerpt: "With an input in the repository root, and with a bundle of one root input and one input under `dossier`, a tag on a commit before S ... is refused naming the cause; a tag on S is accepted."}
- {id: FR-034-AC-16, path: spec/functional/FR-034-document-identity.md, role: examined, excerpt: "... the export exits 3 naming the tag, S and the cause \"not on the first-parent chain\"; a tag of a tree or of a blob exits 3 naming the cause \"tag does not name a commit\"."}
- {id: FR-034-AC-17, path: spec/functional/FR-034-document-identity.md, role: examined, excerpt: "In a shallow clone whose history lacks the commit between S and T, the export exits 3 naming the tag, S, the cause \"history incomplete\" and the missing commit hash ..."}
- {id: FR-040-AC-3, path: spec/functional/FR-040-export-template-and-identity-errors.md, role: examined, excerpt: "... an unresolvable `--release`, a `--release` tag on a commit off the first-parent chain ... each exit 3 naming the tag, the source commit and the cause ..."}
- {id: FR-035, path: spec/functional/FR-035-default-export-template.md, role: context_only, excerpt: "The default template SHALL fix the page size (A4), margins, fonts and heading styles inside the generated template, so they do not depend on the host."}
- {id: FR-005, path: spec/functional/FR-005-path-safety.md, role: context_only, excerpt: "Reject `..` literal segments in any user-supplied path argument (`--module`, positional `<DOC>`, and `edit`'s `--out`/`--in-place` target)."}
```

## Verdict

Mergeable after the fix round for FND-001 (medium); FND-002 to FND-005 are low and should ride in the same round. The behaviour is correct: every brief mutation of the release-ancestor rule and of the Verdict / Verdict file cells is killed by a CLI test, refusals exit 3 and name the tag, the source commit and the cause, and the release error path runs before any converter.

Correctness checks that passed. The first-parent walk stops at a root commit with "not on the first-parent chain", refuses any chain commit whose parent list is not exactly one (merge and octopus), and reports a missing commit object as "history incomplete" naming it; a missing subtree surfaces as a refusal from `gix_diff::tree`. Peeling uses `peel_to_id`, which follows nested annotated tags; a tag that peels to a tree or blob is refused. Lightweight and annotated tags behave the same. The tree difference skips tree-entry additions and tree-to-tree modifications only, so every file below an added directory is checked on its own; deletions, symlinks, gitlinks and mode changes are refused, and the path test is component-wise, exact-byte and case-sensitive, as FR-034 section A states. The directory comes from the canonicalised repository path, so a symlinked input path cannot widen it. The SHA-256 is computed once over the bytes `read_bounded` returned (capped at 16 MiB) and the same buffer becomes the parsed text, so the digest matches the bytes shown. `gix-diff` with `default-features = false` keeps `gix-command` out, and no network crate is linked.

The default PDF template's `#show table: set text(size: 9pt)` is acceptable without a new requirement: FR-035 section C makes fonts a fixed property of the generated template, no AC states a table text size, and FR-038-AC-9's geometry test (which passes) is the requirement it serves. It applies to every table of the document body, not only the approval block; the CHANGELOG says so.
