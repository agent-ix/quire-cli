---
id: SR-967175026302197768900474181941935084883
title: "Gap analysis — PLAT-1890 quire impact command"
type: Review
analysis: gap-analysis
scope: "agent-ix/quire-cli; spec/functional/FR-049-impact-subcommand.md, src/commands/impact.rs, src/commands/mod.rs, src/main.rs, tests"
review_set: subset
relationships:
  - { target: "ix://agent-ix/quire-cli", type: references }
---

## Summary

Repository-driven, planless matrix and reverse-gap checks were run for quire-cli; plan completion was not assessed. In the FR-049 command scope, four test-required criteria have no trace-tagged test, while the engine-backed matrix also reports pre-existing repository-wide gaps outside this PR.

## Verdict

**FAIL** — FR-049-AC-2, AC-4, AC-7, and CON-4 have no test bound to their criterion ids in the computed matrix.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | high | FR-049-AC-2 has no test tagged to verify the human and JSON report fields. | FR-049-AC-2 |
| FND-002 | high | FR-049-AC-4 has no test tagged to verify unsupported-language and extraction-status reporting. | FR-049-AC-4 |
| FND-003 | high | FR-049-AC-7 has no test tagged to verify byte-identical repeated JSON output. | FR-049-AC-7 |
| FND-004 | high | FR-049-CON-4 has no test tagged to verify no network access or writes into the analyzed repository. | FR-049-CON-4 |

## Coverage

- Reconciliation: `quoin matrix --repo . --json` (quoin 0.36.10; quire 0.52.1, engine 0.62.2).
- Plan completion: not assessed
- Repository criteria: tagged 445; untagged 51; tagged-by-ignored-test 15; method-without-symbol 45; total 556.
- Repository evidence: all 556 criteria report no run evidence.
- FR-049 statuses: AC-1 tagged; AC-2 untagged; AC-3 tagged; AC-4 untagged; AC-5 and AC-6 untagged (assigned to PLAT-1901 and PLAT-1902); AC-7 untagged; AC-8 and CON-1..3 method-without-symbol; CON-4 untagged.
- Reverse-gap inspection: the new `impact` command, `--caller-depth` flag, and snapshot processing are owned by FR-049; no unowned user-visible behavior or stub was found in the changed code. Matrix coverage was whole-repository; review findings above are limited to the changed requirement scope.
- Untraced behaviors / stubs: 0 in changed source; 0 in the four new unit tests.
- Semantic review: skipped.

## Dispositions

| FND | outcome | reason |
| --- | --- | --- |
| FND-001 | fixed | `it_impact_json_and_human_reports_required_fields_filters_and_reasons` now traces AC-2 and asserts required JSON fields, filter/reason, and human output. |
| FND-002 | fixed | `it_impact_reports_unsupported_source_and_incomplete_status` now traces AC-4 and asserts the unsupported source and incomplete status in both formats. |
| FND-003 | fixed | `it_impact_json_is_byte_identical_for_identical_inputs` now traces AC-7 and compares two CLI JSON byte streams. |
| FND-004 | fixed | `it_impact_does_not_modify_the_analyzed_repository` now traces CON-4, restricts Git to local file protocol, and verifies HEAD and worktree status remain unchanged. |
