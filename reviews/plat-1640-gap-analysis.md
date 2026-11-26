---
id: SR-967175026302197768900474181935084794
title: "Gap analysis — quire-cli strict selftest trace markers"
type: SpecReview
analysis: gap-analysis
scope: "agent-ix/quire-cli; src/commands/matrix.rs, tests/cli_matrix.rs, spec/functional/FR-047-quire-selftest.md"
review_set: subset
relationships:
  - target: "ix://agent-ix/quire-cli/FR-047"
    type: references
---

## Summary

The new FR-047-AC-3 test tags the criterion and exercises the marker inventory with a custom module. The active module's marker entry paths still exclude selftest scripts, so the matrix cannot bind those real script markers until the companion module change lands. Plan completion: not assessed.

## Verdict

**FAIL** — the changed criterion is tested, but the production matrix binding behavior required by the change is absent under the active module configuration.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | high | The test's synthetic module configuration hides the absent selftest marker entry path in the active module. | tests/cli_matrix.rs:83-90 |

## Coverage

- Plan completion: not assessed.
- Matrix check: FR-047-AC-3 is tagged by selftest_shell_marker_binds_and_removal_names_the_case_script and local_trace_marker_is_retained_in_run_metadata; the changed marker contract has a focused test binding.
- Reverse-gap check: run, check_selftest_trace_markers, and collect_run_scripts are marked implements FR-047-AC-3; the new test and shell case carry trace markers. No other new untagged production behavior was found.
- Stub/coverage check: marker removal and unknown-criterion paths assert nonzero status and the script/criterion diagnostic. The test's synthetic module means this evidence does not cover the active module's production marker path.
- Optional semantic review was not run.

## Finding detail

FR-047-AC-3 is tested in the fixture, but the actual module declaration has entry_paths only for scripts/audits/**/*.sh. Its marker engine therefore does not discover selftest scripts as binders. A separate module change must land before this criterion is satisfied.


## Dispositions

| FND | outcome | reason |
| --- | --- | --- |
| FND-001 | still-open | Companion PR #172 is merged, and the narrow integration passes against module main 17da2da1772e03c300c7e577f4e96c06a7a7b745. The active module remains v0.36.0 at 45ac59ba9d2825c0b5312865dc9e03764930e01b without selftest entry paths; owner/planner routing of an approved module ref is still required. |

Disposition evidence: fix commit fbdefd8c46ddc4ec9f3194d6bb675926fdf11385. The focused marker test passed 1/1. The narrow module-main integration output showed the fixture criterion tagged by selftest/cases/example/run.sh.


## Disposition pass 2

Checked PR #247 at `fbdefd8c46ddc4ec9f3194d6bb675926fdf11385` against the superseding PLAT-1640 scope. The grammar fix remains correct. The published spec-artifacts-process `v0.38.0` at `9f3d3213b182270a67a10dbd9bbe42bd790ce9ee` still declares only `scripts/audits/**/*.sh` for shell marker entry paths. Thus the earlier production-binding finding remains open. No build or broad test ran.

| FND | outcome | reason |
| --- | --- | --- |
| FND-001 | still-open | Published v0.38.0 still omits selftest entry paths, so the fixture-only binding evidence remains insufficient. |

## Current scope assessment

The strict marker inventory remains relevant to artifacts that already exist and does not require high-risk features to gain cases. The new standalone selftest-list-cases asset should be reconciled with the approved normal-suite subset design before merging; artifact retirement remains deferred until the replacement works. The review does not demand additional high-risk cases or zero uncovered requirements.

Current mergeability: **no**.


## Disposition pass 3

Checked `c909b6404af2b31b15de24c4e4bd4575cd68b408`. The FND-003 repository-identity defect is fixed: the retained two-member reproduction now emits each declared identity in the diagnostic subject and message. The standalone selftest-list-cases asset and its embedded metadata are removed, preserving the pre-existing local/linked set. The Linux executable is bound read-only at /tmp/quire/quire before /tmp is remounted read-only; this keeps the command available beneath the empty /home mount. The PR records three focused locked-build tests as passing (identity regression, shell grammar/removal regression, and empty-HOME run record). Independent formatting and diff checks passed. No broad or long test ran in this disposition.

The published spec-artifacts-process v0.38.0 still excludes selftest/**/run.sh from shell entry_paths. That production binding finding remains externally blocked while the planner holds module pins/releases; this pass does not claim it resolved.

| FND | outcome | reason |
| --- | --- | --- |
| FND-001 | still-open | External blocker: published v0.38.0 still omits selftest entry_paths; planner holds module pins/releases. |

Current mergeability after disposition pass 3: **no**.


## Disposition pass 4

Checked full head `e279ee48e43cc7f3a7cce159b0fb5f65777cf86e`. The diff from the previous reviewed head contains only the bare ix_trace_rs::implements import and FR-047-AC-3 marker on Corpus::load. A focused trace query lists Corpus::load with form rust-implements-attribute for FR-047-AC-3, confirming FND-004 is fixed. The query emitted existing symbol-id-collision warnings for cfg-separated is_executable and libc_eloop declarations. No broad tests ran, and no new finding was found.

The remaining blocker is external and unchanged: spec-artifacts-process v0.38.0 at 9f3d3213b182270a67a10dbd9bbe42bd790ce9ee declares only scripts/audits/**/*.sh as shell marker entry_paths, so the real module does not bind selftest/**/run.sh. Planner holds module pins/releases. Strict diagnostics and all locally actionable findings are resolved; the review does not require more cases or zero uncovered high-risk requirements.

| FND | outcome | reason |
| --- | --- | --- |
| FND-001 | still-open | External blocker: published v0.38.0 omits selftest entry_paths and planner holds pins/releases. |

Current mergeability after disposition pass 4: **no**, solely due to the external production-module binding blocker. These scratchpad artifacts are the current review versions; the lead must copy them to the PR's reviews directory in a trailing artifact-only commit before merge.
