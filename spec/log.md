---
type: log
title: "Update Log"
description: "Chronological log of structural changes to this bundle."
---
# Update Log

## History

* **2026-06-15** — Adopted OKF-compatible bundle structure with directory indexes.
* **2026-06-16** — Added [FR-014](./functional/FR-014-validate-okf-bundle.md) (`quire validate --okf` permissive OKF bundle posture: `type` required, unknown-type/broken-link/index-incompleteness warn). Added [FR-003-AC-5](./functional/FR-003-extract-subcommand.md) (extract emits shared `[frontmatter]` untyped-document diagnostic). Backsynced the `artifact_type` → `type` discriminator rename across [FR-003](./functional/FR-003-extract-subcommand.md)/004/007/013 and spec.md via CR notes. Mapped IT-069..072 (`tests/cli_okf.rs`) + IT-026 reuse in tests.md.
* **2026-06-17** — Added [FR-015](./functional/FR-015-fix-subcommand.md) (`quire fix` subcommand, ADR 0007): surfaces quire-rs unlinked-reference suggestions (FR-039) and, with `--write`, applies the auto-fixable ones via byte-exact writeback. Dry-run lists `would-fix`/`warning` and exits 1 when auto-fixes remain (CI gate); `--write` is idempotent; warn-only (unresolved/ambiguous) tokens are never written. Mapped IT-076..080 + AUDIT-002 (thin boundary) in tests.md.
* **2026-06-19** — [FR-004](./functional/FR-004-validate-subcommand.md) scoped discovery now also searches the default install root `~/.ix/filament/modules` and `IX_FILAMENT_MODULES_PATH` (preferred over the legacy `IX_SCHEMA_PATH`); on zero discovered modules it lazy-installs the default set via `quoin plugin ensure-defaults` and reloads once (FR-004-AC-13/AC-14). Added [ADR-0001](./assets/adr/0001-validate-lazy-init-module-bootstrap.md) and amended [NFR-004](./non-functional/NFR-004-no-network.md) via CR note: the no-network guarantee is scoped to quire's own process; the lazy-init's `quoin` child is the sole documented network exception. Mapped IT-081 (scoped discovery network-free) + IT-082 (quoin-absent actionable error) in tests.md.
