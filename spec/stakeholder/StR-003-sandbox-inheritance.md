---
id: StR-003
title: "Sandbox guarantees inherited from quire-rs and FR-035"
artifact_type: StR
---

> **CR note (render removal — 2026-06-04):** Templates are removed (mirrors
> quire-rs render retirement, commit 500a3d3), so the template-sandbox half of this
> need (`{% include %}`/`{% extends %}` / template FS reads) no longer applies. The
> surviving, load-bearing need is **process-boundary path safety** on `--module` and
> the document/data path arguments. The need and AC-3 are revised accordingly.

## Stakeholder Need

Agent-supplied paths and document/data flow into a process that loads schemas and module manifests from disk. Without explicit sandboxing, a malicious or malformed `--module` argument could escape the intended module root via symlinks or `..`.

The CLI MUST add **process-boundary path safety** on `--module` and on the positional document path argument (and `edit` write targets), canonicalizing the module root and refusing to follow symlinks that resolve outside it.

This matches the FR-035 manifest contract.

## Priority

Must-Have

## Acceptance

- **StR-003-AC-1**: `quire validate doc.md --module ../escape` exits 1 with a structured "path safety violation" diagnostic on stderr.
- **StR-003-AC-2**: `quire validate ../../etc/passwd --module $ISO` exits 1 with the same class of diagnostic.
- StR-003-AC-3 (RETIRED): A template containing `{% include "/etc/passwd" %}` is rejected at archetype load time by the upstream `quire-rs` FR-004 strict environment (verified via IT).
- **StR-003-AC-4**: Symlinks under `--module` are not followed past the canonicalized root.
