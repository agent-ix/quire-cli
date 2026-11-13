---
id: US-009
title: "Reviewer exports a spec document or report to PDF and DOCX"
type: US
relationships:
  - target: "ix://agent-ix/quire-cli/spec/stakeholder/StR-004"
    type: "traces_to"
  - target: "ix://agent-ix/quire-cli/spec/functional/FR-031"
    type: "traces_to"
  - target: "ix://agent-ix/quire-cli/spec/functional/FR-032"
    type: "traces_to"
  - target: "ix://agent-ix/quire-cli/spec/functional/FR-033"
    type: "traces_to"
---
# US-009: Reviewer exports a spec document or report to PDF and DOCX

## Story

**As a** reviewer or approver who works in office tools rather than a repository
**I want** a Quire Markdown artifact, or a JSON report the toolchain wrote, as a
PDF or DOCX file
**So that** I can read, circulate and sign it without a Markdown viewer or a JSON reader.

## Context

The toolchain writes Markdown artifacts and JSON reports (`quire matrix`,
`quoin discharge --rmm`, the SOUP report). Reviewers need the same content in
paged and editable form. The export is a one-way view: the Markdown or JSON stays
the source of truth.

## Acceptance Examples (Illustrative)

### US-009-EX-1: Artifact to both formats

- **Given** a validated FR artifact,
- **When** the reviewer exports it to `pdf` and to `docx`,
- **Then** each file contains every heading, list, table and code block of the
  artifact, as specified by
  [FR-031-AC-1](../functional/FR-031-export-subcommand.md#acceptance-criteria).

### US-009-EX-2: Report as tables

- **Given** the JSON output of `quire matrix --format json`,
- **When** the reviewer exports it to `docx`,
- **Then** the criteria appear as a table, as specified by
  [FR-031-AC-4](../functional/FR-031-export-subcommand.md#acceptance-criteria).

### US-009-EX-3: Missing converter explained

- **Given** the converter is not installed,
- **When** the reviewer runs the export,
- **Then** the command names the missing tool and writes no output file, as
  specified by
  [FR-033-AC-1](../functional/FR-033-export-errors.md#acceptance-criteria).
