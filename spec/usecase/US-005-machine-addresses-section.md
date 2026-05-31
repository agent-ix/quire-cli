---
id: US-005
title: "Machine addresses a document section by heading or stable block id"
artifact_type: US
relationships:
  - target: "ix://agent-ix/quire-cli/spec/stakeholder/StR-001"
    type: "implements"
    cardinality: "1:1"
  - target: "ix://agent-ix/quire-cli/spec/stakeholder/StR-004"
    type: "implements"
    cardinality: "1:1"
---

## Story

As an **agent or script working against a rendered spec artifact**, I want to fetch one parsed section by heading, parser-derived section id, or stable Pandoc block id, so that follow-up tools can operate on a narrow section without reimplementing Markdown parsing.

## Context

`quire parse` already exposes the full `QuireDocument` tree. Many automation paths only need one section: for example the H1 title, the `Acceptance` section, or a block with a stable `{#blk-id}` attribute. The CLI should provide that lookup directly while preserving the thin-boundary rule over `quire-rs`.

## Acceptance

- **US-005-AC-1**: `quire lookup doc.md --heading Behavior` returns the first section whose heading matches `Behavior`.
- **US-005-AC-2**: `quire lookup doc.md --heading Title --level 1` returns only an H1 match and fails if the title exists only at another heading level.
- **US-005-AC-3**: `quire lookup doc.md --block-id blk-behavior` returns the section whose heading carries `{#blk-behavior}`.
- **US-005-AC-4**: `quire lookup doc.md --id behavior-L4` returns the section whose parser-derived section id is `behavior-L4`.
- **US-005-AC-5**: A failed lookup exits 1, writes no stdout, and emits a diagnostic naming the missing selector.
