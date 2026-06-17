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
faithfully mirrors the upstream `quire-rs` types with no CLI-introduced fields.
The encoding rules are specified below.

## Behavior

The `parse` and `extract` subcommands SHALL emit JSON on stdout subject to the following rules:

1. **Compact form by default** — one line, no trailing newline beyond the final `\n`. Suitable for piping to `jq` or appending to JSONL files.
2. **`--pretty` flag** — when set, emits pretty-printed JSON with 2-space indentation. Same logical content, different whitespace.
3. **UTF-8 encoding only.** Non-UTF-8 bytes in source documents are rejected at parse time per upstream `quire-rs` [FR-005](./FR-005-path-safety.md).
4. **Stable field ordering** — `QuireDocument`, `ExtractionResult`, and `HarvestedEdge` field order in JSON output SHALL match the public Rust struct declaration order in `quire-rs`.
5. **No additional CLI-introduced fields.** The JSON faithfully mirrors the upstream types; no CLI-side metadata wrapping (no `_quire_cli_version`, no timestamps).

`extract` envelope:

```json
{
  "extraction": <ExtractionResult>,
  "edges":      [<HarvestedEdge>, ...]
}
```

This envelope is the only CLI-introduced structure; both inner values are emitted unmodified.

## Acceptance Criteria

| ID | Criteria | Verification |
|----|----------|--------------|
| FR-008-AC-1 | `quire parse doc.md` output round-trips through `serde_json::from_str::<QuireDocument>` (lib-side smoke test) | Test |
| FR-008-AC-2 | `quire extract doc.md --module $MOD` output deserializes into `{ "extraction": ExtractionResult, "edges": Vec<HarvestedEdge> }` | Test |
| FR-008-AC-3 | `quire parse doc.md --pretty` produces multi-line indented JSON with the same logical content as compact form | Test |
| FR-008-AC-4 | Byte-for-byte output of `parse` is identical across runs against the same input (determinism) | Test |
| FR-008-AC-5 | No CLI version string appears in JSON output | Test |

## Dependencies

- **Upstream**: [FR-002](./FR-002-parse-subcommand.md) parse, [FR-003](./FR-003-extract-subcommand.md) extract (producers of the JSON output).
- **Downstream**: `jq`/JSONL pipeline consumers of `parse`/`extract` output.
