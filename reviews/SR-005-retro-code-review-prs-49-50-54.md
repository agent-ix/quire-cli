---
id: SR-005
title: "Retroactive code review — quire-cli #49, #50, #54 (merged unreviewed)"
type: SpecReview
analysis: code-review
scope: "spec/tests.md, spec/functional/FR-017-coverage-subcommand.md, spec/functional/FR-015-fix-subcommand.md, src/commands/coverage.rs, src/self_update/, tests/cli_coverage.rs, tests/cli_fix.rs, Cargo.toml, Cargo.lock"
review_set: subset
---

## Summary

Retroactive review of the three PRs the trace-status-integrity session merged
into this repository without a code review or gap analysis: #49 (`spec(tests):
retire the warning status marker`, merge c4bf34c), #50 (`feat(coverage): carry
the v0.41.0 engine capabilities to the command line`, merge 160881f), and #54
(`spec(trace): normalize slash-separated id chains`, merge 8f77628). All three
were merged by their author with zero reviews and zero CI runs (CI is
manual-only by policy; nothing manual was dispatched either). The code is
reviewed **as merged** — nothing is fixed here, because every defect found is
already ticketed: agent-ix/quire-cli#51 (grew a fifth finding kind from #50),
agent-ix/quire-cli#52, agent-ix/quire-cli#55 (NC-1) and
agent-ix/quire-cli#56 (NC-2). The companion artifact
[SR-006](./SR-006-gap-analysis-matrix-flips.md) carries the test-execution
evidence for #49's six status flips.

## Verdict

**CONDITIONAL** — one high and four medium findings, all cross-referenced to
open tickets rather than fixed in this pass; three low findings recorded for
the ticket owners. No finding invalidates the merged behavior itself: the
`--strict` contract is unchanged (IT-092 still passes), the six flipped rows
are substantively green (SR-006), and IT-108's `source_exclude` wiring is
sound. What the batch lacked was evidence discipline, and two of its spec
edits made spec text and code disagree.

## Findings

| ID      | Severity | Summary                                                                                                      | Refs |
| ------- | -------- | ------------------------------------------------------------------------------------------------------------ | ---- |
| FND-001 | high     | **#49 → ticket #55.** Six matrix rows flipped ⚠️→✅ justified only by the engine's `backed` predicate — backed is not passing; P0/P1 rows re-declared passing with no recorded test run | spec/tests.md:190-194,200 |
| FND-002 | medium   | **#50 → ticket #51.** Human render of `UndeclaredStatus` prints `reference`/`document`/`status` and drops the `row_id` the record carries — fifth instance of the row_id-omission class | src/commands/coverage.rs:198-208 |
| FND-003 | medium   | **#50 → ticket #56.** IT-107's `--strict` assertion compares two exit codes without pinning either to 0 — satisfiable by a shared failure, so it cannot catch a fixture regression that breaks both invocations identically | tests/cli_coverage.rs (it107, final assert) |
| FND-004 | medium   | **#50 → ticket #56.** FR-017-AC-9 still names `extract_tree_excluding` while this PR moved the call to `extract_tree_scoped` — the staleness is introduced by the very change under review | spec/functional/FR-017-coverage-subcommand.md:78, src/commands/coverage.rs:95 |
| FND-005 | medium   | **#54 → ticket #56.** The comment-only slash→comma edit mints a new `FR-015-AC-5` trace binding on `path_traversal_rejected` with no test change and no recorded coverage-gate run | tests/cli_fix.rs:166 |
| FND-006 | low      | **#50 → ticket #52 (scope note).** `Cargo.lock` records quire-rs `version = "0.33.0"` against `tag=v0.41.0` — the pin itself is **correct** (rev 7278e98); the number is upstream's own unbumped `Cargo.toml`, the same set_version defect class #52 tracks here | Cargo.lock:1020-1022 |
| FND-007 | low      | **#49 → ticket #55.** FR-016 rollup correctly kept 🚧 over "no automated trace" prose, but its `Coverage Status` column is unenforced — `functional_coverage` declares no `column_patterns`, so nothing would have caught a blanket ✅ | spec/tests.md:73 |
| FND-008 | low      | **#50 → ticket #56.** IT-107's human-surface and strict/plain invocations never assert process success; only the `--json` run does | tests/cli_coverage.rs (it107) |

## PR #49 — the six flips and the one non-flip

The diff is spec/tests.md only: six row-level statuses ⚠️→✅ (IT-083, IT-084,
TC-085, TC-086, TC-087, TC-093) and the FR-016 rollup ⚠️→🚧. The PR body's own
justification is explicit: "The same engine run that reads them reports every
one **backed**, which is the exact predicate `status_lies` negates," with
before/after instrument readings (227/252 → 227/252, lies 0 → 0). That is a
category error the batch's own programme exists to eliminate: `backed` means a
tracking tag resolves to a test symbol; ✅ in a Status column means the
verification **passed**. No test execution is recorded anywhere in the PR. The
flip happens to be substantively right — SR-006 runs all six backing tests and
they pass — but at merge time the evidence did not exist, which is precisely
NC-1 (#55). Whether the rows flip back pending recorded runs, or SR-006's runs
stand as the evidence, is #55's decision; this artifact only establishes that
the merged justification was insufficient (FND-001).

The judgment call the PR did get right: the FR-016 rollup went 🚧, not ✅,
because its own Test Cases cell states three criteria have "no automated
trace" (npm-channel check/install AC-3, registry-unreachable exit AC-7, cargo
install inside AC-1's dispatch). The live instrument agrees — FR-016 reads 5/7
with FR-016-AC-3 and FR-016-AC-7 in `unbacked_rows`. But the correctness of
that cell is manual: the `Coverage Status` column carries no `column_patterns`
enforcement, so the restraint was voluntary (FND-007, folded into #55's
"rollup column unenforced" item).

## PR #50 — v0.41.0 adoption

The substantive change is good: `extract_tree_scoped` passes
`model.source_exclude` to the walk (IT-108 verifies the glob subtracts the
fixture tag and spares real source — asserted in both directions, with a
control run proving the test can fail), and `undeclared_statuses` is rendered
on the human surface rather than left machine-only. Four defects ride along:

1. **row_id dropped in the human render** (FND-002). quire-rs's
   `UndeclaredStatus` carries `row_id: Option<String>` (coverage.rs:104-110 at
   rev 7278e98) and IT-107 asserts it on the JSON surface; the human line
   formats only `reference`, `document`, `status`. This is the same class as
   the four finding kinds #51 already tracks — #51's body has been extended
   with this fifth kind. Verified live: the branch binary's census prints
   `verification (spec/functional/FR-016-update-subcommand.md) has no backing
   symbol` — no row id on any finding line.
2. **IT-107 `--strict` assertion is unpinned** (FND-003). The final assertion
   is `assert_eq!(strict.status.code(), plain.status.code())`. The AC-10 text
   it verifies says "the exit code ... is the same with and without the flag",
   and the equality does catch a future `--strict` gate on this class — but it
   is equally satisfied when *both* invocations fail identically (module-load
   error, fixture rot), and neither invocation's success is asserted
   (FND-008). Pinning `plain` to `Some(0)` makes the tautological branch
   unreachable. Tracked as the "IT-107 tautology" item of #56.
3. **FR-017-AC-9 went stale in the same diff that staled it** (FND-004). AC-9
   enumerates the delegated engine calls and still names
   `extract_tree_excluding`; the code this PR merged calls
   `extract_tree_scoped`. An Inspection-verified AC whose named symbols don't
   exist in the code is uninspectable. Second item of #56.
4. **Version metadata** (FND-006). `Cargo.lock`'s quire-rs entry reads
   `version = "0.33.0"` with `source = git+...?tag=v0.41.0#7278e98`. The
   resolution is correct — 0.33.0 is what quire-rs's own `Cargo.toml` declares
   at that tag, eight releases of unbumped upstream metadata. It is the same
   defect #52 tracks in this repo (this crate itself still builds as
   `quire-cli v0.23.0`, and the installed binary reports 0.23.0). Recorded
   here as scope color for #52, not a new ticket.

Also noted, no action: `undeclared_statuses` is `skip_serializing_if =
"Vec::is_empty"` upstream by documented design (byte-identity for conformant
repos), so its absence from a clean repo's JSON is intentional, and IT-107's
fixture exercises the non-empty path.

## PR #54 — one comment line

The entire diff is tests/cli_fix.rs:166 changing `// IT-080 / FR-015-AC-5:` to
`// IT-080, FR-015-AC-5:`. Under the comma-list trace grammar this **mints a
binding** — `FR-015-AC-5 → path_traversal_rejected` — where the slash form
bound nothing, i.e. a trace-graph behavior change shipped as a comment edit
with no test change and no gate run recorded on the PR (FND-005). Two
mitigating facts, both new to this review: FR-015-AC-5 was already bound by
two other comma-form tags (`it080_fix_with_no_positional_uses_the_scope_spec_root`
at cli_fix.rs:178, `it080_fix_without_a_spec_root_is_a_named_error` at
cli_fix.rs:219), so the AC was not orphaned before the edit; and the minted
binding is semantically defensible, since AC-5's text explicitly includes "a
`..` or symlink-escape path on the root is rejected by path-safety" — the
clause `path_traversal_rejected` asserts. The defect is procedural (a minted
binding with no verification run), which is the third item of #56, not a
wrong edge.

## Gates

Run for this review on main @ 8f77628 (post-batch), quire-rs pinned at
v0.41.0 rev 7278e98 from the local cargo git checkout:

- `make test` — 149 tests across 21 binaries, **0 failures** (full log in
  SR-006, which also runs the six flipped rows individually).
- `quire coverage --scope .` with the branch-built binary — **231/256 rows
  backed, `status_lies: []`**, none of the six flipped rows in
  `unbacked_rows`. (The PR-body figure 227/252 was pre-#50; #50's IT-107/108
  rows and FR-017-AC-10/11 account for the drift.)
- No CI ran on any of the three PRs and none was dispatched manually;
  merge-time evidence is therefore whatever the PR bodies recorded, which for
  #49 was the instrument reading only.
