---
id: US-007
title: "Consumer installs the native Quire CLI through npm"
type: US
relationships:
  - target: "ix://agent-ix/quire-cli/spec/stakeholder/StR-001"
    type: "implements"
    cardinality: "1:1"
---

## Story

As an npm consumer, I want one versioned `@agent-ix/quire-cli` package to select
and launch the matching prebuilt native executable, so that I receive the same
`quire` process contract without building Rust or choosing a platform package
myself.

## Acceptance

- **US-007-AC-1**: `npm install -g @agent-ix/quire-cli` installs the matching
  platform package, and `quire --version` executes the packaged native binary.
- **US-007-AC-2**: An unsupported host fails with an actionable diagnostic and
  never substitutes another target.
- **US-007-AC-3**: `quire update` under an npm install defers to npm.

## Boundaries

This story governs distribution only; the package is produced by the shared
publish action (ADR-0003). It adds no document parser, extraction rule,
native-language profile, clause grammar, source semantic, WASM surface, or
browser behavior.
