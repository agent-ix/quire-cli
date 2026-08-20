---
id: FR-003
title: "quire extract subcommand"
type: FR
object_type: cli_command
relationships:
  - target: "ix://agent-ix/quire-cli/spec/usecase/US-004"
    type: "implements"
    cardinality: "1:1"
  - target: "ix://agent-ix/quire-rs/spec/functional/FR-011"
    type: "consumes"
    cardinality: "1:1"
  - target: "ix://agent-ix/quire-rs/spec/functional/FR-015"
    type: "consumes"
    cardinality: "1:1"
---

> **CR note (`type` discriminator rename, 2026-06-16):** OKF adoption renamed the
> archetype discriminator frontmatter key from `artifact_type` to `type`. Step 4
> below and FR-003-AC-2 now read `type`. The implementation reads it via
> `quire_rs::concept_type(&parsed)`.

> **CR note (extract diagnostic consistency, 2026-06-16):** `quire extract` on a
> document with **no `type`** now emits the same `[frontmatter]`-reason
> `ValidationError`-shaped diagnostic on stderr (then exits 1) that `validate`
> emits, instead of a generic anyhow error — aligning extract's vocabulary with
> `validate` ([FR-004](./FR-004-validate-subcommand.md) / [FR-014](./FR-014-validate-okf-bundle.md) §B base-concept contract). See FR-003-AC-5. Verified
> by `tests/cli_okf.rs::okf_untyped_document_is_error` for the shared `[frontmatter]`
> vocabulary.

> **CR note (sugar-field harvesting retired, 2026-08-20):** FR-003-AC-3 required
> `extract` to harvest frontmatter *sugar fields* (`dependencies:`,
> `supersedes:`, …) into typed edges. **No engine ever implemented this.**
> `quire_rs::harvest_edges` (`src/corpus/resolve.rs`) reads exactly two sources —
> the frontmatter `relationships:` array and `ix://` links in the body — and
> `dependencies` appears nowhere in the quire-rs sources. The concept dates to
> the FR-001 render era retired under spec.md §2bis. AC-3 and its trace IT-016
> are therefore **retired**, not deferred: nothing regressed, and the matrix row
> claiming ✅ was reporting evidence that never existed
> (agent-ix/quire-cli#43). Harvesting sugar fields remains a legitimate feature
> request against quire-rs `harvest_edges`; it would arrive as a new AC with a
> new trace, never by reviving this one.

## Description

The CLI SHALL expose an `extract` subcommand that parses a markdown document,
runs the module's body-extraction DSL, and emits the extraction result plus
harvested cross-reference edges as JSON on stdout, delegating all extraction to
`quire-rs`. The behavioral surface is specified below.

## Behavior

The CLI SHALL expose an `extract` subcommand:

```
quire extract <DOC|-> --module <PATH>
```

Required:
- `<DOC|->` — positional, `.md` path or `-` for stdin.
- `--module <PATH>` — Module root containing the body-extraction DSL declarations.

Behavior:
1. Path-safety on `<DOC>` and `--module`.
2. Load Registry from `--module`.
3. Parse document via `quire_rs::parse_document`.
4. Look up the document's archetype kind (from frontmatter `type` field or equivalent declared in the manifest).
5. Run `quire_rs::extract(&compiled_archetype, &doc)` and `quire_rs::harvest_edges(&doc, &compiled_archetype)`.
6. Emit on stdout a single JSON object:

   ```json
   {
     "extraction": <ExtractionResult JSON>,
     "edges": [<HarvestedEdge>, ...]
   }
   ```

7. Edges are deduped by `(source, type, target)` per upstream [FR-015](./FR-015-fix-subcommand.md).

The CLI SHALL NOT mutate, normalize, or filter the upstream extraction or edge output beyond the dedup contract guaranteed by `quire-rs`.

## Acceptance Criteria

| ID | Criteria | Verification |
|----|----------|--------------|
| FR-003-AC-1 | `quire extract sample-fr.md --module $ISO` against a fixture document produces JSON with non-empty `extraction` and at least one edge | Test |
| FR-003-AC-2 | For a document whose frontmatter declares a `type` not present in the module, exit 1 with `UnknownArchetype` on stderr; stdout is empty | Test |
| FR-003-AC-3 | ⛔ RETIRED (CR, 2026-08-20) — was: for a document with frontmatter sugar fields, `.edges` contains a `dependencies`-typed edge. No engine ever harvested sugar fields; see the CR note above | Retired |
| FR-003-AC-4 | Re-running extract on the same input produces byte-identical stdout (determinism, matches upstream) | Test |
| FR-003-AC-5 | For a document with **no `type`** (the discriminator absent from frontmatter) and no `--archetype`, `extract` exits 1 with a `[frontmatter]`-reason `ValidationError`-shaped diagnostic on stderr naming the missing `type` (the same vocabulary `validate` uses), not a generic anyhow error; stdout is empty | Test |

## Dependencies

- **Upstream**: [US-004](../usecase/US-004-cross-reference-extraction.md) cross-reference extraction; quire-rs [FR-011](ix://agent-ix/quire-rs/FR-011), [FR-015](./FR-015-fix-subcommand.md) (extract + edge harvest APIs).
- **Downstream**: graph-ingestion consumers of the `{extraction, edges}` envelope.
