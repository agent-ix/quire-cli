---
name: explore-markdown
description: Use when an agent needs to inspect, outline, or fetch targeted sections from Markdown documents using the quire CLI. Applies to spec artifacts and other heading-structured Markdown where quire parse and quire lookup are faster and safer than ad hoc regex parsing.
metadata:
  short-description: Explore Markdown with quire
---

# Explore Markdown

Use `quire` for structure-aware Markdown exploration. Prefer it over grep/sed when you need headings, frontmatter, section bodies, generated section IDs, or stable `{#block-id}` addressing.

## First Checks

From a repo that has `quire-cli`, use the local binary if available:

```bash
target/debug/quire --help
```

Otherwise use an installed binary:

```bash
quire --help
```

If neither exists and you are in the `quire-cli` repo, build it:

```bash
cargo build
```

## Outline A Document

Use `parse` to get the heading tree and metadata:

```bash
quire --pretty parse path/to/doc.md
```

For a compact outline:

```bash
quire parse path/to/doc.md \
  | jq -r '.. | objects | select(has("heading") and has("level")) | "\(.level) \(.heading) id=\(.id) block_id=\(.block_id // "-") lines=\(.start_line)-\(.end_line)"'
```

The `id` field is parser-derived as `<slug>-L<line>` and can change when lines move. Use `block_id` for stable addressing when the heading is authored with a Pandoc attribute:

```markdown
## Behavior {#blk-behavior}
```

## Fetch One Section

Use `lookup` when you know what section you need:

```bash
quire lookup path/to/doc.md --heading Behavior
quire lookup path/to/doc.md --heading "Document Title" --level 1
quire lookup path/to/doc.md --id behavior-L14
quire lookup path/to/doc.md --block-id blk-behavior
```

Use `--content` when the downstream task only needs the section body:

```bash
quire lookup path/to/doc.md --heading Acceptance --content
quire lookup path/to/doc.md --block-id blk-behavior --content
```

`--heading` returns the first matching section. Add `--level 1` for H1 lookup or another level when duplicate headings are possible.

## Inspect Frontmatter

```bash
quire parse path/to/doc.md | jq '.frontmatter'
quire parse path/to/doc.md | jq -r '.frontmatter.id // empty'
```

Frontmatter parse fallback is tolerant. If frontmatter is malformed, inspect stderr and the `frontmatter` field before assuming the document is invalid.

## Efficient Workflow

1. Run an outline first when you do not know the document shape.
2. Prefer `--block-id` for stable machine operations.
3. Prefer `--heading --level 1` for document-title/H1 lookup.
4. Use `--content` to avoid extra JSON parsing when patching, summarizing, or feeding one section to another tool.
5. Avoid regex parsing of headings unless `quire` is unavailable.

## Failure Handling

No match exits nonzero and writes no stdout. Treat that as “selector not present,” then rerun the outline command to discover the actual heading, generated id, or block id.
