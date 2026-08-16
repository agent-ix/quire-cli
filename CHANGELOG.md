# Changelog

All notable changes to `quire-cli` are documented here.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).
The public contract under SemVer is the subcommand surface, exit codes, and JSON
output schemas (see `spec/non-functional/NFR-006-cli-stability.md`).

## [Unreleased]

## [0.18.0] — 2026-08-16

### Changed

- Engine bumped to **quire-rs v0.26.0** — the SR-006 review follow-up program
  (quire-rs CR-050..CR-058). Behavior reaching this CLI:
  - a declared `document:` that cannot be read, a declared archetype no
    document has, and a model with no trace targets are now **reported** in
    `coverage --json` under a new `diagnostics` key instead of failing open
    (quire-rs FR-050-AC-19). The key is absent when empty, so a healthy
    repository's report is byte-identical to before.
  - the malformed-frontmatter warning carries its own machine reason,
    `malformed-frontmatter`, distinct from `no-frontmatter`
    (quire-rs FR-024-AC-12).
  - `## 3.2 Ubiquitous Language` and other ISO-numbered headings contribute
    glossary terms again (quire-rs FR-044-AC-8).
  - the code walk's document-root exclusion is compared by canonicalized
    identity, so a case-insensitive filesystem or a symlinked `spec/` no
    longer ingests every spec document a second time as source.

### Fixed

- **`--diagnostics json` emitted non-fatal bundle warnings with
  `"severity": "error"`.** Every `validate --okf` warning took the error path,
  which hardcodes error severity, so the machine surface contradicted the exit
  code — which was correctly 0. Warnings now carry `"severity": "warning"` and
  `"kind": "ValidationWarning"`.
- **A symlinked `spec/` produced a silent empty corpus.** `spec_root_of` gated
  on `is_dir()`, which follows symlinks, while the corpus walker does not — so
  the check passed and the run reported `total: 0` and exit 0. The derived root
  is now canonicalized, which also makes `coverage` and `validate --okf` resolve
  the same document root for the same repository; they previously differed,
  since only `validate` canonicalized.
- **The missing-document-root error was a formatted string.** It is now a typed
  `DocumentRootError` carrying a stable `MissingDocumentRoot` kind into
  `--diagnostics json`, so a consumer can branch on it instead of matching prose.
- `coverage` now applies the same path-safety guard to its derived document root
  that `validate` has always applied to its bundle root.

### Added

- **[FR-017](spec/functional/FR-017-coverage-subcommand.md)** and
  **[FR-018](spec/functional/FR-018-properties-subcommand.md)** — `coverage` and
  `properties` shipped in v0.13.0 with no owning requirement, no acceptance
  criteria and no matrix rows. Both are now specified from working code, with
  IT-086..IT-097 covering them. Writing them down corrected two documented
  claims: the human census renders on **stderr** (stdout carries only the
  `--json` payload), and the `properties` payload is a
  `{documents: [{document, archetype, criteria}]}` envelope.
- `fix`'s default-root behavior has coverage for the first time (IT-080), as
  does the code walk's exclusion of `spec/` (IT-087).


## [0.17.0] — 2026-08-15

### Changed

- Engine bumped to **quire-rs v0.25.0**: lazy document bodies, declaration-driven
  body selection, and the frontmatter-less warning inversion (quire-rs CR-046..049).
- A markdown file under the document root with **no frontmatter block** is no
  longer silently ignored: it emits one non-fatal warning naming its path
  (quire-rs FR-024-AC-10). Silence was justified only by tolerating a
  repository-root walk, which 0.16.0 removed — what remains inside `spec/` is
  almost certainly an authoring mistake.
- `validate --okf` now calls `quire_rs::validate_bundle` with the document root
  and the reference root stated separately (quire-rs FR-049-AC-9), so a module's
  `document:`/`exclude:` declarations keep resolving against the repository
  scope.

## [0.16.0] — 2026-08-15

### Changed

- **BREAKING (traversal).** `coverage`, `validate --okf` and `fix` derive **two
  roots from one `--scope`**: the corpus is walked from `<scope>/spec`, while
  the code walk and the module's path-bound declarations keep using `<scope>`
  (quire-rs CR-045, FR-050-AC-17). Engine bumped to **quire-rs v0.24.0**.
- A `--scope` with no `spec/` directory now **exits non-zero** with a diagnostic
  naming the missing document root, instead of silently walking the scope.
  `quire validate --okf --scope path/to/bundle` therefore fails unless
  `path/to/bundle/spec` exists — pass a self-contained bundle as the
  **positional** argument instead, which is honored as given.
- Repository-root files (`README.md`, `CHANGELOG.md`, `plan/*.md`) are no longer
  read as spec documents. **[RAN]** this removed 9,172 `required 'type' is
  missing` errors across 223 repositories — because those files are never
  visited, not because they were classified away.
- The minted-id set over a compliant repository (documents under `spec/`) is
  **byte-identical** to a pre-split run: `--scope` remains the relativization
  base for every emitted path.

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
