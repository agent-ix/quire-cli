---
name: write-markdown
description: Use when an agent needs to generate or update a Markdown artifact with quire-cli from structured data. Covers choosing an archetype, preparing context JSON, rendering with quire render, handling schema or template errors, and verifying the generated document without domain-specific review guidance.
metadata:
  short-description: Generate Markdown artifacts
---

# Write Markdown

Use this skill to generate Markdown artifacts through `quire render`. Keep domain decisions in the calling workflow; this skill only covers the CLI path.

## Workflow

1. Identify the archetype name, for example `FR`, `US`, `NFR`, or a project-specific type.
2. Locate the module directory that contains `manifest.yaml`.
3. Prepare context JSON matching the archetype schema.
4. Render to stdout first unless the user explicitly asked for a file write.
5. Parse or validate the result before considering the artifact done.

## Commands

Render from a JSON file:

```bash
quire render FR --module path/to/module --data ctx.json
```

Render from stdin:

```bash
cat ctx.json | quire render FR --module path/to/module --data -
```

Write directly to a file:

```bash
quire render FR --module path/to/module --data ctx.json --out spec/functional/FR-123.md
```

## Error Handling

Schema and template errors are user errors. Read stderr, fix the context JSON, and rerun. Do not patch rendered Markdown to hide a schema problem; fix the source data unless the user explicitly wants manual Markdown edits.

## Done Check

For a generated artifact, run:

```bash
quire parse path/to/artifact.md | jq '.frontmatter'
quire lookup path/to/artifact.md --heading Acceptance --content
```

Use `quire validate` on the context JSON that produced the artifact:

```bash
quire validate FR --module path/to/module --data ctx.json
```
