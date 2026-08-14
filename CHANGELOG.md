# Changelog

All notable changes to `quire-cli` are documented here.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).
The public contract under SemVer is the subcommand surface, exit codes, and JSON
output schemas (see `spec/non-functional/NFR-006-cli-stability.md`).

## [Unreleased]

## [0.15.0] — 2026-08-14

### Changed

- Engine bumped to **quire-rs v0.21.0**. A legacy trace comment carrying a
  comma-separated list now binds every id it names rather than only the first
  (quire-rs FR-051-AC-16). Paired with `spec-artifacts-process` v0.13.0, which
  widens the declared patterns so there is a list to split — **[RAN]** 205 ids
  across 17 repos start binding with no source edit. Closes agent-ix/quire-rs#68.

## [0.14.0] — 2026-08-14

### Changed

- Engine bumped to **quire-rs v0.20.0**, which brings the traceability model two
  declarations it could not express and stops the symbol adapters losing whole
  files:
  - `exclude:` path globs on trace targets and document references, and
    `archetype` + `document` declared together (FR-050-AC-15).
  - `vocabularies.no_source_symbol` — verification methods that mint no source
    symbol, so `coverage` explains an eval row rather than accusing it
    (FR-050-AC-16). `CoverageReport` gains `no_symbol_rows`, absent when the
    active module declares no such vocabulary, so existing output is unchanged.
  - Rust and TypeScript source scanning is string-aware in one lexer pass
    (FR-051-AC-14/15). 33 files in quire-rs alone had been rejected as
    `unbalanced braces` and yielding zero symbols, so every trace tag in them
    bound to nothing.

This release is what unblocks `spec-artifacts-process` declaring
`no_source_symbol`: a manifest key fails module load outright against an engine
that does not know it, so the CLI has to ship first.

> No changelog entry was written for 0.13.0; this entry does not attempt to
> reconstruct it.

## [0.12.0] — 2026-08-08

### Added
- **`quire properties`** — per-criterion property-shape classification
  (quire-rs FR-052). Emits `row_id`, `statement`, `line`, `shape`, `property`,
  `extractable`, `extraction`, the `{domain, precondition, oracle}` spans and
  the `signals` audit trail, as JSON under `--json` or a census otherwise.
  `quire coverage --json` emits only per-document counts, so the per-criterion
  records a property-test generator reads had no CLI surface before this.
  Never a finding: classification carries no severity and no check id and is
  not addressable by the FR-048 `grammar_severity` registry (FR-052-CON-1).
- **`quire validate --summary`** now also prints the property-extractable
  ratio and the candidate count. Computed by calling the engine directly, not
  by reading a warning message back — classification emits no message, and
  routing it through one would make it a finding.

### Changed
- Pinned to quire-rs **v0.18.0**.

### Fixed
- **`release.yml` could never publish.** The workflow is `workflow_dispatch`
  only by policy, but both publish steps were gated on
  `github.event_name == 'push'`, so a dispatch built four binaries and skipped
  the GitHub Release and the npm publish alike — which is why npm sat at 0.4.1
  against a 0.11.0 `Cargo.toml`. Publishing is now an explicit `publish` input,
  defaulting to false, and the release tag is derived from the resolved version
  rather than from `GITHUB_REF_NAME`.

## [0.2.4] — 2026-06-15

### Added
- `quire validate` now accepts one or more document paths/globs and a scoped
  validation mode: `quire validate --scope <dir> <glob> [glob...]`.
- Scoped validation resolves relative globs under `--scope`, loads repo/module
  search roots, and validates each document using frontmatter `artifact_type`.

### Changed
- `--module` remains available as the exact single-module compatibility path,
  while scoped validation is the ergonomic default for changed spec files.

## [0.2.3] — 2026-06-14

### Added
- Prebuilt binaries for four targets published on each tag: `x86_64`/`aarch64`
  Linux (musl, static), `aarch64` macOS, and `x86_64` Windows.
- npm distribution: `@agent-ix/quire-cli` (GitHub Packages) with per-platform
  optional dependencies carrying the prebuilt binary — no source build or access
  to the private `quire-rs` repo required to install.
- `scripts/set_version.sh` single-sources the release version across `Cargo.toml`,
  the npm packages, and this changelog.

### Changed
- Release profile now strips symbols and uses `panic = "abort"`, so a panic
  SIGABRTs to exit 134 as documented in FR-007.

## [0.2.1] — 2026-06-12

### Changed
- Bump `quire-rs` to v0.4.2 (CR-007: escaped pipes in table cells).

## [0.2.0] — 2026-06-11

### Added
- `quire lint` subcommand — evaluate a module's advisory lint rules against a
  document (FR-013).

### Changed
- Surface module eager-load failures instead of deferring them (FR-004 CR).

## [0.1.1] — 2026-06-06

### Changed
- Depend on `quire-rs` via a pinned git tag instead of a sibling path dependency.

## [0.1.0] — 2026-05-28

### Added
- First release. `quire` binary with `parse`, `extract`, `lookup`, `edit`,
  `validate`, and `schema` subcommands over `quire-rs`. (The render subcommand
  was removed upstream before this line stabilized — see `spec/spec.md` §2bis.)
- Path-safety guard, stdin/stdout/stderr contract, exit-code contract, and JSON
  output encoding (FR-005..008).
- Static-binary, zero-unsafe, no-network, and CLI-stability gates (NFR-002..006).

[Unreleased]: https://github.com/agent-ix/quire-cli/compare/v0.2.4...HEAD
[0.2.4]: https://github.com/agent-ix/quire-cli/compare/v0.2.3...v0.2.4
[0.2.3]: https://github.com/agent-ix/quire-cli/compare/v0.2.1...v0.2.3
[0.2.1]: https://github.com/agent-ix/quire-cli/compare/v0.2.0...v0.2.1
[0.2.0]: https://github.com/agent-ix/quire-cli/compare/v0.1.1...v0.2.0
[0.1.1]: https://github.com/agent-ix/quire-cli/compare/v0.1.0...v0.1.1
[0.1.0]: https://github.com/agent-ix/quire-cli/releases/tag/v0.1.0
