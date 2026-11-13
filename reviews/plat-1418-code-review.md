---
id: SR-20261113155732
title: "PLAT-1418 quire-cli PR196 code-review (rust lane)"
type: SpecReview
analysis: code-review
scope: "agent-ix/quire-cli PR196 head d9788ee; CHANGELOG.md, Cargo.lock, src/commands/matrix.rs, src/commands/relations.rs, tests/cli_relations_design.rs, tests/snapshots/relations-design-worked-example.{txt,json}"
review_set: subset
---

## Summary

Ticket: PLAT-1418 (CLI half). Code review with the Rust lane (rust-style loaded) of `quire relations --design` (quire-cli FR-041) over quire-rs `design_join` (FR-086), scoped to `git diff origin/main...HEAD` (merge base = main c24c393, the #184 squash). Focused gates: `cargo fmt --check` clean; `cargo clippy --all-targets --all-features -D warnings` clean; `cargo test --test cli_relations --test cli_relations_design --test cli_help` 26 passed, 0 failed. Aggregate gate not run (by brief). Mutation spot-check over `src/commands/relations.rs`: see Verdict.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | low | The Cargo.lock bump of quire-rs (9baf155 0.59.1 to 5a45ec9 0.60.0) is not needed by FR-041: `design_join`, `LinkedEntry::Resolved.realized_criteria` and `KindMismatchEntry.archetype: Option` already exist at main's pin, and `src/corpus/relation_report.rs` is unchanged between the two revisions. The bump pulls in unrelated engine changes (carried_fields, design_matrix, when_any, criticality carry) and forces the `carried: None` edit in the matrix.rs unit test, which overlaps PR #195 (it rewrites the same helper and pins quire-rs b502d6c). The PR body's "absorbs quire-rs changes" claim is inaccurate. Drop the Cargo.lock and matrix.rs hunks, or keep them and let whichever of #195/#196 lands second resolve the lock. | Cargo.lock:1674-1721, src/commands/matrix.rs:314 |
| FND-002 | low | Stale docs: the module doc of relations.rs still says the command calls `quire_rs::relation_report` and that rows come from `traceability.required_relations`, and the `--module` help says it supplies `required_relations`; with `--design` the command calls `design_join` over `design_relations`. | src/commands/relations.rs:1-7, src/commands/relations.rs:35-39 |

## Verdict

CONDITIONAL (two low findings, no wrong results). The CLI stays a thin boundary (StR-004, FR-041-AC-11): `--design` swaps `relation_report` for `design_join`, `--json` encodes the engine's `RelationReport` unchanged, and the human renderer only adds the `id (kind) realizes <ids>` element form and the `no design element <check>` gap word. No archetype, verb or relation name in the CLI diff. Exit codes follow FR-007: clap conflicts (`--relation`, `--strict`), missing `design_relations` (CLI pre-check, before any coverage is computed) and the engine's `UnknownDesignRelation` all exit 3 with empty stdout. Determinism: output order is the engine's; two runs byte-identical (tested). Merge against #184's squash: the tree equals main plus the seven listed files; `validate.rs`, `corpus_input.rs`, `tests/cli_relations.rs` are identical to main, so nothing of #184 is lost or duplicated. CHANGELOG entry is accurate. No compat layer, digest, pin or SHA added. No unsafe, panic path or unbounded input. Mutations of the gap word, the `realizes` suffix, the design_relations pre-check, the `--strict` conflict, `design_join` to `relation_report`, the kind label and the design flag to the renderer are each killed by the new tests (see the gap-analysis SR for the surviving ones).
