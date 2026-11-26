# Quire engine dependency move (PLAT-1918)

## Revision

- Old Quire CLI dependency: quire-rs `0.63.2` at `1d8381188b37a95f0fe4cd6b7c57389058ac45b1`.
- New dependency: quire-rs `0.63.2` at `d7d418fa2727f7de87a16dfec5b38ada7d9ee776`.
- The `v0.63.2` tag points at the old commit; this move intentionally pins the compile-fixed public-main revision containing PLAT-1904. No quire-rs release/tag is created here.
- `quire-fixtures` is not a dependency in this repository; no fixture pin moved.

## Upstream diff (`git diff --name-status 1d838... d7d418f`)

```text
M .github/workflows/wheels.yml
M CHANGELOG.md
A reviews/plat-1897-duplicate-trace-dependency-review.md
A reviews/plat-1897-pr750-code-review.md
A reviews/plat-1898-code-review.md
A reviews/plat-1898-gap-analysis.md
A reviews/plat-1904-code-review.md
A reviews/plat-1904-gap-analysis.md
A reviews/plat-1904-spec-review-base.md
A reviews/plat-1904-spec-review-integrity.md
M reviews/plat-1915-code-review.md
A reviews/plat-1915-gap-analysis.md
M spec/functional/FR-092-matrix-design-column.md
M src/coverage.rs
M src/coverage/reconcile.rs
M src/diagnostic.rs
M src/impact.rs
M src/loader/mod.rs
M src/loader/paths.rs
M src/registry.rs
M src/semantic/record.rs
M src/symbols/trace.rs
A tests/fixtures/loader-home/foo/README.md
A tests/fixtures/loader-home/foo/loader-fixture/examples/fixture.record.json
A tests/fixtures/loader-home/foo/loader-fixture/manifest.yaml
A tests/fixtures/loader-home/foo/loader-fixture/schemas/FR.schema.json
M tests/it/relation_report.rs
M tests/it/semantic_model.rs
```

## Public API and local references

The engine adds `quire_rs::coverage::compute_coverage_for_design_matrix`, which preserves valid coverage entries when required-relation declarations were dropped while keeping ordinary coverage strict. Quire CLI uses it only from `src/commands/coverage.rs` in the design-matrix path; other coverage calls continue using strict `compute_coverage`.

The upstream diff also adds `quire_rs::impact::{DependencyMoveInput, UpstreamChange, DependencyLocalReference, DependencyTestReason, DependencyMoveReport, analyze_dependency_move}`. Quire CLI has no references to those items. Exact changed-path searches found no local references to changed engine file paths; the generic `CHANGELOG.md` basename matches are unrelated local corpus-root documentation tests/specs. The matrix regression test now expects the engine-preserved declared module id (`m`) in `DesignMatrix.errors`, rather than the temporary module directory path.

## Verification

- `cargo metadata --locked --no-deps --format-version 1` — passed after pin update.
- `cargo fmt --check` and `git diff --check` — passed.
- `cargo test --locked --test cli_matrix_design dropped_required_relation_errors_preserve_matrix_entries_in_every_format -- --exact` — passed, 1/1.
- `cargo test --locked --test cli_declaration_drop matrix::` — passed, 2/2; checks non-relation and whole-section drops still refuse.
- `cargo run --locked --bin quire -- trace --file src/commands/coverage.rs` — passed (exit 0); current-source trace resolves and reports the new `FR-026-AC-17` and `FR-043-AC-27` implementation tags. It also prints two existing cfg-variant symbol-id collision warnings in `src/commands/selftest.rs` and `src/safety.rs`.
- Cargo tests and source trace run through `worktrees/locked-build.sh`.

## Local Quire CLI changes

`Cargo.toml` and `Cargo.lock` move the engine pin from `1d8381188b37a95f0fe4cd6b7c57389058ac45b1` to `d7d418fa2727f7de87a16dfec5b38ada7d9ee776`, retaining package version `=0.63.2`. `src/commands/coverage.rs` uses the tolerant engine API only for design-matrix coverage; ordinary coverage remains strict. `tests/cli_matrix_design.rs` verifies valid rows and dropped-relation errors survive across formats and asserts the engine’s module id. This record is in `reviews/plat-1918-quire-rs-dependency-move.md`.
