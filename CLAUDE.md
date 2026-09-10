# quire-cli

Static binary CLI wrapping quire-rs (render, parse, extract, validate).

## Commands

```bash
make fmt            # format with rustfmt
make fmt-check      # verify formatting (CI gate)
make lint           # all-target/all-feature clippy with -D warnings
make test           # all-target/all-feature cargo test
make build          # release build
make clean          # cargo clean
make deny           # cargo deny check (advisories, bans, licenses, sources)
make audit-unsafe   # check that every unsafe block has a // SAFETY: comment
make ci             # local format/lint/test/dependency/static/spec gates
```

## Safety scaffolding

Backported from `agent-ix/ecaz`:

- `clippy.toml` pins MSRV to `1.98.1` and caps cognitive complexity / arg count
- `deny.toml` allow-lists licenses and denies unknown registries/git sources
- `scripts/check_unsafe_comments.sh` runs in CI and locally via `make audit-unsafe`. Every `unsafe {` block must have a `// SAFETY:` comment within the 3 preceding lines, or be listed in `scripts/unsafe_comment_baseline.txt`. Update the baseline with `bash scripts/check_unsafe_comments.sh --update-baseline`.
- `rustfmt.toml` uses 100-char width with stable-channel options only. The local
  formatting gate fails on drift.
- `rust-toolchain.toml` pins exact Rust 1.98.1 + rustfmt + clippy.

## Layout

```
src/lib.rs             # crate root
tests/integration.rs   # end-to-end tests
benches/               # criterion benchmarks (opt-in; add criterion to dev-deps)
spec/                  # requirements artifacts (from /spec-create-spec)
scripts/               # local tooling
```
