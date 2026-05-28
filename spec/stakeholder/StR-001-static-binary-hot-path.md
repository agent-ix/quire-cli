---
id: StR-001
title: "Static binary serves the agent generation hot path"
artifact_type: StR
---

## Stakeholder Need

Agent-driven artifact generation (FR / NFR / ADR / Plan / Review / Ledger workflows in `spec-skills`) invokes a CLI **once per artifact**. Today that path uses `minijinja-cli` for render only; a second invocation would be needed to validate, a third to parse the result for cross-references. Three processes per artifact, three cold starts, three sets of context switches.

`quire-cli` SHALL collapse render + parse + extract + validate into a single binary so that:
1. One install (`cargo install quire-cli` or release tarball) gives agents the full surface
2. Each subcommand is one cold-start process (no shared daemon, no IPC)
3. Agent setup documentation pins a single binary version, not three

## Priority

Must-Have

## Acceptance

- **StR-001-AC-1**: `quire --help` lists four subcommands: `render`, `parse`, `extract`, `validate`.
- **StR-001-AC-2**: The binary is statically linked; `ldd quire` on Linux lists only libc and dynamic loader (no `libquire_rs.so`).
- **StR-001-AC-3**: `cargo install --git https://github.com/agent-ix/quire-cli quire-cli` produces a runnable `quire` binary.
- **StR-001-AC-4**: No subcommand opens a network socket (verified by strace / equivalent in IT).
