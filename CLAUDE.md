# quire-cli

Static binary CLI wrapping quire-rs (render, parse, extract, validate).

## Hash / digest / pin antipattern: present, slated for removal

This repository still contains hashes, digests, SHAs, pins, checksum catalogs and
records that track files, versions or tools. That is an antipattern and it is slated
for removal. Do not introduce any new use of it. When you touch code or specs that
contain one, remove it as part of the change. The only hash that stays is a canonical
identity digest that binds a proof to the exact content it proved. Package versions
live in Cargo.toml / package.json and their lockfiles only; reports name the app
version they ran.

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

- `deny.toml` allow-lists licenses and denies unknown registries/git sources
- The Rust `quire-qualify` tool runs locally via `make audit-unsafe`. Every
  parsed `unsafe {` block must have a `// SAFETY:` comment within the three
  preceding lines or be listed in `scripts/unsafe_comment_baseline.txt`.
  Update the baseline explicitly with
  `cargo run --locked -p quire-qualify -- unsafe-comments --update-baseline`.
- `rustfmt.toml` uses 100-char width with stable-channel options only. The local
  formatting gate fails on drift.

## Layout

```
src/lib.rs             # crate root
tests/integration.rs   # end-to-end tests
benches/               # criterion benchmarks (opt-in; add criterion to dev-deps)
spec/                  # requirements artifacts (from /spec-create-spec)
scripts/               # local tooling
```
