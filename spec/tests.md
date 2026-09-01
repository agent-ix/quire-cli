---
id: TM-001
title: "quire-cli Test Matrix"
type: TestMatrix
---

# Test Matrix

## Overview

This matrix maps every Acceptance Criterion in `quire-cli/spec/` to one or more Test Cases. Ids carry one of the two declared evidence-artifact prefixes — `IT-XXX` for an integration test, `TC-XXX` for everything else — and the `Type` column says what kind of evidence each row is (`Integration`, `Unit`, `Benchmark`, `Static`). The benchmark and static-analysis gates formerly numbered `BENCH-XXX` / `AUDIT-XXX` are now `TC-` rows whose `Type` states the same thing (spec-artifacts-process CR-019).

The CLI is a thin process boundary over `quire-rs`; the upstream engine is independently covered by `quire-rs/spec/tests.md`. This matrix tests **only** the CLI's process-level behavior: argv parsing, path-safety, stdin/stdout/stderr contract, exit codes, JSON output encoding, and static-binary properties.

> **Render removal (2026-06-04):** The `render` subcommand, the `validate --json`
> context mode, and the render benchmark are **removed** (see `spec.md` §2bis,
> mirroring quire-rs commit 500a3d3). Render/parity traces (IT-001, IT-009, IT-010,
> IT-017, IT-018, TC-088) and the `--json` context traces (IT-003, IT-050 as
> written) are **retired** — rows marked ⊘ RETIRED below, ids retained, dropped from
> the coverage tally. IT-014 is re-pointed to a direct-markdown sweep (no
> render-then-validate). The retired FR-001/US-001/NFR-001/StR-002 ACs no longer
> require a live trace.

## Matrix Rules

1. **Coverage Rule** — every AC has at least one IT / BENCH / AUDIT trace.
2. **Path-safety boundary rule** — every user-supplied path argument is exercised with `..`, with a symlink escape, and with a valid in-tree path.
3. **Exit-code rule** — every exit code in FR-007 has at least one IT producing it.
4. **Subcommand permutation rule** — for each subcommand (`parse`, `extract`, `lookup`, `edit`, `validate`, `schema`, `assurance`), the success path and each applicable argument/data failure path have a dedicated IT. (`render` removed — §2bis.) `validate` additionally has the `--okf` permissive bundle posture: its hard-error (untyped), warn (unknown-type / broken-link / index-incomplete), and scope-default paths each have a dedicated IT (IT-069..072). `fix` (ADR 0007 unlinked-reference autofix) has its dry-run (would-fix → exit 1), `--write` apply + idempotent re-run, warn-only, clean-bundle, and `--scope`/path-safety paths each covered (IT-076..080). `assurance` has a complete golden export, exact-premise refusals, empty-success distinction, channel separation, no-execution audit, and cross-language schema validation (IT-136..145, TC-814).
5. **Determinism rule** — primary JSON outputs (`parse`, `extract`, `lookup`, `schema`) have deterministic field order through Rust struct serialization.
6. **No-network rule** — IT-008 verifies zero `socket()` calls under strace across all subcommands on their happy path (registry present); IT-081 covers scoped discovery finding modules without network. IT-083/IT-084's no-install/no-network clause is strace-enforced by `update --check` / bare `update` probes in `audit_no_network.rs` — before #56 it held by construction only. The scoped-`validate` empty-discovery lazy-init spawns `quoin` to bootstrap modules — the documented NFR-004 exception (ADR-0001), out of these traces' scope.
7. **Status flip rule** — a row's Status flips to ✅ only on a **recorded passing run** of its backing test: the exact command and result line recorded in a review artifact (`reviews/SR-006-gap-analysis-matrix-flips.md` is the template). The coverage engine's `backed` predicate is a symbol claim, not a run — a tag can resolve to a test that fails, is `#[ignore]`d, or asserts nothing — so backed alone never justifies a flip (#55).

---

## Stakeholder Coverage

| StR | Trace to US/FR | Verifying IT/BENCH/AUDIT | Status |
|-----|---------------|--------------------------|--------|
| StR-001 Static binary hot path (revised — surviving subcommands) | US-002, US-003, US-004, US-005, FR-002..012, FR-016 (binary-lifecycle: keeps the pinned binary current) | IT-002, IT-004, IT-047, IT-033, IT-083, TC-089 (ldd), TC-091 (no-network) | ✅ |
| StR-002 Sub-50 ms render budget | ⊘ RETIRED (§2bis) | — (render bench removed) | ⊘ |
| StR-003 Sandbox inheritance (revised — path-safety) | FR-005 | IT-005 (..), IT-006 (symlink escape), IT-055 (doc path safety) | ✅ |
| StR-004 Thin boundary | FR-002..004, FR-009, FR-011, FR-014, FR-020, NFR-005 | TC-090, TC-814 (src audits for parser/graph/schema duplication), IT-033..038, IT-136 | 🚧 |

## User Story Coverage

| User Story | Acceptance Criteria | Test Cases | Coverage Status |
|----|----|----|--------|
| US-001 Agent renders FR | ⛔ RETIRED (§2bis) | IT-001, IT-009, IT-010, TC-088 (all retired) | ⛔ |
| US-002 Human parses doc | AC-1..4 | IT-002, IT-011 (stdin), IT-012 (malformed frontmatter), IT-013 (empty doc) | ✅ |
| US-003 CI validates | AC-1..3 | IT-003, IT-014 (parametric across 8 ISO archetypes) | ✅ |
| US-004 Extract for graph ingest | AC-1..3 | IT-004 (envelope), IT-015 (edge dedup), IT-020 (determinism), IT-099 (unknown archetype) — IT-016 (sugar field harvest) retired, FR-003 CR 2026-08-20 | ✅ |
| US-005 Machine addresses section | AC-1..5 | IT-033, IT-034, IT-035, IT-036, IT-038 | ✅ |
| US-006 Evidence producer exports assurance facts | AC-1..4 | IT-136, IT-138, IT-139, IT-143, TC-814 | 🚧 |

## Functional Requirement Coverage

| Functional Req | Acceptance Criteria | Test Cases | Coverage Status |
|----|----|----|--------|
| FR-001 render subcommand | ⛔ RETIRED (§2bis) | IT-001, IT-009, IT-010, IT-017, IT-018 (all retired) | ⛔ |
| FR-002 parse subcommand | AC-1..5 | IT-002, IT-011, IT-012, IT-013, IT-019 (byte-offset round-trip) | ✅ |
| FR-003 extract subcommand | AC-1..5 | IT-004, IT-015, IT-016, IT-020 (determinism rerun), IT-069 (untyped doc → shared `[frontmatter]` diagnostic) | ✅ |
| FR-004 validate subcommand (markdown-only; `--json` removed; composed type+object + `--strict`; scoped discovery + lazy-init, ADR-0001; `--summary` + `--severity`; advisory grammar + `--strict` promotion) | AC-1..22 | IT-047 (md valid), IT-048 (md broken), IT-049 (--archetype), IT-014 (md sweep), IT-056 (no frontmatter), IT-057 (no string `type`), IT-050 (unknown archetype), IT-058 (path-safety arg label), IT-059 (stdin `-` exempt + validated), IT-021 (no stdout), IT-073 (unknown `object:` warns, exit 0), IT-074 (`--strict` escalates warning → exit 1), IT-075 (json warning distinct `kind`/severity), IT-081 (scoped env/default-root discovery validates, network-free → AC-13), IT-082 (empty discovery + no quoin → actionable error → AC-14), TC-714 (`--summary` histogram spans every grammar → AC-15), TC-720 (`--severity …=off` suppresses check + histogram row → AC-16), TC-721 (`--severity …=error` promotes and fails the run → AC-17), TC-755 (malformed `--severity` rejected before any read → AC-18), IT-101 (relative path under `--scope`, no `--module` → AC-19), IT-102 (relative glob under `--scope` surfaces the invalid match → AC-20), IT-105 (grammar findings advisory, exit 0 → AC-21), IT-106 (`--strict` promotes them to exit 1 → AC-22), TC-090 (thin boundary) | ✅ |
| FR-010 required-section validation (recast onto FR-032) | AC-1..5 | IT-051 (placeholder), IT-052 (missing), IT-053 (assert), IT-047 (valid exit 0), IT-054 (empty stdout + diagnostics) | ✅ |
| FR-005 path-safety | AC-1..5 | IT-005, IT-006, IT-007, IT-022 (--out reject), IT-023 (stdin bypasses) | ✅ |
| FR-006 IO contract | AC-1..4 | IT-024 (no interleaving), IT-025 (--diagnostics-format=json), IT-011 (stdin) | ✅ |
| FR-007 Exit codes | AC-1..6 | IT-026 (each exit code: 0, 1, 2), IT-027 (no panic on covered inputs) | ✅ |
| FR-008 JSON encoding | AC-1..6 | IT-028 (compact default), IT-029 (--pretty), IT-019 (round-trip), IT-030 (stable field order), IT-100 (envelope shape + no bare version key), IT-123 (`--version` names both), IT-124/IT-125/IT-126 (provenance on extract/properties/coverage), IT-127 (the bare-version ban that survives CR-104), IT-128 (the coverage payload conforms to the published contract), IT-129 (the envelope shape pinned by golden snapshot) | ✅ |
| FR-009 schema subcommand (asserts-based contract) | AC-1..5 | IT-060 (FR schema + asserts), IT-061 (per-section asserts, no template vars), IT-062 (unknown archetype), IT-063 (deterministic stdout), IT-058 (path-safety) | ✅ |
| FR-011 lookup subcommand | AC-1..6 | IT-033, IT-034, IT-035, IT-036, IT-037, IT-038, IT-039 | ✅ |
| FR-012 edit subcommand | AC-1..6 | IT-040, IT-041, IT-042, IT-043, IT-044, IT-045, IT-046 | ✅ |
| FR-013 lint subcommand | AC-1..5 | IT-064 (clean exit 0 silent), IT-065 (warning exit 0 + stderr), IT-066 (error exit 1), IT-067 (--archetype scoping), IT-068 (missing manifest fails fast — also covers the FR-004 CR-note eager-loader behavior for validate/extract/schema) | ✅ |
| FR-014 validate --okf bundle posture (`type` discriminator) | AC-1..9 | IT-069 (untyped → exit 1, `[frontmatter]`), IT-070 (unknown type + broken link → warn, exit 0), IT-071 (index incompleteness → warn, exit 0), IT-072 (defaults to --scope dir), IT-026 (bare `validate` no `--okf` → exit 2, `required_unless_present`), TC-090 (thin boundary) | ✅ |
| FR-015 fix subcommand (unlinked-reference autofix, ADR 0007) | AC-1..6 | IT-076 (dry-run `would-fix` → exit 1, no write), IT-077 (`--write` applies + idempotent re-run exit 0), IT-078 (warn-only never written, no nonzero exit), IT-079 (clean bundle exit 0 empty stdout), IT-080 (`--scope` root + path-safety reject), TC-090 (thin boundary) | ✅ |
| FR-016 update subcommand (install-source-aware self-update) | AC-1..7 | IT-083 (Unknown source: `update --check` prints npm+cargo+releases recipes, exit 0, no install/network), IT-084 (Unknown source: bare `update` also no-install, exit 0), TC-085 (`detect_source` npm/cargo/unknown classification — `self_update::tests`), TC-086 (`registry_args` scope-form for scoped pkg / plain for unscoped / empty when no override), TC-087 (`cargo` `--check` reports branch-tracking, `latest: None`), TC-090 (thin boundary — `update` carries no parser/validator), TC-093 (`self_update` engine imports nothing from quire's `io`/command ctx — package-agnostic) — the untraced criteria are recorded in [FR-016's CR note](functional/FR-016-update-subcommand.md) (#55) | 🚧 |
| FR-017 coverage subcommand (declarative rollup over two roots; agent-sized projection — #53; #51 close-out over quire-rs v0.42.0; CR-012 census to stdout) | AC-1..19 | IT-117 (CR-012 census on stdout, findings on stderr),  IT-107, IT-108 (CR-011 the engine capabilities reach a command line), IT-109 (finding lines lead with `row_id` — #51), IT-110 (`coverage` severity pack — off/error/reject-before-read, incl. typo'd checks — #57), IT-111 (`--format tsv` records on stdout, `line` column populated), TC-812 (TSV escaping guard — #57), TC-813 (#66 census, metric envelopes, diagnostics and suspicions reach the human surface), IT-112 (`--json` honours `--pretty`, compact default), IT-113 (`document:line` locus on finding lines → AC-16), IT-114 (`no_symbol_rows` renders in the census → AC-17), IT-115 (excluded-count census line + extraction diagnostics on stderr → AC-18), IT-116 (`shared_trace_ids` through `--json` → AC-19), IT-089 (human census on stdout — CR-012), TC-740 (JSON payload byte-identical + no-model refusal), TC-810 (document root is `<scope>/spec`), IT-087 (code walk excludes it), TC-811 + IT-086 (missing root named + typed kind), IT-092 (`--strict`), TC-797 (zero matched rows ≠ full coverage), TC-090 (thin boundary) | ✅ Complete |
| FR-018 properties subcommand (per-criterion property classification) | AC-1..9 | IT-093 (census), IT-094 (JSON records + determinism), IT-095 (no criteria → empty, exit 0), IT-096 (non-extractable never fails), IT-097 (path-safety), IT-098 (`exclude:` binds this surface too), IT-103 (envelope conforms to the published schema → AC-8), IT-104 (absent obligation source → `obligation: null`, still conforms → AC-9), TC-090 (thin boundary) | ✅ Complete |
| FR-019 symbols subcommand (the extracted symbol table) | AC-1..6 | IT-130 (human census, stdout clean), IT-131 (JSON record fields + byte-stability), IT-132 (NOT ASKED vs not tagged), IT-133 (both denominators), IT-134 (diagnostics on stderr in JSON mode), IT-135 (a language filter narrows records AND census) | ✅ Complete |
| FR-020 assurance subcommand (source-grounded assurance-v1 export) | AC-1..9 | IT-136 (complete export), IT-137 (upstream schema), IT-138 (compact/pretty determinism), IT-139/IT-140 (premise/load refusal), IT-141 (empty success vs failure), IT-142 (diagnostic channels), TC-814 (thin boundary), IT-143 (no child/network execution), IT-144 (cross-language golden), IT-145 (docs/help/pin) | 🚧 |

## Non-Functional Requirement Coverage

| Non-Functional Req | Verification Method | Evidence/Test Cases | Status |
|-----|--------------|-------|--------|
| NFR-001 render p95 ≤ 50 ms | ⛔ RETIRED (§2bis) | TC-088 (render bench removed) | ⛔ |
| NFR-002 Static binary | static audit | TC-089 (`ldd` IT verifies no project .so) | ✅ |
| NFR-003 Zero unsafe | static audit | TC-092 (`scripts/check_unsafe_comments.sh` CI gate) | ✅ |
| NFR-004 No network (own process; scoped lazy-init via quoin is the ADR-0001 exception) | static + runtime | TC-091 (`cargo deny bans`), IT-008 (strace zero socket(), happy path), IT-081 (scoped discovery network-free), IT-143 (`assurance` no socket/child process) | 🚧 |
| NFR-005 Diagnostic format | unit + IT | IT-031 (each error class parses as Diagnostic JSON) | ✅ |
| NFR-006 CLI stability | snapshot | IT-032 (`quire --help` snapshot pinned) | ✅ |

---

## Test Case Summary

| Test ID | Title | Type | Priority | Traces To | Status |
|---------|-------|------|----------|-----------|--------|
| IT-001 | ⊘ RETIRED (§2bis) — `quire render FR` happy path produces rendered markdown ((retired); (retired)) | Integration | P0 | FR-001-AC-1, US-001-AC-1 | ⛔ |
| IT-002 | `quire parse` emits valid QuireDocument JSON | Integration | P0 | FR-002-AC-1, US-002-AC-1 | ✅ |
| IT-003 | ⊘ RETIRED (§2bis) — `quire validate FR --module $ISO --json <obj>` (context mode removed) ((was context mode; removed)) | Integration | P0 | FR-004-AC-4 | ⛔ |
| IT-004 | `quire extract` emits {extraction, edges} envelope | Integration | P0 | FR-003-AC-1, US-004-AC-1 | ✅ |
| IT-005 | `--module ../escape` exits 1 with PathSafetyViolation | Integration | P0 | FR-005-AC-1, StR-003-AC-1, FR-007-AC-2 | ✅ |
| IT-006 | Symlink under module to /etc/passwd refused at load | Integration | P0 | FR-005-AC-4, StR-003-AC-4 | ✅ |
| IT-007 | ⊘ RETIRED (§2bis) — `--data ../../etc/passwd` exits 1 (replaced by IT-055 on the positional doc path) ((retired)) | Integration | P0 | FR-005-AC-3 | ⛔ |
| IT-008 | No network sockets opened (strace, happy path — registry present) | Integration | P0 | NFR-004-AC-2, StR-001-AC-4 | ✅ |
| IT-009 | ⊘ RETIRED (§2bis) — Render byte-parity vs minijinja-cli (FR archetype) ((retired); (retired)) | Integration | P0 | FR-001-AC-1, US-001-AC-2 | ⛔ |
| IT-010 | ⊘ RETIRED (§2bis) — Schema violation exits 1 before stdout write (render) ((retired); (retired)) | Integration | P0 | FR-001-AC-4, US-001-AC-3 | ⛔ |
| IT-011 | `parse -` reads stdin | Integration | P1 | FR-002-AC-2, US-002-AC-4 | ✅ |
| IT-012 | Malformed frontmatter still parses, stderr warns | Integration | P1 | FR-002-AC-3, US-002-AC-3 | ✅ |
| IT-013 | Empty document → valid empty QuireDocument JSON | Integration | P1 | FR-002-AC-4 | ✅ |
| IT-014 | Parametric **direct-markdown** validate sweep across 8 ISO archetypes (valid + invalid each; no render-then-validate) | Integration | P0 | FR-004-AC-1, FR-004-AC-2, US-003-AC-2 | ✅ |
| IT-100 | The `extract` payload deserializes into the declared `{extraction, edges}` envelope — `extraction` an object, every edge carrying string `target`/`type` — carries no bare `version`/`schema_version` key, and carries the `engine` provenance block (`cli_extract::it_100_*`) | Integration | P1 | FR-008-AC-2, FR-008-AC-5, FR-008-AC-6 | ✅ |
| IT-123 | `quire --version` reports the CLI version and the resolved engine version as distinct, labelled values on one line — a binary linking a stale engine is no longer indistinguishable from one linking a current engine (CR-104, `cli_provenance::it_123_*`) | Integration | P0 | FR-008-AC-6 | ✅ |
| IT-124 | The `extract` payload carries `engine.{cli,engine,capabilities}` with the engine version resolved and distinct from the CLI version, while `extraction` and `edges` are emitted unmodified (`cli_provenance::it_124_*`) | Integration | P0 | FR-008-AC-6 | ✅ |
| IT-125 | The `properties --json` payload carries the provenance block over a non-empty classification — the payload quoin's `spec-correctness` skill derives property tests from (`cli_provenance::it_125_*`) | Integration | P0 | FR-008-AC-6 | ✅ |
| IT-126 | The `coverage --json` payload carries the provenance block over a model that matched a row, so the assertion is not made over the 0/0 state (`cli_provenance::it_126_*`) | Integration | P0 | FR-008-AC-6 | ✅ |
| IT-127 | No payload — `extract`, `properties` or `coverage` — carries a bare `version`, `schema_version` or `$schema` key at any level, checked by walking parsed keys rather than grepping the rendered bytes; the half of AC-5 that survives CR-104, since a loose version lets a payload assert its own contract revision (`cli_provenance::it_127_*`) | Integration | P0 | FR-008-AC-5 | ✅ |
| IT-128 | The `coverage --json` payload conforms to the published `coverage-v1.schema.json` read from the pinned engine checkout — the surface this crate mutates by appending `engine`, and which had no conformance test on either side of the seam (`output_contract::it_128_*`) | Integration | P0 | FR-008-AC-6 | ✅ |
| IT-129 | The `extract` envelope shape is pinned by a golden snapshot with the two version VALUES redacted, so a key that moves, appears or vanishes fails while an engine bump does not train the reader to regenerate without looking (`cli_provenance::it_129_*`, extends IT-032/#60) | Integration | P0 | FR-008-AC-6 | ✅ |
| IT-130 | `quire symbols` exits 0 with NO module: `coverage` bails without a `traceability:` model because it has nothing to reconcile, but "what did the scanner find" is a question about the walk, and demanding a declaration would make the one surface that can size a scanner defect depend on the declaration being right (`cli_symbols::it_130_*`, #309) | Integration | P0 | FR-019-AC-1 | ✅ |
| IT-131 | Every record carries `path`, `symbol`, `kind`, `language`, `line`, `leading_line`, `end_line`, `container`, `id`, `binds_trace_ids`, `carries_implements`, and two runs are byte-identical. `leading_line` sits beside `line` because a marker that failed to match is written at the annotation block, which is the line to edit (`cli_symbols::it_131_*`, #256, #309) | Integration | P0 | FR-019-AC-2 | ✅ |
| IT-132 | Without a module the surface says binding was NOT ASKED; with one, the bound id is reported. An unbound run and a repository nobody tagged otherwise produce the same empty `trace_ids`, and a reader who cannot tell them apart draws the conclusion the 4% figure came from (`cli_symbols::it_132_*`, #309) | Integration | P0 | FR-019-AC-3 | ✅ |
| IT-133 | `by_language` carries `symbols` AND `binding_kinds`, and they differ on a tree with a container — a binding rate over the first reads a tree of containers as untagged, which is the shape EPIC quire-rs#264 opened on (`cli_symbols::it_133_*`) | Integration | P0 | FR-019-AC-4 | ✅ |
| IT-134 | Extraction diagnostics reach stderr in JSON mode and the payload always carries the channel: a file the extractor could not read is indistinguishable from a file with no declarations, and a consumer testing for the key's presence could not tell "none" from "this build does not report them" (`cli_symbols::it_134_*`) | Integration | P0 | FR-019-AC-5 | ✅ |
| IT-135 | `--language` narrows the records AND the census together: a filter that narrowed one would publish a census over a population the payload does not contain (`cli_symbols::it_135_*`) | Integration | P0 | FR-019-AC-6 | ✅ |
| IT-136 | `quire assurance` over the pinned complete fixture emits the upstream `quire-assurance` v1 envelope with artifacts, obligations, symbols, relation kinds, resolved/dangling corpus edges, distinct verifies/implements relations, locators, and every observation availability state (`cli_assurance::it_136_*`) | Integration | P0 | FR-020-AC-1, US-006-AC-1 | 🚧 |
| IT-137 | The emitted golden parses as `AssuranceExport` and validates directly against `quire_rs::assurance::ASSURANCE_V1_SCHEMA`; no vendored CLI schema participates (`cli_assurance::it_137_*`) | Integration | P0 | FR-020-AC-1 | 🚧 |
| IT-138 | Compact and `--pretty` invocations are each byte-identical on repeat; their parsed JSON values are equal, and compact stdout before its newline equals `AssuranceExport::to_json_bytes()` (`cli_assurance::it_138_*`) | Integration | P0 | FR-020-AC-2, US-006-AC-3 | 🚧 |
| IT-139 | Wrong expected module name/version and missing, extra, or wrong schema premises each exit 1, name the rejected premise on stderr, and emit zero stdout bytes (`cli_assurance::it_139_*`) | Integration | P0 | FR-020-AC-3, US-006-AC-2 | 🚧 |
| IT-140 | Malformed premise syntax, invalid revision, unnamed/unversioned module, and archetype load failure are refused before stdout (`cli_assurance::it_140_*`) | Integration | P0 | FR-020-AC-3 | 🚧 |
| IT-141 | A valid empty bounded corpus returns a complete successful envelope and exit 0; one unreadable document remains explicit as an `unknown` observation with a reason; a missing document root, invalid module/source premise, or export-wide upstream error returns exit 1 and empty stdout, so empty, unknown, and unavailable cannot alias (`cli_assurance::it_141_*`) | Integration | P0 | FR-020-AC-4 | 🚧 |
| IT-142 | Registry and extraction diagnostics use stderr in human and JSON diagnostic modes and never appear in the assurance JSON (`cli_assurance::it_142_*`) | Integration | P1 | FR-020-AC-5 | 🚧 |
| TC-814 | Static source audit permits only calls to upstream `Spec`, `extract_tree_scoped`, `trace::bind`, `build_assurance_export`, and `read_assurance_export`, and rejects a CLI-owned assurance schema, graph, or parser (`scripts/check_thin_boundary.sh`) | Static | P0 | FR-020-AC-6, US-006-AC-4, StR-004 | 🚧 |
| IT-143 | Linux `strace -fe network,process` observes neither network syscalls nor spawned commands for successful and premise-refused assurance invocations (`audit_no_network.rs`) | Integration | P0 | FR-020-AC-7, FR-020-CON-2, US-006-AC-4, NFR-004-AC-2 | 🚧 |
| IT-144 | Rust `jsonschema` validates the exact checked-in assurance golden against the upstream schema; Node and Python compatibility probes consume those same bytes, preserve the closed envelope/state tokens, and perform no normalization. An unavailable local runtime is a named skip rather than a fabricated pass (`tests/assurance_cross_language.rs`) | Integration | P0 | FR-020-AC-8 | 🚧 |
| IT-145 | The help snapshot, README command synopsis, changelog, Cargo exact revision, lockfile source, and `assurance_export.v1` capability token agree on the quire-rs 0.46.0/e3352a compatibility boundary (`cli_assurance_contract.rs`) | Integration | P0 | FR-020-AC-9 | 🚧 |
| IT-099 | `extract` on a document whose `type` resolves to no DSL-carrying archetype exits 1 with a diagnostic naming it; no partial extraction reaches stdout (`cli_extract::extract_no_dsl_archetype_errors_cleanly`) | Integration | P1 | FR-003-AC-2, US-004-AC-3 | ✅ |
| IT-015 | Edge dedup by (source, type, target) — a twice-declared relationship and a twice-linked body target each harvest once (`cli_extract::it_015_*`) | Integration | P1 | US-004-AC-2 | ✅ |
| IT-016 | ⊘ RETIRED (FR-003 CR, 2026-08-20) — Frontmatter sugar field `dependencies:` harvested (no engine ever harvested sugar fields) | Integration | P1 | FR-003-AC-3 | ⛔ |
| IT-017 | ⊘ RETIRED (§2bis) — render `--out` flag writes file, empty stdout (the `--out` write-target path-safety survives on `edit`, IT-041) ((retired)) | Integration | P1 | FR-001-AC-5 | ⛔ |
| IT-018 | ⊘ RETIRED (§2bis) — 8-archetype render parity sweep ((retired); (retired)) | Integration | P0 | FR-001-AC-6, StR-002 | ⛔ |
| IT-019 | parse JSON round-trips through QuireDocument deserialize | Integration | P0 | FR-002-AC-5, FR-008-AC-1 | ✅ |
| IT-020 | extract rerun produces byte-identical stdout | Integration | P1 | FR-003-AC-4 | ✅ |
| IT-021 | validate writes nothing to stdout on success | Integration | P1 | FR-004-AC-1, FR-006 | ✅ |
| IT-022 | `edit --out ../escape` rejected (write-target path-safety survives) (note) | Integration | P0 | FR-005-AC-3, FR-012 | ✅ |
| IT-023 | positional `-` (stdin) bypasses path-safety | Integration | P1 | FR-005-AC-5 | ✅ |
| IT-024 | No stdout/stderr interleaving (chunked write test) | Integration | P1 | FR-006-AC-1, FR-006-AC-2 | ✅ |
| IT-025 | `--diagnostics-format=json` produces parseable Diagnostic | Integration | P1 | FR-006-AC-3, NFR-005-AC-2 | ✅ |
| IT-026 | Each documented exit code is produced by at least one input (incl. bare `validate` with no positional and no `--okf` → exit 2, `cli_errors::it_026_exit_code_2_on_argv_error`) | Integration | P0 | FR-007-AC-1, FR-007-AC-2, FR-007-AC-3, FR-007-AC-4, FR-007-AC-5, FR-014-AC-7 | ✅ |
| IT-027 | No panic on randomly malformed inputs (smoke fuzz) | Integration | P1 | FR-007-AC-6 | ✅ |
| IT-028 | Default JSON output is compact (one line) | Integration | P1 | FR-008-AC-1 | ✅ |
| IT-029 | `--pretty` produces multi-line indented JSON | Integration | P2 | FR-008-AC-3 | ✅ |
| IT-030 | JSON field order matches Rust struct order | Integration | P2 | FR-008-AC-4 | ✅ |
| IT-031 | Each error class's stderr deserializes as Diagnostic when JSON format active, with empty stdout and non-empty stderr for every class (`cli_io::it_031_*`) | Integration | P1 | NFR-005-AC-1, NFR-005-AC-2, FR-006-AC-1 | ✅ |
| IT-032 | `quire --help` snapshot pinned | Integration | P2 | NFR-006-AC-2 | ✅ |
| IT-033 | `lookup --heading --level 1` returns the H1 section JSON | Integration | P0 | FR-011-AC-1, US-005-AC-2 | ✅ |
| IT-034 | `lookup --heading Behavior` uses upstream-style heading normalization | Integration | P0 | US-005-AC-1 | ✅ |
| IT-035 | `lookup --block-id blk-behavior` returns stable block section JSON | Integration | P0 | FR-011-AC-2, US-005-AC-3 | ✅ |
| IT-036 | `lookup --id detail-L6` returns parser-derived id section JSON | Integration | P1 | FR-011-AC-3, US-005-AC-4 | ✅ |
| IT-037 | `lookup --content` emits raw section content only | Integration | P1 | FR-011-AC-4 | ✅ |
| IT-038 | Missing lookup selector exits 1 with empty stdout | Integration | P1 | FR-011-AC-5, US-005-AC-5 | ✅ |
| IT-039 | Multiple lookup selectors are rejected by clap as argv error | Integration | P1 | FR-011-AC-5, FR-011-AC-6 | ✅ |
| IT-040 | `edit --heading` replaces section body, rest byte-identical | Integration | P0 | FR-012-AC-1 | ✅ |
| IT-041 | `edit --out <input>` edits the document in place | Integration | P1 | FR-012-AC-3 | ✅ |
| IT-042 | `edit --block-id` replaces the full stable block | Integration | P0 | FR-012-AC-2 | ✅ |
| IT-043 | `edit` reads replacement content from a file | Integration | P1 | FR-012-AC-1 | ✅ |
| IT-044 | `edit` missing section exits 1 without writing the input | Integration | P1 | FR-012-AC-4 | ✅ |
| IT-045 | `edit` with both/neither selector is rejected | Integration | P1 | FR-012-AC-5 | ✅ |
| IT-046 | `edit` with `-` for both doc and content is a user error | Integration | P1 | FR-012-AC-6 | ✅ |
| IT-047 | `quire validate valid-fr.md --module $ISO` exits 0 with no output (markdown default, structure present) | Integration | P0 | FR-004-AC-1, FR-010-AC-4, US-003-AC-1 | ✅ |
| IT-048 | `quire validate broken-fr.md --module $ISO` exits 1; stderr carries a line-numbered diagnostic naming the failing section/assert | Integration | P0 | FR-004-AC-2 | ✅ |
| IT-049 | `quire validate fr.md --module $ISO --archetype FR` overrides frontmatter-derived archetype resolution | Integration | P1 | FR-004-AC-3 | ✅ |
| IT-050 | `quire validate doc.md --module $ISO --archetype NONEXISTENT` exits 1 with `UnknownArchetype` on stderr (re-pointed off the removed `--json` mode) | Integration | P1 | FR-004-AC-6, FR-007-AC-3 | ✅ |
| IT-051 | `quire validate rendered-fr.md --module $ISO` exits 1 when `## Specification` is only `TODO`, reason `placeholder` | Integration | P0 | FR-010-AC-1 | ✅ |
| IT-052 | Validate exits 1 when an FR required section is missing (reason `missing`), naming the section (line absent for a fully-missing section — FR-010 CR-003) | Integration | P0 | FR-010-AC-2 | ✅ |
| IT-053 | Validate exits 1 when the Acceptance Criteria table has wrong columns or zero data rows (reason `assert`) | Integration | P0 | FR-010-AC-3 | ✅ |
| IT-054 | Structural validation failure produces empty stdout + non-empty stderr carrying quire-rs diagnostics unchanged | Integration | P0 | FR-010-AC-5 | ✅ |
| IT-055 | `quire validate ../../etc/passwd --module $ISO` exits 1 with PathSafetyViolation naming the positional document arg | Integration | P0 | FR-005-AC-2, StR-003-AC-2, FR-004-AC-7 | ✅ |
| IT-056 | `quire validate no-frontmatter.md --module $ISO` (no frontmatter, no `--archetype`) exits 1; stderr names missing frontmatter / `--archetype` remedy; empty stdout | Integration | P0 | FR-004-AC-4 | ✅ |
| IT-057 | `quire validate no-type.md --module $ISO` (frontmatter present, `type` absent or non-string; no `--archetype`) exits 1; stderr names `--archetype`/`type` | Integration | P0 | FR-004-AC-5 | ✅ |
| IT-058 | path-safety violation diagnostic names the arg label (`document` / `--module`) | Integration | P1 | FR-004-AC-7, FR-009-AC-5 | ✅ |
| IT-059 | `quire validate - --module $ISO` reads stdin (path-safety-exempt) and still validates structurally | Integration | P1 | FR-004-AC-8 | ✅ |
| IT-060 | `quire schema FR --module $ISO` exits 0; JSON contains FR frontmatter schema + `body_extraction` asserts | Integration | P0 | FR-009-AC-1 | ✅ |
| IT-061 | `schema` JSON describes per-section asserts (headings/columns/id-patterns), no template-variable list | Integration | P0 | FR-009-AC-2 | ✅ |
| IT-062 | `quire schema NONEXISTENT --module $ISO` exits 1 with `UnknownArchetype`, empty stdout | Integration | P1 | FR-009-AC-3 | ✅ |
| IT-063 | Repeated `quire schema FR` calls produce byte-identical stdout | Integration | P2 | FR-009-AC-4 | ✅ |
| IT-064 | `quire lint clean.md --module $M` exits 0, silent on both streams | Integration | P0 | FR-013-AC-1 | ✅ |
| IT-065 | Warning-severity finding: exit 0, stderr `warning: <rule-id>:` + offending value, empty stdout | Integration | P0 | FR-013-AC-2 | ✅ |
| IT-066 | Error-severity finding: exit 1, stderr `error: <rule-id>:` | Integration | P0 | FR-013-AC-3 | ✅ |
| IT-067 | `--archetype NFR` suppresses a rule scoped `archetypes: [FR]` | Integration | P1 | FR-013-AC-4 | ✅ |
| IT-068 | `--module` without manifest.yaml exits 1 naming the missing manifest (eager loader; covers validate/extract/schema too) ((CR eager-load)) | Integration | P0 | FR-013-AC-5, FR-004 | ✅ |
| IT-069 | `validate --okf <DIR>` over a bundle with an **untyped** document exits 1; stderr contains `type` + `[frontmatter]` (`cli_okf::okf_untyped_document_is_error`; also covers extract's shared untyped vocabulary) | Integration | P0 | FR-014-AC-1, FR-014-AC-8, FR-003-AC-5 | ✅ |
| IT-070 | `validate --okf <DIR>` tolerates an **unknown `type`** and a **broken `ix://` link** as warnings: exit 0, stderr `[unknown-type]` + `[dangling-reference]` (`cli_okf::okf_tolerates_unknown_type_and_broken_link`) | Integration | P0 | FR-014-AC-2, FR-014-AC-3 | ✅ |
| IT-071 | `validate --okf <DIR>` warns on an `index.md` omitting a sibling artifact: exit 0, stderr `[index-incomplete]` naming the missing artifact (root `okf_version` completeness in the same posture) (`cli_okf::okf_index_incompleteness_warns`) | Integration | P0 | FR-014-AC-4, FR-014-AC-5 | ✅ |
| IT-072 | `validate --okf --scope <DIR>` with no positional validates `<DIR>/spec` as the bundle root (exit 0, warning-only bundle) (`cli_okf::okf_defaults_to_scope_spec_directory`) | Integration | P1 | FR-014-AC-6 | ✅ |
| IT-085 | `validate --okf --scope <DIR>` where `<DIR>` holds no `spec/` exits non-zero with a diagnostic naming the missing document root — never a silent fallback to walking `<DIR>` (`cli_okf::okf_missing_spec_root_is_a_named_error`) | Integration | P0 | FR-014-AC-6 | ✅ |
| IT-086 | `coverage` missing-root refusal carries machine `kind` `MissingDocumentRoot` under `--diagnostics-format json`, and names the root by interpolated path (`cli_coverage::it086_*`) | Integration | P0 | FR-017-AC-5 | ✅ |
| IT-087 | The code walk excludes the document root: a trace-tagged source file under `<scope>/spec` contributes no symbol while the same shape outside it does (`cli_coverage::it087_*`) | Integration | P0 | FR-017-AC-4 | ✅ |
| IT-088 | A non-fatal bundle warning is emitted with `severity: "warning"` / `kind: "ValidationWarning"` under `--diagnostics json`, exit unchanged; the no-frontmatter and malformed-frontmatter flavors carry distinct machine reasons (`cli_okf::it088_*`) | Integration | P0 | FR-014-AC-6 | ✅ |
| IT-089 | `coverage` renders the human census on stderr and leaves stdout empty without `--json` (`cli_coverage::it089_*`) | Integration | P1 | FR-017-AC-1 | ✅ |
| IT-092 | `coverage --strict` exits 1 on an unbacked row while the same repository exits 0 without it (`cli_coverage::it092_*`) | Integration | P0 | FR-017-AC-7 | ✅ |
| IT-093 | `properties` renders a per-criterion census and exits 0 (`cli_properties::it093_*`) | Integration | P1 | FR-018-AC-1 | ✅ |
| IT-094 | `properties --json` emits one record per binding criterion carrying `row_id`/`shape`/`property`/`extraction`, byte-identical across runs (`cli_properties::it094_*`) | Integration | P0 | FR-018-AC-2 | ✅ |
| IT-095 | A document binding no criteria yields an empty record set and exits 0 (`cli_properties::it095_*`) | Integration | P1 | FR-018-AC-3 | ✅ |
| IT-096 | A non-extractable criterion is reported and never changes the exit code (`cli_properties::it096_*`) | Integration | P0 | FR-018-AC-4 | ✅ |
| IT-097 | `properties` rejects a `..` path by path-safety before any load (`cli_properties::it097_*`) | Integration | P0 | FR-018-AC-5 | ✅ |
| IT-098 | A document excluded by an obligation source's `exclude:` glob carries `obligation: null` in `properties --json` while an included one carries its obligation — the two obligation surfaces cannot disagree about whether a row states one (`cli_properties::it098_*`) | Integration | P0 | FR-018-AC-7 | ✅ |
| IT-073 | `quire validate <doc with `object:` unknown> --module $M` (no `--strict`) → exit 0, empty stdout; stderr carries a `warning:`-prefixed line naming the unknown object, distinct from any error | Integration | P0 | FR-004-AC-10 | ✅ |
| IT-074 | Same doc with `--strict` → exit 1; stderr still carries the `warning:` line; empty stdout. A clean doc (no warnings/errors) under `--strict` still exits 0 | Integration | P0 | FR-004-AC-11 | ✅ |
| IT-075 | Same doc with `--diagnostics-format json --strict` (or no `--strict`) → the warning is a distinct JSON object on stderr carrying a `severity`/`kind` field marking it a warning, separable from an error object | Integration | P1 | FR-004-AC-12 | ✅ |
| IT-081 | Scoped `validate` whose module is reachable ONLY via `IX_FILAMENT_MODULES_PATH` (HOME repointed so the default root is empty) validates the doc (exit 0) and, under `strace -fe network`, opens no inet socket | Integration | P0 | FR-004-AC-13, NFR-004-AC-2 | ✅ |
| IT-082 | Scoped `validate` with zero discoverable modules and no `quoin` on PATH (HOME repointed to an empty dir) exits 1 with a diagnostic naming `quoin plugin ensure-defaults`; empty stdout | Integration | P0 | FR-004-AC-14 | ✅ |
| IT-076 | `quire fix <DIR> --module $M` (dry-run) over a bundle with a bare in-bundle reference → exit 1, stderr `would-fix: <path>: <token> -> [<token>](<rel-path>)`, no file modified | Integration | P0 | FR-015-AC-1 | ✅ |
| IT-077 | `quire fix <DIR> --module $M --write` rewrites the reference to the suggested relative-path link; a second `--write` run changes nothing and exits 0 (idempotence) | Integration | P0 | FR-015-AC-2 | ✅ |
| IT-078 | A warn-only (unresolved/ambiguous) token is surfaced as `warning: … (<reason>)`, never written even under `--write`, and does not alone cause a nonzero exit | Integration | P0 | FR-015-AC-3 | ✅ |
| IT-079 | A clean bundle (no auto-fix findings) exits 0 with empty stdout in both dry-run and `--write` | Integration | P1 | FR-015-AC-4 | ✅ |
| IT-080 | `quire fix --scope <DIR> --module $M` with no positional uses `<DIR>/spec` as root (a repo-root file is never walked), and a `<DIR>` with no `spec/` is the same named error the other commands raise (`cli_fix::it080_*`); a `..`/symlink-escape on root or `--module` is rejected by path-safety before any load | Integration | P0 | FR-015-AC-5, FR-005 | ✅ |
| IT-083 | `update --check` on an Unknown install source (test binary under `target/`) prints manual instructions (npm recipe + cargo recipe + releases URL), exits 0, performs no install/network (`cli_update::update_check_on_unknown_source_prints_manual_instructions_and_exits_zero`) | Integration | P0 | FR-016-AC-1, FR-016-AC-2 | ✅ |
| IT-084 | bare `update` (no `--check`) on an Unknown source also performs no install and exits 0 (`cli_update::update_without_check_on_unknown_source_is_also_safe`) | Integration | P1 | FR-016-AC-2 | ✅ |
| TC-085 | `detect_source` classifies `node_modules` path → Npm, `.cargo` path → Cargo, bare path → Unknown (`self_update::tests::detect_*`) | Unit | P0 | FR-016-AC-1 | ✅ |
| TC-086 | `registry_args` yields the `--@scope:registry=` form for a scoped package, a plain `--registry <url>` for an unscoped package, and an empty vec when no override is supplied (`self_update::tests::registry_args_*`) | Unit | P0 | FR-016-AC-4 | ✅ |
| TC-087 | `run_for_source(Cargo, --check)` reports git-branch tracking with `latest: None` (no cross-scheme version); `run_for_source(Unknown)` emits manual report without installing (`self_update::tests`) | Unit | P1 | FR-016-AC-1, FR-016-AC-5 | ✅ |
| TC-088 | ⊘ RETIRED (§2bis) — hyperfine render p95 ≤ 50 ms on FR archetype (NFR-001-AC-1..2 (retired); (retired)) | Benchmark | P0 | StR-002 | ⛔ |
| TC-089 | `ldd` shows only libc + loader (no project .so) | Static | P0 | NFR-002-AC-1 | ✅ |
| TC-090 | `src/` grep finds no markdown parsing, no structural-validation logic, and **no render/template code** (validation delegated to quire-rs `validate_document` / `validate_bundle`; render removed per §2bis) | Static | P1 | StR-004-AC-2, FR-004-AC-9, FR-014-AC-9, FR-015-AC-6, FR-017-AC-9 | ✅ |
| TC-091 | `cargo deny check bans` rejects HTTP client crates; `deny.toml` still bans each one and `Cargo.lock` links none (`audit_static::tc091_*`) | Static | P0 | NFR-004-AC-1, NFR-004-AC-3 | ✅ |
| TC-092 | `scripts/check_unsafe_comments.sh` reports zero undocumented `unsafe` in src/, AND refuses a synthetic undocumented block — the gate is proven to catch, not merely to pass (`audit_static::tc092_*`) | Static | P0 | NFR-003-AC-1, NFR-003-AC-2 | ✅ |
| TC-093 | `src/self_update/` imports nothing from `quire`'s `io`/command context (engine is package-agnostic, config-struct driven); `commands/update.rs` is the only quire-specific glue and carries no parser/renderer/validator logic | Static | P1 | FR-016-AC-5, FR-016-AC-6, StR-004-AC-2 | ✅ |
| IT-101 | A **relative document path** with `--scope` and no `--module` resolves under the scope and validates against the module that scope carries (exact-module branch — the fixture root holds `manifest.yaml`; discovery is IT-081's): exit 0, empty stdout, empty stderr (`cli_validate::it_101_*`) | Integration | P1 | FR-004-AC-19 | ✅ |
| IT-102 | A **relative glob** with `--scope` expands under the scope; a matching non-conformant document exits 1 with a line-numbered stderr diagnostic naming the offending file, stdout empty (`cli_validate::it_102_*`) | Integration | P0 | FR-004-AC-20 | ✅ |
| IT-121 | `--module` does not move glob resolution off `--scope`: the same document validated with and without it yields identical output (`cli_validate::it_121_*`) | Integration | P0 | FR-004-AC-23, FR-018-AC-5 | ✅ |
| IT-122 | A scoped relative glob ignores the process directory: a decoy of the same relative name in the cwd is not read, so a run cannot silently grade the caller's own repository (`cli_validate::it_122_*`) | Integration | P0 | FR-004-AC-23 | ✅ |
| IT-103 | The `properties --json` envelope validates against quire-rs's published `schemas/output/properties-v1.schema.json`, read from the resolved dependency checkout rather than a vendored copy; the fixture exercises the obligation branch so conformance is not asserted over an empty payload (`output_contract::it_103_*`) | Integration | P0 | FR-018-AC-8 | ✅ |
| IT-104 | Against a module declaring no obligation source, every criterion record carries `obligation: null` and the payload still conforms to the published schema (`output_contract::it_104_*`) | Integration | P1 | FR-018-AC-9 | ✅ |
| IT-105 | A structurally valid FR carrying EARS violations exits 0 — grammar findings are advisory — and each surfaces on stderr under its `[ears:<check>]` label alongside the `--summary` histogram; stdout empty (`cli_validate::it_105_*`) | Integration | P0 | FR-004-AC-21, FR-004-AC-15 | ✅ |
| IT-106 | `--strict` escalates those same advisory grammar findings to exit 1 — the per-repo promotion lever, over the identical document IT-105 runs clean (`cli_validate::it_106_*`) | Integration | P0 | FR-004-AC-22 | ✅ |
| IT-107 | An undeclared status value reaches both the `--json` payload and the human census, verbatim; `--strict`'s exit code is unchanged by it | Integration | P0 | FR-017-AC-10 | ✅ |
| IT-108 | A declared `source_exclude` glob removes the matching file's symbols from the walk while real source survives; undeclared behaves exactly as before | Integration | P0 | FR-017-AC-11 | ✅ |
| IT-117 | `coverage`'s census reaches stdout uncolorized and its findings do not; the unbacked row is still reported, on stderr (`cli_stream_contract::it_117_*`) (CR-012) | Integration | P0 | FR-006-AC-5, FR-017-AC-1 | ✅ |
| IT-118 | `properties`' census is a result on stdout and leads with the specific-shape split alongside the extractable figure (`cli_stream_contract::it_118_*`) (CR-012) | Integration | P0 | FR-006-AC-5 | ✅ |
| IT-119 | `--criteria` renders per-criterion blocks carrying the row id and the extraction spans the census omits; the catch-all is outside the default set and `--all` includes it; the census is unchanged by the flag (`cli_stream_contract::it_119_*`) (CR-012) | Integration | P0 | FR-018-AC-10 | ✅ |
| IT-120 | The diagnostics channel reports a binder that read nothing: a tree of real tests carrying an undeclared marker spelling yields a `binding_census` with candidates and zero bound, and a `no-symbol-bound` diagnostic — the class that was silent while 1,292 symbols went unmentioned in pass 2 (`cli_stream_contract::it_120_*`) | Integration | P0 | FR-006-AC-5 | ✅ |
| IT-109 | Human unbacked-row / status-lie / undeclared-status lines lead with the row's `row_id` (reference kind kept visible in a bracketed trailer); two rows in the same document render distinguishable lines (`cli_coverage::it109_*`) | Integration | P0 | FR-017-AC-12 | ✅ |
| IT-110 | `--severity coverage:<check>=off` drops the kind from human AND `--json` output with the suppression announced and totals still full-computation; `--strict` is unaffected by projection; `=error` exits 1 without `--strict`; a malformed entry — or a typo'd coverage check (#57) — is rejected before any read (`cli_coverage::it110_*`) | Integration | P0 | FR-017-AC-13 | ✅ |
| IT-111 | `--format tsv` emits one tab-separated record per line on stdout: nine-column header, every record fully columned, row id leading, the `line` column carrying the engine's 1-based document line (quire-rs v0.42.0), byte-identical across runs, severity projection applies (`cli_coverage::it111_*`) | Integration | P0 | FR-017-AC-14 | ✅ |
| IT-112 | `coverage --json` is compact by default and indented under the global `--pretty`, parsing to the identical report either way (`cli_coverage::it112_*`) | Integration | P1 | FR-017-AC-15 | ✅ |
| IT-113 | Human unbacked-row / status-lie / undeclared-status lines carry the clickable `document:line` locus when the record has a line, over a fixture whose preamble shifts the table so the numbers can only be document lines; the bare-document form is gone for line-carrying records (`cli_coverage::it113_*`) | Integration | P0 | FR-017-AC-16 | ✅ |
| IT-114 | A `no_symbol_rows` record renders in the human census — row id leading, `document:line` locus, exempting test-type value named, reference kind in the trailer — while a symbol-minting row does not; the TSV projection carries its method and line (`cli_coverage::it114_*`) | Integration | P0 | FR-017-AC-17 | ✅ |
| IT-115 | A declared `source_exclude` that removes one file renders `1 source file(s) excluded by source_exclude` in the census, an undeclared scope renders nothing, and an unreadable source file's `SymbolExtraction` diagnostic reaches stderr; stdout stays empty throughout (`cli_coverage::it115_*`) | Integration | P0 | FR-017-AC-18 | ✅ |
| IT-116 | One status-carrying row id bound by two distinct symbols surfaces as a `shared_trace_ids` record in `--json` with both binders listed; an empty `vocabulary_coverage` stays off the wire (`cli_coverage::it116_*`) | Integration | P1 | FR-017-AC-19 | ✅ |
| TC-812 | A TSV cell carrying tab/newline/CR still yields exactly one nine-column record — the AC-14 escaping guard pinned, which no corpus fixture can exercise (0/1,107 statements carry a structural character) (`commands::coverage::tests::tc812_*`, #57) | Unit | P1 | FR-017-AC-14 | ✅ |
| TC-813 | The binding census renders one line per language directly under the coverage headline — the number and the premise it rests on read together — carrying the forms consulted and an unbound example ONLY where something is unread; measured ratio metrics render their FR-063 envelope and counts do not, a count's value and its `matched` being the same fact (`commands::coverage::tests::tc813_*`, #66) | Unit | P0 | FR-017-AC-18 | ✅ |

---

## Verification Status

GREEN for the v0.1 surface — every IT / BENCH / AUDIT through IT-046 has landed
and passes `make test` + `make bench` on a Linux dev box (WSL2). `make ci` runs
the full gauntlet locally; CI lanes (rust / licenses / bench) mirror the same
gates. Observed `TC-088` p95 is 4.87 ms, well under the 50 ms NFR-001 budget.

GREEN — the markdown-validation slice (ADR 0004): FR-004 recast to a
markdown-default `validate` (structural validation delegated to quire-rs
`validate_document`, FR-032) and the recast FR-010 (AC-1..5). Traces IT-047..054
are implemented in `tests/cli_validate.rs` and pass `make ci`. The ISO +
extract-mod fixtures were migrated off the retired `required_sections` manifest
field (quire-rs FR-031 CR — unified archetype); a `validate-mod` fixture carries a
`body_extraction` DSL with section + table asserts to exercise FR-010.

**Render removal (2026-06-04) — SPEC ONLY, awaiting implementation.** Per
`spec.md` §2bis (mirroring quire-rs commit 500a3d3), the `render` subcommand, the
`validate --json` context mode, and the render benchmark are retired. FR-004 is now
**markdown-only** with new ACs (AC-4..8: archetype-resolution failures, path-safety
arg label, stdin exemption; AC-9 renumbers the thin-boundary AC). Retired rows are
marked ⊘ RETIRED above (ids retained, dropped from the coverage tally). New traces
IT-055..063 (FR-004 failure paths + FR-009 schema coverage) and re-pointed IT-003
/IT-014/IT-050 are **specified here but not yet implemented** — they land with the
render-removal code task, alongside fixtures for no-frontmatter / no-`type`
documents. FR-009 (`schema`) is no longer an uncovered matrix gap.

Coverage tally: render/parity/`--json` traces (IT-001, IT-003, IT-007, IT-009,
IT-010, IT-017, IT-018, TC-088) and the retired FR-001/US-001/NFR-001/StR-002
ACs are dropped from the required-coverage set; every still-active AC retains at
least one IT/AUDIT trace.

GREEN — OKF bundle posture + `type` rename (2026-06-16). FR-014 (`validate --okf`)
adds the permissive bundle posture; traces IT-069..072 are implemented in
`tests/cli_okf.rs` (4 ITs: untyped-error, unknown-type/broken-link warn,
index-incompleteness warn, defaults-to-scope) and FR-014-AC-7 reuses
`cli_errors::it_026_exit_code_2_on_argv_error` (the `required_unless_present =
"okf"` argv behavior). FR-003 gains AC-5 (extract emits the shared `[frontmatter]`
untyped-document vocabulary), traced by IT-069. The `artifact_type` → `type`
discriminator rename was backsynced across FR-003/004/007/013 and spec.md via CR
notes; every FR-014 AC (1..9) and FR-003-AC-5 carries an IT/AUDIT trace.
