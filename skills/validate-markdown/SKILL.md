---
name: validate-markdown
description: Use when an agent needs to check whether an authored Markdown artifact is structurally valid with quire-cli. Covers quire validate (markdown-only), parse-based sanity checks, required-section checks, and how to report actionable validation failures without doing domain review.
metadata:
  short-description: Validate quire artifacts
---

# Validate Markdown

Use this skill to check structure and schema. Do not turn this into domain review; report concrete CLI failures and missing sections.

## Validate a Markdown Document

`quire validate` is markdown-only: it structurally checks an authored artifact
(`body_extraction` asserts + frontmatter schema + per-level heading uniqueness).
`--module` is required so the archetype can be resolved. On success it exits 0
with no output; on failure it exits 1 with line-numbered diagnostics on stderr.

```bash
quire validate path/to/artifact.md --module path/to/module
```

The archetype is resolved from the document's frontmatter `artifact_type`. Pass
`--archetype <NAME>` to override that resolution (or supply it when the document
has no `artifact_type`):

```bash
quire validate path/to/artifact.md --module path/to/module --archetype FR
```

To validate a document streamed on stdin, pass `-` as the positional argument:

```bash
cat path/to/artifact.md | quire validate - --module path/to/module
```

## Inspect Structure with parse

For finer-grained structural inspection beyond pass/fail, use parse:

```bash
quire parse path/to/artifact.md | jq '.frontmatter'
quire parse path/to/artifact.md | jq -r '.. | objects | select(has("heading")) | "\(.level) \(.heading)"'
```

## Required Sections

When required sections are known, fetch each one and check content:

```bash
quire lookup path/to/artifact.md --heading Behavior --content
quire lookup path/to/artifact.md --heading Acceptance --content
```

Treat empty output, placeholder-only content, or missing selectors as validation findings.

## Reporting

Report failures with:

- command run
- exit code
- stderr summary
- missing or invalid field/section
- file path and selector, when relevant
