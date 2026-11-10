---
id: SR-20261110162002
title: "PLAT-1386 quire-cli PR162 gap-analysis"
type: SpecReview
analysis: gap-analysis
scope: "agent-ix/quire-cli PR162 head f08ce1f; spec/functional/FR-017-coverage-subcommand.md, spec/tests.md, src/commands/coverage.rs, tests/cli_coverage.rs"
review_set: subset
---

## Summary

Ticket: PLAT-1386. This is a planless gap pass scoped to the diff. Plan completion: not assessed. The brief limited the review to focused tests, so `quoin matrix` was not run. Instead, I checked each statement of upstream quire-rs (92507f7) TC-2225, TC-2226 and FR-050-AC-64..66 against the CLI code and the CLI tests by hand.

I confirmed one behaviour empirically with the built binary on a scratch fixture with one `🚧 planned` row and one `✅` row with no backing symbol. `--strict` exits 1 and prints `1 unbacked row(s) and 1 contradicted status(es) (--strict)`, and the exempt line prints 1.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | medium | No CLI test covers the TC-2226 leg "an all-exempt fixture that also holds a Complete row without evidence exits 1". IT-181 mixes a planned row with a bare `🚧` row, which gives no status lie. IT-092 has a `✅` lie but no exempt rows. The behaviour is correct (verified by hand), but the strict status-lie premise beside exempt rows is not asserted, for example as `1 contradicted status(es)` with exit 1. | tests/cli_coverage.rs:764, spec/tests.md:299 |
| FND-002 | low | TC-2226's "incomplete explicit evidence inventory" leg cannot be built at this CLI, and neither can FR-050-AC-66's "applies identically on the FR-078 evidence path". `coverage` calls only `quire_rs::compute_coverage`, never `compute_with_evidence`, and has no evidence input. The amended FR-017-AC-7 says only "The other strict premises are unchanged" and does not record that the inventory premise does not exist at the CLI, so a reader of TC-2226 cannot tell this leg is not applicable rather than missing. | spec/functional/FR-017-coverage-subcommand.md:154, src/commands/coverage.rs:279 |

## Examined scope

```yaml
scope:
- id: FR-017-AC-7
  path: spec/functional/FR-017-coverage-subcommand.md
  role: examined
  excerpt: "`--strict` exits 1 when a non-exempt row is unbacked or a live status is contradicted, including beside explicitly retired rows, and exits 0 on the same repository without it. An unbacked row whose status is planned, blocked or parked is strict-exempt (quire-rs FR-050-AC-64): it neither fails `--strict` nor counts toward an `error`-promoted `coverage:unbacked-row`, so a repository whose only unbacked rows are exempt exits 0 under `--strict`, also with that check promoted to `error`. The other strict premises are unchanged. ..."
- id: FR-017-AC-8
  path: spec/functional/FR-017-coverage-subcommand.md
  role: context_only
  excerpt: "A model matching zero live and zero retired rows fails `--strict` with a diagnostic distinct from the unbacked-rows one, rather than reporting full coverage."
- id: TC-2225
  path: "quire-rs spec/tests.md"
  role: examined
  excerpt: "CLI: the text output prints `strict-exempt unbacked rows (planned/blocked/parked): N` only when N is nonzero"
- id: TC-2226
  path: "quire-rs spec/tests.md"
  role: examined
  excerpt: "CLI: `--strict` exits 0 for an all-exempt fixture, also with `coverage:unbacked-row` promoted to `error`, and exits 1 for a mixed fixture; it exits 1 for an all-exempt fixture that also holds a Complete row without evidence or an incomplete explicit evidence inventory, and for the same scope with no declared targets; without `--strict` the exit status is unchanged"
- id: IT-180
  path: spec/tests.md
  role: examined
  excerpt: "`coverage --strict` exits 0 when the only unbacked rows are `🚧 planned`/`🚧 Parked` (also with `coverage:unbacked-row=error`) and the text census prints `strict-exempt unbacked rows (planned/blocked/parked): N`, unchanged under `=off`"
- id: IT-181
  path: spec/tests.md
  role: examined
  excerpt: "A non-exempt unbacked row beside exempt ones fails `--strict` and the `error`-promoted check counting only itself; without `--strict` the exit is unchanged; no exempt rows prints no exempt line"
- id: IT-092
  path: spec/tests.md
  role: examined
  excerpt: "`coverage --strict` exits 1 on a non-exempt unbacked row while the same repository exits 0 without it"
```

## Verdict

Mergeable. FND-001 is a small test addition and FND-002 is a one-sentence spec clarification. Both can go in a fix round inside this PR.

- **TC-2225.** Covered. IT-180 asserts the exact line with N=2, under default severities, under `=error`, and under `=off`. IT-181 asserts no line when N=0.
- **TC-2226.**
  - The all-exempt exit-0 case, the `=error` exit-0 case, the mixed exit-1 case and the non-strict-unchanged case are all covered by IT-180 and IT-181.
  - The empty-population leg is covered by TC-797 (`tc797_zero_matched_rows_is_not_full_coverage`). Exempt rows imply a non-empty population, and the zero-target check runs independently of the exempt count, so that test is sufficient.
  - The status-lie leg is FND-001.
  - The inventory leg is FND-002.
- **`=off`.** The `=off` override is not consulted for the exemption. IT-180 shows the same count under `=off`.
- **Bindings.**
  - The new `#[trace]` bindings are correct: IT-180 and IT-181 bind FR-017-AC-7, and both rows exist in spec/tests.md bound to FR-017-AC-7.
  - The FR-017-AC-7 verification cell lists IT-092, IT-180 and IT-181.
- **Follow-up outside this repo.** The quire-rs matrix still marks TC-2225 and TC-2226 as "awaiting `quire-cli`" and the FR-050 summary row says "CLI cases await". Flipping those rows belongs in quire-rs after this merges.

## Dispositions

| FND | outcome | reason |
| --- | --- | --- |
| FND-001 | still-open | Round 1 (head 0e55041) added IT-182, but its assertion `err.contains("contradicted status")` cannot fail. The strict message is always `"{unbacked} unbacked row(s) and {lies} contradicted status(es) (--strict)"` (src/commands/coverage.rs:209), and in this fixture the `✅` row is itself a non-exempt unbacked row, so it holds with 0 lies too. Exit 1 also comes from the unbacked premise. The test must assert `1 contradicted status(es)` (tests/cli_coverage.rs:828). |
| FND-002 | fixed | Round 1: FR-017-AC-7 now names the surviving premises and states "`quire coverage` calls only `compute_coverage` and has no evidence path, so the engine's incomplete-evidence-inventory premise does not exist at this CLI." |
| FND-001 | fixed | Round 2 (head 102b777): IT-182 now asserts `err.contains("1 contradicted status(es)")`. With 0 lies the message reads `0 contradicted status(es)`, so the assertion can now fail. |
