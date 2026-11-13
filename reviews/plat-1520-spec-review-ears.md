---
id: SR-20261112234814
title: PLAT-1520 quire-cli PR 180 spec-review/ears-conformance
type: SpecReview
analysis: ears-conformance
scope: agent-ix/quire-cli; PR 180; spec/functional/FR-030-relations-subcommand.md, spec/functional/index.md
review_set: subset
---

## Summary

Ticket: PLAT-1520. EARS conformance of the FR-030 statement and its behaviour clauses in quire-cli PR 180.

## Verdict

PASS. The FR statement is a well-formed ubiquitous requirement (`The CLI SHALL provide ...`). The behaviour clauses use the indicative mood (`--strict` exits 1 when ...) in the same event-driven shape as sibling FR-017/FR-026, with explicit trigger, response and actor; no unbounded or optional-feature clause lacks its condition.

## Findings

| ID | Severity | Summary | Refs |
|---|---|---|---|
| FND-001 | low | No findings (placeholder) | - |

## Examined units

```yaml
scope:
- id: FR-030
  path: spec/functional/FR-030-relations-subcommand.md
  role: examined
  excerpt: The CLI SHALL provide a `relations` subcommand that renders the two-way relation coverage report
    of quire-rs [FR-084](ix://agent-ix/quire-rs/FR-084) over one repository or over several repositories
    loaded as one corpus ([FR-085](ix://agent-ix/quire-rs/FR-085)).
- id: 'FR-030 Behavior: strict'
  path: spec/functional/FR-030-relations-subcommand.md
  role: examined
  excerpt: '- The command is report-only. With `--strict` it exits 1 when any row is `uncovered` or has
    `verification` `unverified` or `partial`; without it, such rows exit 0.'
- id: 'FR-030 Behavior: refusals'
  path: spec/functional/FR-030-relations-subcommand.md
  role: examined
  excerpt: '- A scope without `spec/`, a module set with no `required_relations`, and an unknown `--relation`
    name each exit non-zero naming the cause and compute nothing.'
- id: 'FR-030 Behavior: human output'
  path: spec/functional/FR-030-relations-subcommand.md
  role: examined
  excerpt: '- Human output goes to stdout: one block per relation, one line per row, leading with the
    document id, then `state`, `verification`, and the linked document ids with their own verification;
    an `uncovered` row is marked as a gap and names the relation''s `check` token. Dangling and kind-mismatch
    entries print beneath the row, naming the reference as authored.'
- id: 'FR-030 Description: boundary'
  path: spec/functional/FR-030-relations-subcommand.md
  role: examined
  excerpt: 'The subcommand is separate from `quire coverage` ([FR-017](./FR-017-coverage-subcommand.md)):
    that command''s `CoverageReport` payload is its stable interface and gains no row sets. The report
    computation, the row states and the verification statuses belong to quire-rs; the CLI resolves roots,
    applies path safety ([FR-005](./FR-005-path-safety.md)), and renders ([StR-004](../stakeholder/StR-004-thin-boundary-over-quire-rs.md)).
    It holds no archetype, verb or chain name: which rows exist comes from the loaded modules'' `traceability.required_relations`.'
```
