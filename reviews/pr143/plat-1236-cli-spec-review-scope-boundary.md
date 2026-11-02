---
id: SR-14572
title: "PLAT-1236 CLI spec-review/scope-boundary review"
type: SpecReview
analysis: scope-boundary
scope: "agent-ix/quire-cli; src/commands/coverage.rs, src/commands/matrix.rs, tests/cli_coverage_strict.rs, spec/functional/FR-017-coverage-subcommand.md, spec/functional/FR-026-matrix-subcommand.md, Cargo.toml, Cargo.lock, CHANGELOG.md, npm/quire-cli/package.json; PLAT-1236; PR 143"
review_set: subset
---

## Summary

The engine owns declaration classification, locus joins and historical facts; the process module owns Withdrawn vocabulary; CLI owns rendering and exit policy. CLI does not infer retirement from prose, duplicate reconciliation, or synthesize backing. The necessary exhaustive matrix enum consumer is within the same retirement change. The concrete engine dependency is a declared staging prerequisite and remains an explicit merge blocker; no credential, Kani or unrelated pin-removal work is included.

Reviewed frozen CLI head 1de79cffd3d4d5bf0f88657bf221264f56db2f90 against f4f7e0ed791cbb94f7e8841c0a0bee47248c858a. No Cargo runs, source edits, workflow changes or merge. Inspected candidate receipt and focused logs: CLI strict target 6/6, matrix target 15/15, formatting and touched Clippy pass against temporary engine de973da; restoring the old strict condition caused the intended all-retired assertion failure, then restoration passed. Temporary manifest/lock changes were restored before the reviewed commit. These are candidate-only results; final dependency and hosted aggregate remain pending.

## Assurance Context

No AssuranceProfile under this repository spec tree applies: the type search found none. Ordinary code, Rust, gap and selected specification methods apply. No enforced review-policy run is claimed.

## Examined Units

- **FR-017-AC-7** (examined): `--strict` exits 1 when a live row is unbacked or a live status is contradicted, including beside explicitly retired rows, and exits 0 on the same repository without it. Retired records and their authored withdrawal reasons remain in the JSON report without being counted as backing evidence.
- **FR-017-AC-8** (examined): A model matching zero live and zero retired rows fails `--strict` with a diagnostic distinct from the unbacked-rows one, rather than reporting full coverage. A nonempty all-retired population passes this premise when no other strict failure applies.
- **FR-017-AC-9** (context_only): (thin boundary) reconciliation is delegated entirely to quire-rs (`compute_coverage`, `extract_tree_scoped`, `trace::bind`); the CLI resolves the two roots, applies path-safety, and renders ([StR-004](../stakeholder/StR-004-thin-boundary-over-quire-rs.md))
- **FR-017-AC-22** (context_only): A full report containing `status-column-matches-nothing` or `hollow-denominator` makes `--strict` exit 1 after emitting the structured report, including when severity projection hides findings; the same scope without strict remains report-only, and a repaired control succeeds.
- **FR-026-AC-7** (examined): `--format json` emits `{"coverage_matrix": <value>, "engine": {cli, engine, capabilities}}` on stdout; the `coverage_matrix` key carries the engine's own value unmodified — a bare array of requirement entries, never re-wrapped in an object — and is absent when the engine omits it (zero population), never present-but-empty; two runs over identical inputs are byte-identical
- **FR-026-AC-8** (examined): `--format tsv` emits header `document\tcriterion\tstatus\tbinders\tstatement`, one untruncated, unescaped record per criterion in the JSON/markdown order, `binders` comma-separated as `path:line:column` with the same `(ignored)` marker and empty (not `(none)`) at zero; a zero-criteria report (§B) emits the header alone
- **FR-026-AC-10** (examined): `--strict`'s pass condition requires **at least one** criterion, all `tagged`/`retired`/`method-without-symbol`: it exits 1 on the zero-population state (§B) exactly as it exits 1 on an `untagged`/`tagged-by-ignored-test` criterion — a report measuring nothing must not exit 0 (FR-050-AC-14/CR-035 argument, applied to the derived-obligation population)
- **FR-026-AC-12** (examined): `matrix` does not accept `--severity` (an attempt is an ordinary clap argv error, exit 3, FR-007-AC-5) and runs none of the `unbacked-row`/`status-lie`/`untracked-symbol`/`undeclared-status` checks; a module's `grammar_severity` promotion of any of those checks has no effect on `matrix`'s exit code, because `coverage_matrix` carries none of their record kinds. `matrix` also does not inherit `coverage`'s FR-017-AC-22 unread-measurement gate (`status-column-matches-nothing`/`hollow-denominator`): `matrix` consumes the engine's computed status, including locus-scoped retirement metadata, without 

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | low | No findings (placeholder) | - |

## Coverage

Scoped source and spec review; no workflow diff. Plan completion: not assessed. Dependency and final hosted gate remain pending. Failure-domain, integrity and boundary cases relevant to the changed strict checks were inspected; no independent whole-repository semantic audit or unrelated dependency/risk analysis was performed.

## Verdict

PASS for this method within the examined scope; this does not clear the dependency and final-gate merge hold.
