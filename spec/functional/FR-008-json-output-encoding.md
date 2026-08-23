---
id: FR-008
title: "JSON output encoding for parse and extract"
type: FR
object_type: dto
relationships:
  - target: "ix://agent-ix/quire-cli/spec/functional/FR-002"
    type: "consumes"
    cardinality: "1:1"
  - target: "ix://agent-ix/quire-cli/spec/functional/FR-003"
    type: "consumes"
    cardinality: "1:1"
---

## Description

The `parse` and `extract` subcommands SHALL emit deterministic, UTF-8, stably
ordered JSON on stdout — compact by default, pretty under `--pretty` — that
faithfully mirrors the upstream `quire-rs` types. The only CLI-introduced
structure is the envelope and the `engine` provenance block it carries (CR-104);
no upstream value is rewritten. The encoding rules are specified below.

## Behavior

The `parse` and `extract` subcommands SHALL emit JSON on stdout subject to the following rules:

1. **Compact form by default** — one line, no trailing newline beyond the final `\n`. Suitable for piping to `jq` or appending to JSONL files.
2. **`--pretty` flag** — when set, emits pretty-printed JSON with 2-space indentation. Same logical content, different whitespace.
3. **UTF-8 encoding only.** Non-UTF-8 bytes in source documents are rejected at parse time per upstream `quire-rs` [FR-005](./FR-005-path-safety.md).
4. **Stable field ordering** — `QuireDocument`, `ExtractionResult`, and `HarvestedEdge` field order in JSON output SHALL match the public Rust struct declaration order in `quire-rs`.
5. **No CLI-introduced fields on the engine's own values.** Every upstream value
   is emitted unmodified — no timestamps, no per-record annotations, and no key
   naming which revision of a shape a payload conforms to. Structure the CLI
   adds lives on the **envelope**, never inside what the engine produced.

`extract` envelope:

```json
{
  "extraction": <ExtractionResult>,
  "edges":      [<HarvestedEdge>, ...],
  "engine":     { "cli": "…", "engine": "…", "capabilities": ["…"] }
}
```

Both inner values are emitted unmodified.

### Instrument provenance is not a version key (CR-104)

> **CR-104** (#68, EPIC agent-ix/quire-rs#264 Wave 0) — *AC-5 banned "a CLI
> version string in JSON output". That ban conflated two different claims, and
> the one it did not mean to forbid is the one this CLI most needs to make.*
>
> **A contract version** answers *which schema describes this shape*. It belongs
> in the schema's `$id` (quire-rs FR-055-CON-2), never in the payload: a payload
> carrying one asserts its own conformance and puts the contract in two places
> that drift. **That ban is unchanged** — no bare `version`, no `schema_version`,
> no `$schema`, at any level.
>
> **Instrument provenance** answers *which build computed these numbers*, and
> its absence was measured. `quire --version` reported this crate's version
> while the engine is a git dependency pinned by tag in `Cargo.toml:20` that
> **no surface reported at all**. The installed CLI **0.29.0** pins engine
> **v0.42.0**; `binding_census` — the only signal saying whether the trace
> binder read a single test — landed in **v0.43.0**. Four battle-testing passes
> reported ecosystem figures from a binary that could not emit it, and nothing
> in the output said so. This is the same shape as #52, where tags 0.24.0–0.28.0
> shipped binaries reporting 0.23.0 and three SpecReviews cited a binary nobody
> checked; `quoin/scripts/check-version-agreement.mjs` exists because of that,
> and was never applied to the engine-inside-the-CLI seam.
>
> Upgrading the binary fixes today's instance. Carrying provenance on the
> payload fixes the class, because it survives being saved to disk — and a saved
> payload is what a later reader actually reasons from.
>
> `capabilities` is a **token list, not version arithmetic**. A consumer asserts
> it needs `binding_census`; it must not assert `engine >= 0.43.0`, because a
> version comparison in a consumer is a second place the contract lives. The
> vocabulary is open, so adding a token cannot break a consumer written against
> an older list. Each token names an engine surface this binary calls, so a
> build linking an engine that lacks one does not compile — the list cannot
> claim a capability the linked engine does not have.
>
> AC-5 is narrowed to the bare-version case it was arguing for; AC-6 gates the
> provenance block. quire-rs FR-055-CON-2 is narrowed in the same terms, and its
> two published schemas define the optional `engine` object.

## Acceptance Criteria

| ID | Criteria | Verification |
|----|----------|--------------|
| FR-008-AC-1 | `quire parse doc.md` output round-trips through `serde_json::from_str::<QuireDocument>` (lib-side smoke test) | Test |
| FR-008-AC-2 | `quire extract doc.md --module $MOD` output deserializes into `{ "extraction": ExtractionResult, "edges": Vec<HarvestedEdge> }` | Test |
| FR-008-AC-3 | `quire parse doc.md --pretty` produces multi-line indented JSON with the same logical content as compact form | Test |
| FR-008-AC-4 | Byte-for-byte output of `parse` is identical across runs against the same input (determinism) | Test |
| FR-008-AC-5 | No bare `version`, `schema_version` or `$schema` key appears at any level of JSON output: a payload never names the contract revision it claims to conform to (CR-104) | Test |
| FR-008-AC-6 | Every JSON payload — `coverage`, `properties`, `extract` — carries a top-level `engine` object naming the CLI version, the resolved engine version and a capability token list; `quire --version` reports both versions distinctly; the resolved engine version is read from the lockfile and a `-<n>-g<sha>` suffix is reported verbatim rather than rounded to the nearest tag (CR-104) | Test |

## Dependencies

- **Upstream**: [FR-002](./FR-002-parse-subcommand.md) parse, [FR-003](./FR-003-extract-subcommand.md) extract (producers of the JSON output).
- **Downstream**: `jq`/JSONL pipeline consumers of `parse`/`extract` output.
