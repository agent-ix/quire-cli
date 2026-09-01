---
id: US-006
title: "Evidence producer exports source-grounded assurance facts"
type: US
relationships:
  - target: "ix://agent-ix/quire-cli/spec/stakeholder/StR-004"
    type: "implements"
    cardinality: "1:1"
---

## Story

As an **evidence producer or migration adapter**, I want one deterministic CLI
projection of Quire's authoritative artifacts, obligations, symbols, and
relations, so that I can pass static specification facts to a consumer without
reimplementing Quire's graph or confusing an unavailable export with an empty
successful one.

## Acceptance

- **US-006-AC-1**: A producer supplies a bounded repository scope, one exact
  module, a repository identity, and a full immutable revision and receives a
  `quire-assurance` v1 JSON document on stdout.
- **US-006-AC-2**: A producer can pin the expected module version and complete
  active-archetype schema-digest set; any mismatch fails before stdout.
- **US-006-AC-3**: Repeating the command over byte-identical inputs emits
  byte-identical compact JSON.
- **US-006-AC-4**: The command reads documents and source statically and never
  executes a test, proof, solver, consumer, package manager, or Git command.

