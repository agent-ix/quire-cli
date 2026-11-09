---
id: SR-20261102086811
title: "PLAT-847 quire-cli PR160 gap-analysis"
type: SpecReview
analysis: gap-analysis
scope: "agent-ix/quire-cli PR160 head b05771b; src/commands/{matrix,trace,validate}.rs, tests/*.rs, spec/functional/FR-026-matrix-subcommand.md, spec/tests.md"
review_set: subset
---

## Summary

Ticket: PLAT-847. This is a planless gap pass scoped to the diff. Plan completion: not assessed. Quoin's matrix was not run because that needs a build of this head, and the brief confined the review to checks that do not run the gate.

The manual AC-to-test check covered several points. Every id in the 199 added markers is a minted row key. Each required target of the `make spec` assurance gate carries a marker: FR-020-AC-1..8, IT-136..144, TC-814 and StR-004-VC-2/3, the last four through the multi-line marker on `tc090` in tests/audit_static.rs. Only one test in src/ and tests/ cites minted ids without a marker: `cli_errors::unknown_archetype_exits_3_with_named_error`, which is deliberately unbound and documents why.

The author's figure of 65 unbacked rows out of 429 was not re-measured, so whether it is a regression against main is not assessed here.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | low | FR-026-AC-16 requires the unmatched-id advisory to name the symbol and give guidance to write the declared marker form. This PR rewords that guidance, but the test asserts only the id, the path and "was not read by the declared trace grammar"; neither the symbol nor the guidance text is checked. | tests/cli_matrix.rs:1034, src/commands/matrix.rs:90 |
| FND-002 | low | The new `#[trace("FR-004-AC-2")]` is on a unit test that only compares the RowWidth remedy string. FR-004-AC-2 requires exit 3 and a line-numbered diagnostic naming the failing section/assert, which this test does not exercise. IT-048 already backs the AC properly, so this marker inflates the binder count. | src/commands/validate.rs:860 |

## Examined scope

```yaml
scope:
- id: FR-026-AC-16
  path: spec/functional/FR-026-matrix-subcommand.md
  role: examined
  excerpt: "In every output format, `matrix` SHALL emit engine coverage diagnostics and unmatched annotation IDs on stderr with the authored file and line when available. A forbidden source-tag range retains its `range-in-trace-tag` explanation; an ID not read by the module's declared trace grammar is named with the symbol and guidance to write it in the declared marker form. ..."
- id: FR-004-AC-2
  path: spec/functional/FR-004-validate-subcommand.md
  role: examined
  excerpt: "`quire validate broken-fr.md --module $ISO` exits 3; stderr contains a line-numbered diagnostic naming the failing section/assert"
- id: FR-004-AC-15
  path: spec/functional/FR-004-validate-subcommand.md
  role: examined
  excerpt: "With `--summary`, a run over a bundle whose documents emit findings from two different grammars prints a histogram containing a row for each grammar"
- id: FR-004-AC-18
  path: spec/functional/FR-004-validate-subcommand.md
  role: examined
  excerpt: "A malformed `--severity` entry ... exits non-zero with a usage diagnostic before any document is read"
- id: FR-017-AC-8
  path: spec/functional/FR-017-coverage-subcommand.md
  role: examined
  excerpt: "A model matching zero live and zero retired rows fails `--strict` with a diagnostic distinct from the unbacked-rows one"
- id: FR-007-AC-8
  path: spec/functional/FR-007-exit-codes.md
  role: examined
  excerpt: "A failed output write exits 4; typed classification is independent of human diagnostic wording."
- id: NFR-004-AC-2
  path: spec/non-functional/NFR-004-no-network.md
  role: examined
  excerpt: "An IT runs each subcommand under `strace -fe network` ... and asserts zero `socket()` calls"
- id: FR-004-AC-19
  path: spec/functional/FR-004-validate-subcommand.md
  role: examined
  excerpt: "A relative document path given with `--scope <DIR>` resolves under that scope"
- id: FR-004-AC-20
  path: spec/functional/FR-004-validate-subcommand.md
  role: examined
  excerpt: "A relative glob given with `--scope <DIR>` expands under that scope"
```

## Verdict

Minor gaps only (low). The matrix test rewrite still exercises the range and unmatched-tag diagnostics in Rust, Python and TypeScript, then verifies that the repaired marker form passes `--strict` cleanly. The census and capability removals are covered by the edited assertions. Additive retags (IT-083/084 with NFR-004-AC-2, IT-121 with FR-004-AC-19, IT-122 with FR-004-AC-20) match what the tests do. The IT-136 mis-binding is recorded under the code-review artifact, SR-20261102086810, as FND-001.

## Dispositions

| FND | outcome | reason |
| --- | --- | --- |
| FND-001 | fixed | 445d7eb asserts "declared marker form" and that the unmatched FR-001-AC-2 line names the `partial` symbol, in every language and format |
| FND-002 | fixed | 445d7eb removes `#[trace("FR-004-AC-2")]` and the now-unused import from src/commands/validate.rs; IT-048 alone backs FR-004-AC-2 |
