---
type: log
title: "Update Log"
description: "Chronological log of structural changes to this bundle."
---
# Update Log

## History

* **2026-09-09** — Specified the scoped LR05 npm distribution Rust port in
  [US-007](./usecase/US-007-install-quire-through-npm.md),
  [FR-022](./functional/FR-022-npm-launcher-process-contract.md),
  [FR-023](./functional/FR-023-rust-npm-package-assembly.md),
  [FR-024](./functional/FR-024-rust-release-version-contract.md), and
  [NFR-008](./non-functional/NFR-008-native-npm-distribution-tooling.md).
  [ADR-0002](./assets/adr/0002-minimal-node-npm-launcher.md) proposes the
  exact minimal Node host seam and awaits owner disposition. Package decisions,
  generation, version validation, and assertions move to Rust; WASM, Quire
  language/profile semantics, extraction redesign, and generic GitHub release
  archive/checksum cleanup remain out of this change.

* **2026-09-09** — Base `/spec-review`
  [SR-062](./reviews/SR-062-npm-distribution-rust-port-base-review.md) closed
  four drafting gaps and found no scope expansion. The reviewed specification
  is conditional only on the ADR-0002 owner choice; implementation has not
  started.

* **2026-09-09** — The owner approved ADR-0002's bounded Node host for the
  cross-platform npm distribution package. SR-062 FND-004 is closed and the
  reviewed #61 implementation scope is admitted.

* **2026-09-09** — Implemented and locally qualified
  [NFR-007](./non-functional/NFR-007-exact-qualified-rust-toolchain.md). The
  manifest, repository toolchain, Clippy policy, and four manual workflow
  selectors now name Rust 1.98.1; the engine pin is quire-rs #422 merge
  `85dfe9d5a937c52af6456f2e6aa3a6bc4c82db9f`. TC-142 is a native Rust,
  `ix-trace-rs`-marked audit that kills changes and removals across all seven
  declarations plus a `.yaml` escape. Qualification repaired the new Clippy
  and rustdoc findings, removed unsupported nightly rustfmt options, upgraded
  advisory-affected `anyhow` and `crossbeam-epoch`, and made full cargo-deny
  plus cargo-audit recurring local gates. SR-059 records all results; no hosted
  CI or release ran.

* **2026-09-09** — Specified
  [NFR-007](./non-functional/NFR-007-exact-qualified-rust-toolchain.md): one
  exact Rust 1.98.1 policy across the manifest, repository toolchain, Clippy
  policy, and all manual CI/release selections, with a mutation-sensitive
  exhaustive audit and local qualification against the already-qualified
  `quire-rs` revision. SR-057 established the preimplementation gate for #82.

* **2026-09-06** — The #409 integration followup advances FR-020-CON-1 and
  IT-145 to canonical-CI-qualified engine
  `616a7e97c0e8c84aedda71dc198e94e3de3d9da6`. Its four seeded validation-stack
  test pins now match the reviewed module lock; CLI logic, engine runtime
  behavior, the assurance schema, and QA7442 are unchanged.

* **2026-09-06** — The #409 consumer join specifies FR-017-AC-21 and banks
  native IT-152/153 before advancing the engine pin to
  `616a7e97c0e8c84aedda71dc198e94e3de3d9da6`. Two selected tables exercise
  exact global/override headers and default-versus-strict missing-header
  behavior. FR-020-CON-1 and IT-145 retain exact source agreement; no CLI
  production logic or engine-owned assurance schema changes.

* **2026-09-06** — IR51-02: FR-017-AC-22 makes strict coverage fail on the
  engine's structured unread-status and hollow-denominator diagnostics. The
  policy reads the full report, retains report output and default reporting,
  and does not promote unrelated advisories. IT-150/IT-151 precede code.

* **2026-09-01** — Specified [FR-020](./functional/FR-020-assurance-export-subcommand.md)
  and [US-006](./usecase/US-006-export-assurance-facts.md): a deterministic,
  non-executing `quire assurance` boundary over quire-rs `assurance-v1`.
  Source and module premises are explicit, expected module/schema premises are
  exact and fail closed, JSON remains the unmodified upstream envelope, and
  all failures leave stdout empty. The compatibility boundary pins quire-rs
  0.46.0 merge `e3352a0644abcfd5f0ebad348bc7aca235925ecc`. #74;
  agent-ix/quire-rs#389; agent-ix/engineering-assurance#7.

* **2026-08-29** — Canonical Cargo resolution is now locked across build,
  test, lint, license and bans surfaces, and the executable drift audit rejects
  any future unlocked resolver line. CI names a missing `REGISTRY_TOKEN` before
  private-engine fetch, and the engine pin advances to Quire's corresponding
  all-surface drift guard. The final pin also includes the governed benchmark's
  full-SHA corpus/module checks and schema-v2 exporter's consumer, toolchain,
  and executable-digest checks, so this workspace is itself the exact consumer
  the producer attests. #71; agent-ix/quire-rs#379.

* **2026-06-15** — Adopted OKF-compatible bundle structure with directory indexes.
* **2026-06-16** — Added [FR-014](./functional/FR-014-validate-okf-bundle.md) (`quire validate --okf` permissive OKF bundle posture: `type` required, unknown-type/broken-link/index-incompleteness warn). Added [FR-003-AC-5](./functional/FR-003-extract-subcommand.md) (extract emits shared `[frontmatter]` untyped-document diagnostic). Backsynced the `artifact_type` → `type` discriminator rename across [FR-003](./functional/FR-003-extract-subcommand.md)/004/007/013 and spec.md via CR notes. Mapped IT-069..072 (`tests/cli_okf.rs`) + IT-026 reuse in tests.md.
* **2026-06-17** — Added [FR-015](./functional/FR-015-fix-subcommand.md) (`quire fix` subcommand, ADR 0007): surfaces quire-rs unlinked-reference suggestions (FR-039) and, with `--write`, applies the auto-fixable ones via byte-exact writeback. Dry-run lists `would-fix`/`warning` and exits 1 when auto-fixes remain (CI gate); `--write` is idempotent; warn-only (unresolved/ambiguous) tokens are never written. Mapped IT-076..080 + TC-090 (thin boundary) in tests.md.
* **2026-06-20** — Added [FR-016](./functional/FR-016-update-subcommand.md) (`quire update` subcommand): install-source-aware self-update that detects the install channel from `current_exe` (`node_modules` ⇒ npm, `.cargo` ⇒ cargo, else unknown→manual instructions) and drives the matching package manager. No cross-scheme version diff (npm-wrapper and Cargo versions are decoupled); idempotency delegated to npm/cargo. A `--registry` override is applied as the scope-specific `--@agent-ix:registry=` form. Logic lives in a package-agnostic `self_update` engine (kit-extraction unit); the command is a thin wrapper. Deliberate exception to the quire-rs thin boundary ([StR-004](./stakeholder/StR-004-thin-boundary-over-quire-rs.md)) since it manages binary lifecycle, not artifact behavior.
* **2026-06-19** — [FR-004](./functional/FR-004-validate-subcommand.md) scoped discovery now also searches the default install root `~/.ix/filament/modules` and `IX_FILAMENT_MODULES_PATH` (preferred over the legacy `IX_SCHEMA_PATH`); on zero discovered modules it lazy-installs the default set via `quoin plugin ensure-defaults` and reloads once (FR-004-AC-13/AC-14). Added [ADR-0001](./assets/adr/0001-validate-lazy-init-module-bootstrap.md) and amended [NFR-004](./non-functional/NFR-004-no-network.md) via CR note: the no-network guarantee is scoped to quire's own process; the lazy-init's `quoin` child is the sole documented network exception. Mapped IT-081 (scoped discovery network-free) + IT-082 (quoin-absent actionable error) in tests.md.

* **2026-08-15** — **Two roots from one scope, recorded after the fact.** PR #27 changed two documented acceptance criteria's behavior without touching a single spec file. [FR-014](./functional/FR-014-validate-okf-bundle.md)-AC-6 ("validates the `--scope` directory as the bundle root") and [FR-015](./functional/FR-015-fix-subcommand.md)-AC-5 ("uses `--scope` as the bundle root") both now derive `<scope>/spec` through the shared `spec_root_of` helper, and a scope with no `spec/` is a named error rather than a silent repository-wide crawl. Both artifacts gain a CR note; `--help`, `README.md` and the two missing CHANGELOG releases (0.16.0, 0.17.0) are corrected in the same pass, since a **breaking traversal change** that can newly fail `coverage`/`validate --okf`/`fix` on an existing repository was unexplained anywhere — against this repo's own SemVer contract in [NFR-006](./non-functional/NFR-006-cli-stability.md). Stale `validate_bundle_at` references corrected to the two-root `validate_bundle` in the `run_okf` doc comment, FR-014's body and TC-090's matrix row; the two `--okf` root tests gain IT tags; and `spec_root_of`'s doc comment, which had been inserted *inside* `load_module_registry`'s `///` block so rustdoc attached it to the wrong item and left `load_module_registry` undocumented, is split back apart. Closes agent-ix/quire-cli#30 (umbrella agent-ix/quire-rs#106, from SR-006).

* **2026-08-16** — **`coverage` and `properties` gain owning requirements**, and the two-root test gaps close. [FR-017](./functional/FR-017-coverage-subcommand.md) and [FR-018](./functional/FR-018-properties-subcommand.md) are authored for commands that shipped in v0.13.0 and had **no FR, no acceptance criteria and no matrix rows at all** — the most behavior-visible surface added in three releases, changing its default root in PR #27 and what it parses in PR #29 with nothing to be measured against. The criteria are read off working code rather than proposed (the quire-rs CR-042 backfill pattern), and writing them down corrected two claims: the human census renders on **stderr**, not stdout, so `--json` owns stdout alone; and the `properties` payload is a `{documents: [{document, archetype, criteria}]}` envelope, not a bare record array. Test gaps from agent-ix/quire-cli#31: `fix`'s default-root change had **zero** coverage (IT-080 now asserts both that `<scope>/spec` is the root and that a repo-root file is never walked); the missing-root tests asserted only `contains("spec")`, which the unrelated "install spec-artifacts-process" refusal also satisfies (now the interpolated path, plus IT-086 for the typed `MissingDocumentRoot` kind); nothing asserted the *second* half of the derivation, that the code walk excludes `spec/` (IT-087, which a regression to `extract_tree` fails); the extract edge test asserted only that `edges` is an array while the fixture declares a real relationship, so an empty harvest passed both it and the determinism test (now asserts the target and type); and the positional `--okf` form's deliberately different path resolution is stated in FR-014 and the README rather than left implicit. IT-088 covers the machine-surface half of agent-ix/quire-rs#110 end to end: a non-fatal bundle warning now carries `severity: "warning"`, where it used to be emitted through the error path with `severity: "error"` while the exit code correctly said otherwise. Engine bumped to **quire-rs v0.26.0**; `spec_root_of` canonicalizes, `coverage` applies the same path-safety guard `validate` always did, and the exclusion is derived from one `DOCUMENT_ROOT_DIR` constant instead of a second `"spec"` literal (agent-ix/quire-rs#113). Closes agent-ix/quire-cli#31 and the CLI halves of agent-ix/quire-rs#110 and #113 (umbrella agent-ix/quire-rs#106, from SR-006).
