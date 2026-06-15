---
id: FR-004
title: "quire validate subcommand"
artifact_type: FR
object_type: cli_command
relationships:
  - target: "ix://agent-ix/quire-cli/spec/usecase/US-003"
    type: "implements"
    cardinality: "1:1"
  - target: "ix://agent-ix/quire-rs/spec/functional/FR-032"
    type: "consumes"
    cardinality: "1:1"
---

> **CR note (markdown-only — render removal, 2026-06-04):** `validate` is now
> **markdown-only**. The `--json` context/data mode (which dispatched to quire-rs
> `validate(&compiled_archetype, &data)` / FR-002 over a context JSON object) is
> **removed** — no backward-compatibility layer, mirroring the quire-rs render
> retirement (commit 500a3d3). The engine `validate` fn still exists in quire-rs to
> back `validate_document`, but it is no longer reachable from the CLI. The
> consumed-FR-002 relationship is dropped from this FR's frontmatter for that reason.
> The CLI remains a thin wrapper (StR-004) — no validation logic lives here.

> **CR note (`--module` is REQUIRED):** Markdown validation always needs a module
> registry to resolve the archetype and its `body_extraction` asserts, so `--module`
> is mandatory (not bracketed/optional) in the implementation.

> **CR note (eager module-load failure, 2026-06-11):** the tolerant engine load
> reports a missing/unloadable `manifest.yaml` as an `ArchetypeLoadFailure` while
> returning an EMPTY registry (quire-rs FR-013-AC-13). The CLI previously ignored
> `Registry::failures()` and died later with a misleading
> `UnknownArchetype: 'FR' is not registered`. All module-loading subcommands
> (`validate`, `extract`, `schema`, `lint`) now share a loader helper that fails
> fast — exit 1 with the real reason, e.g.
> `module load failed: manifest.yaml not found in module root (<path>/manifest.yaml)`
> — whenever the load yields zero modules and at least one failure. Surfaced by the
> spec-objects format walkthrough (issue #5). Verified by
> `tests/cli_lint.rs::missing_manifest_reports_real_reason_not_unknown_archetype`.

## Behavior

The CLI SHALL expose a single-mode (markdown-only) `validate` subcommand:

```
quire validate <DOC.md|GLOB|->... [--scope <DIR>] [--module <PATH>] [--archetype <NAME>]
```

When the positional argument is a document path, glob, or `-`:
1. Path-safety (FR-005) on each document path, `--scope`, and optional `--module`. A positional `-` is path-safety-exempt (stdin); the document text is read to EOF.
2. Resolve the archetype: read frontmatter `artifact_type` (a string) unless `--archetype <NAME>` overrides it.
3. Dispatch to quire-rs `validate_document(archetype, doc_text)` (FR-032): structural validation over `body_extraction` asserts + frontmatter-schema + per-level heading uniqueness.
4. On success: exit 0, no stdout. On failure: write the line-numbered structured diagnostics to stderr, exit 1.

Scoped validation resolves relative document globs under `--scope`. If `--scope`
itself contains `manifest.yaml`, it is loaded as one exact module; otherwise
Quire loads module search roots from the scope, `--scope/.ix/modules`, and
`IX_SCHEMA_PATH`. `--module` remains the exact single-module compatibility path.

**Archetype-resolution failure paths** (all exit 1, structured diagnostic on
stderr, no stdout):
- No frontmatter block at all → error that the document has no frontmatter from which to resolve the archetype (and `--archetype` was not supplied).
- Frontmatter present but no string `artifact_type` key (absent, or non-string) and no `--archetype` → error directing the author to add `artifact_type` or pass `--archetype`.
- The resolved (or `--archetype`-overridden) name is unknown to the loaded module → quire-rs `UnknownArchetype`.

`validate` SHALL NOT render or write any artifact body. It is a fast CI / authoring gate.

## Acceptance

- **FR-004-AC-1**: `quire validate valid-fr.md --module $ISO` exits 0 with no output (frontmatter valid, all required structure present).
- **FR-004-AC-2**: `quire validate broken-fr.md --module $ISO` exits 1; stderr contains a line-numbered diagnostic naming the failing section/assert.
- **FR-004-AC-3**: `quire validate fr.md --module $ISO --archetype FR` overrides frontmatter-derived archetype resolution.
- **FR-004-AC-4**: A document with **no frontmatter** and no `--archetype` exits 1; stderr names the missing frontmatter / `artifact_type` and points at `--archetype` as the remedy. No stdout.
- **FR-004-AC-5**: A document whose frontmatter is present but has **no string `artifact_type`** (key absent, or a non-string value) and no `--archetype` exits 1; the diagnostic names `--archetype` (or `artifact_type`) as the way to resolve the archetype. No stdout.
- **FR-004-AC-6**: When the resolved or `--archetype`-overridden name is unknown to the loaded module, `validate` exits 1 with quire-rs `UnknownArchetype` on stderr; empty stdout.
- **FR-004-AC-7**: A path-safety violation on the document or `--module` exits 1 with a `PathSafetyViolation` (FR-005) whose diagnostic names the offending argument label (the positional `document`, or `--module`).
- **FR-004-AC-8**: `quire validate - --module $ISO` reads the document from stdin and is **not** subject to path-safety on the document argument (stdin is path-safety-exempt, FR-005-AC-5); the markdown is still validated structurally.
- **FR-004-AC-9**: All validation logic is delegated to quire-rs; an audit confirms the CLI crate contains no structural-validation logic of its own (StR-004 thin boundary).
