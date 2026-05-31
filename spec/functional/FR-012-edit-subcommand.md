---
id: FR-012
title: "quire edit subcommand"
artifact_type: FR
object_type: cli_command
relationships:
  - target: "ix://agent-ix/quire-cli/spec/functional/FR-011"
    type: "references"
    cardinality: "1:1"
  - target: "ix://agent-ix/quire-rs/spec/functional/FR-022"
    type: "consumes"
    cardinality: "1:1"
---

## Behavior

The CLI SHALL expose an `edit` subcommand that performs byte-exact section/block
writeback against an existing rendered document:

```
quire edit <DOC|-> (--heading <TEXT> | --block-id <BLOCK_ID>) --content <FILE|-> [--out <PATH>]
```

Required arguments:
- `<DOC|->` — positional, path to a `.md` file or `-` to read from stdin.
- Exactly one selector:
  - `--heading <TEXT>` — replace the body of the section whose heading matches the
    query (case-insensitive, section-number normalized). The new content is the
    section BODY — everything after the heading line, up to the next heading.
  - `--block-id <BLOCK_ID>` — replace the full block whose stable
    `QuireSection.block_id` equals `<BLOCK_ID>`. The new content is the FULL block
    rendering — the heading line (with its `{#blk-id}` attribute) followed by the body.
- `--content <FILE|->` — source of the replacement content: a file path, or `-` for stdin.

Optional arguments:
- `--out <PATH>` — write the updated full-file markdown to `<PATH>`. When omitted,
  write to stdout. Passing the input path edits the document in place.

Behavior:
1. Read the full document and the replacement content into memory.
2. Dispatch to `quire_rs::parse_document(text)`.
3. For `--heading`, call `quire_rs::update_section`; for `--block-id`, call
   `quire_rs::update_block`.
4. Emit the updated full-file markdown to `--out` or stdout. Frontmatter and every
   untouched section/block stay byte-identical.
5. On no match, exit 1 with empty stdout and a diagnostic naming the selector; the
   input file is left untouched.

`<DOC>` and `--content` SHALL NOT both read from stdin (exactly one stdin source).

## Acceptance

- **FR-012-AC-1**: `quire edit doc.md --heading Description --content body.txt` replaces the Description body and leaves frontmatter and all other sections byte-identical.
- **FR-012-AC-2**: `quire edit doc.md --block-id blk-behavior --content block.txt` replaces the full `{#blk-behavior}` block.
- **FR-012-AC-3**: `--out` pointing at the input path edits the document in place.
- **FR-012-AC-4**: A selector that matches no section exits 1 without writing; the input file is unchanged.
- **FR-012-AC-5**: Passing both `--heading` and `--block-id` is an argv error; passing neither is a user error.
- **FR-012-AC-6**: Passing `-` for both `<DOC>` and `--content` is a user error.
