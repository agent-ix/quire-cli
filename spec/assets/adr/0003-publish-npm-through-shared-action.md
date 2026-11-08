---
id: ADR-0003
title: "Publish npm through the shared native-npm action"
type: ADR
---

# ADR 0003: Publish npm through the shared native-npm action

**Status**: decided (v1) — supersedes ADR-0002
**Date**: 2026-11-08
**Decision authority**: repository owner (PLAT-1012)

## Context

ADR-0002 kept a repository-owned Node launcher and a Rust `quire-dist` tool
that generated and verified the npm packages. Since 2026-09-23
`agent-ix/nodejs-actions/publish-native-npm` does the same for every native
Agent IX CLI (quoin already uses it), so two implementations would drift.

## Decision

Publish `@agent-ix/quire-cli` and its platform packages through
`publish-native-npm`, with `self-update-command: update` and
`static-musl: "true"` (the Linux builds are static musl). The repository keeps
no launcher, no package generator and no registry verifier. `quire-dist` keeps
only `verify-binary`, a pre-publish check on the built executables.

## Consequences

- The npm channel still requires Node, as npm itself does; the launcher is the
  action's.
- The launcher contract (argument and stream forwarding, signal mirroring,
  platform selection, the `update` hint) is owned and tested by the action.
- Package layout, the post-publish registry check and the target catalog are no
  longer specified here.
