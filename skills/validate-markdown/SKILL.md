---
name: validate-markdown
description: Use when an agent needs to check whether a Markdown artifact or JSON context is structurally valid with quire-cli. Covers quire validate, parse-based sanity checks, required-section checks, and how to report actionable validation failures without doing domain review.
metadata:
  short-description: Validate quire artifacts
---

# Validate Markdown

Use this skill to check structure and schema. Do not turn this into domain review; report concrete CLI failures and missing sections.

## Validate Context JSON

Use this before rendering:

```bash
quire validate FR --module path/to/module --data ctx.json
```

For stdin:

```bash
cat ctx.json | quire validate FR --module path/to/module --data -
```

## Check Rendered Markdown

For rendered Markdown, use parse and lookup checks:

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
