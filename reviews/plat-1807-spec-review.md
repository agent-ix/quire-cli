---
id: SR-20261116045015
title: "PLAT-1807 / PLAT-1811 quire-cli PR231 spec-review"
type: SpecReview
analysis: base
scope: "agent-ix/quire-cli PR231 head 569873f; spec/functional/FR-034-document-identity.md, spec/functional/FR-038-approval-block.md, spec/functional/FR-040-export-template-and-identity-errors.md, spec/usecase/US-010-approver-gets-identified-templated-document.md"
review_set: subset
---

## Summary

Tickets: PLAT-1807 (FR-038 approval block) and PLAT-1811 (FR-034 / FR-040 release-ancestor rule). Base checklist review of `git diff origin/main...HEAD`, judged against origin/main of quire-cli, quoin FR-191 / FR-201 and engineering-assurance FR-041 / FR-047. Sub-analyses run: integrity (SR-20261116045016) and failure-domain (SR-20261116045017). `quire validate` on the four changed files exits 0; it does not detect the duplicate FR-034-AC-12 recorded in the integrity review.

The FR-038 rewrite delivers the ticket intent (every Verdict cell prefixed, Verdict file column, fixed scope statement), and the old "label only approved as an approval" bullets are removed. Three clauses no longer fit together: the `claimed:` rule now reaches `released` entries, "reads as an approval" is untestable next to a heading literally named `Approval`, and the scope statement names a digest of "this document" that no one can compute. The FR-034 rule is sound in shape, but the Release field hides that the source commit is later than the tag.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | medium | FR-038 Behavior: "For an entry whose `status` is not `approved` the command SHALL prefix a recorded Meaning with `claimed:`". A release result's status is `released` or `not_released`, never `approved`, and the removed scoping sentence ("an entry of a release result is labelled under Release-verify input") no longer limits it, so a `released` entry's Meaning is prefixed `claimed:`. That contradicts FR-038-AC-10 (Meaning byte-identical to the standalone row). Scope the bullet to standalone verdicts and leave release results to the Release-verify section. | FR-038 Behavior bullet 4; FR-038-AC-10 |
| FND-002 | medium | "SHALL NOT present any entry as verified: no heading, label, badge or cell text reads as a verified or confirmed approval" and FR-038-AC-2 / AC-19 "no heading or label of the block reads as verified or as an approval" have no oracle. The block's fixed heading is `Approval`, the fixed statement contains `re-verified`, and fixture meanings contain `approved`. "The word `approved` appears in a Verdict cell only after the prefix" is vacuous because every Verdict cell starts with the prefix. Replace with checkable text: every Verdict cell begins with `unverified claim: `; the block's only heading is `Approval`; no cell other than Meaning contains `approved`, `verified` or `confirmed` outside the prefixed Verdict text. | FR-038 Behavior bullet 3; FR-038-AC-2; FR-038-AC-19 |
| FND-003 | medium | The fixed statement says an approval does not cover "this document, unless the digest of this document equals the record digest". "This document" is undefined: the PDF/DOCX bytes (which can never equal an EA record digest), the committed source blob, or a bundle. The statement also names no algorithm and no bytes, so a reader cannot perform the comparison it offers. quoin FR-201 already phrases it against "the exact bytes of this DOS-REL document". Name the bytes (the committed source blob of the input at the Source commit), or drop the "unless" clause. | FR-038 Behavior bullet 6; FR-038-AC-21 |
| FND-004 | low | The Verdict file cell holds "a second line `tag object: `", but a pandoc pipe-table cell cannot hold a raw line break, and the same requirement replaces line breaks in verdict values with spaces. The rendering mechanism (hard line break, `<br>`-equivalent, or a `; ` separator) is unspecified, and FR-038-AC-20 depends on what text extraction calls a "line". | FR-038 Entry content, Verdict file row; FR-038-AC-20 |
| FND-005 | medium | FR-034: when the tag commit T differs from S, the identity shows `Release: v1.2` with the hash and date of S. For an input outside a dossier, such as `spec/functional/FR-001.md`, an edit to that file after the tag is accepted (it is "under the input's directory"), so post-release content carries the release name with no visible sign. Show the tag commit when it differs from the source commit (for example `v1.2 (tag commit <T>)`), and add that to FR-034-AC-12. | FR-034 section A bullet 3; FR-034 Release row; FR-034-AC-12 (new) |
| FND-006 | low | The FR-040 Source commit unavailable diagnostic says it "Names the repository, the revision or tag, and the first cause". The new FR-034 ACs require the tag, the source commit and the offending commit or path. Align the FR-040 row to name tag, source commit, and the offending commit and path. | FR-040 Source commit unavailable row; FR-034-AC-13 (new); FR-034-AC-14 (new) |

## Examined scope

```yaml
scope:
- {id: FR-038, path: spec/functional/FR-038-approval-block.md, role: examined, excerpt: "Because the export does not verify, it presents every entry, whatever its status, as an unverified claim and never as a verified approval."}
- {id: FR-038-AC-1, path: spec/functional/FR-038-approval-block.md, role: examined, excerpt: "... the meaning and the verdict `unverified claim: approved`, in a block under `Approval` after the document ..."}
- {id: FR-038-AC-2, path: spec/functional/FR-038-approval-block.md, role: examined, excerpt: "... the first shows the verdict `unverified claim: approved`; no entry is labelled verified or as an approval."}
- {id: FR-038-AC-9, path: spec/functional/FR-038-approval-block.md, role: examined, excerpt: "... and a `tag_object_id` of 40 hexadecimal digits, exports to PDF with every cell's content inside its own column, the Verdict file column included ..."}
- {id: FR-038-AC-10, path: spec/functional/FR-038-approval-block.md, role: examined, excerpt: "... its Verdict cell is exactly `unverified claim: approved`, it carries no label reading as verified, and its Person, Signed at, Meaning and Record cells are byte-identical ..."}
- {id: FR-038-AC-11, path: spec/functional/FR-038-approval-block.md, role: examined, excerpt: "... Verdict cell exactly `unverified claim: not_released; reasons: [...]; nested: untrusted_signer` ..."}
- {id: FR-038-AC-12, path: spec/functional/FR-038-approval-block.md, role: examined, excerpt: "... Verdict cell exactly `unverified claim: not_released; reasons: [\"record_invalid\"]` ..."}
- {id: FR-038-AC-13, path: spec/functional/FR-038-approval-block.md, role: examined, excerpt: "... holds the row of A, Verdict file cell included, byte-identical in all three ..."}
- {id: FR-038-AC-16, path: spec/functional/FR-038-approval-block.md, role: examined, excerpt: "... the export starts no process to compute the Verdict file digest."}
- {id: FR-038-AC-17, path: spec/functional/FR-038-approval-block.md, role: examined, excerpt: "... Verdict cell exactly `unverified claim: not_released; reasons: [\"decision_owner_mismatch\"]` ..."}
- {id: FR-038-AC-19, path: spec/functional/FR-038-approval-block.md, role: examined, excerpt: "A fabricated verdict file whose `status` is `approved` ... exports with the Verdict cell exactly `unverified claim: approved`; no cell of the table equals `approved` ..."}
- {id: FR-038-AC-20, path: spec/functional/FR-038-approval-block.md, role: examined, excerpt: "The Verdict file cell of an entry holds `sha256:` followed by the SHA-256 of the verdict file's bytes ..."}
- {id: FR-038-AC-21, path: spec/functional/FR-038-approval-block.md, role: examined, excerpt: "The text under the `Approval` heading, before the table, is exactly the statement of this requirement ..."}
- {id: FR-034, path: spec/functional/FR-034-document-identity.md, role: examined, excerpt: "The command SHALL resolve `--release <TAG>` to a commit, peeling an annotated tag, and SHALL accept it when that commit is the source commit, or is reached from the source commit by following first parents only ..."}
- {id: FR-034-AC-12, path: spec/functional/FR-034-document-identity.md, role: examined, excerpt: "A fixture whose tag `v1.2` names commit T, with the source commit S a child of T ..."}
- {id: FR-034-AC-13, path: spec/functional/FR-034-document-identity.md, role: examined, excerpt: "With `--release` naming T, an export is refused naming the tag, the source commit and the offending path or commit ..."}
- {id: FR-034-AC-14, path: spec/functional/FR-034-document-identity.md, role: examined, excerpt: "With `--release` naming a commit that is not on the first-parent chain of the source commit ... is refused ..."}
- {id: FR-034-AC-6, path: spec/functional/FR-034-document-identity.md, role: context_only, excerpt: "During an export, `quire` starts no process other than `engineering-assurance` ..."}
- {id: FR-040-AC-3, path: spec/functional/FR-040-export-template-and-identity-errors.md, role: examined, excerpt: "... a `--release` tag on a commit off the first-parent chain ... each exit 3 naming the cause ..."}
- {id: US-010-EX-3, path: spec/usecase/US-010-approver-gets-identified-templated-document.md, role: examined, excerpt: "Then the document shows the approver, the time the signer claimed, the meaning and the verdict as an unverified claim"}
- {id: quoin FR-191, path: "quoin spec/functional/FR-191-dossier-command.md", role: context_only, excerpt: "`export` SHALL pass `--release`, each user-supplied `--approval` and `--template` to `quire export` unchanged."}
- {id: quoin FR-201, path: "quoin spec/functional/FR-201-dossier-release-record-content.md", role: context_only, excerpt: "Approval entries in this export are treated as unbound to the exact bytes of this DOS-REL document ..."}
```

## Verdict

Not mergeable as is: the integrity review's FND-001 (duplicate FR-034-AC-12) is high, and FND-001 to FND-003 and FND-005 here are medium spec defects, all fixable inside this PR.

Checks that passed. The old "label only an `approved` entry as an approval" rules are gone. No stale AC still expects a bare `approved` Verdict cell. US-010-EX-3 matches. The "no re-verify / no external process" invariants still hold, because SHA-256 over the file bytes is in-process data work (FR-038-AC-16 and FR-034-AC-6). The verdict-file SHA-256 is the only new hash and is the owner-approved content identity. No migration or compat language, no pins or ledgers, no paywalled text and no machine paths were found. Refusals reuse exit 3 (FR-007: invalid request); exit 4 is internal or tool failure, so 3 is correct.

Owner question 1 (single dossier root vs per-input directory): keep per-input directory. quoin FR-191 writes every dossier document under one `<DIR>` (`dossier`, or `--out`), so for the dossier export the two rules give the same answer. A single root would need either a new quire flag or a hard-coded `dossier` name, which breaks `--out other` (FR-191-AC-10). Close the gaps instead: component-wise path matching with an AC, a defined rule for bundles that mix a root input with a non-root one, and a visible tag commit when it differs (FND-005).

Owner question 2 (AC-6): FR-034-AC-6 stays as written. The first-parent walk and tree comparison can use the gix-odb / gix-object readers already linked (identity.rs already walks trees by hand), and the SHA-256 needs only an in-process pure-Rust digest crate. Neither starts a process or links a transport. If an implementer prefers `gix-diff`, `gix-traverse` or `gix-revwalk`, the crate list in FR-034 section B must name them. Hand-rolling needs no spec change.

## New findings (disposition pass 1)

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-007 | high | The fix's new FR-038 bullet "If the result `status` is `not_released`, then the command SHALL take no Person, Signed at or Record value from the nested `approval`" covers every `not_released` result. The Release-verify table reads Person, Signed at and Record from the nested `approval` for every `not_released` result except one whose nested status is `approved`. The Verdict file row also shows ` tag object: ` when "the Record cell is read from the nested `approval` (not in the `claimed:` case)". For a `not_released` result with a nested `untrusted_signer` (FR-038-AC-11) the requirement contradicts itself. Limit the bullet to a nested `approval.status` of `approved`. | FR-038 Release-verify input bullet 1; FR-038 Release-verify table; FR-038-AC-11; FR-038-AC-20 |
| FND-008 | low | FR-038-AC-11 and FR-038-AC-12 still end with "is not labelled an approval". The fix round removed every rule that defined labelling an entry as an approval, so the clause has no oracle. Replace it with the cell-content rule (the Verdict cell begins with `unverified claim: `), or drop it. | FR-038-AC-11; FR-038-AC-12 |
| FND-009 | low | FR-034-AC-13 says "when a non-dossier input is edited after the tag, so the later commit shows in the Release field". The Release field shows the tag commit T, not the later commit; the source commit S shows in its own field. Reword to "so the Release field names the tag commit". | FR-034-AC-13 |
| FND-010 | low | The Release field is now "shown as `<TAG> (tag commit <hash>)`" when T differs from S. FR-034 section C writes `release` to `in/metadata.yaml`, which organisation templates read (FR-036), and the spec does not say whether that value is the tag name alone or the shown text. State which, and add it to FR-034-AC-12 or FR-034-AC-13. | FR-034 Release row; FR-034 section C; FR-034-AC-12 |

## Dispositions

| FND | outcome | reason |
| --- | --- | --- |
| FND-001 | fixed | - |
| FND-002 | fixed | - |
| FND-003 | fixed | - |
| FND-004 | fixed | - |
| FND-005 | fixed | - |
| FND-006 | fixed | - |
| FND-007 | fixed | - |
| FND-008 | fixed | - |
| FND-009 | fixed | - |
| FND-010 | fixed | - |
