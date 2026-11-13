---
id: SR-20261112234821
title: "PLAT-1605 quire-cli PR182 spec-review/ears-conformance"
type: SpecReview
analysis: ears-conformance
scope: "agent-ix/quire-cli PR182 head 9194f3a; spec/functional/FR-031,FR-032,FR-034..FR-040, spec/non-functional/NFR-010,NFR-011, spec/stakeholder/StR-004, spec/usecase/US-010"
review_set: subset
---

## Summary

Ticket: PLAT-1605. EARS grammar of the new FR and NFR statements.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | low | Non-singular statement: one bullet carries two SHALL NOT obligations (no git process; no linked network transport). quire validate flags it as `ears:non-singular` at FR-034:54. Split it into two bullets. | spec/functional/FR-034-document-identity.md:54 |
| FND-002 | low | FR-039's Behavior bullets and FR-035 section C (Layout) are descriptive ('Every input is validated', 'The bundle limit is 256 inputs', 'The default template fixes the page size') with no SHALL. That leaves their obligation strength unclear next to the SHALL-marked FRs in the same PR. Restate them as `The command SHALL ...`. | spec/functional/FR-039-export-bundle.md:32-54, spec/functional/FR-035-default-export-template.md:54-59 |

## Examined scope

```yaml
scope:
- id: FR-034-git
  path: spec/functional/FR-034-document-identity.md
  role: examined
  excerpt: The command SHALL NOT start a `git` process and SHALL NOT link a network transport, so FR-032 stays the only source of child processes.
- id: FR-039-bullets
  path: spec/functional/FR-039-export-bundle.md
  role: examined
  excerpt: Every input is validated as FR-031 section A requires and is given a document identity as FR-034 defines ... The bundle identity is its title (`--bundle-title`), its id (`--bundle-id`) ... The bundle limit is 256 inputs.
- id: FR-035
  path: spec/functional/FR-035-default-export-template.md
  role: examined
  excerpt: 'The default template SHALL produce, in this order: 1. Title page: the title, then the identity fields of FR-034 as labelled lines (Document id, Version, Release when present, Source commit, Date), and nothing from the body. ... 2. Table of contents ... 3. Body ...'
- id: FR-038-all
  path: spec/functional/FR-038-approval-block.md
  role: examined
  excerpt: The command SHALL show every supplied verdict, whatever its status, in the order given on the command line, so a failed approval is never omitted. It SHALL label only the status `approved` as an approval; any other status SHALL show as written, with no wording that reads as an approval.
```

## Verdict

Minor only. The FR-034, FR-036, FR-037, FR-038 and FR-040 statements are ubiquitous or unwanted-behaviour SHALL / SHALL NOT forms with a named subject (`quire export`, 'the command').

## Dispositions

| FND | outcome | reason |
| --- | --- | --- |
| FND-001 | fixed | - |
| FND-002 | fixed | - |
