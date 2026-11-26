---
id: SR-967175026302197768900474181941935084793
title: "Code review — quire-cli strict selftest trace markers"
type: SpecReview
analysis: code-review
scope: "agent-ix/quire-cli; src/commands/matrix.rs, src/commands/selftest.rs, tests/cli_matrix.rs, tests/cli_selftest.rs, selftest/cases/selftest-list-cases, spec/functional/FR-047-quire-selftest.md"
review_set: subset
relationships:
  - target: "ix://agent-ix/quire-cli/FR-047"
    type: references
---

## Summary

Reviewed quire-cli PR #247 at 2e61933d49f08c821fa3783da0fcacd6d70687c7 against origin/main. The focused test and formatting checks pass, but the new marker parser rejects valid forms of the declared shell trace grammar, and the fixture test assumes a marker entry path absent from the active module.

## Verdict

**FAIL** — strict mode can reject valid shell markers, and the test's synthetic module configuration hides that actual shell markers are not bound by the active repository module.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | medium | Strict marker inventory recognizes only the literal # trace: prefix, although the declared shell trace grammar also accepts #trace: and other whitespace forms. | src/commands/matrix.rs:313-315 |
| FND-002 | high | The fixture adds a selftest entry path absent from the active module, so actual markers do not bind into the computed matrix. | tests/cli_matrix.rs:83-90 |

## Coverage

- Examined FR-047-AC-3, including its strict selftest marker and binding contract.
- Examined run, check_selftest_trace_markers, and collect_run_scripts in src/commands/matrix.rs; the embedded case inventory and trace test in src/commands/selftest.rs.
- Examined selftest_shell_marker_binds_and_removal_names_the_case_script, its synthetic marker configuration, and the changed selftest integration assertions.
- Examined the shell marker grammar in active spec-artifacts-process@v0.36.0: pattern #\\s*trace:\\s*(.+)$ and only entry path scripts/audits/**/*.sh. It does not currently include selftest paths. The new test supplies selftest/cases/**/*.sh itself, so it does not verify current module configuration.
- Focused locked-build test passed (1/1); cargo fmt --all -- --check passed; strict validation of the edited FR passed. No aggregate or long-running check was run.


## Dispositions

| FND | outcome | reason |
| --- | --- | --- |
| FND-001 | fixed | The inventory now applies the declared shell marker regex; the focused test covers #trace: and a whitespace variant. |
| FND-002 | still-open | Companion PR #172 is merged, and a narrow strict-matrix integration passes with module main 17da2da1772e03c300c7e577f4e96c06a7a7b745. The active module remains v0.36.0 at 45ac59ba9d2825c0b5312865dc9e03764930e01b, whose entry_paths omit selftest scripts. Keep this finding open until the owner/planner routes an approved module ref. |

Disposition evidence: fix commit fbdefd8c46ddc4ec9f3194d6bb675926fdf11385. Focused test passed 1/1. The module-main integration reported FR-001-AC-1 tagged by selftest/cases/example/run.sh:1:1. Active module inspection confirmed selftest paths are absent.


## Disposition pass 2

Checked PR #247 at `fbdefd8c46ddc4ec9f3194d6bb675926fdf11385` against the superseding PLAT-1640 scope. The grammar fix remains correct. The published spec-artifacts-process `v0.38.0` at `9f3d3213b182270a67a10dbd9bbe42bd790ce9ee` still declares only `scripts/audits/**/*.sh` for shell marker entry paths. Thus the earlier production-binding finding remains open. No build or broad test ran.

## New findings (disposition pass 2)

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-003 | medium | Missing-marker diagnostics discard the source repository identity, so equal relative case paths in two members are indistinguishable. | src/commands/matrix.rs:319-325,363-364 |

FR-047-AC-3 requires missing markers to be reported with their source repository identities. The checker strips each script to the member-relative path and stores no identity. A direct run of the existing PR binary over two corpus members, `agent-ix/a` and `agent-ix/b`, both containing an untagged `selftest/cases/example/run.sh`, emitted the identical relative-path diagnostic twice, without either identity. The matrix rows show repo names but do not identify these untagged scripts. Reproduction inputs and captured output are retained at `/tmp/review247-7krzb3vs/`. Include the declared repo identity or an unambiguous root-qualified script path in the diagnostic and add a focused two-member regression.

| FND | outcome | reason |
| --- | --- | --- |
| FND-002 | still-open | Published v0.38.0 still omits selftest entry paths. |
| FND-003 | still-open | The direct two-member reproduction emits two indistinguishable diagnostics; no fix exists on this head. |

## Current scope assessment

The strict marker inventory remains relevant to artifacts that already exist and does not require high-risk features to gain cases. The new standalone selftest-list-cases asset should be reconciled with the approved normal-suite subset design before merging; artifact retirement remains deferred until the replacement works. The review does not demand additional high-risk cases or zero uncovered requirements.

Current mergeability: **no**.


## Disposition pass 3

Checked `c909b6404af2b31b15de24c4e4bd4575cd68b408`. The FND-003 repository-identity defect is fixed: the retained two-member reproduction now emits each declared identity in the diagnostic subject and message. The standalone selftest-list-cases asset and its embedded metadata are removed, preserving the pre-existing local/linked set. The Linux executable is bound read-only at /tmp/quire/quire before /tmp is remounted read-only; this keeps the command available beneath the empty /home mount. The PR records three focused locked-build tests as passing (identity regression, shell grammar/removal regression, and empty-HOME run record). Independent formatting and diff checks passed. No broad or long test ran in this disposition.

The published spec-artifacts-process v0.38.0 still excludes selftest/**/run.sh from shell entry_paths. That production binding finding remains externally blocked while the planner holds module pins/releases; this pass does not claim it resolved.

## New findings (disposition pass 3)

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-004 | low | The changed corpus loader retains the repository identity used by FR-047-AC-3 diagnostics but lacks its required implementation trace marker. | src/commands/corpus_input.rs:83-101 |

The session Requirement tags policy requires a new or changed function implementing an acceptance criterion to carry the bare implements attribute. Corpus::load now retains identity on Member so the matrix diagnostic can fulfill FR-047-AC-3, but this function has no implements marker and the module has no ix_trace_rs::implements import. Add the bare marker for that criterion. The new identity test already carries the correct test trace.

| FND | outcome | reason |
| --- | --- | --- |
| FND-002 | still-open | External blocker: published v0.38.0 still omits selftest entry_paths; planner holds module pins/releases. |
| FND-003 | fixed | Member identity is preserved and emitted in the diagnostic subject and message; retained two-member reproduction passes. |
| FND-004 | still-open | Changed Corpus::load has no implementation trace marker for FR-047-AC-3. |

Current mergeability after disposition pass 3: **no**.


## Disposition pass 4

Checked full head `e279ee48e43cc7f3a7cce159b0fb5f65777cf86e`. The diff from the previous reviewed head contains only the bare ix_trace_rs::implements import and FR-047-AC-3 marker on Corpus::load. A focused trace query lists Corpus::load with form rust-implements-attribute for FR-047-AC-3, confirming FND-004 is fixed. The query emitted existing symbol-id-collision warnings for cfg-separated is_executable and libc_eloop declarations. No broad tests ran, and no new finding was found.

The remaining blocker is external and unchanged: spec-artifacts-process v0.38.0 at 9f3d3213b182270a67a10dbd9bbe42bd790ce9ee declares only scripts/audits/**/*.sh as shell marker entry_paths, so the real module does not bind selftest/**/run.sh. Planner holds module pins/releases. Strict diagnostics and all locally actionable findings are resolved; the review does not require more cases or zero uncovered high-risk requirements.

| FND | outcome | reason |
| --- | --- | --- |
| FND-002 | still-open | External blocker: published v0.38.0 omits selftest entry_paths and planner holds pins/releases. |
| FND-004 | fixed | Corpus::load has the bare implements marker and import; the trace query reports FR-047-AC-3 as rust-implements-attribute. |

Latest local outcomes: FND-001 fixed; FND-003 fixed; FND-004 fixed. FND-002 alone remains open.

Current mergeability after disposition pass 4: **no**, solely due to the external production-module binding blocker. These scratchpad artifacts are the current review versions; the lead must copy them to the PR's reviews directory in a trailing artifact-only commit before merge.
