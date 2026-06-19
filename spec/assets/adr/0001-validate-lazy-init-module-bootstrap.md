---
id: ADR-0001
title: "validate lazy-installs default modules via quoin (no-network exception)"
type: ADR
---

# ADR 0001: `validate` lazy-installs default modules via quoin

**Status**: decided (v1)
**Date**: 2026-06-19
**Decision authority**: quire-cli maintainer

## Context

`quire validate` in scoped mode (no `--module`) resolves its archetype
registry from a set of module search roots. Historically those roots were the
`--scope` directory, `--scope/.ix/modules`, and `IX_SCHEMA_PATH`. The canonical
location where `quoin` materializes the default module set —
`~/.ix/filament/modules` (also the default root `quire-rs`
`loader::paths::default_module_root()` reads) — was **not** searched. As a
result, scoped validation on an otherwise correctly-provisioned machine failed
with "no modules found" unless `IX_SCHEMA_PATH` was set by hand.

Two fixes were considered to make scoped validation work out of the box:

1. **Discovery fallback** — add the default install root (and the preferred
   `IX_FILAMENT_MODULES_PATH` env var) to the scoped search set. Pure
   filesystem; no behavioural risk.
2. **Lazy-init** — additionally, when discovery finds *zero* modules, bootstrap
   the default set before failing, so a fresh machine self-heals.

Lazy-init is the better UX, but it collides with
[NFR-004](../../non-functional/NFR-004-no-network.md): quire-cli is a static,
no-network CI gate, and `NFR-004` forbids opening any network socket "at any
point during any subcommand's execution," enforced by `IT-008`
(`strace -fe network`, which follows child processes) and `AUDIT-003`
(`cargo deny` bans HTTP-client crates). The default module set is installed by
**git-cloning public GitHub repos** — unavoidably network I/O.

## Decision

**Adopt both fixes, and bootstrap by delegating to `quoin` as a child process.**

- The discovery roots gain the default install root and
  `IX_FILAMENT_MODULES_PATH` (preferred) alongside the legacy `IX_SCHEMA_PATH`.
  This is pure filesystem work and remains fully within `NFR-004`.
- When scoped discovery yields zero modules, `validate` shells out **once** to
  `quoin plugin ensure-defaults`, then reloads the registry one time. `quoin`
  owns the default-module manifest (`default-modules.yaml`), the pinned tags,
  and the `ts-plugin-kit` reconcile/clone logic — quire neither links a network
  crate nor implements any clone logic itself.
- The network access therefore happens **only** in the `quoin` child, only on
  the empty-discovery path, and only when `quoin` is present on `PATH`. If
  `quoin` is absent or fails, `validate` falls through to an actionable error.

`NFR-004` is **amended** (CR note, 2026-06-19) to scope its guarantee precisely:
*quire-cli's own process* opens no network socket during any subcommand; the
scoped `validate` lazy-init MAY spawn `quoin`, whose network I/O for module
bootstrap is the single documented exception. `IT-008` keeps `strace -f` and
continues to assert zero sockets on the happy path (modules already present —
including the `--module` and populated-search-root cases); the empty-discovery
lazy-init path is explicitly out of its happy-path scope and verified by
demonstration plus the quoin-absent error test (`IT-082`).

## Consequences

- **Positive**: scoped validation self-heals on a fresh machine with no env
  vars; in the normal `quoin`-first workflow (e.g. the `specify` skill runs
  `quoin write` before `quire validate`) modules are already present and quire
  never spawns anything. The static-binary / no-HTTP-crate posture
  (`AUDIT-003`, `NFR-004-AC-1/AC-3`) is untouched — quire links no network
  crate.
- **Negative**: `validate`'s worst-case behaviour now depends on an external
  binary (`quoin`) and may, on the empty-discovery path only, cause network
  I/O via that child. This is a deliberate, documented weakening of the
  absolute "no socket ever" reading of `NFR-004`, accepted for the bootstrap
  UX. Sandboxed/offline callers that must guarantee zero network can keep
  modules pre-installed (the happy path never networks) — the lazy-init is a
  fallback, not the primary path.

## Alternatives rejected

- **Discovery fallback only** (no lazy-init): preserves `NFR-004` verbatim but
  a truly standalone `quire validate` on a bare machine still fails until the
  user runs `quoin`. Rejected in favour of self-heal.
- **Opt-in flag** (`--install-missing`): keeps the default no-network, but adds
  surface area and a footgun (silent no-install by default). Rejected as the
  less ergonomic default.
- **Reimplement the clone in quire** (link a git/HTTP crate): directly violates
  `AUDIT-003` / `NFR-004-AC-1` and duplicates `quoin`'s manifest + reconcile
  logic. Rejected outright.
