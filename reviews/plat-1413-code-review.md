---
id: SR-20261112032005
title: "PLAT-1413 quire-cli PR169 code-review"
type: SpecReview
analysis: code-review
scope: "agent-ix/quire-cli PR169 head 6031b6f; src/commands/matrix.rs, tests/cli_matrix.rs, tests/snapshots/matrix-worked-example.md, spec/tests.md, Cargo.lock"
review_set: subset
---

## Summary

Ticket: PLAT-1413. This review covers `git diff origin/main...HEAD` for quire-cli PR169, the `quire matrix` Requirement Criticality column (FR-028). It includes the rust-review lane, with rust-style loaded.

The checks run were as follows. `cargo check --locked --tests` was run against the PR's own Cargo.lock, and it fails. The lock was then bumped locally to quire-rs 0575fd4 (#648), and that bump was not committed. Against the bumped lock, these checks pass: `cargo test --bin quire --test cli_matrix --test output_contract --test cli_coverage` (86 tests), the 7 `fr028_*` tests, `cargo clippy --all-targets -D warnings`, and `cargo fmt --check`.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | high | Cargo.lock still pins quire-rs at 05afc408, which is before #648. At the PR's own lock, `CoverageMatrixRequirement` has no `criticality` field: `cargo check --locked --tests` fails with E0609 at matrix.rs:232 and E0560 at matrix.rs:313. A plain `cargo build` uses the same lock, and every `make` target runs `--locked`. Merging as-is turns main red. The Cargo.lock bump to quire-rs 0575fd4 has to land in this PR. | Cargo.lock:1204, src/commands/matrix.rs:232, src/commands/matrix.rs:313 |
| FND-002 | low | There is no CHANGELOG `[Unreleased]` entry. The change is user-visible: the Markdown and TSV output of `quire matrix` gains a column, which shifts positional TSV consumers. Recent feature PRs (for example PLAT-1386) add an entry. | CHANGELOG.md:10 |
| FND-003 | low | FR-028-AC-11 asks for identical `--strict` and exit code across all-high, all-missing and mixed values. The test compares stderr and exit only across all-high, all-missing and all-low. For the mixed corpus it asserts only exit 0. Every corpus is fully tagged, so the strict-failing path (an untagged criterion) is never compared across criticality values. | tests/cli_matrix.rs:1217-1248 |

## Examined scope

```yaml
scope:
- id: src/commands/matrix.rs
  path: src/commands/matrix.rs
  role: examined
  excerpt: "fn criticality_text(requirement: &CoverageMatrixRequirement) -> String { match &requirement.criticality { None => UNASSIGNED.to_string(), Some(serde_json::Value::String(text)) => text.clone(), Some(other) => other.to_string(), } }"
- id: Cargo.lock
  path: Cargo.lock
  role: examined
  excerpt: "source = \"git+https://github.com/agent-ix/quire-rs?branch=main#05afc408f0a1cb01b9999a90f4380e4dddf09433\""
- id: tests/cli_matrix.rs
  path: tests/cli_matrix.rs
  role: examined
  excerpt: "fr028_* tests (7) plus updated it168/169/170/172/173/175 assertions"
- id: FR-028-AC-11
  path: spec/functional/FR-028-matrix-criticality-column.md
  role: examined
  excerpt: "For the same corpus, `--strict` and the exit code are identical whether every requirement is `high`, every requirement is missing a value, or the values are mixed."
```

## Verdict

Changes requested. The rendering code is correct and idiomatic:

- `Option<Value>` is matched exhaustively.
- Nothing in production code panics.
- The cell goes through the existing `markdown_cell` and `tsv_cell` rules, so `|` is escaped and tab, LF and CR are scrubbed.
- The value is not truncated.
- JSON passes through via the engine struct, whose `skip_serializing_if` keeps an absent key absent.
- The engine maps a YAML null to `None`, so a null renders `unassigned`. This is consistent with the spec.
- The snapshot and the IT-168/169/170/172/173/175 assertions are updated coherently.

FND-001 blocks the merge: the PR does not compile at its own lockfile.

A minor point, not a finding: `render_tsv` recomputes `criticality_text` per criterion, while `render_markdown` computes it once per requirement.
