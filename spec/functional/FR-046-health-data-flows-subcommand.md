---
id: FR-046
title: "quire relations health-data-flows mode"
type: FR
relationships:
  - target: "ix://agent-ix/quire-cli/spec/functional/FR-030"
    type: "extends"
  - target: "ix://agent-ix/quire-cli/spec/functional/FR-044"
    type: "references"
  - target: "ix://agent-ix/quire-rs/spec/functional/FR-084"
    type: "references"
  - target: "ix://agent-ix/quire-rs/spec/functional/FR-096"
    type: "references"
  - target: "ix://agent-ix/spec-clauses-us-health/FR-005"
    type: "references"
  - target: "ix://agent-ix/spec-objects-architecture/FR-007"
    type: "references"
---
# FR-046: quire relations health-data-flows mode

## Description

The CLI SHALL expose a `health-data-flows` record through
`quire relations --health-data-flows`, so a consumer can obtain health-classified
connection facts from the declared architecture corpus. The CLI SHALL build the
health-specific projection from the generic relation facts in quire-rs
[FR-084](ix://agent-ix/quire-rs/spec/functional/FR-084); it SHALL NOT modify or
specialize the engine report. The engine report supplies only author-declared
model facts. The CLI does not determine whether a flow is legally permissible.

## Inputs

- The corpus root selected by `--scope` or the ordered corpus selected by `--repo`, as defined by [FR-030](./FR-030-relations-subcommand.md).
- Optional `--module` paths supplying `traceability.required_relations` and their selected `record_keys`.

## Outputs

- A complete JSON envelope with schema id `quire.health-data-flows/v1`, canonical output metadata, a deterministically ordered `flows` array, and a deterministically ordered `gaps` array.

## Behavior

- `quire relations --health-data-flows` SHALL accept `--scope`, repeated `--repo`, repeated `--module`, `--json`, and `--strict` with the meanings in [FR-030](./FR-030-relations-subcommand.md).
- The mode SHALL use the generic `relations` section from the quire-rs FR-084 report.
- The mode SHALL NOT require `traceability.flow_rules` or a flow-rule denial.
- The mode SHALL read connection endpoint values only from `sourceEnd` and `targetEnd` selected by the loaded connection relation's `record_keys`.
- The mode SHALL read health categories only from the related classification's selected `regulatedCategories` record value.
- The mode SHALL read crossings only from related trust-boundary entries and their selected `trust_level` record value.
- The mode SHALL ignore the optional `flow` section appended under quire-rs FR-096, leaving it unaltered for `--flow` callers.
- When `--json` is present, stdout SHALL contain only the complete CLI-projected document.
- With `--json`, the envelope SHALL carry keys in this order: `schema`, `output`, `flows`, `gaps`.
- The `schema` value SHALL equal `quire.health-data-flows/v1`.
- `output` SHALL be exactly `{"name":"health-data-flows","kind":"record","description":"Health-data flow records; PLAT-1756 in the Data-Flow Model project owns the producer definition."}`.
- The CLI SHALL emit the envelope, `output`, every flow, crossing, gap, missing-reference object, `ModelDocument`, and `DeclaredReference` as closed objects with no additional keys.
- A `ModelDocument` value SHALL have exactly `repo`, `id`, `archetype`, and `path`, in that order.
- The `repo` field in `ModelDocument` SHALL be a string containing the `<ORG/REPO>` identity or `null` for `--scope`.
- The `id`, `archetype`, and `path` fields in `ModelDocument` SHALL be strings.
- The `path` field in `ModelDocument` SHALL be relative to that corpus member's root.
- A `DeclaredReference` value SHALL have exactly `repo`, `id`, `archetype`, and `declared_by`, in that order.
- The `repo` field in `DeclaredReference` SHALL be a string containing the target member identity or `null` for `--scope`.
- The `id` and `archetype` fields in `DeclaredReference` SHALL be strings.
- The `declared_by` field in `DeclaredReference` SHALL be a `ModelDocument` for the artifact whose relation or record field authored the reference.
- Each `flows` item SHALL have exactly `connection`, `source`, `destination`, `classifications`, and `crossings`, in that order.
- The `connection` field in a flow SHALL be a `ModelDocument`.
- The `source` and `destination` fields in a flow SHALL be `DeclaredReference` values.
- The `classifications` field in a flow SHALL be an array of `ModelDocument` values.
- The `crossings` field in a flow SHALL be an array of objects with exactly `boundary` followed by `trust_level`.
- The `boundary` field in a crossing SHALL be a `ModelDocument`.
- The `trust_level` field in a crossing SHALL be a string.
- Each complete flow SHALL represent one connection whose related classification record states `regulatedCategories` containing `health`.
- The `source` and `destination` references in a flow SHALL come from the connection's declared `sourceEnd.type` and `targetEnd.type` record values.
- The `crossings` array in a flow SHALL include every resolved related trust boundary and its declared `trust_level` value.
- Each `gaps` item SHALL have exactly `connection` followed by `missing`.
- The `connection` field in a gap SHALL be a `ModelDocument`.
- The `missing` field in a gap SHALL be a non-empty array.
- Each missing-reference item SHALL have exactly `field` followed by `reference`.
- The `field` value in a missing-reference item SHALL be a string equal to one of `connection_record`, `source`, `destination`, `crossing`, or `trust_level`.
- The `reference` value in a missing-reference item SHALL be a `DeclaredReference` or `null` when the corresponding reference identifier is absent or the selected connection record is unavailable.
- When FR-084 omits a health-classified connection's selected record, the CLI SHALL emit a missing-reference item with `field` equal to `connection_record` and `reference` equal to `null`.
- When FR-084 omits a health-classified connection's selected record, the CLI SHALL NOT report either endpoint as missing because the report does not establish which endpoint value is absent.
- When the selected connection record is present, the CLI SHALL emit a gap for a missing endpoint record value, an explicitly declared dangling crossing, or a resolved boundary that lacks `trust_level`.
- A connection with no declared crossing SHALL have no crossing entry.
- A connection with no declared crossing SHALL NOT be treated as a missing boundary.
- The CLI SHALL NOT emit a complete flow entry for a connection with any missing endpoint or boundary fact.
- The CLI SHALL NOT infer, substitute, or default a missing identifier or value.
- A connection is a health flow only when a related, author-declared classification record includes `health` in `regulatedCategories`.
- Other classifications SHALL not be promoted to health flows.
- The CLI SHALL NOT derive classifications from source code, schema content, flow-rule outcomes, or legal criteria.
- Complete flow entries SHALL sort by connection `repo` then `id`.
- Classification arrays SHALL sort by target `repo` then `id`.
- Crossing arrays SHALL sort by boundary `repo` then `id`.
- Gaps SHALL sort by connection `repo` then `id`.
- Each `missing` array SHALL sort by `field`, then target `repo` and `id`.
- The `flows`, `gaps`, `classifications`, and `crossings` arrays SHALL always be present, including when empty.
- The `missing` array SHALL be present and non-empty in every gap.
- When sorting repository identities, the CLI SHALL sort `null` before any string identity.
- When sorting strings and identifiers, the CLI SHALL sort them lexically.
- The envelope SHALL contain no timestamps, host paths, or run-specific values.
- Human output SHALL provide one line per complete flow.
- Human output SHALL provide one line per gap.
- A report containing gaps SHALL still be emitted completely.
- By default, gaps SHALL not change the exit status.
- With `--strict`, the CLI SHALL emit the complete report and exit 1 when `gaps` is non-empty.
- With `--strict`, the CLI SHALL emit the complete report and exit 0 when `gaps` is empty.
- Flow-report denials SHALL not affect this exit rule.
- Relation verification status SHALL not affect this exit rule.
- Invalid corpus or module input follows [FR-030](./FR-030-relations-subcommand.md).
- The mode SHALL conflict with `--relation`, `--design`, and `--flow`, naming both conflicting modes and writing no report to stdout.
- The existing `--flow` output SHALL remain unchanged.
- The default `relations` output SHALL remain unchanged.
- The CLI SHALL obtain the generic relation report from quire-rs and build this health projection from its `document`, `linked`, `dangling`, and selected `record` values.
- The CLI SHALL NOT read source files outside the report.
- The CLI SHALL NOT infer missing facts.
- The CLI SHALL NOT decide permissibility.

## Acceptance Criteria

| ID | Criteria | Verification |
|----|----------|--------------|
| FR-046-AC-1 | `quire relations --scope <DIR> --module $M --health-data-flows --json` over a fixture module with `required_relations` for connection-to-classification and connection-to-trust-boundary facts, and `record_keys` selecting `sourceEnd`, `targetEnd`, `regulatedCategories`, and `trust_level`, emits a complete JSON envelope. The fixture model has one health-classified connection with source and destination record values and one declared boundary crossing. | Test |
| FR-046-AC-2 | The AC-1 envelope's `schema`, `output`, `flows`, `gaps`, `ModelDocument`, `DeclaredReference`, flow, crossing, and gap keys have the exact names, order, types, nullability, and closed shapes specified in this requirement; each reference carries repository identity and a source artifact path or `declared_by` document. | Test |
| FR-046-AC-3 | A health-classified connection whose selected FR-084 record is omitted emits a `connection_record` missing item with a null reference, without claiming either endpoint is missing; independently observed crossing gaps remain present. If no other missing facts exist, that gap has exactly one missing item. When the selected record is present but only one endpoint value is missing, the gap names only that endpoint; a dangling crossing or boundary without `trust_level` names its missing reference. Each case emits no complete entry and substitutes no identifier or value. A connection with no declared crossing emits no boundary gap. | Test |
| FR-046-AC-4 | A connection carrying a declared category other than `health` is absent from the complete flow entries and gaps, even when another category is regulated. | Test |
| FR-046-AC-5 | Two runs over the same corpus produce byte-identical JSON. With multiple repositories, references retain each `--repo` identity and records, crossings, and gaps follow the specified order. | Test |
| FR-046-AC-6 | The emitted metadata's name, kind, and description exactly match the `health-data-flows` row in US pack FR-005, and the pack's catalog test accepts those values. | Test |
| FR-046-AC-7 | A module set with the required `required_relations` and record keys but no `flow_rules` can produce a record; a corpus with no health-classified connections succeeds with empty `flows` and `gaps`. | Test |
| FR-046-AC-8 | With gaps, default mode emits the full report and exits 0; with `--strict`, it emits the full report and exits 1. With no gaps, both modes emit the full report and exit 0, regardless of generic relation verification status or flow-report denials. | Test |
| FR-046-AC-9 | A request combining `--health-data-flows` with `--relation`, `--design`, or `--flow` exits 3, names the conflicting modes, and writes no report to stdout; requests in existing relations modes retain their prior outputs. | Test |
| FR-046-AC-10 | The CLI derives health selection and record structure only from the generic FR-084 relation report's `document`, `linked`, `dangling`, and selected `record` values; it reads no source files and does not alter the report used by `--flow`. | Inspection |

## Dependencies

- **Upstream**: [FR-030](./FR-030-relations-subcommand.md), [FR-044](./FR-044-relations-flow-mode.md), quire-rs [FR-084](ix://agent-ix/quire-rs/spec/functional/FR-084) for generic relation facts, architecture [FR-007](ix://agent-ix/spec-objects-architecture/FR-007), and the `health-data-flows` output metadata in US pack [FR-005](ix://agent-ix/spec-clauses-us-health/FR-005). The existing `connection-carries-classification` relation SHALL select `sourceEnd` and `targetEnd` in addition to `regulatedCategories`; the `connection-crosses-trust-boundary` relation SHALL select `trust_level`. Updating these `record_keys` is part of PLAT-1756 implementation and needs no new engine requirement. Quire-rs [FR-096](ix://agent-ix/quire-rs/spec/functional/FR-096) is referenced only to distinguish its optional flow section; this mode does not require `flow_rules`.
- **Consumer**: the US Health-Law Compliance Pack consumes this record and validates its metadata against its output catalog; the CLI does not discharge or evaluate HIPAA clauses.
