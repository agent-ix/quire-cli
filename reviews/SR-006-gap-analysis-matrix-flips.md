---
id: SR-006
title: "Gap analysis — the six ⚠️→✅ matrix flips of #49, verified by execution"
type: SpecReview
analysis: gap-analysis
scope: "spec/tests.md, src/self_update/, tests/cli_update.rs, tests/audit_static.rs"
review_set: subset
---

## Summary

PR #49 flipped six Test Matrix rows — IT-083, IT-084, TC-085, TC-086, TC-087,
TC-093 — from ⚠️ to ✅ on the strength of the engine's `backed` predicate
alone; no test run was recorded (SR-005 FND-001, ticket
agent-ix/quire-cli#55 / NC-1). Backed is not passing: a tag can resolve to a
test that fails, is `#[ignore]`d, or asserts nothing. This artifact is the
missing evidence. Each of the six rows' backing tests was located in the
source, run individually with the exact command recorded, and the full suite
was run once end-to-end. Environment: main @ 8f77628 (the merged state of
#49+#50+#54), quire-rs resolved offline from the cargo git checkout at tag
v0.41.0 (rev 7278e98) — the build completed with no network dependency.

## Verdict

**PASS — zero over-claims.** All six rows have a real, currently-passing
backing test; none is an over-claim requiring a flip-back under #55. The full
suite is green (149 tests, 0 failures) and the coverage instrument on the same
tree reads 231/256 backed with `status_lies: []` and none of the six rows in
`unbacked_rows`. The gap #49 created was evidentiary, not substantive: the ✅
marks were true, and unproven at merge time. Three partial-assertion gaps
inside the backing tests are recorded below for #55/#56 to weigh — they narrow
what "passing" attests, without unseating any row.

## Findings

| ID      | Severity | Summary                                                                                                   | Refs |
| ------- | -------- | ---------------------------------------------------------------------------------------------------------- | ---- |
| FND-001 | low      | IT-083/IT-084's "performs no install/network" clause holds by construction (Unknown source path never shells out), not by observation — `audit_no_network.rs` probes parse/validate/extract/schema/lookup but not `update` | tests/cli_update.rs:1-6, tests/audit_no_network.rs |
| FND-002 | low      | TC-087's row text claims "reports git-branch tracking"; the backing test asserts `Action::Checked { latest: None }` and never inspects `report.messages`, so the tracking-report wording is unasserted | src/self_update/mod.rs:348-358 |
| FND-003 | low      | TC-093's static gate is substring-based: a grouped import (`use crate::{io, ...}`) would not contain any `FORBIDDEN` needle, so the inspection can be evaded without tripping | tests/audit_static.rs:139-157 |

## Per-row execution evidence

All commands run 2026-08-21 from `/home/peter/dev/quire-cli` on main @
8f77628. Result lines are quoted verbatim from cargo's output.

### IT-083 — `update --check` on Unknown source (P0)

- Backing test: `tests/cli_update.rs` →
  `update_check_on_unknown_source_prints_manual_instructions_and_exits_zero`,
  tagged `// IT-083, FR-016-AC-1, FR-016-AC-2` (line 12).
- Command: `cargo test --test cli_update update_check_on_unknown_source_prints_manual_instructions_and_exits_zero`
- Result: `test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1 filtered out`
- Verdict: **PASS** — asserts exit 0, Unknown-source guidance, npm recipe and
  cargo recipe on stdout. The no-network clause is by construction (FND-001).

### IT-084 — bare `update` on Unknown source (P1)

- Backing test: `tests/cli_update.rs` →
  `update_without_check_on_unknown_source_is_also_safe`, tagged
  `// IT-084, FR-016-AC-2` (line 37).
- Command: `cargo test --test cli_update update_without_check_on_unknown_source_is_also_safe`
- Result: `test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1 filtered out`
- Verdict: **PASS**.

### TC-085 — `detect_source` classification (P0)

- Backing tests: `src/self_update/mod.rs` `tests` module — four `detect_*`
  tests tagged `// TC-085, FR-016-AC-1` (lines 295, 303, 310, plus the
  lookalike-path guard).
- Command: `cargo test --lib self_update::tests::detect_`
- Result: `test result: ok. 4 passed; 0 failed; 0 ignored; 0 measured; 20 filtered out`
- Verdict: **PASS** — covers node_modules→Npm, .cargo→Cargo, bare→Unknown,
  and lookalike components staying Unknown (more than the row claims).

### TC-086 — `registry_args` forms (P0)

- Backing tests: `src/self_update/mod.rs` `tests` module — three
  `registry_args_*` tests tagged `// TC-086, FR-016-AC-4` (lines 266, 272, 283).
- Command: `cargo test --lib self_update::tests::registry_args`
- Result: `test result: ok. 3 passed; 0 failed; 0 ignored; 0 measured; 21 filtered out`
- Verdict: **PASS** — all three claimed shapes (scoped form, plain
  `--registry`, empty vec) have one test each.

### TC-087 — `run_for_source` Cargo/--check and Unknown (P1)

- Backing tests: `src/self_update/mod.rs` `tests` module —
  `cargo_check_reports_branch_tracking_without_a_version` and
  `unknown_source_emits_manual_instructions_without_installing`, both tagged
  `// TC-087, FR-016-AC-1, FR-016-AC-5` (lines 330, 348).
- Commands:
  `cargo test --lib self_update::tests::cargo_check_reports_branch_tracking_without_a_version`
  and
  `cargo test --lib self_update::tests::unknown_source_emits_manual_instructions_without_installing`
- Results: `test result: ok. 1 passed; 0 failed; ...` for each (23 filtered out).
- Verdict: **PASS**, with FND-002's caveat: the `latest: None` oracle is
  asserted; the "reports git-branch tracking" phrasing is not.

### TC-093 — self_update engine is package-agnostic (P1, Static)

- Backing test: `tests/audit_static.rs` →
  `tc093_self_update_engine_is_package_agnostic`, tagged
  `// TC-093, FR-016-AC-5, FR-016-AC-6, StR-004-AC-2` (line 133). The row is
  type Static, and the gap SR-002 closed made it executable: the inspection is
  a real test that greps `src/self_update/mod.rs` for forbidden context
  imports and `src/commands/update.rs` for parser/validator reach-ins.
- Command: `cargo test --test audit_static tc093_self_update_engine_is_package_agnostic`
- Result: `test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 3 filtered out`
- Verdict: **PASS**, with FND-003's caveat on substring evasion.

## Whole-suite and instrument corroboration

- Command: `make test` (→ `cargo test`)
- Result: 21 test binaries, **149 passed, 0 failed, 0 ignored** — including
  IT-107/IT-108 from #50 and the full cli_fix set containing #54's retagged
  `path_traversal_rejected`.
- Command: branch-built binary, `quire coverage --scope /home/peter/dev/quire-cli --json`
- Result: `status_lies: []`; `unbacked_rows` row_ids are
  `FR-001 render subcommand, NFR-001-AC-1..3, NFR-002-AC-2, NFR-006-AC-1,
  IT-001, IT-009, IT-010, IT-016, IT-017, IT-018, TC-088, FR-001-AC-1..6,
  FR-003-AC-3, FR-006-AC-4, FR-016-AC-3, FR-016-AC-7` — the six flipped rows
  appear in none of them; human census reads **231/256 rows backed (90%)**.
  FR-016-AC-3/AC-7 unbacked matches the 🚧 rollup #49 deliberately kept.

## Disposition for #55

The six ✅ marks stand as substantively correct; this artifact supplies the
recorded runs that were missing at merge time. What remains for #55 is
policy, not evidence: (a) decide whether SR-006 retroactively satisfies the
"recorded passing run" requirement or the rows flip back until the gate is
mechanical; (b) the FR-016 rollup's `Coverage Status` column is still
unenforced (`functional_coverage` declares no `column_patterns`), so the next
#49-shaped edit is still uncatchable by the instrument. No spec/tests.md edit
is made here — the flip-back decision and any rollup enforcement land under
#55.
