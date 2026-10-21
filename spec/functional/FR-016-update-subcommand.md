---
id: FR-016
title: "quire update subcommand (install-source-aware self-update)"
type: FR
object_type: cli_command
relationships:
  - target: "ix://agent-ix/quire-cli/spec/stakeholder/StR-001"
    type: "implements"
    cardinality: "1:1"
---

## Description

The CLI SHALL expose an `update` subcommand that upgrades the installed `quire`
binary to the latest published version. Because the single binary
([StR-001](../stakeholder/StR-001-static-binary-hot-path.md)) is distributed
through more than one channel — the npm prebuilt-binary wrapper
(`@agent-ix/quire-cli`) and `cargo install --git` — `update` SHALL first detect
**how the running binary was installed** and then drive the matching package
manager.

`update` is the one subcommand exempt from the quire-rs thin-boundary
([StR-004](../stakeholder/StR-004-thin-boundary-over-quire-rs.md)): it manages
binary lifecycle, not artifact behavior, so it carries no parser/renderer/
validator logic and wraps no `quire-rs` API. Its logic lives in a deliberately
**package-agnostic** `ix-cli-kit::self_update` engine (config-struct driven, no
quire coupling). The `update` command is a thin wrapper that supplies quire's
distribution coordinates.

The npm wrapper version and the Cargo crate version are independently numbered,
so `update` SHALL NOT compare the running binary's version against a registry —
a cross-scheme diff is meaningless. Idempotency is delegated to npm/cargo, which
already no-op (or rebuild) when current.

## Behavior

```
quire update [--check] [--registry <URL>]
```

1. Resolve the running executable path (`current_exe`) and classify the install
   source: a path under a `node_modules` tree ⇒ **npm**; a path under `.cargo`
   ⇒ **cargo**; anything else ⇒ **unknown**.
2. **npm channel**:
   - `--check`: run `npm view @agent-ix/quire-cli version` and report the latest
     published version; install nothing.
   - otherwise: run `npm install -g @agent-ix/quire-cli@latest` with inherited
     stdio.
   - When `--registry <URL>` is given, the override is applied as the
     **scope-specific** `--@agent-ix:registry=<URL>` form, because a plain
     `--registry` is silently ignored for a scoped package when the user's npmrc
     pins a `@scope:registry`. With no `--registry`, the ambient npm config
     resolves the package (mirroring how it was installed).
3. **cargo channel**:
   - `--check`: report that cargo installs track the git default branch (no
     single published version to compare); install nothing.
   - otherwise: run `cargo install --git https://github.com/agent-ix/quire-cli
     --force` with inherited stdio.
4. **unknown source**: never guess and clobber a binary the tool did not place.
   Print manual upgrade instructions (the npm and cargo recipes plus the
   releases URL) and exit **0** — performing no install, touching no network.
5. The summary lines describing the detected source and action are the
   subcommand's primary output on **stdout**; any npm/cargo progress is the
   child process's own inherited output.
6. Exit codes: **0** on success (including `--check` and the unknown-source
   manual path); **4** when an invoked `npm`/`cargo` command fails or the
   registry cannot be reached.

## Acceptance Criteria

| ID | Criteria | Verification |
|----|----------|--------------|
| FR-016-AC-1 | A binary path under `node_modules/...` classifies as the npm channel; a path under `.cargo/...` classifies as the cargo channel; any other path classifies as unknown | Test |
| FR-016-AC-2 | On an unknown source, `quire update` (with or without `--check`) prints manual upgrade instructions including the npm recipe, the cargo recipe, and the releases URL, exits **0**, and performs no install | Test |
| FR-016-AC-3 | On the npm channel, `--check` runs `npm view @agent-ix/quire-cli version` and reports the latest version without installing | Test |
| FR-016-AC-4 | A `--registry <URL>` override for the scoped package is passed as `--@agent-ix:registry=<URL>` (scope form), not a bare `--registry`; with no override no registry flag is added | Test |
| FR-016-AC-5 | `update` performs no version comparison against the running binary and shells out to npm/cargo for idempotency (no cross-scheme version diff in `src/`) | Inspection |
| FR-016-AC-6 | `commands/update.rs` uses the shared `ix-cli-kit::self_update` engine, supplies quire's coordinates through `SelfUpdateConfig`, and contains no parser, renderer, or validator logic; quire carries no local copy of the engine | Inspection |
| FR-016-AC-7 | A failing `npm`/`cargo` invocation, or an unreachable registry, exits **4** | Test |

> **CR note (untraced criteria, updated 2026-10-20, #55):** four paths carry **no
> automated trace**, because exercising them needs live network access and
> global-install side effects: the npm-channel `--check`/install of AC-3, the
> registry-unreachable / failing-`npm`/`cargo` exit-4 of AC-7, and the
> `cargo install` dispatch half of AC-1. The registry-argument construction of
> AC-4 is tested by `ix-cli-kit`, but its upstream tests do not bind quire's
> local trace matrix. Their `Verification: Test` states the
> intended method, not an existing trace. This statement lives here rather
> than in the Test Matrix's Coverage Status column because that column
> declares no `column_patterns` and is therefore unenforced — prose there is
> validated by nothing (agent-ix/spec-artifacts-process#57 tracks declaring
> the column contract; once it lands the rollup row can carry an enforced
> status instead). The FR-016 matrix row stays 🚧 until these paths gain a
> recorded trace.

## Dependencies

- **Upstream**: `ix-cli-kit` supplies the shared `self_update` engine;
  `quire-rs` is not involved in this binary-lifecycle feature.
- **Downstream**: agent/CI setup docs that pin a single binary version
  ([StR-001](../stakeholder/StR-001-static-binary-hot-path.md)) — `update`
  keeps that pinned binary current.
