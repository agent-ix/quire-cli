---
id: FR-007
title: "Exit code contract"
type: FR
object_type: cli_command
relationships:
  - target: "ix://agent-ix/quire-cli/spec/stakeholder/StR-001"
    type: "implements"
    cardinality: "1:1"
---

## Description

The CLI SHALL use a uniform exit-code contract across all subcommands — 0
success, 1 user error, 2 argv error, 134 internal panic — and no other code, so
callers can branch reliably on outcome. The behavioral surface is specified
below.

## Behavior

The CLI SHALL use the following exit codes uniformly across all subcommands:

| Code | Meaning |
|------|---------|
| 0 | Success. Primary result on stdout (or empty for `validate`). |
| 1 | **User error** — recoverable by the caller: path-safety violation, unknown archetype, structural-validation failure, archetype-resolution failure (no frontmatter / no `type`), missing file, module load failure. Diagnostic on stderr. |
| 2 | **Argument parsing error** — `clap` could not parse argv. clap-generated message on stderr. |
| 134 | Internal panic (SIGABRT). Indicates a bug; should never happen in normal operation. |

The CLI SHALL NOT use any other exit code. In particular, structural-validation failures, archetype-resolution failures, and parse errors all exit 1 — the diagnostic on stderr discriminates among them.

## Acceptance Criteria

| ID | Criteria | Verification |
|----|----------|--------------|
| FR-007-AC-1 | For each subcommand, success exits 0 | Test |
| FR-007-AC-2 | Path-safety violation exits 1 | Test |
| FR-007-AC-3 | Unknown archetype exits 1 | Test |
| FR-007-AC-4 | Structural-validation failure (`validate_document`) exits 1 | Test |
| FR-007-AC-5 | `quire --bogus-flag` exits 2 | Test |
| FR-007-AC-6 | No test triggers exit 134 (no panics on any covered input) | Test |

## Dependencies

- **Upstream**: StR-001 single-binary hot path.
- **Downstream**: every subcommand and FR-006 I/O contract rely on these exit codes.
