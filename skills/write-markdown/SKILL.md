---
name: write-markdown
description: Use when an agent needs to author or update a Markdown artifact for a quire module. Covers choosing an archetype, fetching its input contract with quire schema, authoring the Markdown directly, and validating the document with quire validate without domain-specific review guidance.
metadata:
  short-description: Author Markdown artifacts
---

# Write Markdown

Use this skill to author Markdown artifacts for a quire module by hand, guided by the archetype's input contract. Keep domain decisions in the calling workflow; this skill only covers the CLI path. There is no `render` step — quire-cli does not generate Markdown from data; you write the Markdown directly and validate it.

## Workflow

1. Identify the archetype name, for example `FR`, `US`, `NFR`, or a project-specific type.
2. Locate the module directory that contains `manifest.yaml`.
3. Fetch the archetype's input contract with `quire schema` (see below). It describes the required frontmatter (a JSON Schema) and the body structure — required headings, table columns, and id-patterns — that validation enforces.
4. Author the Markdown document directly: write frontmatter satisfying the schema and a body that satisfies the contract's structural asserts.
5. Validate the finished document with `quire validate` before considering the artifact done.

## Commands

Get the input contract for an archetype:

```bash
quire schema FR --module path/to/module
```

This emits deterministic JSON: the frontmatter JSON Schema plus the `body_extraction` asserts (required headings, table columns, id-patterns) that `quire validate` checks. Use it as the skeleton for the document you author.

Validate an authored document:

```bash
quire validate path/to/artifact.md --module path/to/module
```

Validate against a specific archetype (when it cannot be inferred from frontmatter):

```bash
quire validate path/to/artifact.md --module path/to/module --archetype FR
```

`quire validate` exits 0 with no stdout on success, and exits 1 with diagnostics on stderr on failure. It writes nothing to stdout.

## Error Handling

Validation failures are authoring errors. Read stderr, fix the Markdown — the frontmatter or the body structure — and rerun. Do not work around a contract failure; bring the document into line with the contract reported by `quire schema`.

## Done Check

For an authored artifact, confirm the structure parses and read back the sections you care about:

```bash
quire parse path/to/artifact.md | jq '.frontmatter'
quire lookup path/to/artifact.md --heading Acceptance --content
```

Then validate:

```bash
quire validate path/to/artifact.md --module path/to/module
```
