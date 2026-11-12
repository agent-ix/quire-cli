---
id: SR-20261112032001
title: "Base spec-review of FR-028 matrix criticality column (quire-cli PR 166)"
type: SpecReview
analysis: base
scope: "agent-ix/quire-cli PR166 head 0ad25b1; spec/functional/FR-028-matrix-criticality-column.md, spec/functional/index.md"
review_set: subset
relationships:
  - target: ix://agent-ix/quire-cli/FR-028
    type: references
---

# Base spec-review of FR-028 matrix criticality column (quire-cli PR 166)

## Summary

Ticket: PLAT-1412. Base checklist over `git diff origin/main...HEAD` at 0ad25b1: the new FR-028 and its index line. Read against FR-026 (the requirement it extends), `src/commands/matrix.rs`, `src/commands/coverage.rs` and `src/commands/trace.rs` on main, quire-rs FR-082 (quire-rs#645) and spec-artifacts-iso FR-008 (spec-artifacts-iso#70). `quire validate` (quire 0.40.0, engine 0.56.0) exits 0 on both changed files; the one warning is about the local spec-artifacts-app module, not these files. No hash, digest, pin or tracking-record antipattern is present.

Deviation from the ticket's `quire trace` wording: judged correct. `src/commands/trace.rs` `TraceReport` holds `claims.verifies[]`, `claims.implements[]` and `citations[]` records keyed by trace id, with no requirement rows. The requirement-level model is FR-050's `coverage_matrix` `requirements[]`, which `quire matrix` renders. This agrees with FR-082's own re-hosting.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | medium | FR-028 renders the engine value "verbatim" in Markdown and TSV cells, but quire-rs FR-082-AC-6 carries an out-of-enumeration value as written. FR-028 does not say how a value containing `\|`, a tab or a newline is rendered (FR-026 §C/§D pipe-escapes and scrubs Statement cells for exactly this), nor what a present empty string `""` renders as (an empty cell, which FR-026 §D calls indistinguishable from a rendering bug), nor a non-string YAML value. Fix: apply FR-026's tab/newline scrub and pipe-escape to the cell, and state the `""` case. | spec/functional/FR-028-matrix-criticality-column.md:25-30, FR-028-AC-1, FR-028-AC-2 |
| FND-002 | medium | The `Criticality` column sits on criterion rows but carries the requirement document's value, while quire-rs FR-053 defines a different per-criterion `criticality` that `quire coverage --format tsv` already renders on obligation rows (`src/commands/coverage.rs:504`, absent as an empty cell). A reader of a matrix row will take the cell as that criterion's criticality. FR-028 does not mention FR-053's value, whether the matrix shows it, or why the two absent conventions (`unassigned` here, empty in `coverage`) differ. Fix: name the column for its level (for example `Requirement criticality`) or state that the per-criterion value is not shown and why. | spec/functional/FR-028-matrix-criticality-column.md:25-27, src/commands/coverage.rs:504 |
| FND-003 | low | The Description carries change history ("The originating ticket words this as a `quire trace` column; that wording is a deviation this requirement makes deliberately"). The spec should state the target design; the first sentence of that paragraph already says why the column is on the matrix. Fix: drop the ticket sentence, or move it into a CR note as FR-026 does. | spec/functional/FR-028-matrix-criticality-column.md:21-23 |

## Verdict

Changes requested (two medium, one low). Clean units: FR-028-AC-3, AC-4 and AC-5 are testable and single-outcome; AC-4 correctly keeps an absent key distinguishable from an authored `unassigned`, which matches FR-082-AC-2/AC-3. The index line matches the title. The `quire matrix` host is the right one.

## Dispositions

| FND | outcome | reason |
| --- | --- | --- |
| FND-001 | fixed | Round 1 (2a9935b): "Missing, present and odd values" applies FR-026's pipe-escape and tab/newline/CR scrub, states the present empty string renders an empty cell distinct from `unassigned`, and renders non-strings as compact JSON; FR-028-AC-7..AC-10 test each. |
| FND-002 | fixed | Round 1 (2a9935b): column renamed `Requirement Criticality` / `requirement_criticality`; the FR-053 per-criterion value and its empty-when-absent convention are explained, and the matrix does not show it because FR-050-AC-48 criterion entries do not carry it (confirmed in quire-rs `CoverageMatrixCriterion`). |
| FND-003 | fixed | Round 1 (2a9935b): the ticket-wording sentence moved into a `CR note (PLAT-1412)` block. |
