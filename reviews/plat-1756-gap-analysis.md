---
id: SR-20261116045010
title: gap-analysis review — PLAT-1756 quire-cli
type: SpecReview
analysis: gap-analysis
scope: agent-ix/quire-cli; spec/functional/FR-046-health-data-flows-subcommand.md;
  src/commands/relations.rs; tests/cli_health_data_flows.rs; context ix://agent-ix/quire-rs/spec/functional/FR-084
  and ix://agent-ix/spec-objects-architecture/spec/functional/FR-007
review_set: subset
---

## Summary

The computed matrix binds tests to FR-046 AC-1 through AC-9 and treats AC-10 as inspection-only. Review of the tagged tests found incomplete evidence for the exact nested wire-shape and ordering requirements. Plan completion was not assessed; optional semantic review was not requested.

## Verdict

**CONDITIONAL** — two medium evidence gaps remain.

## Scope

- Examined: FR-046-AC-1 through FR-046-AC-10; changed relation projection and CLI integration tests.
- Context only: quire-rs FR-084 record omission contract and architecture FR-007 required connection endpoints.
- Ticket: PLAT-1756; PR #212; reviewed published head `39ea93741fd79f2a52bd12d562861fdc6af2e3f6`.
- Applicable AssuranceProfile: none found in `spec/`.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | medium | The test checks the top-level schema value, exact output object, selected flow values, and key order for the envelope and flow. It does not assert exact key sets, order, types, or nullability for ModelDocument, DeclaredReference, crossings, gaps, and missing-reference objects, so those closed-shape requirements can regress without this AC test failing. | tests/cli_health_data_flows.rs:195-242 |
| FND-002 | medium | The determinism test emits one flow and one crossing, and the multi-repository test emits one flow with one reference of each kind. Those cases cannot fail when array sorting is removed or reversed, so the specified ordering across multiple flows, classifications, crossings, gaps, and missing references is not verified. | tests/cli_health_data_flows.rs:176-250,252-303 |

## Evidence

`quoin matrix --repo . --json` reports test binders for AC-1 through AC-9 and `method-without-symbol` for inspection-only AC-10. `quire matrix --scope . --format json` reports the same test bindings. The coverage gaps below are narrower than the marker coverage: the tests do not assert the full nested shape for AC-2 or exercise multiple values to verify AC-5 ordering. Reverse-gap and stub scan found no other unowned behavior or stubs. The full workspace tests, formatter, Clippy, rustdoc, and dist-test passed; remaining make-ci gates (deny, unsafe comments, thin boundary, and spec traceability) passed when rerun with a writable task-local Cargo home where needed.

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

## Coverage

The matrix and tests were inspected for all ten criteria. `Plan completion: not assessed`. Semantic review was skipped because it was not requested.

## New findings (round 1)

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-003 | medium | AC-3 explicitly requires independently observed crossing gaps to remain in a gap when FR-084 omits the selected connection record. The implementation preserves those items, but the unavailable-record test supplies no crossing, so a regression that drops crossing gaps on this path would pass. | tests/cli_health_data_flows.rs:693-750 |

## Dispositions

Round 1, published head 89cd7594e6accae571b8209f10bfe0f9525637b1.

| FND | outcome | reason |
| --- | --- | --- |
| FND-001 | fixed | `emits_the_canonical_health_flow_envelope_from_generic_relation_facts` now asserts closed, ordered object shapes for the envelope, output, flow, ModelDocument, DeclaredReference, crossing and boundary, including field types and nullable repository identity. |
| FND-002 | fixed | `sorts_multiple_flows_classifications_crossings_and_missing_references` now exercises reverse-authored multiple flows, classifications, crossings, gaps and missing references, asserts lexical output order, and compares repeat output byte-for-byte. |
| FND-003 | still-open | The unavailable-record test checks only `connection_record` by itself. Add an unavailable selected record and a dangling crossing in the same fixture, and assert both missing items remain in the emitted gap. The production code currently retains that crossing item, but no test protects the explicit AC-3 combination. |


Round 2, published head d47d870ed714206ebf5451cf462361506ec4e407.

| FND | outcome | reason |
| --- | --- | --- |
| FND-001 | fixed | The canonical output test continues to assert closed ordered shapes for the nested model and reference objects. |
| FND-002 | fixed | The multi-value test continues to assert ordering across multiple flows, classifications, crossings, gaps, and missing references, plus byte-identical repeated output. |
| FND-003 | fixed | The added invalid-record case includes a dangling boundary crossing; it asserts exactly two missing items (`connection_record` with null reference and the `crossing` reference to BOUNDARY-1 authored by CONN-1). The focused test passed. |
