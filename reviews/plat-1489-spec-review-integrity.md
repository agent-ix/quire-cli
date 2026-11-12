---
id: SR-20261112034128
title: "PLAT-1489 quire-cli PR173 spec-review integrity"
type: SpecReview
analysis: integrity
scope: "agent-ix/quire-cli PR173 head 01136b3; spec/functional/FR-014-validate-okf-bundle.md, spec/tests.md"
review_set: subset
---

## Summary

Ticket: PLAT-1489. Integrity review of the FR-014 prose and AC-12..15 additions and the spec/tests.md rows. The `make spec` equivalent passes at head: `quire validate --summary` over the assurance docs and the edited files, `quire coverage --json`, and `quire-qualify assurance-traceability` (20/20). AC numbering is contiguous (AC-12..15 follow AC-11). The FR-014 summary row reads AC-1..15.

The IT ids were checked against origin/main and open PRs #119 and #123. PR #172 has since merged to main.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | high | IT-258, IT-259, IT-260 and IT-261 are already taken on origin/main. PR #172 (PLAT-1419, FR-029 publish-check) merged as 1df1208 after this branch's base and minted the same four ids. `git merge-tree` shows spec/tests.md merging textually clean, so landing PR173 silently produces two rows per id and two meanings per trace tag. Rebase and renumber to the next free ids (IT-262..IT-265 at 1df1208; open PRs #119 and #123 add no IT ids in that range). Update the FR-014 summary row, the IT rows and the `#[trace]` tags in tests/cli_bundle_line.rs. | spec/tests.md:74, spec/tests.md:306-309, tests/cli_bundle_line.rs:131 |
| FND-002 | low | FR-014-AC-13 is compound: a new obligation (the `line` field present or absent) plus a restated pre-existing invariant (warnings-then-errors order) that FR-014 prose already states. Split the order clause into its own AC, or drop it from AC-13, so a test can bind each part independently. | spec/functional/FR-014-validate-okf-bundle.md:119 |

## Examined scope

```yaml
scope:
- id: FR-014
  path: spec/functional/FR-014-validate-okf-bundle.md
  role: examined
  excerpt: "A finding the engine locates (BundleFinding.line, quire-rs FR-049-AC-19..22) carries its line exactly as a located document diagnostic does (<path>: line <N>: <message> [<reason>], FR-014-AC-12) and its JSON record gains a numeric line field (FR-014-AC-13)."
- id: FR-014-AC-12
  path: spec/functional/FR-014-validate-okf-bundle.md
  role: examined
  excerpt: "A bundle finding whose BundleFinding.line is Some(N) ... is printed in the located form <path>: line <N>: <message> [<reason>]"
- id: FR-014-AC-13
  path: spec/functional/FR-014-validate-okf-bundle.md
  role: examined
  excerpt: "the record of a finding with Some(N) carries a numeric line field equal to N; a finding with line None carries no line key. Record order (warnings first, then errors, engine order) is unchanged"
- id: FR-014-AC-14
  path: spec/functional/FR-014-validate-okf-bundle.md
  role: examined
  excerpt: "A bridged document error under --okf --strict carries the line of its document-validation result"
- id: FR-014-AC-15
  path: spec/functional/FR-014-validate-okf-bundle.md
  role: examined
  excerpt: "A finding with line None renders byte-identically to the pre-FR-014-AC-12 form <path>: <message> [<reason>]"
- id: IT-258
  path: spec/tests.md
  role: examined
  excerpt: "validate --okf: a dangling-trace-reference finding prints <path>: line <N>: at its referencing row ... and its JSON record carries line"
```

## Verdict

Changes requested. FND-001 (IT id collision with merged #172) must be fixed before merge. FND-002 is low.

## Dispositions

| FND | outcome | reason |
| --- | --- | --- |
| FND-001 | fixed | b1ccfb3 merges origin/main (1df1208) and renumbers to IT-262..IT-266. spec/tests.md has no duplicate IT row: FR-029 keeps IT-258..IT-261. Open PRs #119 and #123 add no IT-262..IT-269. The branch is up to date with main. |
| FND-002 | fixed | b1ccfb3: AC-13 now covers only the line/path/reason fields. Ordering is its own criterion, FR-014-AC-16, bound to IT-266. |
