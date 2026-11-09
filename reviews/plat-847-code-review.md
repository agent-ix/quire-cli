---
id: SR-20261102086810
title: "PLAT-847 quire-cli PR160 code-review"
type: SpecReview
analysis: code-review
scope: "agent-ix/quire-cli PR160 head b05771b; Cargo.toml, Cargo.lock, src/commands/{coverage,matrix,trace,validate}.rs, src/engine.rs, tests/*.rs, tests/fixtures/assurance/v1.json, tests/fixtures/reference-status-column/manifest.yaml, tests/snapshots/extract-envelope.json, CHANGELOG.md"
review_set: subset
---

## Summary

Ticket: PLAT-847. Code review with the rust-review lane folded in, scoped to `git diff origin/main...HEAD`. The PR moves quire-cli to quire-rs main at 0.54.0, where only explicit markers bind (CR-215). Commit 1 adapts the code: it drops `self_named`, `provenance` and the `binding_census.self_named` capability, and propagates the new `Result` from `trace_search::search`. Commit 2 adds `#[trace(...)]` markers to 197 tests.

The adaptation is mechanical and correct, with no compatibility shim. Of 340 distinct ids in 199 added markers, every one is a row key minted under `spec/`. 185 of 185 test-id/AC pairs agree with or extend their `spec/tests.md` row. One retag binds a `validate` test to an assurance-export gate row. 48 `Trace:` comments that no longer bind were left above the new markers.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | medium | `#[trace("IT-136")]` binds the validate JSON-diagnostics test to IT-136. IT-136 is the FR-020 assurance complete-export row and is on the quire-qualify `make spec` required list. That puts two symbols on one gate row: deleting the real assurance test would leave IT-136 green. The retag copied a pre-existing id collision from prose into a binding. | tests/cli_validate.rs:61 |
| FND-002 | medium | 48 legacy `// Trace:` / `/// Trace:` comments remain directly above the new markers, and an obsolete note says `rust-comment-id` binds. After CR-215 these bind nothing, yet they read as trace declarations and have already drifted: the cli_validate.rs:589 comment omits FR-004-AC-19, which the marker below it carries. | tests/cli_validate.rs:589, src/commands/matrix.rs:318, tests/cli_matrix.rs:173, tests/cli_errors.rs:101 |
| FND-003 | low | The new `?` propagation of `trace_search::search` errors (ExtractionUnavailable) has no test, and the error leaves as the generic `QuireError` kind. The CLI cannot reach it in a default-feature build, but a unit test can populate the public `SymbolExtraction.unavailable_languages` and call `run_query`. CHANGELOG.md:12 advertises the refusal. | src/commands/trace.rs:359 |

## Examined scope

```yaml
scope:
- id: Cargo.toml
  path: Cargo.toml
  role: examined
  excerpt: "quire-rs = { git = \"https://github.com/agent-ix/quire-rs\", branch = \"main\" }"
- id: src/commands/trace.rs
  path: src/commands/trace.rs
  role: examined
  excerpt: "let result = trace_search::search(graph, extraction, &Query::Id(id.clone()))?;"
- id: src/engine.rs
  path: src/engine.rs
  role: examined
  excerpt: "binding_census.self_named capability, witness and test assertion removed"
- id: src/commands/coverage.rs
  path: src/commands/coverage.rs
  role: examined
  excerpt: "BindingCensus literals drop self_named, self_named_bound, self_named_unbound_example"
- id: src/commands/matrix.rs
  path: src/commands/matrix.rs
  role: examined
  excerpt: "write it in the module's declared marker form"
- id: IT-136
  path: spec/tests.md
  role: examined
  excerpt: "`quire assurance` over the pinned complete fixture emits the upstream `quire-assurance` v1 envelope with artifacts, obligations, symbols, relation kinds, ..."
- id: tests/cli_matrix.rs
  path: tests/cli_matrix.rs
  role: examined
  excerpt: "matrix_explains_ranges_and_partially_read_trace_lists_in_every_format rewritten to marker sources"
```

## Verdict

Changes requested: FND-001 and FND-002 should be fixed in-PR, and FND-003 is low.

Rust lane: clean. `run_query` and `search_by_prefix` now return `anyhow::Result`, and `?` is applied at all four call sites, so no error is swallowed or unwrapped. The removed census fields and the capability are gone from code, from the snapshot `tests/snapshots/extract-envelope.json`, from the fixture `tests/fixtures/assurance/v1.json`, and from the tests. Nothing in `spec/`, README or `skills/` still references them. The diff adds no compatibility shim and no legacy reader. The fixture's `legacy:` block is deleted.

Dependency: `branch = "main"` is appropriate. `Cargo.lock` records 7602ea4, and every build, lint, test, deny and release path runs `--locked`. `ix-cli-kit` already follows the same convention, and `deny.toml` allow-lists the git source. The old `rev` plus `=0.53.0` pin duplicated the lockfile and guarded nothing.

Commit 2 was not test-run by the author, but a static check is clean: every file that uses `#[trace]` imports it, and every import is used. The single pre-merge `make ci` (including `make spec`) on b05771b must still run.

## Dispositions

| FND | outcome | reason |
| --- | --- | --- |
| FND-001 | fixed | 445d7eb removes `#[trace("IT-136")]` from cli_validate::it_136_json_validation_error_is_locatable_and_actionable; IT-136 is now bound only by the assurance tests |
| FND-002 | still-open | 445d7eb deletes the drifted comments and the obsolete `rust-comment-id` note, but two dead `Trace:` comments remain above markers: tests/audit_static.rs:20-21 and tests/cli_stream_contract.rs:269 |
| FND-003 | fixed | 445d7eb adds trace::tests::run_query_refuses_when_a_language_in_the_tree_is_unreadable, covering --id, --symbol and --file and downcasting to ExtractionUnavailable. --prefix stays uncovered because an empty graph never calls search |
