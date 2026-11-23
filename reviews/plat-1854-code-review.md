---
id: SR-20261116045013
title: "PLAT-1854 quire-cli PR230 code-review"
type: SpecReview
analysis: code-review
scope: "agent-ix/quire-cli PR230 head e4ede7f; Cargo.toml, Cargo.lock, src/commands/mod.rs, tests/cli_clauses.rs, tests/cli_declaration_drop.rs"
review_set: subset
---

## Summary

Ticket: PLAT-1854. Code review with the rust-review lane folded in, scoped to `git diff origin/main...HEAD`. Behaviour owner: quire-rs FR-058-AC-47. A declaration that cannot be loaded drops only itself. Every consumer that computes over the merged model must exit Invalid (3) and name each dropped declaration on stderr. That includes the case where the whole `traceability:` section is unreadable.

The PR makes `require_traceability_model` call `Registry::require_complete_model()` before the "no model declared" refusal. It moves quire-rs to `rev = b6beef4`, which is the #715 merge commit and the current quire-rs `main` (compare status: identical). Two `ExpectedOutput` fixtures gain `producers: None`. A new CLI test file covers `coverage` and `trace`.

The core change is correct. `coverage`, `trace` and `matrix` all reach `require_traceability_model`: `matrix` goes through `compute_report_over`. All three exit 3 and name `module 'm' flow_rule 'bad'` or `module 'm' traceability_section 'traceability'`, including when the whole section is dropped. I probed `matrix` against the built binary. Reverting `src/commands/mod.rs` to origin/main makes all four new tests fail with the "no module in scope declares a `traceability:` model" message, so the tests prove the change through the real CLI.

The focused suites pass at this head: cli_declaration_drop 4/4, cli_clauses 6/6, cli_coverage 28/28, cli_trace 25/25, cli_matrix 23/23, cli_relations 17/17, cli_symbols 6/6 and cli_assurance 7/7. The quire-rs bump breaks nothing in these suites.

The `rev` dependency has the right shape. This repo has used `rev =` for quire-rs before (c7a256f, 727f830), no tag contains #715 yet, and `Cargo.lock` records the commit. No further pin or ledger is needed. The other Cargo.lock changes come from quire-rs's own manifest: `semantic-schema` moves from `branch=main` to the tag at the same commit, and `quire-code-parse` goes to v0.2.3.

The PR still has defects. Its trace tags name an id this corpus does not define, which turns the repo's own `make spec` traceability gate red. Two other consumers of the merged model still compute without naming the drop.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | high | The four new tests carry `#[trace("FR-058-AC-47")]`, a quire-rs id with no criterion in quire-cli's corpus. `quire coverage --scope . --json` at this head reports exactly these 4 `untracked_symbols`, and `quire-qualify assurance-traceability` (the `make spec` step in `make ci`) fails with `UntrackedSymbols { count: 4 }`. The `//!` header that names the id also raises a coverage diagnostic. Untag the tests, or bind them to a local AC once one exists. | tests/cli_declaration_drop.rs:2, tests/cli_declaration_drop.rs:72-113, Makefile:103 |
| FND-002 | medium | `relations` tests `registry.traceability()` list emptiness before any computation and never calls `require_complete_model()`. When a drop empties the needed list or the whole section, stderr does not name the drop. The default and `--design` modes print "no loaded module declares `traceability.required_relations`/`design_relations`". `--flow` prints only `registry.failures().first()`. I reproduced this with both fixtures: exit 3, drop not named. Call `super::require_traceability_model` (or `require_complete_model`) before the prechecks. | src/commands/relations.rs:124-165 |
| FND-003 | medium | `symbols --module` binds trace ids over `registry.traceability()` and its `source_exclude` without `require_complete_model()`. With the whole section dropped it exits 0, falls back to an unbound walk, and its human output says "no --module" although `--module` was passed. A dropped `source_exclude` or `trace_tags` entry would widen or change the walk silently. | src/commands/symbols.rs:94-125 |
| FND-004 | low | `assurance` refuses with exit 3 through the engine's `validate_registry`, which reports only `registry.failures().first()`. With several dropped declarations only one is named. The CLI could call `require_complete_model()` first, as the PR does for coverage/trace. | src/commands/assurance.rs:94-103 |
| FND-005 | low | The new test file covers `coverage` and `trace` only. `matrix` behaves correctly by probe but has no test, so a later split of `compute_report_over` could lose the refusal unnoticed. `relations` and `symbols` also have no tests (see FND-002/FND-003). | tests/cli_declaration_drop.rs |

## Examined scope

```yaml
scope:
- id: src/commands/mod.rs
  path: src/commands/mod.rs
  role: examined
  excerpt: "registry.require_complete_model().map_err(|dropped| invalid_error!(\"{dropped}\"))?;"
- id: Cargo.toml
  path: Cargo.toml
  role: examined
  excerpt: "quire-rs = { git = \"https://github.com/agent-ix/quire-rs\", rev = \"b6beef4df8326fdd4323dfeb803075563ccdc350\" }"
- id: Cargo.lock
  path: Cargo.lock
  role: examined
  excerpt: "quire-rs 0.62.3 rev=b6beef4; quire-code-parse 0.2.3 tag v0.2.3; semantic-schema tag semantic-schema-v0.6.0"
- id: tests/cli_clauses.rs
  path: tests/cli_clauses.rs
  role: examined
  excerpt: "producers: None,"
- id: tests/cli_declaration_drop.rs
  path: tests/cli_declaration_drop.rs
  role: examined
  excerpt: "coverage/trace refuse with exit 3 naming flow_rule 'bad' and traceability_section"
- id: src/commands/coverage.rs
  path: src/commands/coverage.rs
  role: context_only
  excerpt: "compute_report_over -> super::require_traceability_model(registry)?"
- id: src/commands/matrix.rs
  path: src/commands/matrix.rs
  role: context_only
  excerpt: "compute_matrix -> compute_report_over"
- id: src/commands/relations.rs
  path: src/commands/relations.rs
  role: context_only
  excerpt: "registry.traceability().is_none_or(|model| model.required_relations.is_empty())"
- id: src/commands/symbols.rs
  path: src/commands/symbols.rs
  role: context_only
  excerpt: "let model = registry.as_ref().and_then(|r| r.traceability());"
- id: src/commands/assurance.rs
  path: src/commands/assurance.rs
  role: context_only
  excerpt: "let model = registry.traceability();"
```

## Verdict

Not mergeable as is. FND-001 turns the repo's `make spec` gate red. Fix FND-001 in this PR. FND-002 and FND-003 are real misses of FR-058-AC-47's "every consumer" clause, and each is a few lines plus a test, so they belong in this PR too. The rust-review lane found no idiom, safety or error-handling defects. The change reuses the repo's `invalid_error!` mapping, adds no shim, and the tests drive the real binary with an exact exit code and a stderr assertion.

## New findings (disposition pass 1)

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-006 | low | The `//!` header of the new test file still names `FR-058-AC-47`. `quire coverage --scope . --json` reports this as an advisory diagnostic: "`FR-058-AC-47` is written in a comment or docstring on `common` … a container binds no trace id". It does not gate, and `assurance-traceability` passes. The edited header line also runs past the line width of the lines around it and reads "refuses … and name each". To clear the diagnostic, refer to the upstream behaviour without the AC id, for example "quire-rs PLAT-1804". | tests/cli_declaration_drop.rs:1-5 |

## Dispositions

| FND | Outcome | Reason |
| --- | --- | --- |
| FND-001 | fixed | 7445147 removes every `#[trace]` tag from tests/cli_declaration_drop.rs. At this head, `quire coverage --scope . --json` reports 0 untracked_symbols and `quire-qualify assurance-traceability` prints "ok: 21/21 required targets backed" (exit 0). The residual comment diagnostic is FND-006. |
| FND-002 | fixed | `load_registry_for` now calls `require_complete_model` before returning, so `relations` refuses before its emptiness prechecks. The relations, relations --design and relations --flow tests assert exit 3 and name both dropped rules and the whole section. They fail when that call is removed. |
| FND-003 | fixed | `symbols --module` goes through `load_registry_for` and now refuses. Its two tests fail when the call is removed. |
| FND-004 | fixed | `assurance` calls `super::require_complete_model(&registry)?` right after loading, and its test asserts that both `flow_rule 'bad'` and `flow_rule 'worse'` are named. |
| FND-005 | fixed | 16 real-CLI tests now cover coverage, trace, matrix, relations (default, --design, --flow), symbols and assurance, each with two dropped rules and with a whole dropped section. |
