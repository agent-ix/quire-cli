---
id: SR-20261112184201
title: "PLAT-1421 quire-cli PR174 code-review"
type: SpecReview
analysis: code-review
scope: "agent-ix/quire-cli PR174 head b59c201; src/commands/publish_check.rs, src/main.rs, src/commands/mod.rs, tests/cli_publish_check.rs, tests/snapshots/help.txt, CHANGELOG.md, Cargo.lock"
review_set: subset
---

## Summary

Ticket: PLAT-1421. This review covers `git diff origin/main...HEAD` for quire-cli PR174, the `quire publish-check` subcommand (FR-029). It includes the rust-review lane, with rust-style loaded. The engine API was read at the locked quire-rs rev a7266b8 (`check_publishable`, `Registry::load_module_strict`, `loader::load_single_module`).

Checks run: `cargo test --test cli_publish_check --test cli_help` (7 passed), `cargo test --bin quire publish_check` (1 passed), and `cargo clippy --all-targets`, which reported nothing for publish_check. Lead ruling applied: the outside-root `clause_sets` symlink exiting 3 is accepted behaviour, not a finding.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | medium | The module is parsed twice. `Registry::load_module_strict` runs `load_single_module` and its failures are checked. Then `load_single_module` runs again and its `failures` are ignored. The loader drops a clause set that fails to load from `LoadedModule.clause_sets` and only records it in `failures`. So if a licensed set's file changes between the two reads (TOCTOU), it is silently missing from the second load. The command then prints `no restricted clause sets` and exits 0, which fails open on a gate that must fail closed. Fix: parse once. Call `load_single_module` once, refuse with exit 3 when `failures` or `manifest_errors` is non-empty, then call `check_publishable(&outcome.modules[0])`. A cleaner option is for quire-rs to expose the verdict over a `Registry`, which already holds `clause_sets()`. At minimum, check the second outcome's `failures`. | src/commands/publish_check.rs:50-68 |
| FND-002 | medium | The `..` refusal test is vacuous. `{root}/../module` canonicalizes to the restricted module itself, so with `reject_dotdot` removed the command would load the restricted sets and still exit 2. Fix: point the traversal at `public_module`, or assert that stderr carries the path refusal and names no set. | tests/cli_publish_check.rs:201-206 |
| FND-003 | medium | The AC-6 byte-identity check brackets only two runs that never open the module: the traversal run is refused before load, and the missing path is a different directory. Fix: take the snapshot around a full human run and a full json run on the restricted module. | tests/cli_publish_check.rs:200-219 |
| FND-004 | low | The AC-8 token scan is case-sensitive, so `TextRights::Licensed` or a `Rights` type would pass. Fix: lowercase the source before scanning. | tests/cli_publish_check.rs:275-283 |
| FND-005 | low | Cargo.lock also moves agent-ix-semantic-schema (filament-core-data 60d46cf to 327a49c), beyond the quire-rs-only refresh. Fix: `cargo update -p quire-rs` only, unless quire-rs a7266b8 needs the newer revision. | Cargo.lock:8 |

## Examined scope

```yaml
scope:
- id: src/commands/publish_check.rs
  path: src/commands/publish_check.rs
  role: examined
  excerpt: "let registry = quire_rs::Registry::load_module_strict(&module) ... let outcome = quire_rs::loader::load_single_module(&module); let Some(loaded) = outcome.modules.first() else { ... }; Ok(check_publishable(loaded))"
- id: tests/cli_publish_check.rs
  path: tests/cli_publish_check.rs
  role: examined
  excerpt: "IT-258..IT-261 process-boundary tests plus the IT-261 unit test in src"
- id: tests/snapshots/help.txt
  path: tests/snapshots/help.txt
  role: examined
  excerpt: "publish-check  Refuse a module that carries restricted clause text"
- id: CHANGELOG.md
  path: CHANGELOG.md
  role: examined
  excerpt: "PLAT-1421: `quire publish-check --module <dir> [--format human|json]` refuses a module whose clause sets carry restricted text."
- id: Cargo.lock
  path: Cargo.lock
  role: examined
  excerpt: "quire-rs 66889cf -> a7266b8; agent-ix-semantic-schema 60d46cf -> 327a49c"
```

## Verdict

Changes requested on FND-001 through FND-003.

The boundary is thin: the CLI holds no rights policy, and the verdict comes from the engine `check_publishable`. Exit codes use the existing paths: `policy_refusal!` maps to Refused (2) and `invalid_request!` / `failure::invalid` map to 3, matching `clauses.rs`. Streams follow AC-1 and AC-3. In human mode the clean line goes to stdout, and a refusal writes nothing to stdout and puts the names on stderr. In json mode stdout carries `restricted` plus engine provenance. No clause text is emitted. Cargo.toml is unchanged. The help snapshot and CHANGELOG are correct. Nothing in production code panics.
