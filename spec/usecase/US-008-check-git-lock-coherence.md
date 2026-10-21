---
id: US-008
title: "Developer checks Git lock coherence"
type: US
relationships:
  - target: "ix://agent-ix/quire-cli/StR-004"
    type: "traces_to"
---
# US-008: Developer checks Git lock coherence

## Story

**As a** developer maintaining related Rust packages
**I want** dependency gates to detect inconsistent resolutions from the same
Git repository
**So that** different crate names cannot hide a split dependency revision.

## Context

IR, CG, and RT currently copy a checker that detects repeated agent-ix crate
names but accepts distinct crate names from one repository at different
resolved commits. One distributed developer command avoids three diverging
executable implementations. Different independently checked lockfiles can
legitimately select different revisions.

## Acceptance Examples (Illustrative)

### US-008-EX-1: Aligned packages pass the gate

- **Given** a developer's lock selects multiple crates from one repository at
  the same resolved commit,
- **When** the developer runs the shared gate,
- **Then** the developer receives a successful result, as specified by
  [FR-027-AC-1](../functional/FR-027-check-lock-subcommand.md#acceptance-criteria).

### US-008-EX-2: Split revisions explain the refusal

- **Given** a lock selects different crate names from the same repository at
  different resolved commits,
- **When** the developer runs the shared gate,
- **Then** the refusal identifies the lock, repository, crates, and revisions,
  as specified by
  [FR-027-AC-2](../functional/FR-027-check-lock-subcommand.md#acceptance-criteria).

These examples illustrate the developer outcomes; the linked FR criteria own
acceptance and the command's normative rules.

## Dependencies (Contextual)

[FR-027](../functional/FR-027-check-lock-subcommand.md) defines the command.
IR-641 owns subsequent adoption and removal of the copied consumer scripts.
