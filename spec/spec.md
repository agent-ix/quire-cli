---
artifact_type: master-requirements
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

The CLI is the **hot path** for FR-035 manifest-driven artifact generation: an agent invokes `quire render <archetype> --module <path> --data <ctx>` once per artifact, expects sub-50 ms end-to-end (NFR-006), and writes the result to disk. The same binary covers parsing, body-extraction, and schema validation so that one tool serves the full generate → inspect → validate loop without requiring a Python or Node runtime.

It establishes:
- The CLI surface (subcommands, flags, stdin/stdout/stderr contract)
- The sandbox boundary inherited from `quire-rs` and FR-035 (no template FS reads, path-safety on input arguments)
- The performance budget that justifies replacing `minijinja-cli` in agent setup
- The relationship between `quire-rs` library FRs and this consumer's command-line surface

This document is the **top-level requirements artifact** for the repository.

---

## 2. Scope

### 2.1 In Scope

This specification governs:
- The `quire` binary's subcommand surface (`render`, `parse`, `extract`, `validate`)
- Argument parsing, exit codes, and stdout/stderr conventions
- Path-safety checks on user-supplied `--module` and `--data` arguments
- JSON encoding of `QuireDocument`, `ExtractionResult`, `HarvestedEdge[]` on stdout
- Process-level performance budgets (cold-start render + write)
- Sandbox carry-over from `quire-rs` (no `{% include %}`/`{% extends %}`, no FS reads in templates)
- Distribution and installation surface (`cargo install`, release binary tarballs)

### 2.2 Out of Scope

This specification does not govern:
- The parsing, rendering, extraction, or validation logic itself — those are normatively defined in **`quire-rs`** FRs (`FR-001` render dispatch, `FR-002` validation pipeline, `FR-005` parse, `FR-010` query, `FR-011` body extraction, `FR-013` archetype loader, `FR-014` module activation, `FR-015` relationship harvesting). This CLI is a thin process boundary around those APIs.
- Filament Module manifest schema — owned by `filament-core-service/spec/functional/FR-035-module-manifest-schema.md`.
- Python bindings (owned by `quire-py`) or WASM bindings (owned by `quire-wasm`).
- Server-side validation in `filament-core-service` (uses `quire-py`, not this CLI).
- Editor live preview (uses `quire-wasm`).

---

## 3. System Overview

### 3.1 System Description

`quire-cli` ships a single Rust binary, `quire`, with four subcommands:

- `quire render <archetype> --module <path> --data <file|->` — schema-validate `data` against `<archetype>`'s frontmatter schema and render the archetype's `.md.j2` template, emitting the rendered markdown on stdout. Wraps `quire_rs::render_by_name` (consumer of `quire-rs` FR-001 + FR-002).
- `quire parse <doc.md>` — emit a `QuireDocument` (heading tree, frontmatter, byte slices) as JSON on stdout. Wraps `quire_rs::parse_document` (consumer of `quire-rs` FR-005 / FR-006 / FR-007 / FR-008).
- `quire extract <doc.md> --module <path>` — run the body-extraction DSL declared in the module against the document and emit `{extraction, edges}` as JSON. Wraps `quire_rs::extract` + `quire_rs::harvest_edges` (consumer of `quire-rs` FR-011 + FR-015).
- `quire validate <archetype> --module <path> --data <file|->` — schema-validate without rendering; exit 0 on valid, 1 with structured errors on stderr otherwise. Wraps `quire_rs::validate` (consumer of `quire-rs` FR-002 + FR-017).

All subcommands accept `--data -` for stdin. The binary statically links `quire-rs` and ships as a single platform-specific executable.

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

- `--data -` reads the entire JSON object from stdin until EOF
- Successful operations write the primary result to stdout (rendered markdown, JSON document, JSON extraction)
- Diagnostics (schema errors, render errors, sandbox violations) write to stderr in the structured format defined by `quire-rs` FR-017
- Exit code 0 on success, 1 on user error (bad path, schema violation), 2 on internal error

### 8.3 Sandbox Inheritance

The CLI inherits sandbox guarantees from `quire-rs`:
- MiniJinja v2 with strict-undefined + `{% include %}`/`{% extends %}` rejected (`quire-rs` FR-004)
- No filesystem reads from inside templates
- Schema resolver pinned to `resolve-file` (no network)

The CLI **adds**:
- Path-safety check: reject `..` in `--data` / `--module` arguments
- No symlink-following on `--module` root (resolves once at startup, then operates on the canonicalized path)

---

## 9. Performance Model

### 9.1 Budget

- p95 end-to-end render (cold start → load module → render → write) ≤ **50 ms** on a modern dev workstation (NFR-006 carry-over from filament-core).
- Measurement harness: `hyperfine` against `templates/fr.md.j2` from `spec-artifacts-iso`.

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
| Schema validation error | `quire-rs` FR-002 | 1 | stderr (structured per FR-017) |
| Render error | `quire-rs` FR-001 / FR-004 | 1 | stderr (structured) |
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
- **Render parity tests** comparing stdout against vendored fixtures from `quire-rs/tests/render_parity/`
- **Benchmark harness** (`hyperfine`) gating the p95 ≤ 50 ms NFR
- **Sandbox tests** with deliberately malformed `..` / symlink inputs

---

## 13. Change Management

- CRs follow the cross-repo change-management policy
- Changes that modify CLI surface require a deprecation cycle: warn-on-stderr for one release, remove the next
- Upstream `quire-rs` API changes propagate via Cargo version pin

---

## 14. Lifecycle Status

DRAFT — initial requirements authoring.

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
