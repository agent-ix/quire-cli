---
id: SR-004
title: "Gap analysis — quire-cli trace id collisions (#45)"
type: SpecReview
analysis: gap-analysis
scope: "spec/functional/, spec/tests.md, tests/"
review_set: subset
---

## Summary

Post-implementation verification of `agent-ix/quire-cli#45` on branch
`fix/45-trace-id-collisions`. The deliverable was that four colliding trace ids
each name one behaviour, and that the behaviours involved own a matrix row and
an acceptance criterion. Both collisions are removed and verified by
falsification. Two further gaps beyond the ticket were found and closed here:
two tests carrying no trace tag at all, and a comment-shaped hazard introduced
by the fix itself.

## Verdict

**PASS** — no matrix row claims evidence it lacks, no test carries a tag that
resolves to nothing, and every test in `tests/` now carries a tag or is
documented as deliberately claiming none. The rollup is 227/252 with zero
status lies and zero untracked symbols; all 23 unbacked targets are unbackable
by construction and unchanged by this branch.

## Findings

| ID      | Severity | Summary                                                                  | Refs                                |
| ------- | -------- | ------------------------------------------------------------------------ | ----------------------------------- |
| FND-001 | medium   | **FIXED IN BRANCH.** `ears_grammar_warnings_are_advisory_and_summarized` and `ears_grammar_warnings_fail_under_strict` carried no trace tag in any form, so the advisory-grammar and `--strict`-promotion behaviours had no owning criterion. `quire coverage` cannot see this class: it reports tags that resolve to nothing, never a test that was never tagged | tests/cli_validate.rs:469, :490     |
| FND-002 | low      | **FIXED IN BRANCH.** The explanatory comment added above the untagged `cli_errors` test placed `IT-050` at line-initial position. It does not bind today — `rust-comment-id` requires a delimiter after the id and the following word saves it — but reflowing the comment would silently recreate the collision the branch removes | tests/cli_errors.rs:95              |
| FND-003 | low      | The coverage report exposes no per-target list of which symbols back a row, so a collision is invisible in the output and can only be found by removing a tag and watching a count move. This is the root reason `#45` had to be discovered by hand; it is a quire-rs concern, recorded not fixed | —                                   |

## Matrix verification

Measured with the repo's own release build under a repo-local
`CARGO_TARGET_DIR`. The `quire` on PATH is 0.23.0 and must not be used.

| Measure | Before (`main`) | After |
| --- | --- | --- |
| Backed | 215/240 | **227/252** |
| Status lies | 0 | **0** |
| Untracked symbols | 0 | **0** |
| Tests with no trace tag | 2 | **0** |

The +12 is six new rows (IT-101..106) and six new criteria
(FR-004-AC-19..22, FR-018-AC-8/9), all backed.

### The collisions, verified by falsification

A rising `backed` proves nothing about collisions — it rises whenever rows are
added. The property under test is that a row **fails** when its binder goes.

| Probe | Result | Reads as |
| --- | --- | --- |
| on `main`: retag `output_contract.rs` IT-095 → undeclared id | backed 215 → **215**, dead 0 → 1 | row stayed green on its twin — collision real |
| on `main`: rename `fn it_060_…` → `fn it_902_…` | **no change at all** | never bound — `rust-test-name-id` is `\bfn (?i:tc)(\d+)_`, TC only |
| after fix: retag `cli_properties.rs` IT-095 → undeclared id | backed 223 → **222**, dead 0 → 1 | single binder — collision gone |
| after fix: retag `cli_validate.rs` IT-050 → undeclared id | backed 223 → **222**, dead 0 → 1 | FND-002's prose does not bind |

The second row is why the ticket's diagnosis needed correcting: IT-060/061 was
never a double-bind. The two `cli_validate.rs` tests carried no tag in any
form, and the behaviour underneath — scoped path and glob resolution — had no
matrix row anywhere. The collision existed only in the function names.

## Underspecified code (reverse gap)

This is where the branch grew past its ticket. Four behaviours were being
tested while no requirement owned them:

| Behaviour | Now owned by |
| --- | --- |
| relative path under `--scope`, no `--module` (exact-module branch) | FR-004-AC-19 / IT-101 |
| relative glob under `--scope` surfacing the invalid match | FR-004-AC-20 / IT-102 |
| `properties --json` conforms to the published `properties-v1.schema.json` | FR-018-AC-8 / IT-103 |
| absent obligation source → `obligation: null`, still conforms | FR-018-AC-9 / IT-104 |
| grammar findings advisory — valid document with violations exits 0 | FR-004-AC-21 / IT-105 |
| `--strict` promotes grammar findings to exit 1 | FR-004-AC-22 / IT-106 |

The criteria were backfilled rather than pointing the tests at whichever
existing criterion was nearest. Two cases were close enough to be tempting and
are deliberately kept distinct: `FR-004-AC-10/11` escalate the unknown-`object:`
warning, which is a different finding source from grammar checks; and
`FR-004-AC-13`/IT-081 cover module **discovery** from search roots, whereas
IT-101's fixture root carries `manifest.yaml` and therefore takes the
exact-module branch.

No source code changed on this branch, so there is no new unowned
implementation.

## Unbacked targets

23 reference rows remain unbacked, the same set `#43` recorded, none introduced
here:

| Targets | Why unbackable |
| --- | --- |
| FR-001-AC-1..6, NFR-001-AC-1..3, and rows IT-001/009/010/017/018, TC-088, `FR-001 render subcommand` | the `render` subcommand and its latency budget, retired under spec.md §2bis |
| FR-003-AC-3, IT-016 | sugar-field harvesting, retired — no engine ever implemented it |
| FR-006-AC-4 | its text names the retired FR-001 |
| NFR-002-AC-2 | `Demonstration` — a fresh-machine `cargo install` |
| NFR-006-AC-1 | a CHANGELOG process rule; any test would assert its own fixture |
| FR-016-AC-3, FR-016-AC-7 | npm install and registry-unreachable paths (network + global-install side effects) |

## Coverage

Plan completion does not apply — this is a ticket, not a plan bundle;
`plan/plan.md` predates it. Semantic review was **not** re-run here: it ran as
part of `/code-review` on the same diff and is recorded in
`reviews/26-08-20-trace-id-collisions.md` (SR-003), which found and fixed two
acceptance criteria that claimed more than their tests assert.

Gates re-run rather than assumed: `make ci` exits 0 (fmt-check, clippy
`-D warnings`, tests, `cargo deny` licenses + bans, unsafe audit, thin-boundary
audit). `quire validate --scope . "spec/**/*.md"` exits 0 with zero errors; the
19 grammar warnings are pre-existing and confined to files this branch does not
touch.
