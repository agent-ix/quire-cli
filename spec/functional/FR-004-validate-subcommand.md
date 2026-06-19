---
id: FR-004
title: "quire validate subcommand"
type: FR
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
> `validate(&compiled_archetype, &data)` / [FR-002](./FR-002-parse-subcommand.md) over a context JSON object) is
> **removed** — no backward-compatibility layer, mirroring the quire-rs render
> retirement (commit 500a3d3). The engine `validate` fn still exists in quire-rs to
> back `validate_document`, but it is no longer reachable from the CLI. The
> consumed-[FR-002](./FR-002-parse-subcommand.md) relationship is dropped from this FR's frontmatter for that reason.
> The CLI remains a thin wrapper ([StR-004](../stakeholder/StR-004-thin-boundary-over-quire-rs.md)) — no validation logic lives here.

> **CR note (`--module` is REQUIRED):** Markdown validation always needs a module
> registry to resolve the archetype and its `body_extraction` asserts, so `--module`
> is mandatory (not bracketed/optional) in the implementation.

> **CR note (eager module-load failure, 2026-06-11):** the tolerant engine load
> reports a missing/unloadable `manifest.yaml` as an `ArchetypeLoadFailure` while
> returning an EMPTY registry (quire-rs [FR-013-AC-13](ix://agent-ix/quire-rs/FR-013-AC-13)). The CLI previously ignored
> `Registry::failures()` and died later with a misleading
> `UnknownArchetype: 'FR' is not registered`. All module-loading subcommands
> (`validate`, `extract`, `schema`, `lint`) now share a loader helper that fails
> fast — exit 1 with the real reason, e.g.
> `module load failed: manifest.yaml not found in module root (<path>/manifest.yaml)`
> — whenever the load yields zero modules and at least one failure. Surfaced by the
> spec-objects format walkthrough (issue #5). Verified by
> `tests/cli_lint.rs::missing_manifest_reports_real_reason_not_unknown_archetype`.

> **CR note (`type` discriminator rename, 2026-06-16):** OKF adoption renamed the
> archetype discriminator frontmatter key from `artifact_type` to `type`. All
> prose and ACs below (notably the archetype-resolution paths and AC-4/AC-5) now
> read `type`; the implementation reads it via `quire_rs::concept_type(&parsed)`.
> The new `--okf` bundle posture is specified separately in
> [FR-014](./FR-014-validate-okf-bundle.md); the default per-file strict path here
> is unchanged. The base concept contract (`type` required + non-empty, optional
> `description`/`tags` typed) is now enforced upstream in quire-rs for every
> validated document — see [FR-014](./FR-014-validate-okf-bundle.md) §B.

> **CR note (composed type+object validation + `--strict`, 2026-06-16):** The
> per-file `validate` path now dispatches to quire-rs
> `validate_document_in_registry(&registry, archetype, doc_text)` (FR-032-AC-11..13),
> which composes the `type` archetype with the frontmatter `object:` archetype.
> The result carries both `errors` (exit-failing) and `warnings` (advisory). The
> CLI surfaces BOTH on stderr: warnings are clearly marked (`warning:` prefix in
> human format; a distinct `severity`/`kind` field in `--diagnostics-format json`)
> and distinct from errors. A new boolean `--strict` flag escalates warnings to
> exit-failing. Exit code: **1 if any error**; with `--strict`, **also 1 if any
> warning**; otherwise **0** (warnings alone, no `--strict` → exit 0, still
> printed). The CLI remains a thin wrapper ([StR-004](../stakeholder/StR-004-thin-boundary-over-quire-rs.md)) — all validation/composition
> logic lives in quire-rs.

> **CR note (scoped discovery roots + lazy-init, 2026-06-19):** Scoped
> validation now also searches the canonical install root
> `~/.ix/filament/modules` (the same default `quire-rs`
> `loader::paths::default_module_root()` reads, where `quoin` materializes the
> default module set) and the `IX_FILAMENT_MODULES_PATH` env var (preferred;
> `IX_SCHEMA_PATH` retained as the legacy alias). These additions are pure
> filesystem discovery — no network. Additionally, when scoped discovery finds
> **zero** modules, `validate` lazy-installs the default set by shelling out
> once to `quoin plugin ensure-defaults` and reloading the registry one time
> before failing; if `quoin` is absent or fails, it falls through to an
> actionable error. The lazy-install delegates all network I/O to the `quoin`
> child — quire links no network crate — and is the documented exception to
> [NFR-004](../non-functional/NFR-004-no-network.md). See
> [ADR-0001](../assets/adr/0001-validate-lazy-init-module-bootstrap.md).

## Description

The CLI SHALL expose a single-mode (markdown-only) `validate` subcommand that
structurally validates committed markdown artifacts against their archetype,
delegating all validation and composition to `quire-rs`. It is a fast CI /
authoring gate that writes nothing to stdout. The behavioral surface is
specified below.

## Behavior

The CLI SHALL expose a single-mode (markdown-only) `validate` subcommand:

```
quire validate <DOC.md|GLOB|->... [--scope <DIR>] [--module <PATH>] [--archetype <NAME>] [--strict]
```

When the positional argument is a document path, glob, or `-`:
1. Path-safety ([FR-005](./FR-005-path-safety.md)) on each document path, `--scope`, and optional `--module`. A positional `-` is path-safety-exempt (stdin); the document text is read to EOF.
2. Resolve the archetype: read frontmatter `type` (a string) unless `--archetype <NAME>` overrides it.
3. Dispatch to quire-rs `validate_document_in_registry(&registry, archetype, doc_text)` (FR-032-AC-11..13): composed structural validation over the `type` archetype AND the frontmatter `object:` archetype. The result carries `errors` (exit-failing) and `warnings` (advisory — today only the unknown-`object:` case).
4. Surface BOTH errors and warnings on stderr (line-numbered, structured). Warnings are clearly marked: a `warning:` prefix in the human format; a distinct `severity`/`kind` field under `--diagnostics-format json`. Errors keep their existing shape.
5. Exit code: **1** if any **error**; with `--strict`, **also 1** if any **warning**; otherwise **0** (warnings alone, no `--strict`, still printed). On a clean success: exit 0, no stdout.

Scoped validation resolves relative document globs under `--scope`. If `--scope`
itself contains `manifest.yaml`, it is loaded as one exact module; otherwise
Quire loads module search roots from the scope, `--scope/.ix/modules`, the
`IX_FILAMENT_MODULES_PATH` / `IX_SCHEMA_PATH` env vars, and the default install
root `~/.ix/filament/modules`. If that search yields **zero** modules, `validate`
lazy-installs the default module set by shelling out once to
`quoin plugin ensure-defaults` and reloading the registry a single time; only
this child performs network I/O (the [NFR-004](../non-functional/NFR-004-no-network.md)
exception, [ADR-0001](../assets/adr/0001-validate-lazy-init-module-bootstrap.md)).
When the set is still empty (e.g. `quoin` not installed), it exits 1 with an
actionable diagnostic. `--module` remains the exact single-module compatibility
path and never triggers discovery or lazy-init.

**Archetype-resolution failure paths** (all exit 1, structured diagnostic on
stderr, no stdout):
- No frontmatter block at all → error that the document has no frontmatter from which to resolve the archetype (and `--archetype` was not supplied).
- Frontmatter present but no string `type` key (absent, or non-string) and no `--archetype` → error directing the author to add `type` or pass `--archetype`.
- The resolved (or `--archetype`-overridden) name is unknown to the loaded module → quire-rs `UnknownArchetype`.

`validate` SHALL NOT render or write any artifact body. It is a fast CI / authoring gate.

## Acceptance Criteria

| ID | Criteria | Verification |
|----|----------|--------------|
| FR-004-AC-1 | `quire validate valid-fr.md --module $ISO` exits 0 with no output (frontmatter valid, all required structure present) | Test |
| FR-004-AC-2 | `quire validate broken-fr.md --module $ISO` exits 1; stderr contains a line-numbered diagnostic naming the failing section/assert | Test |
| FR-004-AC-3 | `quire validate fr.md --module $ISO --archetype FR` overrides frontmatter-derived archetype resolution | Test |
| FR-004-AC-4 | A document with **no frontmatter** and no `--archetype` exits 1; stderr names the missing frontmatter / `type` and points at `--archetype` as the remedy. No stdout | Test |
| FR-004-AC-5 | A document whose frontmatter is present but has **no string `type`** (key absent, or a non-string value) and no `--archetype` exits 1; the diagnostic names `--archetype` (or `type`) as the way to resolve the archetype. No stdout | Test |
| FR-004-AC-6 | When the resolved or `--archetype`-overridden name is unknown to the loaded module, `validate` exits 1 with quire-rs `UnknownArchetype` on stderr; empty stdout | Test |
| FR-004-AC-7 | A path-safety violation on the document or `--module` exits 1 with a `PathSafetyViolation` ([FR-005](./FR-005-path-safety.md)) whose diagnostic names the offending argument label (the positional `document`, or `--module`) | Test |
| FR-004-AC-8 | `quire validate - --module $ISO` reads the document from stdin and is **not** subject to path-safety on the document argument (stdin is path-safety-exempt, [FR-005-AC-5](./FR-005-path-safety.md)); the markdown is still validated structurally | Test |
| FR-004-AC-9 | All validation logic is delegated to quire-rs; an audit confirms the CLI crate contains no structural-validation logic of its own ([StR-004](../stakeholder/StR-004-thin-boundary-over-quire-rs.md) thin boundary) | Inspection |
| FR-004-AC-10 | A document that is otherwise conformant but declares a frontmatter `object:` the registry cannot resolve produces a quire-rs **warning**. Without `--strict`, `validate` exits **0** and prints the warning to stderr, clearly marked (`warning:` prefix in human format) and distinct from any error; stdout stays empty | Test |
| FR-004-AC-11 | With `--strict`, the same unknown-`object:` warning becomes exit-failing: `validate` exits **1**, the warning still appears on stderr; stdout stays empty. A document with NO warnings and no errors still exits 0 under `--strict` | Test |
| FR-004-AC-12 | Under `--diagnostics-format json`, a warning is emitted as a distinct JSON object carrying a `severity`/`kind` field marking it a warning (not an error), so machine consumers can tell warnings from errors. An error retains its error `kind` | Test |
| FR-004-AC-13 | Scoped validation discovers modules from the default install root `~/.ix/filament/modules` and from `IX_FILAMENT_MODULES_PATH` (in addition to `--scope`, `--scope/.ix/modules`, and the legacy `IX_SCHEMA_PATH`) with no env var required and no network: a document validates against a module provided only via the default-root/env discovery path, and the run opens no inet socket | Test |
| FR-004-AC-14 | When scoped discovery finds zero modules and `quoin` is not available on PATH, `validate` exits 1 with an actionable diagnostic naming `quoin plugin ensure-defaults` (and `IX_FILAMENT_MODULES_PATH`); empty stdout. When `quoin` IS available, the empty-discovery path shells out to `quoin plugin ensure-defaults` once and reloads before validating (the [NFR-004](../non-functional/NFR-004-no-network.md) network exception, [ADR-0001](../assets/adr/0001-validate-lazy-init-module-bootstrap.md)) | Test (quoin-absent path) + Demonstration (lazy install) |

## Dependencies

- **Upstream**: [US-003](../usecase/US-003-ci-validates-archetype-conformance.md) CI validates archetype conformance; quire-rs [FR-032](ix://agent-ix/quire-rs/FR-032) (`validate_document_in_registry`).
- **Downstream**: [FR-010](./FR-010-required-section-validation.md) structural-validation surfacing; [FR-014](./FR-014-validate-okf-bundle.md) `--okf` bundle posture.
