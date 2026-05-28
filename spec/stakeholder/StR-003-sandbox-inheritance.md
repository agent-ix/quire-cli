---
id: StR-003
title: "Sandbox guarantees inherited from quire-rs and FR-035"
artifact_type: StR
---

## Stakeholder Need

Agent-supplied paths and JSON data flow into a process that loads templates and JSON Schemas from disk. Without explicit sandboxing, a malicious or malformed `--module` argument could escape the intended module root via symlinks or `..`; a hand-rolled template could read arbitrary files via `{% include %}`.

The CLI MUST inherit `quire-rs`'s template sandbox (no `{% include %}`/`{% extends %}`, no FS reads from templates) and add **process-boundary path safety** on `--module` and `--data` arguments.

This matches the sandbox carry-over called out in the parent plan ("Sandbox carry-over from NFR-006") and the FR-035 manifest contract.

## Priority

Must-Have

## Acceptance

- **StR-003-AC-1**: `quire render FR --module ../escape ...` exits 1 with a structured "path safety violation" diagnostic on stderr.
- **StR-003-AC-2**: `quire render FR --data ../../etc/passwd ...` exits 1 with the same class of diagnostic.
- **StR-003-AC-3**: A template containing `{% include "/etc/passwd" %}` is rejected at archetype load time by the upstream `quire-rs` FR-004 strict environment (verified via IT).
- **StR-003-AC-4**: Symlinks under `--module` are not followed past the canonicalized root.
