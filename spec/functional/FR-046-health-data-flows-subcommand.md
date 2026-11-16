---
id: FR-046
title: "quire relations health-data-flows mode"
type: FR
relationships:
  - target: "ix://agent-ix/quire-cli/spec/functional/FR-030"
    type: "extends"
  - target: "ix://agent-ix/quire-cli/spec/functional/FR-044"
    type: "references"
  - target: "ix://agent-ix/spec-clauses-us-health/FR-005"
    type: "references"
  - target: "ix://agent-ix/spec-objects-architecture/FR-007"
    type: "references"
---
# FR-046: quire relations health-data-flows mode

## Description

The CLI SHALL expose the engine's `health-data-flows` record through
`quire relations --health-data-flows`, so a consumer can obtain health-classified
connection facts from the declared architecture corpus. The mode SHALL pass
the engine's record through unchanged. The engine selects flows only from
author-declared classifications, connection endpoints and trust-boundary
crossings; it does not determine whether a flow is legally permissible.

## Inputs

- The corpus root selected by `--scope` or the ordered corpus selected by `--repo`, as defined by [FR-030](./FR-030-relations-subcommand.md).
- Optional `--module` paths supplying the data-flow model declarations.

## Outputs

- A `health-data-flows` record with the canonical output metadata and a deterministically ordered list of complete health-classified flows.
- A deterministically ordered list of gaps for health-classified connections whose declared endpoint or boundary reference cannot be resolved.

## Behavior

- `quire relations --health-data-flows` SHALL accept `--scope`, repeated `--repo`, repeated `--module`, and `--json` with the meanings in [FR-030](./FR-030-relations-subcommand.md).
- The health-data-flows mode SHALL NOT require `traceability.flow_rules` or a flow-rule denial.
- When `--json` is present, stdout SHALL contain only the complete engine-produced document.
- The document metadata SHALL carry:
  - `name`: `health-data-flows`;
  - `kind`: `record`;
  - `description`: `Health data flow records; PLAT-1756 in the Data-Flow Model project owns the producer definition.`
- Each flow record SHALL identify its connection, source, destination, declared classification, and every resolved trust-boundary crossing.
- Each reference SHALL retain its repository identity when the input uses `--repo`.
- Each reference SHALL identify the source model artifact sufficiently to trace the value to that artifact.
- When a health-classified connection has a missing or unresolved source endpoint, destination endpoint, or explicitly declared crossing target, the engine SHALL produce a gap naming the connection and each missing reference.
- The engine SHALL NOT produce a complete flow entry with an inferred, substituted, or defaulted identifier for a missing or unresolved reference.
- A connection with no declared crossing SHALL have no crossing entry.
- A connection with no declared crossing SHALL NOT be treated as having a missing boundary.
- A connection is a health flow only when an author-declared classification on that connection includes `health` in `regulated_categories`.
- Other classifications SHALL not be promoted to health flows.
- The engine SHALL NOT derive classifications from source code, schema content, flow-rule outcomes, or legal criteria.
- Complete flow entries SHALL sort by repository identity and connection identifier.
- Each entry's boundary references SHALL sort by repository identity and boundary identifier.
- Gaps SHALL sort by repository identity and connection identifier.
- Missing-reference names SHALL use a stable lexical order.
- Human output SHALL provide one line per complete flow and per gap. A report containing gaps is still a successful report; a gap does not change the CLI exit status. Invalid corpus or module input follows [FR-030](./FR-030-relations-subcommand.md).
- The mode SHALL conflict with `--relation`, `--design`, and `--flow`, naming both conflicting modes and writing no report to stdout.
- The existing `--flow` output SHALL remain unchanged.
- The default `relations` output SHALL remain unchanged.
- The CLI SHALL pass the mode to quire-rs and render its result.
- The CLI SHALL NOT filter categories, resolve endpoints, inspect boundary declarations, infer missing facts, or decide permissibility.

## Acceptance Criteria

| ID | Criteria | Verification |
|----|----------|--------------|
| FR-046-AC-1 | `quire relations --scope <DIR> --module $M --health-data-flows --json` over a model with one health-classified connection, resolved source and destination, and one declared crossing emits one complete flow entry naming those model references and the crossing; the output metadata exactly names `health-data-flows` and kind `record`. | Test |
| FR-046-AC-2 | A health-classified connection with a missing source, missing destination, or dangling declared crossing emits a gap naming the connection and the missing reference, emits no complete entry for that connection, and substitutes no identifier. A connection with no declared crossing emits no boundary gap. | Test |
| FR-046-AC-3 | A connection carrying a declared category other than `health` is absent from the complete flow entries and gaps, even when another category is regulated. | Test |
| FR-046-AC-4 | Two runs over the same corpus produce byte-identical JSON. With multiple repositories, references retain each `--repo` identity and records, crossings, and gaps follow the specified order. | Test |
| FR-046-AC-5 | The emitted metadata's name, kind, and description exactly match the `health-data-flows` row in US pack FR-005, and the pack's catalog test accepts those values. | Test |
| FR-046-AC-6 | A module set declaring the architecture data-flow model but no `flow_rules` can produce a record; a corpus with no health-classified connections succeeds with empty flow and gap lists. | Test |
| FR-046-AC-7 | A request combining `--health-data-flows` with `--relation`, `--design`, or `--flow` exits 3, names the conflicting modes, and writes no report to stdout; requests in existing relations modes retain their prior outputs. | Test |
| FR-046-AC-8 | The CLI delegates selection and record construction to quire-rs: its source contains no category filtering, endpoint or boundary resolution, missing-fact inference, or legal-permissibility decision for this mode. | Inspection |

## Dependencies

- **Upstream**: [FR-030](./FR-030-relations-subcommand.md), [FR-044](./FR-044-relations-flow-mode.md), architecture [FR-007](ix://agent-ix/spec-objects-architecture/FR-007), and the `health-data-flows` output metadata in US pack [FR-005](ix://agent-ix/spec-clauses-us-health/FR-005).
- **Consumer**: the US Health-Law Compliance Pack consumes this record and validates its metadata against its output catalog; the CLI does not discharge or evaluate HIPAA clauses.
