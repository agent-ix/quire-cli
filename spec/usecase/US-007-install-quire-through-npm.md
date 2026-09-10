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

- **US-007-AC-1**: A clean offline installation from the five locally packed
  npm artifacts installs one launcher and the matching optional platform
  package, and `quire --version` executes the packaged native binary.
- **US-007-AC-2**: Arguments, standard streams, normal exit status, and
  terminating signals cross the launcher boundary without reinterpretation.
- **US-007-AC-3**: An unsupported host or an absent matching optional package
  fails with an actionable diagnostic and never substitutes another target.
- **US-007-AC-4**: Every launcher release selects platform packages at the
  identical version and ships the repository's declared license.

## Boundaries

This story governs distribution and process launching only. It adds no
document parser, extraction rule, native-language profile, clause grammar,
source semantic, WASM surface, or browser behavior.
