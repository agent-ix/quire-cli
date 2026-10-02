---
id: SR-178
title: Shared CLI outcome Rust diff review
type: SpecReview
relationships:
  - target: "ix://agent-ix/quire-cli/FR-007"
    type: reviews
---

# SR-178: Shared CLI outcome Rust diff review

## Summary

Reviewed the approved outcome migration against the shared taxonomy and Rust review checklist. Classification is driven by typed sources and explicit domain verdicts, never message matching. Shared JSON and primary writer calls retain the utility migration's boundary.

## Scope

Reviewed revision 46b6956cee3c37b7889bb9dabce93d4cbfebf7e3. Workflow diff is empty. Files examined:

- `README.md`
- `reviews/2026-10-02-cli-kit-outcome-spec-review.md`
- `spec/functional/FR-003-extract-subcommand.md`
- `spec/functional/FR-004-validate-subcommand.md`
- `spec/functional/FR-005-path-safety.md`
- `spec/functional/FR-007-exit-codes.md`
- `spec/functional/FR-009-schema-subcommand.md`
- `spec/functional/FR-011-lookup-subcommand.md`
- `spec/functional/FR-012-edit-subcommand.md`
- `spec/functional/FR-013-lint-subcommand.md`
- `spec/functional/FR-014-validate-okf-bundle.md`
- `spec/functional/FR-015-fix-subcommand.md`
- `spec/functional/FR-016-update-subcommand.md`
- `spec/functional/FR-026-matrix-subcommand.md`
- `spec/tests.md`
- `src/commands/assurance.rs`
- `src/commands/clauses.rs`
- `src/commands/coverage.rs`
- `src/commands/edit.rs`
- `src/commands/extract.rs`
- `src/commands/failure.rs`
- `src/commands/fix.rs`
- `src/commands/lint.rs`
- `src/commands/lookup.rs`
- `src/commands/matrix.rs`
- `src/commands/mod.rs`
- `src/commands/parse.rs`
- `src/commands/properties.rs`
- `src/commands/provenance.rs`
- `src/commands/schema.rs`
- `src/commands/symbols.rs`
- `src/commands/trace.rs`
- `src/commands/validate.rs`
- `src/io.rs`
- `src/main.rs`
- `src/safety.rs`
- `tests/cli_assurance.rs`
- `tests/cli_clauses.rs`
- `tests/cli_coverage.rs`
- `tests/cli_edit.rs`
- `tests/cli_errors.rs`
- `tests/cli_extract.rs`
- `tests/cli_fix.rs`
- `tests/cli_lint.rs`
- `tests/cli_lookup.rs`
- `tests/cli_matrix.rs`
- `tests/cli_okf.rs`
- `tests/cli_reference_status_column.rs`
- `tests/cli_sandbox.rs`
- `tests/cli_schema.rs`
- `tests/cli_validate.rs`

## Verdict

Fix the primary human result emitter before delivery. The typed failure and input classifications preserve diagnostic chains; the open upstream enum has an explicit Internal fallback. No credentials or compatibility layer are introduced.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | medium | A closed stdout pipe on human coverage, trace, property or validation summary output reaches println and can panic instead of returning Internal code 4. The remaining local emitter must delegate to the fallible shared writer and propagate errors. | src/io.rs:151 at 46b6956cee3c37b7889bb9dabce93d4cbfebf7e3 |

## Validation

The first complete make ci gate passed. Real process tests cover successful, invalid, refusal and partial reports; a closed parse-output pipe returns 4. The human output fix needs expanded process coverage and complete gates before delivery.

## Dispositions

- FND-001: fixed 02b668f6a134a32075db49ed8c6fbf17ede51058. Removed the local result emitter, propagated shared writer errors through human report and summary renderers, and expanded the real closed-pipe test to parse and human properties. The complete make ci gate passes after this fix. No diagnostic text is used for outcome classification.
