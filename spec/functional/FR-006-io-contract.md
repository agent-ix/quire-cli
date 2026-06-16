---
id: FR-006
title: "Stdin / stdout / stderr contract"
type: FR
object_type: cli_command
relationships:
  - target: "ix://agent-ix/quire-cli/spec/stakeholder/StR-001"
    type: "implements"
    cardinality: "1:1"
  - target: "ix://agent-ix/quire-rs/spec/functional/FR-017"
    type: "consumes"
    cardinality: "1:1"
---

> **CR note (render removal — 2026-06-04):** The `render` row of the stream table
> and the `--data -` stdin trigger are removed (mirrors quire-rs render retirement,
> commit 500a3d3). The uniform I/O contract is otherwise unchanged and applies to the
> surviving subcommands. AC-4 is rephrased onto a stdin-reading surviving subcommand.

## Behavior

Every subcommand SHALL adhere to a uniform I/O contract:

| Stream | Content |
|--------|---------|
| stdout | The **primary result** of the subcommand: JSON document (`parse`), JSON `{extraction, edges}` (`extract`), JSON contract (`schema`), looked-up section/block (`lookup`), edited document (`edit`). `validate` writes nothing to stdout. |
| stderr | All diagnostics — informational, warning, and error — emitted in the structured format defined by `quire-rs` FR-017. Free-form text is permitted only for `clap` argument-parsing errors. |
| stdin | Used only when a positional `<DOC>` is `-`; read to EOF as a single unit. |

The CLI SHALL NOT interleave diagnostics with stdout content. Stdout is either:
- The successful primary result, written as one contiguous payload, or
- Empty (in failure cases).

This guarantees that downstream pipelines (`quire parse … | jq …`, `quire extract … | grep …`) see only well-formed output on success, and `2>/dev/null` cleanly suppresses diagnostics without affecting result correctness.

## Acceptance

- **FR-006-AC-1**: For each subcommand, a failure case produces empty stdout and non-empty stderr.
- **FR-006-AC-2**: For each subcommand, a success case produces non-empty stdout (except `validate`) and empty stderr (except for non-fatal advisories explicitly allowed by upstream `quire-rs` FRs).
- **FR-006-AC-3**: All structured stderr diagnostics are valid `quire-rs::Diagnostic` JSON when the `--diagnostics-format=json` flag is set (default is human-readable per upstream FR-017).
- **FR-006-AC-4**: `quire parse - --module $ISO <<< '---\nid: FR-001\n...'` works in bash (positional `-` stdin handling is correct for piped input).
