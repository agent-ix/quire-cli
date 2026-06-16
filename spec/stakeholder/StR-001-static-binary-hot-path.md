---
id: StR-001
title: "Static binary serves the agent generation hot path"
type: StR
---

> **CR note (render removal — 2026-06-04):** The render half is removed (mirrors
> quire-rs render retirement, commit 500a3d3). The single-binary hot-path need now
> covers `validate` / `parse` / `extract` / `lookup` / `edit` (no `render`).
> Artifacts are authored as markdown directly and checked by `validate`. The
> stakeholder need below is revised accordingly; AC-1 is updated to the surviving
> subcommand set.

## Stakeholder Need

Agent-driven artifact workflows (FR / NFR / ADR / Plan / Review / Ledger workflows in `spec-skills`) invoke a CLI **once per artifact** to validate, parse for cross-references, or address a section. Without a single binary, each is a separate process, cold start, and set of context switches.

`quire-cli` SHALL collapse parse + extract + validate + lookup + edit into a single binary so that:
1. One install (`cargo install quire-cli` or release tarball) gives agents the full surface
2. Each subcommand is one cold-start process (no shared daemon, no IPC)
3. Agent setup documentation pins a single binary version, not three

## Priority

Must-Have

## Acceptance

- **StR-001-AC-1**: `quire --help` lists the subcommands `parse`, `extract`, `validate`, `schema`, `lookup`, and `edit` (no `render`).
- **StR-001-AC-2**: The binary is statically linked; `ldd quire` on Linux lists only libc and dynamic loader (no `libquire_rs.so`).
- **StR-001-AC-3**: `cargo install --git https://github.com/agent-ix/quire-cli quire-cli` produces a runnable `quire` binary.
- **StR-001-AC-4**: No subcommand opens a network socket (verified by strace / equivalent in IT).
