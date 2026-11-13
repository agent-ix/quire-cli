---
id: US-010
title: "Approver receives an identified, templated, signed export"
type: US
relationships:
  - target: "ix://agent-ix/quire-cli/spec/stakeholder/StR-004"
    type: "traces_to"
  - target: "ix://agent-ix/quire-cli/spec/functional/FR-034"
    type: "traces_to"
  - target: "ix://agent-ix/quire-cli/spec/functional/FR-035"
    type: "traces_to"
  - target: "ix://agent-ix/quire-cli/spec/functional/FR-036"
    type: "traces_to"
  - target: "ix://agent-ix/quire-cli/spec/functional/FR-037"
    type: "traces_to"
  - target: "ix://agent-ix/quire-cli/spec/functional/FR-038"
    type: "traces_to"
  - target: "ix://agent-ix/quire-cli/spec/functional/FR-039"
    type: "traces_to"
  - target: "ix://agent-ix/quire-cli/spec/functional/FR-040"
    type: "traces_to"
  - target: "ix://agent-ix/quire-cli/spec/non-functional/NFR-011"
    type: "traces_to"
---
# US-010: Approver receives an identified, templated, signed export

## Story

**As an** approver or auditor who keeps controlled documents outside the repository
**I want** every exported document to carry its identity, follow my organisation's
layout, show who approved it and when, and come out the same each time
**So that** I can file the PDF or DOCX as a controlled copy and prove which source
revision it came from.

## Context

[US-009](./US-009-export-document-to-pdf-docx.md) gives a reviewer a PDF or DOCX
of an artifact. A controlled copy needs more: a title page and table of contents,
the document id, version and source commit, the approvals recorded for it, the
organisation's own look, and the guarantee that exporting the same revision twice
gives the same document. The look comes from a template the organisation holds
locally; Quire ships only a default.

## Acceptance Examples (Illustrative)

### US-010-EX-1: Default controlled copy

- **Given** a committed artifact with an id, a title and a version,
- **When** the approver exports it to `pdf` with no template,
- **Then** the file opens on a title page that states the identity fields and the
  source commit date, as specified by
  [FR-034-AC-1](../functional/FR-034-document-identity.md#acceptance-criteria) and
  [FR-035-AC-1](../functional/FR-035-default-export-template.md#acceptance-criteria).

### US-010-EX-2: Organisation template

- **Given** the organisation's template file on a local path,
- **When** the approver exports with `--template <path>`,
- **Then** the document uses that template and the template file is never copied
  into a repository, as specified by
  [FR-036-AC-1](../functional/FR-036-organisation-export-template.md#acceptance-criteria).

### US-010-EX-3: Approval block

- **Given** the verdict file of a signed approval tag,
- **When** the approver exports with `--approval <verdict>`,
- **Then** the document shows the approver, the time, the meaning and the verdict,
  as specified by
  [FR-038-AC-1](../functional/FR-038-approval-block.md#acceptance-criteria).

### US-010-EX-4: Same revision, same document

- **Given** one committed artifact and one template,
- **When** the approver exports it twice on different days,
- **Then** the extracted text and structure are equal, as specified by
  [NFR-011-AC-1](../non-functional/NFR-011-export-reproducibility.md#acceptance-criteria).

### US-010-EX-5: Bundle

- **Given** three committed artifacts,
- **When** the approver exports them together,
- **Then** one document holds all three in the order given, behind one title page,
  as specified by
  [FR-039-AC-1](../functional/FR-039-export-bundle.md#acceptance-criteria).
