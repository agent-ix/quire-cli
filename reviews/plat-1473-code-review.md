---
id: SR-20261112205302
title: "PLAT-1473 quire-cli PR176 code-review"
type: SpecReview
analysis: code-review
scope: "agent-ix/quire-cli PR176 head 4681c37; Makefile, CHANGELOG.md, README.md, tests/audit_no_network.rs, tests/cli_stream_contract.rs, tests/fixtures/iso (deleted), tests/common/mod.rs (context)"
review_set: subset
---

## Summary

Ticket: PLAT-1473. Code review of `git diff origin/main...HEAD` for quire-cli PR176, with the rust-review lane (rust-style loaded). The PR deletes `tests/fixtures/iso` and the `refresh-fixtures` target, and re-points `audit_no_network`, `cli_stream_contract` and `make bench` at quire-cli's own `tests/fixtures/validate-mod`.

The checks run were as follows. `cargo test --locked --no-fail-fast` over audit_no_network, cli_stream_contract, cli_validate, cli_errors, cli_sandbox, cli_schema and cli_extract: 10 tests fail. `cargo clippy --locked --all-targets -D warnings` passes. `cargo fmt --check` passes. Two mutations were run. Removing the `iso-spec-core` grammar rename in cli_stream_contract makes the `[ears:vague-response]` assertion fail at line 419. Pointing `IX_FILAMENT_MODULES_PATH` at a missing dir makes the IT-081 test fail. Both replacements are faithful. The bench command (`quire validate validate-mod/docs/valid-fr.md --module validate-mod`) exits 0.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | high | `tests/common/mod.rs` `iso_module()` still returns `fixture_root().join("iso")`, the directory this PR deletes. Ten tests in five binaries now fail with `--module .../tests/fixtures/iso: No such file or directory`: cli_validate it_014, it_055, it_056, it_057, it_059 and it_105; cli_errors failed_primary_output_returns_internal_without_panicking; cli_extract extract_no_dsl_archetype_errors_cleanly; cli_sandbox it_023; and cli_schema it_062. PR CI runs only the CLA checks, so nothing caught this. Merging turns main red. Fix: re-point each CLI-behaviour test at validate-mod, or at a tempdir module built from quire-cli's own fixtures. it_014's per-ISO-archetype sweep (`ISO_ARCHETYPES`) checks spec-artifacts-iso's schemas, not CLI behaviour, so rewrite it over validate-mod or delete it. Then delete `iso_module()`, `ISO_ARCHETYPES`, and every `iso-docs` file left without a reader. | tests/common/mod.rs:18-20, tests/common/mod.rs:51-61, tests/cli_validate.rs:16, tests/cli_errors.rs:12, tests/cli_extract.rs:245, tests/cli_sandbox.rs:14, tests/cli_schema.rs:13 |
| FND-002 | low | `tests/common/mod.rs` `ctx_path()` has no caller, so `tests/fixtures/contexts/*.json` (10 render-context files from the retired render command) are dead fixtures. They sit in the same helper module FND-001 must edit. Delete the helper and the directory. | tests/common/mod.rs:30-32, tests/fixtures/contexts |

## Examined scope

```yaml
scope:
- id: tests/audit_no_network.rs
  path: tests/audit_no_network.rs
  role: examined
  excerpt: "std::os::unix::fs::symlink(validate_module(), modroot.join(\"validate-mod\")) ... let scope = ...join(\"tests/fixtures/validate-mod/docs\");"
- id: tests/cli_stream_contract.rs
  path: tests/cli_stream_contract.rs
  role: examined
  excerpt: "manifest.replace(\"validate-mod-core\", \"iso-spec-core\")"
- id: Makefile
  path: Makefile
  role: examined
  excerpt: "'$(CURDIR)/target/release/quire validate $(CURDIR)/tests/fixtures/validate-mod/docs/valid-fr.md --module $(CURDIR)/tests/fixtures/validate-mod'"
- id: tests/common/mod.rs
  path: tests/common/mod.rs
  role: examined
  excerpt: "pub fn iso_module() -> PathBuf { fixture_root().join(\"iso\") }"
- id: tests/fixtures/iso-docs
  path: tests/fixtures/iso-docs
  role: examined
  excerpt: "21 small quire-cli-authored docs (FR-valid.md, FR-ears-warn.md, ...); no matching content in quire-rs history or elsewhere under ~/dev"
```

## Verdict

Changes requested. FND-001 blocks the merge: the deletion is incomplete, and 10 tests fail at the PR head.

The rest is correct:

- Both audit_no_network rewrites assert the same property. The IT-081 env-discovery path is load-bearing, as the mutation shows.
- The cli_stream_contract tempdir module is built only from quire-cli's own `validate-mod` files. The grammar rename really triggers the EARS warning; without it the assertion fails.
- No new copy of another repo's files is introduced.
- `tests/fixtures/iso-docs` is not a vendored copy. It holds 21 small quire-cli-authored docs. None of their content occurs in quire-rs history, or elsewhere under ~/dev outside quire-cli checkouts.

## New findings (disposition pass 1)

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-003 | medium | `.github/workflows/release.yml` sets `core.protectNTFS=false` for every git call. Its comment, and the sparse-checkout comment, say this exists only for `tests/fixtures/contexts/CON.json`, described there as "an upstream fixture". This PR deletes that file. `git ls-files` now finds no Windows-reserved name in the tree, so a git path-safety check stays off for no remaining reason. Fix: delete the three `GIT_CONFIG_*` lines and the stale comments. This is a workflow edit, so it needs the owner's clearance first. | .github/workflows/release.yml:36-43, .github/workflows/release.yml:65-68 |
| FND-004 | low | it_022 passes `--heading "FR-001 FR sweep fixture"`, the heading of the deleted `iso-docs/FR-valid.md`. The new `valid-fr.md` heading is `[FR-001] A valid functional requirement`. The test still passes because path-safety rejects `--out` first. But the request now has two faults, where the original isolated the `--out` one. Fix: use the real heading. | tests/cli_sandbox.rs:41 |
| FND-005 | low | `it_026_exit_code_3_on_validation_failure` used to fail on a frontmatter-schema violation (`iso-docs/FR-invalid.md`, `id: bad id`). It now uses `broken-fr.md`, which fails a body table assert. it_014, whose invalid docs were also frontmatter-schema violations, is deleted. So no non-OKF `validate --module` test now drives a frontmatter JSON-schema violation to exit 3. Fix: give it_026 (or a sibling test) an inline tempdir doc with `id: bad id` against validate-mod. | tests/cli_errors.rs:56-66 |

## Dispositions

| FND | outcome | reason |
| --- | --- | --- |
| FND-001 | fixed | 89274c7 deletes `iso_module()`, `iso_doc()`, `ISO_ARCHETYPES` and `tests/fixtures/iso-docs`, and re-points the 10 tests at validate-mod or a tempdir `ears_module()`. Full `cargo test --locked --no-fail-fast` at 89274c7: 307 passed, 0 failed. Mutations: it_055 with a valid doc path, it_105 with plain validate-mod, and the extract test with `body_extraction` kept all fail, so the re-pointed tests still discriminate |
| FND-002 | fixed | 89274c7 deletes `ctx_path()` and `tests/fixtures/contexts/*.json` |
