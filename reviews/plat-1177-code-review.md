---
id: SR-182
title: "PLAT-1177 quire-cli PR159 code-review (Rust lane)"
type: SpecReview
analysis: code-review
scope: "agent-ix/quire-cli PR159; tests/cli_update.rs, tests/audit_ldd.rs, tests/audit_no_network.rs, tests/audit_static.rs, tests/cli_assurance.rs, tests/cli_io.rs, tools/quire-qualify/tests/policy.rs"
review_set: subset
---

## Summary

Ticket: PLAT-1177. Reviewed `git diff origin/main...HEAD` at head d80d1f3: three new unix-only process-boundary tests in `tests/cli_update.rs::npm_channel` (plus a fourth for absent npm) and seven trace-tag rebindings on existing tests. Rust lane (rust-review, rust-style loaded) folded in. The npm-channel harness copies the real `quire` binary under `node_modules/`, puts a copied stock `echo`/`false` on a private `PATH` as `npm`, and asserts the exact argv reported back by the ix-cli-kit engine (`run_capture` returns npm's trimmed stdout as the "latest published version"). Checked against ix-cli-kit ec1b262 `npm_update`: a wrong argv, a missing `version`, a bare `--registry`, or an install on `--check` each fail a test. `PATH` is set per `Command` (no leakage), `TempDir` cleans up, no shell, no network.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | medium | FR-016-AC-7 is tagged on tests that exercise only npm; the cargo half ("a failing npm/cargo invocation ... exits 4") is untested although a `.cargo/bin` copy with `false` as `cargo` would cover it in the same harness | tests/cli_update.rs:132 |
| FND-002 | low | Freshly copied executables are exec'd while sibling test threads fork; a forked child can hold the write fd from fs::copy and execve then fails ETXTBSY (rust-lang/rust#114554), an intermittent failure | tests/cli_update.rs:80 |
| FND-003 | low | stock_program copies /bin/echo or /bin/false; on a busybox or multicall-coreutils host that binary dispatches on argv[0], so as `npm` it fails and the AC-3/AC-4 tests break | tests/cli_update.rs:65 |

## Examined scope

```yaml
scope:
- id: FR-016-AC-3
  path: spec/functional/FR-016-update-subcommand.md
  role: examined
  excerpt: "On the npm channel, `--check` runs `npm view @agent-ix/quire-cli version` and reports the latest version without installing"
- id: FR-016-AC-4
  path: spec/functional/FR-016-update-subcommand.md
  role: examined
  excerpt: "A `--registry <URL>` override for the scoped package is passed as `--@agent-ix:registry=<URL>` (scope form), not a bare `--registry`; with no override no registry flag is added"
- id: FR-016-AC-7
  path: spec/functional/FR-016-update-subcommand.md
  role: examined
  excerpt: "A failing `npm`/`cargo` invocation, or an unreachable registry, exits **4**"
- id: npm_update
  path: ix-cli-kit src/self_update.rs (ec1b262)
  role: context_only
  excerpt: "let mut args = vec![\"view\", cfg.npm_package, \"version\"]; args.extend(reg_args...); let latest = run_capture(\"npm\", &args, ...)?; ... format!(\"latest published version: {latest}\")"
```

## Verdict

Changes requested (medium). The AC-3 and AC-4 tests genuinely assert their criteria and can fail on a wrong implementation; cfg(unix) gating is correct (PermissionsExt); hermetic. Add a cargo-channel exit-4 case so the FR-016-AC-7 tag is not a partial claim (FND-001). FND-002/003 are robustness nits: retry the spawn on ETXTBSY or serialize the copy-then-exec, and prefer a stand-in that does not depend on argv[0] (or skip when the stock binary is a multicall link).

## Dispositions

| FND | outcome | reason |
| --- | --- | --- |
| FND-001 | fixed | aee9bff: failing_or_missing_cargo_exits_four copies quire under .cargo/bin, runs bare update with a failing and an absent cargo on PATH, asserts exit 4, empty stdout, non-empty stderr; tagged FR-016-AC-7 |
| FND-002 | fixed | aee9bff: static EXEC_LOCK serializes write-then-exec across the channel tests and spawn_retrying retries ExecutableFileBusy up to 50 times; TempDir still owns cleanup |
| FND-003 | fixed | aee9bff: stand-ins are #!/bin/sh scripts (echo "$@" / exit 1) written per test, so nothing depends on argv[0] dispatch of a copied system binary |
