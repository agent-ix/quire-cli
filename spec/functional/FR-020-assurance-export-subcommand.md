---
id: FR-020
title: "quire assurance subcommand"
type: FR
relationships:
  - target: "ix://agent-ix/quire-cli/spec/usecase/US-006"
    type: "implements"
    cardinality: "1:1"
  - target: "ix://agent-ix/quire-cli/spec/stakeholder/StR-004"
    type: "implements"
    cardinality: "1:1"
  - target: "ix://agent-ix/quire-rs/spec/functional/FR-067"
    type: "implements"
    cardinality: "1:1"
  - target: "ix://agent-ix/quire-rs/spec/functional/FR-068"
    type: "implements"
    cardinality: "1:1"
---

## Description

The CLI SHALL provide an `assurance` subcommand that exposes quire-rs's
implemented `assurance-v1` export through a thin offline process boundary.

```text
quire assurance --scope <DIR> --module <PATH>
                --expect-module <NAME@VERSION>
                [--expect-schema <MODULE/ARCHETYPE>]...
```

The exact module and expected premises are mandatory. An explicit empty
`--expect-schema` set means that the expected module has no active archetype;
it does not disable schema checking.

## Inputs

- `--scope` is the repository root. Documents are loaded only from
  `<scope>/spec`; source symbols are extracted from `<scope>` with `spec/` and
  the module-declared `source_exclude` globs excluded.
- `--module` names one exact module directory containing `manifest.yaml`; the
  command performs no environment/default module discovery and no lazy
  installation.
- `--expect-module` names the only accepted module and semantic version.
- Each `--expect-schema` names one accepted active archetype.
  The supplied set is compared exactly with the set emitted by quire-rs.

## Behavior

1. The command applies the existing path-safety guards, loads the exact module,
   and derives the existing two roots from `--scope`.
2. It constructs `Spec`, `SymbolExtraction`, and `SymbolGraph` only through the
   authoritative quire-rs loaders and binder. It then calls
   `build_assurance_export`; no CLI-owned artifact, obligation, symbol,
   relation, availability, or freshness logic is permitted. A module without a
   `traceability:` model is valid: static artifacts and symbols are exported,
   while obligations and `verifies`/`implements` relations are empty.
3. It validates the completed bytes with `read_assurance_export` and the
   caller's accepted module/schema premises, and separately requires the
   accepted set to equal the emitted set so an unused extra premise cannot pass.
4. Only after construction, upstream schema validation, and exact-premise
   comparison succeed does it write JSON to stdout. Compact output is the exact
   `AssuranceExport::to_json_bytes()` value plus one newline. Global `--pretty`
   parses and re-indents those validated compact bytes, changes whitespace only,
   and remains deterministic.
5. Registry, corpus, and symbol-extraction diagnostics use the established
   stderr channel. An error exits non-zero with empty stdout; a successful
   export may legitimately contain empty record arrays and still exits zero
   with a complete envelope.

## Supported producer and consumer releases

The assurance payload's `format_version` alone does not establish semantic
compatibility. The following released producer/consumer pairs are the
compatibility boundary for this command:

| Producer | Consumer | Contract shape | Support |
|---|---|---|---|
| quire-rs 0.46.0 at `a874fb641cb70da83c8c8b23f9fea0a44255b88a` | quire-cli 0.32.0 | Earlier `assurance-v1`: source premise, schema digests, locator digests, statement hashes, hashed symbol identities, and relation provenance | Historical fixture only; not accepted by current quire-cli |
| quire-rs 0.47.1 at `92dbebc49f354f7a3d5050b94cd2d57d9084d99f` | quire-cli 0.33.0 | Earlier `assurance-v1` with the same schema and field meanings as the 0.46.0 pair | Historical fixture only; not accepted by current quire-cli |
| quire-rs 0.62.2 (`v0.62.2`) | quire-cli 0.52.0 | Current `assurance-v1`: no source premise or digest fields, structured symbol identities, no relation provenance, and shell/entry-point symbol forms | Supported by the current release |

These release identities come from the versioned CLI release manifests and
lockfiles and the exact quire-rs schema at each pinned producer revision. The
two historical producer revisions publish the same schema bytes; the current
producer revision publishes a different schema while retaining the v1 format
marker.

The current CLI SHALL validate the entire payload against its linked
engine's published schema and SHALL preserve the current schema's field
identities and meanings. It SHALL NOT infer compatibility from the shared
`assurance_export.v1` capability or `format_version: 1` alone. A payload from a
producer shape outside the current consumer's supported row SHALL be refused
before any typed record is returned; the consumer SHALL NOT discard or
reinterpret fields to make an unsupported shape fit. Adding or changing a
field's meaning requires a new format version and a corresponding explicit
consumer-support decision before that producer/consumer pair is supported.

## Constraints

| ID | Constraint | Type | Validation |
|----|------------|------|------------|
| FR-020-CON-2 | The command SHALL execute no test, proof, solver, consumer, package-manager, Git, or network command. It performs parsing and static source extraction only. | Responsibility | Test |
| FR-020-CON-3 | The CLI SHALL add no verdict, execution result, evidence freshness claim, generic evidence envelope, or tool-provenance field to the closed upstream payload. | Responsibility | Inspection |
| FR-020-CON-4 | Unknown or malformed module versions, schema premises, and incomplete module loads SHALL fail closed before any stdout byte. | Integrity | Test |
| FR-020-CON-5 | The CLI SHALL support a producer/consumer pair only when its complete fixture preserves every required field identity and semantic meaning; matching a capability string or `format_version` is insufficient, and unsupported shapes are refused without dropping or reinterpreting fields. | Compatibility | Test |

## Acceptance Criteria

| ID | Criteria | Verification |
|----|----------|--------------|
| FR-020-AC-1 | A pinned fixture containing artifacts, obligations, symbols, resolved and dangling corpus relations, `verifies` and `implements` bindings, locators, relation-kind capabilities, and `available`, `missing`, `not_applicable`, and `unknown` observations emits a complete `quire-assurance` v1 document that validates against `quire_rs::assurance::ASSURANCE_V1_SCHEMA`. | Test |
| FR-020-AC-2 | Two compact runs over identical fixture bytes and arguments produce byte-identical stdout; `--pretty` changes only whitespace and is independently byte-identical across runs. | Test |
| FR-020-AC-3 | A mismatched module name/version, missing or extra schema premise, malformed premise syntax, or unnamed/unversioned module exits non-zero with empty stdout and a diagnostic naming the refused premise. | Test |
| FR-020-AC-4 | A valid corpus with zero artifacts, obligations, symbols, or relations still emits the complete successful envelope and exits zero. A module without a `traceability:` model exports its static artifacts and symbols with empty obligations and no `verifies`/`implements` relations. A document the corpus walker cannot read remains a successful export with an `unknown` relation observation and non-empty reason as quire-rs FR-068 requires; a missing root, invalid module premise, or export-wide upstream error exits non-zero with empty stdout. | Test |
| FR-020-AC-5 | Module-loader and symbol-extraction diagnostics are emitted on stderr in human or JSON diagnostic form and never enter the assurance payload. | Test |
| FR-020-AC-6 | The command delegates construction to `build_assurance_export`, validation to `read_assurance_export`, corpus loading to `Spec`, extraction to `extract_tree_scoped`, and binding to `trace::bind`; a static boundary audit rejects a second graph, schema, or direct parser in the CLI. | Inspection |
| FR-020-AC-7 | The command opens no network socket and spawns no child process on success or refusal paths. | Test |
| FR-020-AC-8 | The checked-in golden JSON fixture validates against the upstream schema in Rust and independently pins every v1 field and state token. | Test |
| FR-020-AC-9 | Independent fixtures captured from the quire-cli 0.32.0 / quire-rs 0.46.0 pair, the quire-cli 0.33.0 / quire-rs 0.47.1 pair, and the current quire-cli 0.52.0 / quire-rs 0.62.2 pair validate under their respective published schemas and preserve the full field-identity oracle for modules, artifacts, obligations, symbols, relations, locators, and observations; the current pair's fixture is accepted by the current CLI, while each historical fixture is refused by the current CLI before typed records are returned. The current CLI refuses a fixture when a shared capability or format-version string is present but required fields or identity meanings differ. Verify with `tests/cli_assurance.rs` and `tests/assurance_cross_language.rs`. | Test |

## Dependencies

- **Upstream**: quire-rs [FR-067](ix://agent-ix/quire-rs/FR-067) and
  [FR-068](ix://agent-ix/quire-rs/FR-068).
- **Ownership gate**: `agent-ix/engineering-assurance#5`, accepted before this
  command was specified.
- **Downstream**: `agent-ix/quoin#322` and the common compatibility fixtures in
  `agent-ix/engineering-assurance#9` consume this static export; they own
  retention, audit, attestations, and verdicts.
