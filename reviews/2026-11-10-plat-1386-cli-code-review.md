---
id: SR-20261110162001
title: "PLAT-1386 quire-cli PR162 code-review"
type: SpecReview
analysis: code-review
scope: "agent-ix/quire-cli PR162 head f08ce1f; src/commands/coverage.rs, tests/cli_coverage.rs, Cargo.lock, CHANGELOG.md"
review_set: subset
---

## Summary

Ticket: PLAT-1386. This is a code review with a Rust lane (`/rust-review`) of `git diff origin/main...origin/plat-1386-cli` at head f08ce1f. The upstream spec is quire-rs 92507f7, covering FR-050 "Strict exemption of planned, blocked and parked rows", FR-050-AC-64..66, and TC-2225/TC-2226.

The focused test run passed: `cargo test --test cli_coverage` reported 27 passed and 0 failed.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | low | IT-181 checks the strict and `error`-promoted runs with `!status.success()`. TC-2226 specifies exit 1, and the same file already asserts `status.code() == Some(1)` for promotion (line 1197). As written, an exit 2 would also pass. The stderr message assertions narrow this, but the exit code is not pinned. | tests/cli_coverage.rs:771-787 |

## Examined scope

```yaml
scope:
- id: src/commands/coverage.rs
  path: src/commands/coverage.rs
  role: examined
  excerpt: "(\"unbacked-row\", report.non_exempt_unbacked()), ... if t.strict_exempt_unbacked > 0 { primary_line!(\"strict-exempt unbacked rows (planned/blocked/parked): {}\", ...) }"
- id: tests/cli_coverage.rs
  path: tests/cli_coverage.rs
  role: examined
  excerpt: "strict_exempt_fixture, it180_strict_exempts_planned_rows_and_prints_the_exempt_line, it181_strict_still_fails_on_a_non_exempt_unbacked_row"
- id: Cargo.lock
  path: Cargo.lock
  role: examined
  excerpt: "quire-rs 7e61e14 -> 92507f7; agent-ix-semantic-schema (filament-core-data) 6505433 -> ce72bc5"
```

## Verdict

Mergeable. The one finding is a low test-strength nit.

- **Gate ordering.** `full_counts` is captured from the full report before `project_by_severity`. Both the `error`-promotion filter and the `--strict` unbacked premise read `full_counts[0]`, which is now `non_exempt_unbacked()`. Exempt rows therefore never reach either gate, and a `=off` projection cannot change the verdict. The zero-population check and the status-lie count are independent of the exempt count, so they are unchanged.
- **Non-strict behaviour.** Without promotion, behaviour is unchanged (IT-181 `plain`). With promotion, exempt rows are no longer counted, which FR-050-AC-66 requires.
- **Text line.** It reads `totals.strict_exempt_unbacked`, which projection does not touch, so it prints the same count under `=off`. It goes to stdout with the rest of the census (CR-012). JSON passes the engine's `totals.strict_exempt_unbacked` and per-row `strict_exempt` through unmodified, so text and JSON agree. Under `=off`, JSON keeps the exempt total while `unbacked_rows` is cleared. This is consistent with the existing projection semantics.
- **Rust.** The change is minimal and idiomatic, with no new arithmetic in the CLI. The subtraction lives in the engine's `non_exempt_unbacked()` and is bounded by construction. There is no panic path.
- **Cargo.lock side effect.** The bump moved filament-core-data from 6505433 to ce72bc5. quire-rs 92507f7's own Cargo.lock pins the same ce72bc5, so the bump aligns the CLI with what the engine was built and tested against. For the `agent-ix-semantic-schema` crate, the only content difference is an additive optional `import_types` property in `module-manifest.schema.json` and `module-semantic-block.schema.json`. This is acceptable.
- **Existing tests.** No existing test was weakened. The diff only adds tests. IT-092's code is unchanged and only its matrix wording changed.
