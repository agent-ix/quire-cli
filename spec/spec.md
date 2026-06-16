---
type: master-requirements
name: quire-cli
org: agent-ix
component_type: rust-cli
tags:
  - rust
  - cli
  - templating
  - markdown
  - filament
implementation_language: rust
depends_on:
  - quire-rs
relationships:
  - target: "ix://agent-ix/quire-rs"
    type: "consumes"
    cardinality: "1:1"
  - target: "ix://agent-ix/spec-artifacts-iso"
    type: "consumes"
    cardinality: "1:N"
  - target: "ix://agent-ix/spec-artifacts-process"
    type: "consumes"
    cardinality: "1:N"
  - target: "ix://agent-ix/spec-artifacts-app"
    type: "consumes"
    cardinality: "1:N"
  - target: "ix://agent-ix/filament-core-service/FR-035"
    type: "implements"
    cardinality: "1:1"
standards_alignment:
  - iso-iec-ieee-29148
  - ieee-828
---
# Master Requirements Specification
## quire-cli — Static Binary CLI for Quire/Filament Artifact Generation

---

## 1. Purpose

This document defines the **scope, intent, and governing requirements framework** for `quire-cli`, a single static Rust binary that exposes `quire-rs` as a command-line surface for agent-driven and human-driven artifact workflows.

The CLI is the **hot path** for FR-035 manifest-driven authoring: an agent authors a markdown artifact and invokes `quire validate <doc.md> --module <path>` once per artifact, expecting a fast structural check. The same binary covers parsing, body-extraction, the asserts-based input contract (`schema`), and section-level read/edit so that one tool serves the full author → inspect → validate loop without requiring a Python or Node runtime. (The render/templating path is **removed** — see §2bis.)

It establishes:
- The CLI surface (subcommands, flags, stdin/stdout/stderr contract)
- The path-safety boundary on input arguments (templates removed; see §2bis)
- The fast-CLI target for the surviving subcommands
- The relationship between `quire-rs` library FRs and this consumer's command-line surface

This document is the **top-level requirements artifact** for the repository.

---

## 2. Scope

### 2.1 In Scope

This specification governs:
- The `quire` binary's subcommand surface (`parse`, `extract`, `validate`, `schema`, `lookup`, `edit`, `lint`) — **`render` is removed** (see §2bis)
- Argument parsing, exit codes, and stdout/stderr conventions
- Path-safety checks on user-supplied `--module` and positional document arguments (and `edit` write targets)
- JSON encoding of `QuireDocument`, `ExtractionResult`, `HarvestedEdge[]` on stdout
- Process-level performance budgets (cold-start validate / parse / extract)
- Distribution and installation surface (`cargo install`, release binary tarballs)

### 2.2 Out of Scope

This specification does not govern:
- The parsing, extraction, or validation logic itself — those are normatively defined in **`quire-rs`** FRs (`FR-002` data-schema validation, `FR-005` parse, `FR-010` query, `FR-011` body extraction, `FR-013` archetype loader, `FR-014` module activation, `FR-015` relationship harvesting, `FR-032` `validate_document`). This CLI is a thin process boundary around those APIs. (Rendering/templating is **removed** upstream — see §2bis.)
- Filament Module manifest schema — owned by `filament-core-service/spec/functional/FR-035-module-manifest-schema.md`.
- Python bindings (owned by `quire-py`) or WASM bindings (owned by `quire-wasm`).
- Server-side validation in `filament-core-service` (uses `quire-py`, not this CLI).
- Editor live preview (uses `quire-wasm`).

---

## 2bis. Render Removal (2026-06-04)

The render/templating half of `quire-rs` is **removed** — **no backward-compatibility
layer**, no deprecated-but-kept flag, no dual-read (mirrors the upstream quire-rs
render retirement, commit 500a3d3). `quire-cli` is now a **parse / validate /
extract / schema / lookup / edit** boundary. Artifacts are authored as markdown
directly (via the `/specify` flow) and checked structurally by `validate` →
quire-rs `validate_document` (FR-032). The `validate --json` context/data mode
(which dispatched to quire-rs `validate`/FR-002 over a context JSON object) is
**also removed**: the engine `validate` fn survives upstream to back
`validate_document`, but it is no longer reachable from the CLI.

This supersedes the render-centric prose retained elsewhere in this document for
history (§3.1, §8.2, §8.3, §9, §12) — those passages are kept for traceability but
are governed by this entry.

### Retired (kept for history, ACs dropped from the required-coverage tally)

| Artifact | Kind | Why |
|---|---|---|
| **FR-001** `render` subcommand | FR | No render path; subcommand removed |
| **US-001** Agent renders an FR | US | Render-centric; author markdown directly + `validate` |
| **NFR-001** Render latency budget | NFR | No render path to bench |
| **StR-002** Sub-50 ms render budget | StR | Render-centric budget; fast-CLI need carried by StR-001 |

### Revised (CR-noted, kept and active)

| Artifact | Change |
|---|---|
| **StR-001** Static-binary hot path | AC-1 lists the surviving subcommands (no `render`) |
| **StR-003** Sandbox inheritance | Template-include sandbox half dropped (templates gone); path-safety half kept; AC-3 retired |
| **FR-004** `validate` subcommand | **Markdown-only**; `--json` context mode removed; FR-002 consumed-relationship dropped; +archetype-resolution failure ACs (no frontmatter / no string `artifact_type` / unknown), +path-safety arg-label AC, +stdin-exempt AC |
| **FR-005** Path-safety | `--data`/`--out` (render) examples rephrased onto `validate`/`edit`; `--data` AC retired; generic semantics preserved |
| **FR-006** I/O contract | `render` stdout row + `--data -` stdin trigger removed; AC-4 rephrased onto `parse -` |
| **FR-009** `schema` subcommand | Asserts-based input contract (FR-029 recast by ADR 0004); "template variables" wording removed |

### Decision: `validate` is markdown-only (no `--json`)

The `--json` context/data validation mode is removed entirely (not kept as a
legacy flag). This **moots gap G1** (the `--json`-vs-positional dispatch rule):
there is exactly one mode, so there is nothing to dispatch on.

---

## 3. System Overview

### 3.1 System Description

`quire-cli` ships a single Rust binary, `quire`, with these subcommands (render
removed — see §2bis):

- `quire parse <doc.md|->` — emit a `QuireDocument` (heading tree, frontmatter, byte slices) as JSON on stdout. Wraps `quire_rs::parse_document` (consumer of `quire-rs` FR-005 / FR-006 / FR-007 / FR-008).
- `quire extract <doc.md> --module <path>` — run the body-extraction DSL declared in the module against the document and emit `{extraction, edges}` as JSON. Wraps `quire_rs::extract` + `quire_rs::harvest_edges` (consumer of `quire-rs` FR-011 + FR-015).
- `quire validate <doc.md|glob|->... [--scope <dir>] [--module <path>] [--archetype <name>]` — **markdown-only** structural validation; exit 0 on valid, 1 with structured errors on stderr otherwise. In scoped mode, relative globs are resolved under `--scope` and frontmatter `artifact_type` selects the archetype. Wraps `quire_rs::validate_document` (consumer of `quire-rs` FR-032).
- `quire schema <archetype> --module <path>` — emit the asserts-based input contract (FR-029) as JSON.
- `quire lookup` / `quire edit` — read / byte-splice a section or stable block (consumer of `quire-rs` query + `update_section`/`update_block`).

The positional document argument accepts `-` for stdin. The binary statically links `quire-rs` and ships as a single platform-specific executable.

### 3.2 Intended Users

- **Agents** generating FR/NFR/ADR/Plan/Review/Ledger artifacts during `/spec-write-*`, `/spec-create-spec`, `/spec-to-plan`, and similar workflows (primary consumer; hot path)
- **Human authors** invoking the CLI directly from a shell during spec authoring
- **CI pipelines** validating that committed `.md` artifacts still satisfy their archetype schemas
- **Distribution channels** (`cargo install quire-cli`, GitHub release tarballs, future homebrew tap)

---

## 4. Requirements Architecture

```
spec/
├── spec.md                # This document
├── stakeholder/           # StR-XXX
├── usecase/               # US-XXX
├── functional/            # FR-XXX (CLI surface + sandbox + perf gates)
├── non-functional/        # NFR-XXX
├── tests.md               # Bidirectional FR ↔ IT matrix
└── assets/                # Diagrams, fixture inventory
```

---

## 5. Requirement Classes

### 5.1 Stakeholder Requirements

- Format: `StR-XXX`
- Location: `stakeholder/`
- Drive system requirements

### 5.2 User Stories

- Format: `US-XXX`
- Location: `usecase/`
- Drive functional requirements

### 5.3 Functional Requirements

- Format: `FR-XXX`
- Location: `functional/`
- Normative system behavior; each consumer FR cites the upstream `quire-rs` FR it surfaces

### 5.4 Non-Functional Requirements

- Format: `NFR-XXX`
- Location: `non-functional/`
- Performance, sandbox, distribution constraints

### 5.5 Acceptance Criteria

- Format: `{FR-XXX}-AC-N`

---

## 6. Identifier Schema

| Artifact | Format | Example |
|----------|--------|---------|
| Stakeholder Requirement | `StR-XXX` | `StR-001` |
| User Story | `US-XXX` | `US-002` |
| Functional Requirement | `FR-XXX` | `FR-001` |
| Non-Functional Requirement | `NFR-XXX` | `NFR-001` |
| Acceptance Criteria | `{FR}-AC-N` | `FR-001-AC-1` |
| Test Case / IT | `IT-XXX` | `IT-001` |

Identifiers are immutable once assigned.

---

## 7. Requirement Quality Policy

All FRs SHALL:
- Define observable CLI behavior (stdout, stderr, exit code, written file)
- Be unambiguous and atomic
- Reference upstream `quire-rs` FRs by ID for any pass-through behavior
- Be testable via process-level integration tests (`assert_cmd`)

FRs SHALL NOT:
- Re-specify rendering/parsing/validation logic — cite the upstream `quire-rs` FR instead
- Encode application-specific policy (artifact templates live in `spec-artifacts-*`)

---

## 8. Process Boundary Model

### 8.1 Process Lifecycle

Each invocation is a **one-shot process**: parse argv → load `Registry` from `--module` → run the requested operation → write to stdout / file → exit. There is no daemon mode, no long-lived state, no IPC channel.

### 8.2 Stdin / Stdout / Stderr Contract

- A positional `-` reads the entire document from stdin until EOF
- Successful operations write the primary result to stdout (JSON document, JSON extraction, JSON contract, looked-up/edited section); `validate` writes nothing on success
- Diagnostics (schema errors, structural-validation errors, sandbox violations) write to stderr in the structured format defined by `quire-rs` FR-017
- Exit code 0 on success, 1 on user error (bad path, validation failure), 2 on internal error

### 8.3 Sandbox Inheritance

> **§2bis note:** Templates are removed, so the template-include sandbox guarantees
> below no longer apply; the path-safety guarantees the CLI **adds** are the
> surviving, load-bearing ones.

The CLI inherits the schema-resolution guarantee from `quire-rs`:
- Schema resolver pinned to `resolve-file` (no network)

The CLI **adds**:
- Path-safety check: reject `..` in positional document / `--module` arguments (and `edit` write targets)
- No symlink-following on `--module` root (resolves once at startup, then operates on the canonicalized path)

---

## 9. Performance Model

### 9.1 Budget

> **§2bis note:** The render-latency NFR (NFR-001) and the render-centric StR-002
> budget are retired with render. A fast-CLI target is retained at the stakeholder
> level (StR-001) for the surviving subcommands; no dedicated render bench remains.

- Target: a one-shot `validate` / `parse` / `extract` (cold start → load module → run → write) stays in the low tens of milliseconds on a modern dev workstation.
- Measurement harness: `hyperfine` against a representative `spec-artifacts-iso` document (no render bench).

### 9.2 Strategy

- Static link of `quire-rs`; LTO thin + 1 codegen unit (matches `quire-rs/Cargo.toml` release profile)
- Lazy-load `Registry` per-invocation (no warm cache; first-invocation latency is the headline metric)
- Single allocation for stdout buffer where possible

---

## 10. Error and Failure Model

### 10.1 Error Classification

| Class | Source | Exit | Channel |
|-------|--------|------|---------|
| Sandbox violation (`..` in path, symlink escape) | This crate | 1 | stderr (structured) |
| Argument parsing error | `clap` | 2 | stderr |
| Module load error | `quire-rs` FR-013 / FR-014 | 1 | stderr (structured) |
| Structural validation error | `quire-rs` FR-032 (`validate_document`) | 1 | stderr (structured per FR-017) |
| Archetype-resolution error (no frontmatter / no string `artifact_type` / unknown) | This crate + `quire-rs` `UnknownArchetype` | 1 | stderr (structured) |
| Internal panic | Bug | 134 | stderr (rust panic message) |

### 10.2 Diagnostic Shape

Structured diagnostics follow `quire-rs` `Diagnostic` + `format_violation` (FR-017). No new diagnostic shape is introduced by this CLI.

---

## 11. Traceability

Bidirectional traceability SHALL be maintained between:
- StR → US / FR
- US → FR
- FR → AC
- AC → IT

Each FR that wraps a `quire-rs` API SHALL declare the upstream FR ID in its `relationships:` frontmatter (`type: implements`, `target: ix://agent-ix/quire-rs/FR-XXX`).

---

## 12. Verification Strategy

FRs are verified by:
- **Integration tests** (`tests/`) driving the compiled binary with `assert_cmd` + `predicates`
- **Path-safety tests** with deliberately malformed `..` / symlink inputs

(Render parity tests and the render benchmark are removed with render — see §2bis.)

---

## 13. Change Management

- CRs follow the cross-repo change-management policy
- Changes that modify CLI surface require a deprecation cycle: warn-on-stderr for one release, remove the next
- Upstream `quire-rs` API changes propagate via Cargo version pin

---

## 14. Lifecycle Status

BASELINED — v0.2.x released. The surface is seven subcommands: the six in §3.1
(`parse`, `extract`, `lookup`, `edit`, `validate`, `schema`) plus `lint`
(FR-013, added in v0.2.0); `render` is removed (§2bis). Under SemVer the
subcommand surface, exit codes, and JSON output schemas are the stable contract
(NFR-006). Further changes follow §13 change management.

---

## 15. Governance Notes

- This CLI is a **thin process boundary**. Substantive parse/render/extract/validate logic belongs in `quire-rs`. If a feature request cannot be satisfied by adding flags or composition over existing `quire-rs` APIs, file the upstream FR first.
- The CLI MUST remain a single static binary. No optional dynamic linking, no plugin loading, no eval-from-env code paths.
- No network calls of any kind (matches `quire-rs` StR-001 hard rule).

---

## 16. References

- `quire-rs/spec/spec.md` and its FR-001 through FR-018
- `filament-core-service/spec/functional/FR-035-module-manifest-schema.md`
- `filament-core-service/spec/non-functional/NFR-006-generation-performance.md` (if extant; otherwise the parent plan's design target)
- ISO/IEC/IEEE 29148 — Requirements Engineering
- IEEE 828 — Configuration Management

---
