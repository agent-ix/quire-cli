---
id: SR-20261116045009
title: code-review review — PLAT-1756 quire-cli
type: SpecReview
analysis: code-review
scope: agent-ix/quire-cli; src/commands/relations.rs; tests/cli_health_data_flows.rs;
  FR-046; context ix://agent-ix/quire-rs/spec/functional/FR-084 and ix://agent-ix/spec-objects-architecture/spec/functional/FR-007
review_set: subset
---

## Summary

Reviewed the Rust CLI projection against merged FR-046 and the generic quire-rs FR-084 record contract. One concrete failure remains for a partially missing required connection endpoint because FR-084 suppresses the whole invalid record.

## Verdict

**FAIL** — a medium-severity projection defect remains.

## Scope

- Examined: FR-046-AC-1 through FR-046-AC-10; changed relation projection and CLI integration tests.
- Context only: quire-rs FR-084 record omission contract and architecture FR-007 required connection endpoints.
- Ticket: PLAT-1756; PR #212; reviewed published head `39ea93741fd79f2a52bd12d562861fdc6af2e3f6`.
- Applicable AssuranceProfile: none found in `spec/`.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | medium | When a real connection has only `targetEnd` authored, FR-084 omits the entire selected record because the Connection schema requires both endpoints; this projection then emits both `source` and `destination` as missing with null references. The test fixture makes both endpoint properties optional, so it misses the false gap for the authored destination. | src/commands/relations.rs:289-300,359-369 |

## Evidence

FR-084 says `record` is omitted when record validation fails. The architecture Connection schema requires both `sourceEnd` and `targetEnd` (and `flowDirection`). I reproduced this with only `targetEnd` authored: generic `quire relations --json` omitted the connection record, and `--health-data-flows --json` returned missing `destination` and `source`, both with null references. The new fixture Connection schema at `tests/cli_health_data_flows.rs:49-57` omits a `required` array, so it accepts partial records and cannot reveal this behavior. Source code uses only FR-084 report fields and does not read corpus documents; the other inspected projection paths match FR-046 shape and selection. Gates: fmt-check, all-target/all-feature Clippy, workspace tests, rustdoc, dist-test, deny, audit-unsafe, audit-thin-boundary, and spec traceability passed. The initial `make ci` stopped at cargo-deny because the default advisory lock is in a read-only path; deny and later gates passed using a task-owned writable Cargo home. `cargo audit` passed with one repository-allowed `RUSTSEC-2026-0486` warning for rustix 1.1.4.

## Examined Units

- `FR-046-AC-1` (examined): `quire relations --scope <DIR> --module $M --health-data-flows --json` over a fixture module with `required_relations` for connection-to-classification and connection-to-trust-boundary facts, and `record_keys` selecting `sourceEnd`, `targetEnd`, `regulatedCategories`, and `trust_level`, emits a complete JSON envelope. The fixture model has one health-classified connection with source and destination record values and one declared boundary crossing.
- `FR-046-AC-2` (examined): The AC-1 envelope's `schema`, `output`, `flows`, `gaps`, `ModelDocument`, `DeclaredReference`, flow, crossing, and gap keys have the exact names, order, types, nullability, and closed shapes specified in this requirement; each reference carries repository identity and a source artifact path or `declared_by` document.
- `FR-046-AC-3` (examined): A health-classified connection with a missing source value, missing destination value, dangling crossing, or boundary without `trust_level` emits a gap naming the connection and missing reference, emits no complete entry for that connection, and substitutes no identifier or value. A connection with no declared crossing emits no boundary gap.
- `FR-046-AC-4` (examined): A connection carrying a declared category other than `health` is absent from the complete flow entries and gaps, even when another category is regulated.
- `FR-046-AC-5` (examined): Two runs over the same corpus produce byte-identical JSON. With multiple repositories, references retain each `--repo` identity and records, crossings, and gaps follow the specified order.
- `FR-046-AC-6` (examined): The emitted metadata's name, kind, and description exactly match the `health-data-flows` row in US pack FR-005, and the pack's catalog test accepts those values.
- `FR-046-AC-7` (examined): A module set with the required `required_relations` and record keys but no `flow_rules` can produce a record; a corpus with no health-classified connections succeeds with empty `flows` and `gaps`.
- `FR-046-AC-8` (examined): With gaps, default mode emits the full report and exits 0; with `--strict`, it emits the full report and exits 1. With no gaps, both modes emit the full report and exit 0, regardless of generic relation verification status or flow-report denials.
- `FR-046-AC-9` (examined): A request combining `--health-data-flows` with `--relation`, `--design`, or `--flow` exits 3, names the conflicting modes, and writes no report to stdout; requests in existing relations modes retain their prior outputs.
- `FR-046-AC-10` (examined): The CLI derives health selection and record structure only from the generic FR-084 relation report's `document`, `linked`, `dangling`, and selected `record` values; it reads no source files and does not alter the report used by `--flow`.
- `FR-084` (context_only): the generic relation report omits `record` when record validation fails.
- `FR-007` (context_only): a connection requires `sourceEnd`, `targetEnd`, and `flowDirection`.

## Dispositions

Round 1, published head 89cd7594e6accae571b8209f10bfe0f9525637b1.

| FND | outcome | reason |
| --- | --- | --- |
| FND-001 | fixed | `project_health_data_flows` now detects a missing selected connection record from a health-classified relation, removes guessed endpoint gaps, and emits one `connection_record` item with a null reference. `an_invalid_connection_record_reports_unavailability_without_guessing_endpoints` uses a schema requiring both endpoints and flowDirection, then verifies the unavailable-record output for each missing endpoint. |


Round 2, published head d47d870ed714206ebf5451cf462361506ec4e407.

| FND | outcome | reason |
| --- | --- | --- |
| FND-001 | fixed | The production projection is unchanged from the round-1 fix; the required-connection-schema test still asserts the unavailable record produces one `connection_record` item and no guessed endpoint. Focused integration test passed at this head. |
