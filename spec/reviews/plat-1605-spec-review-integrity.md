---
id: SR-20261112234818
title: "PLAT-1605 quire-cli PR182 spec-review/integrity"
type: SpecReview
analysis: integrity
scope: "agent-ix/quire-cli PR182 head 9194f3a; spec/functional/FR-031,FR-032,FR-034..FR-040, spec/non-functional/NFR-010,NFR-011, spec/stakeholder/StR-004, spec/usecase/US-010"
review_set: subset
---

## Summary

Ticket: PLAT-1605. Integrity of the new and amended artifacts against each other and against StR-004, NFR-004, NFR-010 and the merged FR-031..FR-033.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | high | StR-004-VC-2 was not updated. It still forbids 'template rendering' and 'JSON Schema definition/validation logic' in src/, and it names only the FR-027 and FR-031 exceptions. The PR adds three src/ duties: FR-035 generates the default template (Typst source and a DOCX reference document), FR-034 reads git's object store (commit, committer date, tag, and the blob compare), and FR-038 parses and checks the EA `approval-verify-result/v1` document. The validation criterion now contradicts FR-034, FR-035 and FR-038. The prose exception is also unbounded: it calls these 'likewise output data' without limiting the git read to the operations FR-034 and FR-038 need. Fix: name FR-034, FR-035 and FR-038 in VC-2, and bound the exception to read-only lookup of one commit, its committer date, a tag's target and date, and one blob compare, with no index or worktree writes, no ref updates and no fetch. | spec/stakeholder/StR-004-thin-boundary-over-quire-rs.md:26, :55 |
| FND-002 | low | The exclusion is circular. NFR-011 says FR-037 names the properties excluded from the stability property, and FR-037 says they are 'excluded ... by NFR-011 and named there'. Neither file names any. Extracted text and structure do not include embedded identifiers or compression order in any case, so the exclusion is vacuous. Delete the Excluded bullet and the matching clause of FR-037. | spec/non-functional/NFR-011-export-reproducibility.md:34-36, spec/functional/FR-037-export-clock-free-output.md:40-42 |
| FND-003 | low | The Statement promises equality 'whatever the ... host name' of the run, but Scope limits the property to 'one host'. Typst also resolves fonts from the host when the template's fonts lack a glyph, and FR-032 fixes the typst arguments to exactly `compile`, input, output, so `--ignore-system-fonts` cannot be passed. Say 'on one host, whatever its host name string, user ...', or add `--ignore-system-fonts` to the typst request and widen the property. | spec/non-functional/NFR-011-export-reproducibility.md:20-24, :38-42 |
| FND-004 | low | FR-031-AC-1 was changed to a 'committed fixture' because Markdown export now needs a committed source and an identity (title, id, and version or release). FR-031-AC-2's passing half ('the same file passes against a module set that admits it') was not changed. A test of it now also needs the file committed and carrying `version`, or the export fails under FR-040 for an unrelated reason. Add 'committed' and the identity precondition to AC-2. | spec/functional/FR-031-export-subcommand.md:51 |

## Examined scope

```yaml
scope:
- id: StR-004-VC-2
  path: spec/stakeholder/StR-004-thin-boundary-over-quire-rs.md
  role: examined
  excerpt: 'StR-004-AC-2: `src/` contains no markdown parsing, template rendering, graph construction, assurance projection, or JSON Schema definition/validation logic — only argv/premise parsing, path-safety checks, stdin/stdout wiring, calls into `quire-rs`, and the explicitly owned FR-027 Cargo.lock developer gate and FR-031 export presentation transform.'
- id: StR-004
  path: spec/stakeholder/StR-004-thin-boundary-over-quire-rs.md
  role: examined
  excerpt: The identity read of FR-034 (commit, date and tag from the repository's object store) and the approval entries of FR-038 are likewise output data for the converter, not Markdown semantics. This exception authorizes no document parser, engine model, graph, assurance, source-analysis, or general qualification-tool duplication.
- id: NFR-011
  path: spec/non-functional/NFR-011-export-reproducibility.md
  role: examined
  excerpt: For one committed input, one source commit, one template (or the default), the same approval verdict files and the same converter versions, `quire export` SHALL produce documents whose extracted text and extracted structure are equal, whatever the wall-clock time, time zone, host name, user, working directory or capability-root path of the run. Byte equality of the file is not required.
- id: NFR-011-excl
  path: spec/non-functional/NFR-011-export-reproducibility.md
  role: examined
  excerpt: 'Excluded: embedded random identifiers and compression order, which FR-037 names where a converter cannot suppress them.'
- id: FR-037
  path: spec/functional/FR-037-export-clock-free-output.md
  role: examined
  excerpt: Where a converter would record a path or a random identifier in the file, the command SHALL set the option or metadata that suppresses it; a property it cannot suppress is excluded from the stability property by NFR-011 and named there.
- id: FR-031-AC-2
  path: spec/functional/FR-031-export-subcommand.md
  role: examined
  excerpt: Exporting a Markdown file that fails validation against the `--module` set exits 3, writes no output file and starts no converter; the same file passes against a module set that admits it.
- id: FR-031
  path: spec/functional/FR-031-export-subcommand.md
  role: context_only
  excerpt: The last two lines of options, and the identity, title page and layout a Markdown input receives, are specified by FR-034 through FR-039; a Markdown `<INPUT>` may repeat for a bundle. ... A Markdown input exported by the commands of those FRs needs a committed source and an identity; JSON inputs carry none.
```

## Verdict

Changes required (FND-001). The FR-031, FR-032, NFR-010 and spec.md edits are consistent with each other and with FR-033 (exit codes 2, 3 and 4 match FR-007/FR-033). FR-040 cleanly extends the FR-033 error table without overlap. The FR and NFR relationship edges are well formed, and every cross-reference resolves to an existing file.
