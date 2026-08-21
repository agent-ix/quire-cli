---
id: SR-008
title: "WP5 gap analysis — ticket acceptance and release preconditions for v0.29.0"
type: SpecReview
analysis: gap-analysis
scope: "spec/functional/FR-016-update-subcommand.md, spec/functional/FR-017-coverage-subcommand.md, spec/tests.md, src/commands/coverage.rs, src/self_update/mod.rs, tests/, .github/workflows/release.yml, CHANGELOG.md, npm/quire-cli/package.json, Cargo.toml, Cargo.lock"
review_set: subset
---

## Summary

Companion to [SR-007](./SR-007-wp5-code-review.md). This artifact answers
two questions for the v0.29.0 gate — the phase's only npm publish: does each
closed ticket's acceptance actually hold on this tree, with evidence; and do
the release runbook's preconditions hold. Environment: main @ 30b04ef,
quire-rs resolved offline from the cargo git checkout at tag v0.42.0
(rev 991ac32). All commands run 2026-08-21 from `/home/peter/dev/quire-cli`;
result lines quoted from the recorded runs.

## Findings

| ID      | Severity | Summary | Refs |
| ------- | -------- | --------- | ---- |
| FND-001 | medium   | **FIXED (30b04ef).** Release precondition failed as found: the changelog the GitHub Release points readers at ended at 0.23.0, with `[Unreleased]` empty and v0.24.0..v0.28.0 never recorded (same code-level finding as SR-007 FND-003; carried here because it is a runbook precondition, not a code defect) | CHANGELOG.md, #52 |
| FND-002 | low      | **NO REPO ACTION.** First `--version` observation read `quire 0.23.0` — traced to a stale binary under the repo-local `target/` shadowed by the machine-wide `CARGO_TARGET_DIR=/home/peter/.cargo-target`; the fresh build reports `quire 0.29.0` and CI reads the path the fresh build populates. Dev-machine artifact only | Release preconditions §1 |

## Ticket acceptance

### #51 — coverage findings omit the row_id the record carries (CLOSED — holds)

- **Row-id-leading lines** (item 1): FR-017-AC-12, IT-109 — two same-document
  rows render distinguishable lines, reference kind kept in the bracketed
  trailer, old kind-leading shape asserted absent. Mutation 2 (SR-007)
  proves the TSV id column is load-bearing; mutation 3 proves the human
  render is.
- **`document:line` loci** (item 3): FR-017-AC-16, IT-113 — fixture preamble
  shifts the table so the asserted `:13`/`:14` can only be document lines;
  bare-document form asserted gone for line-carrying records. it107/it109/
  it111/it114 re-pinned to lined loci.
- **`no_symbol_rows` rendered**: FR-017-AC-17, IT-114 — exemption line with
  row id, locus, verbatim test-type value; a symbol-minting row asserted NOT
  to render it. The decision (outside the AC-13 pack) is recorded in the FR
  CR note.
- **`source_exclude` observable**: FR-017-AC-18, IT-115 — `1 source file(s)
  excluded…` at N=1, silence at zero, `SymbolExtraction` unreadable-file
  diagnostic on stderr, stdout empty throughout.
- **Advisory passthrough**: FR-017-AC-19, IT-116 — `shared_trace_ids` with
  both binders; empty `vocabulary_coverage` off the wire.
- **Item 2 (findings to stdout)**: formally WONTFIX, superseded by
  `--format tsv`; recorded in the FR-017 CR note; AC-1 stands unamended.
- Recorded run: `cargo test --test cli_coverage` → `ok. 22 passed; 0 failed`.

### #52 — binary version drift / release staging (OPEN — staging half holds)

- `Cargo.toml` `version = "0.29.0"`; `Cargo.lock` `quire-cli 0.29.0`
  (regenerated in 841b224); `npm/quire-cli/package.json` `0.29.0` with all
  four `@agent-ix/quire-cli-*` optionalDependencies at `0.29.0` in lockstep
  — the `set_version.sh` output of 841b224 verified intact on HEAD.
- The release.yml guard runs in the build job for every matrix target and
  fails closed; see Release preconditions below.
- The ticket rightly stays open until the publish itself; the changelog
  precondition it implied is now met (30b04ef).

### #53 — agent-first coverage output (CLOSED — holds)

- **Severity pack**: FR-017-AC-13, IT-110 — `off` drops the kind from
  human AND `--json` (and TSV, it111) with the suppression announced
  (`2 coverage:status-lie finding(s) suppressed`); `totals` pinned to the
  full reconciliation (`totals.total == 2` with both finding kinds
  projected off); `--strict` fails on the full computation with both
  gated kinds rendered off; `=error` exits 1 without `--strict`; malformed
  entries rejected before read. Post-review hardening: a typo'd check is
  now rejected too (#57, SR-007 FND-001).
- **`--format tsv`**: FR-017-AC-14, IT-111 — nine-column header, every
  record fully columned, row id leading, `line` column carrying the
  engine's document line, byte-identical across runs, projection applies.
  The escaping clause is now pinned by TC-812 (#57, SR-007 FND-002): a
  cell carrying tab/newline/CR still yields one nine-column record —
  the answer to "what happens when a statement DOES carry one" is
  *replaced with spaces, structure intact*, and it is now enforced.
- **`--pretty` honoured**: FR-017-AC-15, IT-112 — compact by default,
  indented under the flag, identical parsed value.
- Recorded runs: it110/it111/it112 within the 22-pass cli_coverage suite;
  `cargo test --bin quire tc812` → `ok. 1 passed`.

### #55 — backed is not passing (CLOSED — holds)

- FR-016's untraced-AC prose relocated from the unenforced matrix
  `Coverage Status` column into FR-016's own CR note (the validated
  document); the rollup cell reduced to a pointer; the FR-016 row stays 🚧.
- Matrix Rule 7 (status flip requires a recorded passing run; `backed` is a
  symbol claim, not a run) recorded in `spec/tests.md` §Rules. This
  review's own flips comply: TC-812's ✅ carries its recorded run (SR-007
  gates section and the 0f6fe2f commit message).
- The six #49 flips stand on SR-006's six/six PASS evidence, unchanged.

### #56 — v0.41.0-adoption test debt (CLOSED — all six discharged)

1. IT-107 absolute exit codes — `strict`/`plain` each pinned `Some(0)`
   (the near-tautological equality removed).
2. FR-017-AC-9 names `extract_tree_scoped` — matches the call at
   `src/commands/coverage.rs` (verified on HEAD).
3. FR-015-AC-5 binding — `path_traversal_rejected` states its oracle and
   pins exit 1 + `PathTraversal` naming the `..` + no finding line
   (rejection before any load).
4. TC-087 — branch-tracking wording asserted against `report.messages`
   (phrase + tracked repo), not just `Action::Checked { latest: None }`.
5. TC-093 — grouped imports expanded to full paths, whitespace-normalized
   inline references checked, and the gate proven against four evasion
   shapes in the test itself.
6. IT-083/IT-084 no-network — observed under strace
   (`update_check_does_not_open_inet_socket`,
   `update_without_check_does_not_open_inet_socket`); Matrix Rule 6 records
   the probe. Fail-open skip when strace is absent is the file's documented
   convention (SR-007 FND-005, accepted).

## Release preconditions (runbook)

- **The binary builds and reports 0.29.0.** Fresh build on this tree:
  `cargo build` → `Compiling quire-cli v0.29.0 … Finished`; `quire
  --version` → `quire 0.29.0`. (First observation of `0.23.0` traced to the
  stale binary under the repo-local `target/` shadowed by the machine-wide
  `CARGO_TARGET_DIR=/home/peter/.cargo-target` — a dev-machine artifact,
  not a tree defect; CI sets no `CARGO_TARGET_DIR` and reads
  `target/<target>/release/quire`, the path the fresh build populates.)
- **The guard would fail a mismatched tag.** Its comparison logic was run
  verbatim against the fresh binary for all four cases:
  dispatch `0.29.0` → `OK: 0.29.0 == 0.29.0`;
  tag `v0.29.0` → `OK`;
  dispatch `0.30.0` → `FAIL: binary 0.29.0 vs release 0.30.0`, exit 1;
  tag `v0.28.0` → `FAIL`, exit 1. The guard sits in the build job after the
  smoke test, per matrix target, `shell: bash` on all four runners.
- **npm staging in lockstep.** Launcher `0.29.0`; the four platform
  optionalDependencies `0.29.0`; per-platform packages are generated at
  release time by `npm/build-packages.mjs` from the same resolved version.
- **CHANGELOG names the release.** `## [0.29.0]` authored (30b04ef); the
  GitHub Release note's "See CHANGELOG.md" no longer points at a log ending
  at 0.23.0.
- **Gates.** `make ci` → exit 0 (fmt-check, clippy `-D warnings`, 160
  tests / 23 suites / 0 failures, deny licenses+bans, unsafe audit,
  thin-boundary audit). `quire validate` (0.29.0 binary) → exit 0 over
  `spec/tests.md`, FR-016, FR-017, and both SR artifacts.
- **Not done here, by instruction:** no tag pushed, nothing published, no
  release workflow dispatched.

## Verdict

Every closed ticket's acceptance holds with recorded evidence; the one open
ticket (#52) is open for the right reason — the publish itself. The three
review findings that could have shipped (silent no-op flag, unenforced
escaping clause, changelog ending six versions early) are fixed on this
tree and gated green.

v0.29.0: GO
