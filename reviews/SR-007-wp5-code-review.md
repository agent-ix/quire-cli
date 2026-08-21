---
id: SR-007
title: "WP5 pre-release code review — the seven unreviewed commits staged for v0.29.0"
type: SpecReview
analysis: code-review
scope: "src/commands/coverage.rs, src/commands/validate.rs, src/self_update/mod.rs, tests/cli_coverage.rs, tests/cli_fix.rs, tests/audit_no_network.rs, tests/audit_static.rs, .github/workflows/release.yml, Cargo.toml, Cargo.lock, npm/quire-cli/package.json, spec/functional/FR-016-update-subcommand.md, spec/functional/FR-017-coverage-subcommand.md, spec/tests.md"
review_set: subset
---

## Summary

Final pre-release review gate for v0.29.0 — the phase's only npm publish —
over the seven commits this session landed on main without review
(`git log 5ecec96..HEAD` at review start): 841b224 (#52 version staging +
release.yml binary-version guard), ea84563 (#55 FR-016 relocation + Matrix
Rule 7), 8ce7c35 (#56 six test-debt items), 3ec380a (#51 row_id half),
b1533a1 (#53 severity pack + TSV + `--pretty`), 06633bb (quire-rs v0.42.0),
faacec7 (#51 close-out). Reviewed under the `rust-review` discipline:
tautology/stub detection over IT-107..116 and the reworked tc810/tc093,
mutation spot-checks on every load-bearing behavior, seam and panic-surface
checks, and the gates actually run. The companion
[SR-008](./SR-008-wp5-gap-analysis.md) carries the per-ticket acceptance
evidence and the release verdict.

Unlike SR-005 (retroactive, nothing fixed), this review **fixed its
findings in-tree**: two code-level defects were ticketed (#57) and closed by
0f6fe2f, and the release-precondition gap was closed by 30b04ef (#52).

## Verdict

**PASS after fixes** — three findings fixed on this tree (two medium via
#57, one medium via the #52 changelog commit), three low findings recorded
(one tracked on #57, two accepted with rationale). No finding invalidated a
shipped behavior: every mutation aimed at a claimed contract was caught by
the tests that claim it, the release guard fails closed on both trigger
paths, and the staged versions are in lockstep.

## Findings

| ID      | Severity | Summary | Refs |
| ------- | -------- | --------- | ---- |
| FND-001 | medium   | **FIXED (0f6fe2f, #53 → #57).** A `--severity coverage:<check>` entry naming a check outside the pack's four was a silent no-op — FR-048 validates only key shape, so `coverage:unbaked-row=off` merged, matched nothing, and the run proceeded as if the flag were absent. Now rejected before any document is read; IT-110 pins it | src/commands/coverage.rs (`reject_unknown_pack_checks`), tests/cli_coverage.rs (it110) |
| FND-002 | medium   | **FIXED (0f6fe2f, #53 → #57).** The `tsv_cell` tab/newline/CR replacement (FR-017-AC-14's column-integrity clause) was pinned by nothing: mutating it to the identity left all 159 tests green, because the #53 measurement's 0/1,107 means no corpus fixture carries a structural character. TC-812 (unit) now proves a hostile cell yields exactly one nine-column record | src/commands/coverage.rs (`tests::tc812_*`) |
| FND-003 | medium   | **FIXED (30b04ef, #52).** Release precondition: `[Unreleased]` was empty and the changelog stopped at 0.23.0 while v0.24.0..v0.28.0 shipped unrecorded — and the GitHub Release this workflow creates says "See CHANGELOG.md for details". The 0.29.0 entry and a retroactive 0.24.0–0.28.0 block are authored | CHANGELOG.md |
| FND-004 | low      | **RECORDED on #57.** FR-017-AC-12's fallback clause — "a record without a row id renders the reference kind leading, exactly as before" — is pinned by no test: every fixture producing unbacked rows either declares a `row_id_column` or asserts nothing about the rendered line, so mutating the `None` arm of `finding_identity` survives the suite | src/commands/coverage.rs (`finding_identity`), agent-ix/quire-cli#57 (comment) |
| FND-005 | low      | **ACCEPTED.** The new `update`/`update --check` strace probes (#56) fail **open** when `strace` is absent (skip + eprintln) — the file's documented convention (`#![cfg(target_os = "linux")]`, "stays no-op on minimal containers"), matching the rust-review external-dependency rule; CI runs ubuntu-24.04, where strace is present. A fail-closed lane would need a CI-only env assert, out of scope | tests/audit_no_network.rs:13-16,42-48 |
| FND-006 | low      | **ACCEPTED.** `Cargo.lock` records quire-rs `version = "0.33.0"` against `tag=v0.42.0` — the pin is correct (rev 991ac32, updated in the same commit as Cargo.toml, 06633bb); the number is upstream's own unbumped manifest, the identical class SR-005 FND-006 recorded for v0.41.0 | Cargo.lock:1019-1022 |
| FND-007 | low      | **ACCEPTED (style note, no scenario).** The release guard's `else` branch (`${GITHUB_REF_NAME#v}`) is dead code today — `release.yml` triggers on `workflow_dispatch` only — but it is the correct comparison if a tag-push trigger is ever added, and it was verified by simulation (SR-008 §Release preconditions) | .github/workflows/release.yml:121-137 |

## Mutation spot-checks (all reverted; recorded runs on this tree)

Each pinning test was proven to *catch*, not merely to pass:

| # | Mutation | Expectation | Result |
|---|----------|-------------|--------|
| 1 | `project_by_severity` stops clearing `status_lies` under `off` | it110/it111 fail | **CAUGHT** — `cargo test --test cli_coverage it11` → `FAILED. 5 passed; 2 failed` (`it110_severity_pack_projects_and_promotes`, `it111_tsv_emits_one_record_per_line_on_stdout`) |
| 2 | TSV `status-lie` row emits `""` for the `row_id` column | it111 fails | **CAUGHT** — `FAILED. 0 passed; 1 failed` (`it111`) |
| 3 | `finding_identity` ignores `line` (bare-document locus always) | locus tests fail | **CAUGHT** — `FAILED. 18 passed; 4 failed` (`it107`, `it109`, `it113`, `it114`) |
| 4 | release.yml guard comparison, wrong version | guard exits 1 | **CAUGHT** — guard logic simulated verbatim against the fresh 0.29.0 binary: dispatch `0.29.0` → OK; tag `v0.29.0` → OK; dispatch `0.30.0` → `FAIL … exit=1`; tag `v0.28.0` → `FAIL … exit=1` |
| 5 | `tsv_cell` → identity (probe) | some test fails | **SURVIVED** on the reviewed tree (`ok. 22 passed`) → FND-002; **CAUGHT** after the fix (`cargo test --bin quire tc812` → `FAILED. 0 passed; 1 failed`) |

## Review notes by lens

- **Tautology/stub detection (IT-107..116, tc810, tc093).** None found.
  Every new assertion names a value the implementation computes, not one the
  test set: it109/it113 pin exact rendered lines including engine-provided
  document line numbers over a fixture whose preamble shifts the table (so a
  table-relative index would fail loudly); it110 pins absolute exit codes
  and full-computation totals; tc810's rework replaced the pretty-spacing
  substring needles (vacuous under the #53 compact default) with parsed-
  payload assertions; tc093's rework carries its own evasion-proof loop —
  the four grouped-import shapes the old substring gate missed are each
  asserted to be caught. it107's `strict == plain` near-tautology from
  SR-005 FND-003 is discharged (both pinned to `Some(0)`).
- **Release guard.** Runs in the `build` job after the smoke test, per
  matrix target, `shell: bash` (Windows included). Both trigger-path
  expressions correct; `awk '{print $2}'` matches the clap `quire X.Y.Z`
  shape; comparison fails closed (a `v`-prefixed dispatch input would fail
  the guard rather than pass a wrong binary). Verified by simulation
  (table above) since CI is dispatch-only by policy.
- **Severity totals semantics.** Consistent across all three surfaces by
  construction: one projected struct feeds human/JSON/TSV; `totals`/`groups`
  are never touched by projection; `--strict` and `error` promotion judge
  `full_counts` captured before projection. it110 pins each leg.
- **Seam compliance.** `apply_severity_overrides` widened to `pub(crate)`
  and reused rather than reimplemented — parsing/precedence/reject-before-
  read cannot drift between `validate` and `coverage`. No `#[cfg(test)]`
  branches in production code; tc093 (source inspection) remains the
  sanctioned last resort for an absence no runtime path can observe.
- **Panic surface / conversions.** New code paths are infallible string
  assembly plus `bail!`-mediated refusals; no `unwrap`/indexing on request
  paths, no `as` casts, no new unsafe (`make audit-unsafe` green). `line`
  renders via `Option<usize>` → `to_string`, no arithmetic.
- **Wire contract.** `--json` unchanged except: whitespace (AC-15,
  `--pretty` now honoured — parsed-identity pinned by it112) and the
  spec'd AC-13 projection under an explicit flag. New v0.42.0 fields ride
  the wholesale `CoverageReport` serialization, absent-when-empty (it116).
- **Integrity.** No new `#[allow]`, no gate weakening, no lint changes.
  The one clippy finding introduced during this review's own fix
  (`items_after_test_module`) was fixed before commit; `make ci` green
  before every push.

## Gates (recorded run, this tree, 2026-08-21)

- `make ci` → exit 0: fmt-check, clippy `-D warnings`, **160 tests / 23
  suites, 0 failures**, `cargo deny` licenses+bans ok, unsafe-comment audit
  ok, thin-boundary audit ok.
- `quire validate spec/tests.md spec/functional/FR-017-coverage-subcommand.md
  spec/functional/FR-016-update-subcommand.md` (the freshly built 0.29.0
  binary) → exit 0.
