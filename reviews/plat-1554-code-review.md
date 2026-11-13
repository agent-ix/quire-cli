---
id: SR-20261113045746
title: "PLAT-1554 quire-cli PR184 code-review (rust lane)"
type: SpecReview
analysis: code-review
scope: "agent-ix/quire-cli PR184 head b429ebb; src/commands/corpus_input.rs, src/commands/relations.rs, src/commands/validate.rs, src/commands/mod.rs, src/main.rs, tests/cli_relations.rs, tests/snapshots/relations-worked-example.{txt,json}, tests/snapshots/help.txt, CHANGELOG.md"
review_set: subset
---

## Summary

Ticket: PLAT-1554. Code review with the Rust lane (rust-style loaded) of `quire relations` and `validate --okf --repo` (FR-030), scoped to `git diff origin/main...HEAD`. Built against a local path patch of quire-rs main+#666 (8b45bcb). Focused gates: `cargo fmt --check` clean; `cargo clippy --all-targets --all-features -D warnings` clean; `cargo test --test cli_relations --test cli_help` 15 passed, 0 failed. Aggregate gate not run (by brief). Cargo.lock refresh for #666 is out of scope.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | low | The human header renders `RelationDirection` with `format!("{:?}", ..).to_lowercase()`. Debug output is not a rendering contract: a renamed or restructured variant changes the human output with no compile error, while the sibling `state_word` / `verification_word` use exhaustive matches. Use an exhaustive match for direction too. | src/commands/relations.rs:185 |
| FND-002 | low | An unknown `--relation` name is refused only by the engine, after the CLI has computed the coverage report (symbol extraction, with its stderr diagnostics) for every member. FR-030 says an unknown `--relation` name 'computes nothing'. The CLI pre-checks the empty `required_relations` case but not the name case; check the names against `registry.traceability().required_relations` in the same pre-check. | src/commands/relations.rs:65-95 |
| FND-003 | low | A `--repo` member with no `spec/` gets the `MissingDocumentRoot` text 'point --scope at the repository root, or create spec/', which names a flag the user did not pass and cannot combine with `--repo`. The context 'member o/c' is right; the remediation hint is wrong for this input. | src/commands/corpus_input.rs:81-83, src/commands/mod.rs:103 |

## Verdict

CONDITIONAL: three low findings, no wrong results. The CLI stays a thin boundary (StR-004). `--json` encodes the engine's `RelationReport` unchanged through `ix_cli_kit::json::encode`. The human renderer adds no data the engine did not produce. Exit codes follow FR-007, checked by test and by hand: clap misuse, malformed `--repo`, duplicate identity, unknown relation, missing `required_relations` and `MissingDocumentRoot` exit 3; a path-safety failure on a `--repo` root exits 2; `--strict` failures exit 1 after the full report. Coder's notes, ruled:

- `--strict` lists every applicable cause in one exit-1 diagnostic. `empty-population` and `nothing-measured` stay distinct tokens and never appear together, so the combined line meets FR-030. Accepted.
- The `required_relations` pre-check is a guard against computing coverage for nothing. The engine has the same refusal (`NoRelationsDeclared`), but the guard is cheap and is not ceremony. Accepted; FND-002 asks for the same guard on names.
- `with_bundle_package` returning `Result` is mapped to an exit-3 invalid request. Correct.

The CLI source holds no hazard, threat or classification name (grep of the diff). The snapshots are meaningful: they pin the rendered rows, gap markers and summaries of a three-state worked example, and the JSON uses relative paths, so it is stable across tempdirs. No ceremony was added. No unsafe, panic path or unbounded input was found: `members[0]` is guaranteed non-empty by the `--scope` default or by `--repo` being non-empty.

## Examined scope

```yaml
scope:
- {id: src/commands/corpus_input.rs, path: src/commands/corpus_input.rs, role: examined, excerpt: "parse_repo, Corpus::from_scope/from_repos/load, root_pairs"}
- {id: src/commands/relations.rs, path: src/commands/relations.rs, role: examined, excerpt: "Args, run, strict_problems, render_human"}
- {id: src/commands/validate.rs, path: src/commands/validate.rs, role: examined, excerpt: "--repo arg, run scope selection, run_okf_corpus, okf_verdict, bundle_message member prefix, with_bundle_package Result"}
- {id: src/main.rs, path: src/main.rs, role: examined, excerpt: "Relations subcommand wiring"}
- {id: tests/cli_relations.rs, path: tests/cli_relations.rs, role: examined, excerpt: "14 FR-030 integration tests"}
- {id: tests/snapshots/relations-worked-example.txt, path: tests/snapshots/relations-worked-example.txt, role: examined, excerpt: "human worked-example snapshot"}
- {id: tests/snapshots/relations-worked-example.json, path: tests/snapshots/relations-worked-example.json, role: examined, excerpt: "pretty JSON worked-example snapshot"}
- {id: FR-030, path: spec/functional/FR-030-relations-subcommand.md, role: context_only, excerpt: "The CLI SHALL provide a `relations` subcommand that renders the two-way relation coverage report of quire-rs FR-084 over one repository or over several repositories loaded as one corpus (FR-085)."}
```

## Dispositions

| FND | outcome | reason |
| --- | --- | --- |
| FND-001 | still-open | No fix landed. Head 603492f only merges main into b429ebb; src/commands/relations.rs:185 still renders the direction with `format!("{:?}", section.direction).to_lowercase()`. |
| FND-002 | still-open | No fix landed. src/commands/relations.rs:77-94 still computes `compute_report` for every member before `relation_report` refuses an unknown `--relation` name. |
| FND-003 | still-open | No fix landed. A `--repo` member with no `spec/` still gets "point --scope at the repository root" (src/commands/mod.rs:105), reproduced at 603492f with `--repo o/a=bare`. |

## New findings (disposition pass 1)

Re-review at head 603492f (merge of origin/main 5a051d3; Cargo.lock now resolves quire-rs main 9baf155, which contains quire-rs#666). Gates run on this head: `cargo fmt --check` clean; `cargo clippy --all-targets --all-features -D warnings` clean; `cargo test --test cli_relations --test cli_okf --test cli_validate --test cli_help` 55 passed, 0 failed. Merge resolution in src/commands/validate.rs checked: `run` delegates to `run_with(.., Source::Files)`; the `--repo` scope derivation sits ahead of the registry load in `run_with`; both export call sites pass `repo: Vec::new()`, so `Source::Committed` never reaches `run_okf_corpus`; `run_okf` keeps main's `with_context` for `with_bundle_package`. No defect in the resolution.

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-004 | medium | Without `--module`, ambient module discovery is bounded by the first `--repo` member only, in both `relations` (`corpus.members[0].root`) and `validate --okf` (`args.repo.first()`). A member's own `manifest.yaml` or `.ix/modules` is used when it is listed first and ignored otherwise, so reordering `--repo` flags can change the loaded module set and with it the rows and findings. FR-030 does not state which member bounds discovery, and the `relations --repo`/`--module` help does not say it either; no test covers it. State the rule in FR-030 and in the `--repo` help (or require `--module` when several members are given). | src/commands/relations.rs:64-65, src/commands/validate.rs:168-175 |
| FND-005 | low | The untyped kind-mismatch target has three spellings: this PR's human output prints `is an untyped document` (added in the merge), the engine's `validate --okf` finding for the same edge says `of archetype '(untyped)'`, and stacked PR #196 introduces `UNTYPED = "untyped"` on the same line, which will conflict when #196 is rebased onto this head. Pick one spelling (the engine's) and use it in both PRs. | src/commands/relations.rs:228 |
