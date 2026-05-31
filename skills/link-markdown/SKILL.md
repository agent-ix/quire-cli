---
name: link-markdown
description: Use when an agent needs to inspect relationships, ix links, dependency edges, or blast radius in Markdown artifacts with quire-cli. Covers quire extract, frontmatter relationship edges, body ix:// link harvest, and practical link-audit workflows.
metadata:
  short-description: Inspect artifact links
---

# Link Markdown

Use this skill to inspect artifact relationships and link blast radius. It is for graph and dependency questions, not prose review.

## Extract Edges

Use `quire extract` when the document has an archetype/module with extraction rules:

```bash
quire extract path/to/doc.md --module path/to/module
```

If the archetype cannot be inferred:

```bash
quire extract path/to/doc.md --module path/to/module --archetype FR
```

Edges appear in `.edges` and include frontmatter relationships plus body `ix://` links harvested by quire-rs.

```bash
quire extract path/to/doc.md --module path/to/module | jq '.edges'
```

## Quick Link Audit

For one file:

```bash
quire extract path/to/doc.md --module path/to/module \
  | jq -r '.edges[] | "\(.type) \(.target)"'
```

For many files, loop in the shell and keep the output line-oriented:

```bash
for f in spec/**/*.md; do
  quire extract "$f" --module path/to/module \
    | jq -r --arg file "$f" '.edges[] | "\($file)\t\(.type)\t\(.target)"'
done
```

## Blast Radius Questions

For “what depends on X,” extract all edges and filter targets:

```bash
jq -r 'select(.[2] == "ix://target") | @tsv'
```

Use exact targets when possible. If target formats vary, first list unique target strings and normalize only after inspecting examples.

## Failure Handling

`extract` requires a module and an archetype with `body_extraction`. If it fails because there is no DSL, use `quire parse` for frontmatter and `rg 'ix://'` as a fallback, and state that the fallback is not equivalent to quire edge harvest.
