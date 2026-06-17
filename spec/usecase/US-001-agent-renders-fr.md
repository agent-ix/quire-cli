---
id: US-001
title: "Agent renders a new FR artifact during /spec-write-fr"
type: US
relationships:
  - target: "ix://agent-ix/quire-cli/spec/stakeholder/StR-001"
    type: "implements"
    cardinality: "1:1"
  - target: "ix://agent-ix/quire-cli/spec/stakeholder/StR-002"
    type: "implements"
    cardinality: "1:1"
---

> **RETIRED (render removal — 2026-06-04):** The render path is removed (mirrors
> quire-rs [US-001](ix://agent-ix/quire-rs/US-001)/US-009 retirement, commit 500a3d3). Agents executing
> `/spec-write-fr` now author the FR markdown directly via the `/specify` flow and
> validate it with `quire validate <doc.md>` ([FR-004](../functional/FR-004-validate-subcommand.md)); there is no `quire render`
> call. Kept for history and traceability only; ACs dropped from the
> required-coverage tally (ids retained, immutable). Recorded in `spec.md` §2bis.

## Story

As an **agent executing `/spec-write-fr`**, I want to invoke `quire render FR --module ~/dev/spec-artifacts-iso/spec_artifacts_iso --data -` with the FR context JSON on stdin and receive a fully-rendered, schema-validated markdown body on stdout in under 50 ms, so that authoring an FR is a single tool call with no wrapper script and no follow-up validation step.

## Context

The FR archetype lives in `spec-artifacts-iso` as `templates/fr.md.j2` plus `schemas/fr-frontmatter.schema.json`. Today the agent runs `minijinja-cli` which renders but does not validate — schema violations only surface later at `/spec-review` or commit time. With `quire render` the schema check is part of the render call (per upstream `quire-rs` [FR-001](../functional/FR-001-render-subcommand.md) / [FR-002](../functional/FR-002-parse-subcommand.md)), so invalid frontmatter is caught immediately.

## Acceptance

- US-001-AC-1 (RETIRED): `echo '{...valid FR ctx...}' | quire render FR --module $ISO --data -` writes the rendered markdown to stdout and exits 0.
- US-001-AC-2 (RETIRED): The rendered output is byte-identical to `minijinja-cli templates/fr.md.j2 ctx.json` against the same template+context.
- US-001-AC-3 (RETIRED): Invalid frontmatter (e.g. missing `id`) exits 1 with a `SchemaViolation` diagnostic on stderr naming the offending field, before any output is written to stdout.
- US-001-AC-4 (RETIRED): End-to-end wall-time for one render is ≤ 50 ms at p95 on a modern dev workstation.
