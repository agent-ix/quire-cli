---
id: SR-20261113155733
title: "PLAT-1418 quire-cli PR196 gap-analysis"
type: SpecReview
analysis: gap-analysis
scope: "agent-ix/quire-cli PR196 head d9788ee; spec/functional/FR-041-relations-design-mode.md, spec/functional/FR-030-relations-subcommand.md (AC-25), src/commands/relations.rs, tests/cli_relations_design.rs"
review_set: subset
---

## Summary

Ticket: PLAT-1418 (CLI half). Planless gap analysis of FR-041 against the PR's tests and code. `quire matrix --scope . --format json` (quire 0.44.0, engine 0.59.1) shows FR-041-AC-1..10 (Test) all `tagged`; FR-041-AC-11 (Inspection) is `method-without-symbol` and was checked by inspection: relations.rs holds no join, edge-matching or row logic, it calls `quire_rs::design_join`. Plan completion: not assessed. Each tagged test was read against its AC and spot-checked by mutating relations.rs. Reverse gap: none; every added behaviour traces to FR-041.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | medium | FR-041-AC-4 human element form has an untested branch: an element linked to the requirement as a whole (`realized_criteria: []`) must print `id (kind)` with no `realizes` suffix, but the only fixture element (BEA-001) links both to SIG-001 and to SIG-001-AC-1, so its list is never empty. Dropping the `.filter(\|c\| !c.is_empty())` in `design_element` prints a bare trailing `realizes ` and no test fails (mutation run: 8 passed, 0 failed). Add a whole-requirement-only element to the fixture and assert its line. | src/commands/relations.rs:222-228, tests/cli_relations_design.rs:96-112 |
| FND-002 | low | FR-041-AC-10 requires the complete join in human and `--json` output over invented names; the AC-10-tagged test asserts JSON only. The human side is asserted by `worked_example_matches_its_snapshots` and `human_output_prints_elements_gaps_and_a_summary_equal_to_the_json`, which are not tagged AC-10. Tag one of them FR-041-AC-10. | tests/cli_relations_design.rs:178-202, tests/cli_relations_design.rs:254-266 |
| FND-003 | low | No `--design` test covers FR-030-AC-25 first-member module discovery: both `design_repos` invocations pass `--module`. The code path is shared and not branched on `--design` (`corpus.members[0].root` to `load_registry_for`), and FR-041 states `--design` combines with `--repo`/`--module` exactly as FR-030; no FR-041 AC demands it, so the risk is a future divergence going unnoticed. | src/commands/relations.rs:75-76, tests/cli_relations_design.rs:125-137 |

## Verdict

CONDITIONAL: one medium (untested branch of AC-4), two low. AC-1/2 (join only, JSON-only stdout, byte-identical reruns), AC-3 (relation outside `design_relations` absent in JSON and human), AC-4/5 (element form, gap word with check token, summary equal to JSON), AC-6 (sibling element `o/b`, dangling inverse without `o/b`), AC-7/8 (exit 3 naming `design_relations` / the entry, empty stdout), AC-9 (both conflicts, naming both flags) each have a test that fails when the behaviour is removed: mutations of the gap word, the `realizes` suffix, the design_relations pre-check, the `--strict` conflict, `design_join` to `relation_report`, the kind label and the renderer's design flag are each killed. The `--design` side of the `--relation` conflict is declared twice (on both args), so removing one survives; harmless.
