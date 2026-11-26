---
id: US-012
title: "Developer inspects source-change impact"
type: US
relationships:
  - target: "ix://agent-ix/quire-cli/spec/stakeholder/StR-004"
    type: "traces_to"
---
# US-012: Developer inspects the impact of a source change

## Story

**As a** developer reviewing a source change
**I want** to see the tests and related code that may be affected, including
unlinked or incomplete areas
**So that** I can decide what to inspect and verify before integrating the
change.

## Context

The source-change impact analysis is produced by quire-rs and surfaced by the
quire CLI. This story captures the developer's need for actionable impact
information without prescribing the command's inputs or output format.

## Acceptance Examples (Illustrative)

### US-012-EX-1: Developer receives actionable impact information

- **Given** a developer has a local source change
- **When** the developer requests its impact
- **Then** the developer can identify potentially affected tests and related
  code, as specified by [FR-049-AC-1](../functional/FR-049-impact-subcommand.md#acceptance-criteria)

### US-012-EX-2: Developer can recognize incomplete impact information

- **Given** part of the source change cannot be analyzed completely
- **When** the developer requests its impact
- **Then** the developer can identify the incomplete or unlinked area, as
  specified by [FR-049-AC-3](../functional/FR-049-impact-subcommand.md#acceptance-criteria)

These examples illustrate the user outcome; FR-049 defines the command's
normative behavior.
