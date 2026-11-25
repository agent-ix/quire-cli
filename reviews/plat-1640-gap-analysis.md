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

