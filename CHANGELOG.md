# Changelog

All notable changes to `quire-cli` are documented here.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).
The public contract under SemVer is the subcommand surface, exit codes, and JSON
output schemas (see `spec/non-functional/NFR-006-cli-stability.md`).

## [Unreleased]

## [0.30.2] — 2026-08-22

**First release since 0.27.0 that actually publishes.** `Cargo.toml` sat at
0.29.0 while v0.28.0, v0.29.0, v0.30.0 and v0.30.1 were tagged, and the
release workflow is `workflow_dispatch`-only — so no tag ran it, and the one
dispatch that did run refused to publish a binary reporting a stale version.
Those four tags shipped nothing.

### Fixed

- **Engine bumped to `quire-rs` v0.44.1**, which corrects two checks v0.44.0
  shipped that were each measured against one corpus
  (`agent-ix/quire-rs#235`, `#229`).
  - `coverage` no longer reports `vacuous-under-guard` against TypeScript
    arrow functions. On `agent-ix/quoin` that was **549 suspicions from 551
    candidates**; it is now 0, and the genuine Rust positives are unchanged.
  - `coverage` no longer reports `hollow-denominator` for a count-shaped
    metric reading an honest zero. This repository's own
    `coverage.implements` reads 0 of 214 and was flagged as arithmetic over
    nothing.
  - Every metric in the `coverage --json` payload now carries a required
    `shape` (`ratio` | `count`), declared in `coverage-v1.schema.json`.

## [0.30.0] — 2026-08-22

Output-contract release. Part of the metric-integrity programme
(agent-ix/quoin#197).

### Changed

- **BREAKING (human surface): results go to stdout, diagnostics to stderr**
  (#59, #60, CR-012, FR-006-AC-5, FR-017-AC-1 amended). `quire coverage > out.txt`
  produced a **0-byte file** while 90,462 bytes went to stderr, and
  `Coverage: 1238/2390 rows backed (51%)` — a census — rendered in the same red
  as every finding.

  This corrects a contradiction rather than introducing a contract: FR-006 has
  required *primary result on stdout* since v0.1, and FR-017-AC-1 said the
  opposite. **`--json` and `--format tsv` are unaffected** — the census is
  emitted only in the human branch, so `| jq` is untouched and the #51 WONTFIX
  stands.

### Added

- **`properties --criteria`** (#59, FR-018-AC-10) renders one block per
  criterion — row id, `document:line`, shape, and the extraction spans. Those
  fields were `--json`-only, and `--json` on the pass-2 corpus is 597,636 bytes
  against an 869-byte census, so quoin's `spec-correctness` could not be driven
  from the compact surface at all. Defaults to the actionable set; `--all`
  includes `example` and `unclassified`.
- **The properties census carries the specific-shape split** (quire-rs CR-095),
  so `54%` no longer travels without the `8%` beside it.
- **Row ids in `validate`'s assert findings** (#58, engine CR-097). Was 15 of
  496 findings carrying an id, with one distinct line per document; now every
  row-scoped failure carries its own line and its declared `id_column` cell.
- **`suspicions` in the coverage payload** (engine FR-064): a property suite
  whose assertions may never run, and an oracle that copies the code it judges.

### Engine

- quire-rs **v0.44.0** (from v0.42.0): the metric provenance envelope, the
  binding census, the honest properties headline, the skeptic layer, the corpus
  benchmark and the cross-corpus overfit check.


## [0.29.0] — 2026-08-21

The first npm publish since 0.12.0. `release.yml` now asserts the built
binary's `--version` matches the version being released (#52) — the guard the
0.24.0–0.28.0 tags shipped without, every one of whose binaries reported
`0.23.0`.

### Added

- **`coverage` human findings are actionable lines (#51).** Every
  unbacked-row, status-lie, undeclared-status — and now no-symbol-row —
  census line leads with the row's own id and a clickable `document:line`
  locus (`TC-123 (spec/tests.md:9) has no backing symbol [traces-to]`),
  instead of the reference kind repeated identically per row. `no_symbol_rows`
  renders for the first time; what `source_exclude` subtracted is counted on
  the census, and `SymbolExtraction` diagnostics (refused glob list,
  unreadable source file) reach stderr instead of being dropped.
- **Agent-sized `coverage` output (#53).** A `coverage` severity pack —
  `--severity coverage:{unbacked-row|status-lie|untracked-symbol|undeclared-status}=<off|warning|error>`
  on the FR-048 machinery `validate` uses; `off` projects a kind out of every
  output surface (suppression announced with its count), `error` is a
  per-check gate. Totals and `--strict` always judge the full computation,
  never the projection. And `--format tsv`: one nine-column tab-separated
  record per line on stdout (~36% of the JSON size), the first rendered
  surface `no_symbol_rows`, `diagnostics`, `obligations` and `implements`
  have had. A typo'd coverage check in `--severity` is rejected, not silently
  ignored (#57).
- **`shared_trace_ids` and `vocabulary_coverage` pass through `--json`**
  (quire-rs v0.42.0 advisory lists), both absent when empty.

### Changed

- **quire-rs v0.40.0 → v0.42.0.** v0.41.0 brought `undeclared_statuses`
  reporting (CR-083) and `source_exclude` (CR-085), both wired to the command
  line in the same release; v0.42.0 adds 1-based `line` on the five finding
  record kinds, `excluded_source_files`, `shared_trace_ids` and
  `vocabulary_coverage`.
- **`coverage --json` honours the global `--pretty` (#53).** Compact
  single-line by default like every other JSON surface (FR-008-AC-1);
  `--pretty` restores the previous indented shape. Whitespace-only — the
  payload parses identically.

### Fixed

- npm launcher and platform packages had sat at 0.12.0 against a 0.23.0
  Cargo.toml; versions are staged in lockstep by `scripts/set_version.sh`
  and guarded in CI (#52).

## [0.24.0] – [0.28.0] — 2026-08-19 .. 2026-08-20

Tagged without CHANGELOG entries (and without `set_version.sh` — the defect
#52 closes; none of these reached npm). What each tag carried:

- **0.28.0** — quire-rs v0.41.0 capabilities reach the command line (#50):
  undeclared statuses on both surfaces, `source_exclude` wired to the walk.
- **0.27.0** — quire-rs v0.38.0 → v0.40.0 (#48).
- **0.26.2** — every published npm package declares AGPL-3.0, not MIT (#47).
- **0.26.1** — matrix trace/criterion bindings repaired (#44, #46).
- **0.26.0** — quire-rs v0.36.0 → v0.38.0 (#42).
- **0.25.1** — CI: bounded, retried musl toolchain install (#41).
- **0.25.0** — quire-rs v0.34.0 → v0.36.0 (#40).
- **0.24.0** — quire-rs v0.33.0 → v0.34.0 (#39).

## [0.23.0] — 2026-08-18

### Changed

- **quire-rs v0.30.0 → v0.33.0.** The engine had moved five releases ahead of
  this CLI, so **every capability added by ADR-0011 Phase 2 waves A–D was
  unreachable from any command line**:

  | Engine FR | What was unreachable |
  |---|---|
  | FR-057 | per-check corpus severity (`trace:`/`refs:`/`edges:`/`bundle:` keys) |
  | FR-058 | upward-trace completeness — orphan requirements and unimplemented needs |
  | FR-059 | declared-vocabulary coverage — which values no document claims |
  | FR-060 | `from_vocabulary` / `column_vocabularies` in body-extraction asserts |
  | FR-061 | combinatorial obligations from declared configuration dimensions |

  Nothing in this crate changed to expose them: `validate` already routes the
  corpus packs and `coverage` already emits the obligation contract, so the
  bump *is* the fix. That is also why it went unnoticed — the CLI kept working,
  and simply answered from an older engine.

## [0.22.0] — 2026-08-17

### Changed

- Engine bumped to **quire-rs v0.30.0** — the post-merge review follow-ups for
  the ADR-0011 P1 wave (agent-ix/quire-rs#150–#153, CR-063..CR-065). Reaching
  this CLI:
  - `coverage --json` gains two diagnostic reasons:
    `obligation-row-states-nothing` (a row whose statement cell is empty — the
    diagnostic FR-053-AC-8 always promised and never emitted) and
    `uncatalogued-verification-method` (a `Verification` cell naming neither a
    catalog method id nor a catalog class, which nothing reported before).
    `diagnostics[].reason` is a deliberately open vocabulary, so neither is a
    contract break for a consumer that pins the published schema.
  - `statement_hash` now normalizes to **NFC** before trimming, so an editor
    rewriting a decomposed accent no longer reads as a reworded requirement.
  - The `obligations` list is ordered by source **declaration** order rather
    than source name.

- **`properties --json` and `validate --summary` now pass each document's
  scope-relative path to the engine** (new **FR-018-AC-7**, IT-098). quire-rs
  FR-053-AC-14 makes an obligation source's `exclude:` globs bind the
  classification surface as well as the coverage rollup, and it can only do so
  if this crate hands over the path. Before, a criterion in an excluded fixture
  stated no obligation in `coverage --json` and stated one here — and this
  payload is what `spec-correctness` generates property tests from, so the
  asymmetry became a generated test carrying a trace tag for an id nothing
  mints. Stdin passes no path, having no location a glob could match.

### Fixed

- `make fmt-check` was red on `main`: `tests/output_contract.rs` landed
  unformatted in v0.21.0 (#37). The same class of miss as
  agent-ix/quire-rs#150, found by the same review.

## [0.21.0] — 2026-08-17

### Changed

- Engine bumped to **quire-rs v0.29.0** — the ADR-0011 engine surface
  (agent-ix/quire-rs#81 P1: FR-053 obligation record, FR-054 verification-method
  catalog, FR-055 published output contract, FR-056 requirement-quality lints).
  Reaching this CLI:

  - **`properties --json` records gain `obligation`** (quire-rs FR-053).
    `null` for a module declaring no `traceability.obligations:` source, so a
    corpus that has not adopted them sees the key with a null rather than a
    shape change. The nested object carries `source`, `statement_hash`,
    `method`, `criticality` and optional `parameters` — and deliberately **not**
    `id`, `statement` or `document`, because the record and its enclosing object
    already carry all three.
  - **`coverage --json` gains `obligations`**, absent when the model declares no
    sources — so the payload is byte-identical for every module that has not
    adopted them.
  - **`validate` gains the `quality:*` grammar** (quire-rs FR-056):
    `ambiguous-term`, `agentless-passive`, `mixed-modal`. All **advisory**, and
    each addressable by `--severity quality:<check>=off|warning|error` like any
    other check. **Report change**: measured across 239 repositories,
    20.2% of FR/NFR/StR documents gain at least one warning.

### Added

- **Output-contract conformance tests** (IT-095, IT-096) validating the emitted
  `properties --json` envelope against quire-rs's published
  `properties-v1.schema.json`. The engine publishes both schemas and gates the
  parts it emits; it never constructs this envelope, so without a test here the
  published schema would describe a shape nothing checked. The schema is read
  from the resolved quire-rs source rather than vendored — a copy is a second
  artifact that drifts.

## [0.20.0] — 2026-08-17

### Changed

- Engine bumped to **quire-rs v0.28.0** — archetype-only trace binding
  (quire-rs CR-062). Behavior reaching this CLI:
  - **A module declaring `document:` on a trace target or a document reference
    no longer loads.** The key is retired and the nested structs are
    `deny_unknown_fields`, so `quire validate` / `quire coverage` fail loudly
    against a stale module rather than silently minting nothing. Pair this CLI
    with `spec-artifacts-process` **v0.14.0** or later, which ships the matching
    collapse of nine declarations to three.
  - **`coverage --json` reaches nested matrices.** Path binding enumerated one
    target per filename convention and could not see
    `spec/<module>/matrix/tests.md`; archetype binding types the document
    instead. **Report change**: repositories authoring nested module matrices
    gain minted ids and backed rows — measured across 238 repositories, dead
    trace tags fall 1,401 → **1,207** occurrences, and `filament-ide-rs` alone
    goes 17/850 → **473/2,184** rows backed.
  - **A mistyped matrix now mints nothing**, where under path binding
    frontmatter was irrelevant. **Report change**: a repository whose Test
    Matrix declares the wrong `type:` sees its test-case ids disappear — the
    fix is to correct the frontmatter, and the six ecosystem cases were
    corrected before quire-rs cut the release.
  - The `unreadable-declared-document` and `absent-declared-document` machine
    reasons are **withdrawn** — v0.19.0 shipped them for the code path CR-062
    deletes. `archetype-matches-nothing` is the surviving reason. Anything
    keying on the two withdrawn tokens must migrate.

  Cut now because the ADR-0011 verification program (agent-ix/quire-rs#81)
  works against the installed CLI: an engine-before-module release ordering is
  unverifiable if the CLI the modules are validated with lags the engine.

## [0.19.0] — 2026-08-16

### Changed

- Engine bumped to **quire-rs v0.27.0** — the SR-007 blockers
  (quire-rs CR-059..CR-061). Behavior reaching this CLI:
  - `coverage --json` distinguishes an **absent** declared auxiliary
    `document:` from an **unreadable** one: `absent-declared-document` is a new
    machine reason, and `unreadable-declared-document` narrows to the
    always-wrong case (quire-rs FR-050-AC-19). A fleet module shipping an
    optional declaration across many repositories no longer reports a fault
    where there is none.
  - a model-level `traceability.exclude:` scopes the criteria walk as well as
    every declaration (quire-rs FR-050-AC-13/15). **Report change**: a
    repository declaring the new key with criteria under those paths sees
    smaller `totals.criteria` / `totals.property_shaped`.
  - trace tags on **benchmarks** and **fuzz targets** now bind — a
    `criterion_group!`-registered function or a `fuzz_target!` invocation is
    leaf evidence, where before it minted no binding (quire-rs FR-051-AC-17).
    **Report change**: coverage rises for repositories whose benches or fuzz
    targets carry tracking tags, and correspondingly fewer tags land in
    `untracked_symbols`.

  This last one is why the bump is cut now rather than batched: the corpus
  measurements on agent-ix/quire-rs#75 and #78 must run on an engine that binds
  leaf evidence, or their numbers are stale on arrival.

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
