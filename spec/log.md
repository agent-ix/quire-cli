---
type: log
title: "Update Log"
description: "Chronological log of structural changes to this bundle."
---
# Update Log

## History

* **2026-06-15** — Adopted OKF-compatible bundle structure with directory indexes.
* **2026-06-16** — Added FR-014 (`quire validate --okf` permissive OKF bundle posture: `type` required, unknown-type/broken-link/index-incompleteness warn). Added FR-003-AC-5 (extract emits shared `[frontmatter]` untyped-document diagnostic). Backsynced the `artifact_type` → `type` discriminator rename across FR-003/004/007/013 and spec.md via CR notes. Mapped IT-069..072 (`tests/cli_okf.rs`) + IT-026 reuse in tests.md.
* **2026-06-17** — Added FR-015 (`quire fix` subcommand, ADR 0007): surfaces quire-rs unlinked-reference suggestions (FR-039) and, with `--write`, applies the auto-fixable ones via byte-exact writeback. Dry-run lists `would-fix`/`warning` and exits 1 when auto-fixes remain (CI gate); `--write` is idempotent; warn-only (unresolved/ambiguous) tokens are never written. Mapped IT-076..080 + AUDIT-002 (thin boundary) in tests.md.
