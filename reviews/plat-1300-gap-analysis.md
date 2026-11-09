---
id: SR-20261102086809
title: "PLAT-1300 quire-cli PR158 gap-analysis"
type: SpecReview
analysis: gap-analysis
scope: "agent-ix/quire-cli PR158; src/commands/validate.rs, tests/cli_okf.rs, spec/functional/FR-014-validate-okf-bundle.md, spec/tests.md"
review_set: subset
---

## Summary

Ticket: PLAT-1300. Planless scoped gap pass over the PR diff against FR-014 (validate --okf bundle) and spec/tests.md. The PR changes observable `--okf` behavior (cross-repo dangling refs are no longer reported) without a requirement or test-catalog entry, and labels its new test with an id already owned by a coverage test. Plan completion: not assessed.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | medium | New test is labeled IT-089, which spec/tests.md already assigns to cli_coverage::it089 (FR-017-AC-1); the new test has no tests.md row and no AC binding | tests/cli_okf.rs:219 |
| FND-002 | medium | No FR-014 acceptance criterion covers the cross-repo skip; FR-014-AC-3 still says any broken ix:// link yields a dangling-reference warning, which is now false for cross-repo targets | spec/functional/FR-014-validate-okf-bundle.md:102 |

## Examined scope

```yaml
scope:
- id: FR-014-AC-3
  path: spec/functional/FR-014-validate-okf-bundle.md
  role: examined
  excerpt: "A broken ix:// link under --okf is a warning: the command exits 0 and stderr carries a [dangling-reference] diagnostic"
- id: FR-014-AC-6
  path: spec/functional/FR-014-validate-okf-bundle.md
  role: context_only
  excerpt: "quire validate --okf --scope <DIR> --module $M with no positional validates <DIR>/spec as the bundle root, with <DIR> itself kept as the reference root"
- id: IT-089
  path: spec/tests.md
  role: examined
  excerpt: "coverage renders the human census on stderr and leaves stdout empty without --json (cli_coverage::it089_*) | Integration | P1 | FR-017-AC-1"
```

## Verdict

Gaps found (medium). Add an FR-014 AC for the cross-repo skip (package declared from the repo origin; foreign-authority miss skipped; own-authority miss still warns; no package means every miss warns), amend AC-3 accordingly, give the new test a fresh IT id with a tests.md row bound to that AC.

## Dispositions

| FND | outcome | reason |
| --- | --- | --- |
| FND-001 | fixed | 8291a73: tests renamed it178/it179 with IT-178/IT-179 rows in spec/tests.md bound to FR-014-AC-3/AC-10; no collision on origin/main or open PRs #119/#123 |
| FND-002 | fixed | 8291a73: FR-014-AC-10 added and AC-3 amended to the own-authority / no-package case; FR-014 coverage row now AC-1..10 |
