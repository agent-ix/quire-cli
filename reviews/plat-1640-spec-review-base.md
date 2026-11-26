---
id: SR-967175026302197768900474181935084795
title: "Base spec review — quire-cli selftest matrix criterion"
type: SpecReview
analysis: base
scope: "agent-ix/quire-cli; spec/functional/FR-047-quire-selftest.md"
review_set: subset
relationships:
  - target: "ix://agent-ix/quire-cli/FR-047"
    type: references
---

## Summary

Reviewed the changed FR-047-AC-3 against the base requirement checklist. The criterion is observable and verifiable, but the code/test evidence depends on a marker path not present in the active module configuration.

## Verdict

**CONDITIONAL** — the requirement is understandable and testable; the companion module configuration must be present for the stated binding behavior to hold.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | medium | AC evidence uses a fixture-only entry path, so it does not prove the active module binds selftest markers. | FR-047-AC-3 |

## Validation

- The changed criterion has a unique ID and Test verification method.
- Its new obligations are externally observable: strict failure and exact script-path reporting.
- The focused test covers a valid marker, removal, and an unknown criterion, but its fixture-local module rule does not cover the active module configuration.
- No other FR-047 requirement or acceptance criterion changed.
- No applicable AssuranceProfile was found under spec/.
- quire validate --scope . spec/functional/FR-047-quire-selftest.md --strict — exit 0.


## Dispositions

| FND | outcome | reason |
| --- | --- | --- |
| FND-001 | still-open | The merged companion module main supports selftest paths, but the active module ref remains v0.36.0 at 45ac59ba9d2825c0b5312865dc9e03764930e01b, where selftest paths are absent. The criterion evidence remains incomplete until the owner/planner routes an approved module ref. |

Disposition evidence: the focused integration passed against module main commit 17da2da1772e03c300c7e577f4e96c06a7a7b745; the active module ref inspection still shows v0.36.0.


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
