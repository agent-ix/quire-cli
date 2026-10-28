---
id: SR-14563
title: "Gap analysis — quire-cli repository after QSL-782 engine pin"
type: SpecReview
analysis: gap-analysis
scope: "spec/, src/, tests/, tools/, Cargo.toml, Cargo.lock, npm/quire-cli/package.json; planless repository audit"
review_set: subset
relationships:
  - { target: "ix://agent-ix/quire-cli/spec", type: references }
---

## Summary

Audited the quire-cli requirements, computed matrix, trace tags, coverage diagnostics, and source/test surface. The release-pin diff adds no source behavior, but the repository-wide audit remains below the assurance bar because 51 criteria are untagged and the coverage model reports selector and binding diagnostics.

## Verdict

**FAIL** — the computed matrix has 51 untagged criteria; coverage also reports unread target sections and a hollow Rust self-named-binding denominator. These are pre-existing repository gaps and are not introduced by PR #138.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | high | FR-001-AC-1 has no test binder in the computed matrix (pre-existing; unchanged by PR #138). | FR-001-AC-1 |
| FND-002 | high | FR-001-AC-2 has no test binder in the computed matrix (pre-existing; unchanged by PR #138). | FR-001-AC-2 |
| FND-003 | high | FR-001-AC-3 has no test binder in the computed matrix (pre-existing; unchanged by PR #138). | FR-001-AC-3 |
| FND-004 | high | FR-001-AC-4 has no test binder in the computed matrix (pre-existing; unchanged by PR #138). | FR-001-AC-4 |
| FND-005 | high | FR-001-AC-5 has no test binder in the computed matrix (pre-existing; unchanged by PR #138). | FR-001-AC-5 |
| FND-006 | high | FR-001-AC-6 has no test binder in the computed matrix (pre-existing; unchanged by PR #138). | FR-001-AC-6 |
| FND-007 | high | FR-003-AC-3 has no test binder in the computed matrix (pre-existing; unchanged by PR #138). | FR-003-AC-3 |
| FND-008 | high | FR-004-AC-23 has no test binder in the computed matrix (pre-existing; unchanged by PR #138). | FR-004-AC-23 |
| FND-009 | high | FR-006-AC-4 has no test binder in the computed matrix (pre-existing; unchanged by PR #138). | FR-006-AC-4 |
| FND-010 | high | FR-006-AC-5 has no test binder in the computed matrix (pre-existing; unchanged by PR #138). | FR-006-AC-5 |
| FND-011 | high | FR-016-AC-3 has no test binder in the computed matrix (pre-existing; unchanged by PR #138). | FR-016-AC-3 |
| FND-012 | high | FR-016-AC-4 has no test binder in the computed matrix (pre-existing; unchanged by PR #138). | FR-016-AC-4 |
| FND-013 | high | FR-016-AC-7 has no test binder in the computed matrix (pre-existing; unchanged by PR #138). | FR-016-AC-7 |
| FND-014 | high | FR-017-AC-23 has no test binder in the computed matrix (pre-existing; unchanged by PR #138). | FR-017-AC-23 |
| FND-015 | high | FR-018-AC-10 has no test binder in the computed matrix (pre-existing; unchanged by PR #138). | FR-018-AC-10 |
| FND-016 | high | FR-019-AC-1 has no test binder in the computed matrix (pre-existing; unchanged by PR #138). | FR-019-AC-1 |
| FND-017 | high | FR-019-AC-2 has no test binder in the computed matrix (pre-existing; unchanged by PR #138). | FR-019-AC-2 |
| FND-018 | high | FR-019-AC-3 has no test binder in the computed matrix (pre-existing; unchanged by PR #138). | FR-019-AC-3 |
| FND-019 | high | FR-019-AC-4 has no test binder in the computed matrix (pre-existing; unchanged by PR #138). | FR-019-AC-4 |
| FND-020 | high | FR-019-AC-5 has no test binder in the computed matrix (pre-existing; unchanged by PR #138). | FR-019-AC-5 |
| FND-021 | high | FR-019-CON-1 has no test binder in the computed matrix (pre-existing; unchanged by PR #138). | FR-019-CON-1 |
| FND-022 | high | FR-020-CON-2 has no test binder in the computed matrix (pre-existing; unchanged by PR #138). | FR-020-CON-2 |
| FND-023 | high | FR-020-CON-4 has no test binder in the computed matrix (pre-existing; unchanged by PR #138). | FR-020-CON-4 |
| FND-024 | high | FR-021-AC-2 has no test binder in the computed matrix (pre-existing; unchanged by PR #138). | FR-021-AC-2 |
| FND-025 | high | FR-021-AC-3 has no test binder in the computed matrix (pre-existing; unchanged by PR #138). | FR-021-AC-3 |
| FND-026 | high | FR-021-AC-5 has no test binder in the computed matrix (pre-existing; unchanged by PR #138). | FR-021-AC-5 |
| FND-027 | high | FR-021-AC-6 has no test binder in the computed matrix (pre-existing; unchanged by PR #138). | FR-021-AC-6 |
| FND-028 | high | FR-027-AC-8 has no test binder in the computed matrix (pre-existing; unchanged by PR #138). | FR-027-AC-8 |
| FND-029 | high | FR-027-AC-10 has no test binder in the computed matrix (pre-existing; unchanged by PR #138). | FR-027-AC-10 |
| FND-030 | high | NFR-001-AC-2 has no test binder in the computed matrix (pre-existing; unchanged by PR #138). | NFR-001-AC-2 |
| FND-031 | high | NFR-001-M-1 has no test binder in the computed matrix (pre-existing; unchanged by PR #138). | NFR-001-M-1 |
| FND-032 | high | NFR-001-M-2 has no test binder in the computed matrix (pre-existing; unchanged by PR #138). | NFR-001-M-2 |
| FND-033 | high | NFR-001-M-3 has no test binder in the computed matrix (pre-existing; unchanged by PR #138). | NFR-001-M-3 |
| FND-034 | high | NFR-002-AC-2 has no test binder in the computed matrix (pre-existing; unchanged by PR #138). | NFR-002-AC-2 |
| FND-035 | high | NFR-002-M-1 has no test binder in the computed matrix (pre-existing; unchanged by PR #138). | NFR-002-M-1 |
| FND-036 | high | NFR-002-M-2 has no test binder in the computed matrix (pre-existing; unchanged by PR #138). | NFR-002-M-2 |
| FND-037 | high | NFR-003-M-1 has no test binder in the computed matrix (pre-existing; unchanged by PR #138). | NFR-003-M-1 |
| FND-038 | high | NFR-004-M-1 has no test binder in the computed matrix (pre-existing; unchanged by PR #138). | NFR-004-M-1 |
| FND-039 | high | NFR-004-M-2 has no test binder in the computed matrix (pre-existing; unchanged by PR #138). | NFR-004-M-2 |
| FND-040 | high | NFR-005-M-1 has no test binder in the computed matrix (pre-existing; unchanged by PR #138). | NFR-005-M-1 |
| FND-041 | high | NFR-005-M-2 has no test binder in the computed matrix (pre-existing; unchanged by PR #138). | NFR-005-M-2 |
| FND-042 | high | NFR-006-M-1 has no test binder in the computed matrix (pre-existing; unchanged by PR #138). | NFR-006-M-1 |
| FND-043 | high | NFR-006-M-2 has no test binder in the computed matrix (pre-existing; unchanged by PR #138). | NFR-006-M-2 |
| FND-044 | high | NFR-008-M-1 has no test binder in the computed matrix (pre-existing; unchanged by PR #138). | NFR-008-M-1 |
| FND-045 | high | NFR-008-M-2 has no test binder in the computed matrix (pre-existing; unchanged by PR #138). | NFR-008-M-2 |
| FND-046 | high | NFR-008-M-3 has no test binder in the computed matrix (pre-existing; unchanged by PR #138). | NFR-008-M-3 |
| FND-047 | high | NFR-008-M-4 has no test binder in the computed matrix (pre-existing; unchanged by PR #138). | NFR-008-M-4 |
| FND-048 | high | NFR-009-M-1 has no test binder in the computed matrix (pre-existing; unchanged by PR #138). | NFR-009-M-1 |
| FND-049 | high | NFR-009-M-2 has no test binder in the computed matrix (pre-existing; unchanged by PR #138). | NFR-009-M-2 |
| FND-050 | high | NFR-009-M-3 has no test binder in the computed matrix (pre-existing; unchanged by PR #138). | NFR-009-M-3 |
| FND-051 | high | NFR-009-M-4 has no test binder in the computed matrix (pre-existing; unchanged by PR #138). | NFR-009-M-4 |
| FND-052 | medium | Trace declarations select no table for FR-021 Constraints and no Acceptance Criteria section in NFR-009, leaving those declared targets unread. | spec/functional/FR-021-clauses-subcommand.md; spec/non-functional/NFR-009-native-qualification-boundary.md |
| FND-053 | medium | The Rust self-named binding denominator includes 84 evidence symbols but reads none, so this published ratio is hollow. | tests/assurance_cross_language.rs:14; coverage.self_named_binding.rust |
| FND-054 | medium | Five production functions carry trace ids as evidence tags, but production functions do not bind evidence ids. | src/commands/coverage.rs:215,520; src/commands/mod.rs:136; src/commands/validate.rs:138 |
| FND-055 | medium | 31 verification labels are not declared classes or catalog method ids, so their evidence methods cannot be classified by the active module. | spec/functional/FR-023-rust-npm-package-assembly.md; spec/non-functional/NFR-004-no-network.md; spec/non-functional/NFR-009-native-qualification-boundary.md |

## Coverage

- Reconciliation: `quoin matrix --repo <repo> --module spec-artifacts-process@8bbd969 --json` (Quoin 0.29.1); no run-evidence records were present.
- Plan completion: not assessed
- Criteria: tagged 199 · untagged 51 · tagged-by-ignored-test 0 · method-without-symbol 21 (271 total)
- Evidence: no run evidence for all 271 criteria
- Coverage diagnostics: 40 total; 1 `section-holds-no-table`, 1 `section-matches-nothing`, 1 `marker-form-mismatch` affecting 84 symbols, 5 `tag-on-non-binding-symbol`, 31 `uncatalogued-verification-method`, and 1 `hollow-denominator`; 0 untracked symbols.
- Reverse-gap inventory: 18 CLI commands and the `quire-dist` / `quire-qualify` internal tools map to existing FR/NFR requirements in the current source inventory; this diff adds no behavior. Untraced behaviors found: 0; source stubs: 0; test stubs: 0. The one ignored network test has an explicit real-npm reason; the TODO hit is fixture text exercised by tests, not a placeholder test.
- Semantic review: skipped (not requested).
