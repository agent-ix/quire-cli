# Changelog

All notable changes to `quire-cli` are documented here.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).
The public contract under SemVer is the subcommand surface, exit codes, and JSON
output schemas (see `spec/non-functional/NFR-006-cli-stability.md`).

## [Unreleased]

### Added
- Prebuilt binaries for five targets published on each tag: `x86_64`/`aarch64`
  Linux (musl, static), `x86_64`/`aarch64` macOS, and `x86_64` Windows.
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

[Unreleased]: https://github.com/agent-ix/quire-cli/compare/v0.2.1...HEAD
[0.2.1]: https://github.com/agent-ix/quire-cli/compare/v0.2.0...v0.2.1
[0.2.0]: https://github.com/agent-ix/quire-cli/compare/v0.1.1...v0.2.0
[0.1.1]: https://github.com/agent-ix/quire-cli/compare/v0.1.0...v0.1.1
[0.1.0]: https://github.com/agent-ix/quire-cli/releases/tag/v0.1.0
