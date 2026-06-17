---
id: FR-011
title: "quire lookup subcommand"
type: FR
object_type: cli_command
relationships:
  - target: "ix://agent-ix/quire-cli/spec/usecase/US-005"
    type: "implements"
    cardinality: "1:1"
  - target: "ix://agent-ix/quire-rs/spec/functional/FR-005"
    type: "consumes"
    cardinality: "1:1"
  - target: "ix://agent-ix/quire-rs/spec/functional/FR-010"
    type: "consumes"
    cardinality: "1:1"
  - target: "ix://agent-ix/quire-rs/spec/functional/FR-019"
    type: "consumes"
    cardinality: "1:1"
---

## Description

The CLI SHALL expose a `lookup` subcommand that locates a single section or block
in a markdown document by heading, parser id, or stable block id, emitting either
the `QuireSection` JSON or its raw content, delegating all parsing and lookup to
`quire-rs`. The behavioral surface is specified below.

## Behavior

The CLI SHALL expose a `lookup` subcommand with the following surface:

```
quire lookup <DOC|-> (--heading <TEXT> [--level <1..6>] | --id <ID> | --block-id <BLOCK_ID>) [--content]
```

Required arguments:
- `<DOC|->` — positional, path to a `.md` file or `-` to read from stdin.
- Exactly one selector:
  - `--heading <TEXT>` — locate the first section whose heading matches the query using `quire-rs` heading-query semantics.
  - `--id <ID>` — locate the first section whose parser-derived `QuireSection.id` equals `<ID>`.
  - `--block-id <BLOCK_ID>` — locate the first section whose stable `QuireSection.block_id` equals `<BLOCK_ID>`.

Optional arguments:
- `--level <1..6>` — constrain `--heading` lookup to a single ATX heading level. This option is invalid with `--id` or `--block-id`.
- `--content` — write only the selected section's raw `content` bytes to stdout. When omitted, write the selected `QuireSection` as JSON.

Behavior:
1. Read full document into memory.
2. Dispatch to `quire_rs::parse_document(text)`.
3. Apply the selected lookup against the parsed `QuireDocument`.
4. On success with default output, serialize the matching `QuireSection` as JSON using the global `--pretty` flag.
5. On success with `--content`, write the section content exactly as stored in `QuireSection.content`.
6. On no match, exit 1 with empty stdout and a diagnostic naming the selector.

`QuireSection.id` is parser-derived from `<slug>-L<line>` and is not stable across line shifts. Stable machine addressing SHOULD use `--block-id` with authored Pandoc heading attributes such as `## Behavior {#blk-behavior}`.

## Acceptance Criteria

| ID | Criteria | Verification |
|----|----------|--------------|
| FR-011-AC-1 | Given `# Title`, `quire lookup doc.md --heading Title --level 1` emits a section JSON object with `level: 1` and `heading: "Title"` | Test |
| FR-011-AC-2 | Given `## Behavior {#blk-behavior}`, `quire lookup doc.md --block-id blk-behavior` emits a section JSON object with `block_id: "blk-behavior"` | Test |
| FR-011-AC-3 | Given `## Behavior` on body line 4, `quire lookup doc.md --id behavior-L4` emits that section | Test |
| FR-011-AC-4 | With `--content`, stdout is the selected section's content only, not a JSON string | Test |
| FR-011-AC-5 | Passing multiple selectors is an argv error | Test |
| FR-011-AC-6 | Passing `--level` without `--heading` is an argv error | Test |

## Dependencies

- **Upstream**: US-005 machine addresses section; quire-rs FR-005, FR-010, FR-019 (parse + heading/block lookup APIs).
- **Downstream**: FR-012 edit (shares heading/block addressing).
