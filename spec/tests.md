---
id: TM-001
title: "quire-cli Test Matrix"
type: TestMatrix
---

# Test Matrix

## Overview

This matrix maps every Acceptance Criterion in `quire-cli/spec/` to one or more Test Cases (IT-XXX integration tests, BENCH-XXX benchmark gates, AUDIT-XXX static-analysis gates).

The CLI is a thin process boundary over `quire-rs`; the upstream engine is independently covered by `quire-rs/spec/tests.md`. This matrix tests **only** the CLI's process-level behavior: argv parsing, path-safety, stdin/stdout/stderr contract, exit codes, JSON output encoding, and static-binary properties.

> **Render removal (2026-06-04):** The `render` subcommand, the `validate --json`
> context mode, and the render benchmark are **removed** (see `spec.md` §2bis,
> mirroring quire-rs commit 500a3d3). Render/parity traces (IT-001, IT-009, IT-010,
> IT-017, IT-018, BENCH-001) and the `--json` context traces (IT-003, IT-050 as
> written) are **retired** — rows marked ⊘ RETIRED below, ids retained, dropped from
> the coverage tally. IT-014 is re-pointed to a direct-markdown sweep (no
> render-then-validate). The retired FR-001/US-001/NFR-001/StR-002 ACs no longer
> require a live trace.

## Matrix Rules

1. **Coverage Rule** — every AC has at least one IT / BENCH / AUDIT trace.
2. **Path-safety boundary rule** — every user-supplied path argument is exercised with `..`, with a symlink escape, and with a valid in-tree path.
3. **Exit-code rule** — every exit code in FR-007 has at least one IT producing it.
4. **Subcommand permutation rule** — for each subcommand (`parse`, `extract`, `lookup`, `edit`, `validate`, `schema`), the success path, the unknown-archetype path, and the validation-failure path each have a dedicated IT where applicable. (`render` removed — §2bis.) `validate` additionally has the `--okf` permissive bundle posture: its hard-error (untyped), warn (unknown-type / broken-link / index-incomplete), and scope-default paths each have a dedicated IT (IT-069..072). `fix` (ADR 0007 unlinked-reference autofix) has its dry-run (would-fix → exit 1), `--write` apply + idempotent re-run, warn-only, clean-bundle, and `--scope`/path-safety paths each covered (IT-076..080).
5. **Determinism rule** — primary JSON outputs (`parse`, `extract`, `lookup`, `schema`) have deterministic field order through Rust struct serialization.
6. **No-network rule** — IT-008 verifies zero `socket()` calls under strace across all subcommands.

---

## Stakeholder Coverage

| StR | Trace to US/FR | Verifying IT/BENCH/AUDIT | Status |
|-----|---------------|--------------------------|--------|
| StR-001 Static binary hot path (revised — surviving subcommands) | US-002, US-003, US-004, US-005, FR-002..012 | IT-002, IT-004, IT-047, IT-033, AUDIT-001 (ldd), AUDIT-003 (no-network) | ✅ |
| StR-002 Sub-50 ms render budget | ⊘ RETIRED (§2bis) | — (render bench removed) | ⊘ |
| StR-003 Sandbox inheritance (revised — path-safety) | FR-005 | IT-005 (..), IT-006 (symlink escape), IT-055 (doc path safety) | ✅ |
| StR-004 Thin boundary | FR-002..004, FR-009, FR-011, FR-014, NFR-005 | AUDIT-002 (src grep for parse logic), IT-033..038 | ✅ |

## User Story Coverage

| US | AC | IT | Status |
|----|----|----|--------|
| US-001 Agent renders FR | ⊘ RETIRED (§2bis) | IT-001, IT-009, IT-010, BENCH-001 (all retired) | ⊘ |
| US-002 Human parses doc | AC-1..4 | IT-002, IT-011 (stdin), IT-012 (malformed frontmatter), IT-013 (empty doc) | ✅ |
| US-003 CI validates | AC-1..3 | IT-003, IT-014 (parametric across 8 ISO archetypes) | ✅ |
| US-004 Extract for graph ingest | AC-1..4 | IT-004, IT-015 (edge dedup), IT-016 (sugar field harvest) | ✅ |
| US-005 Machine addresses section | AC-1..5 | IT-033, IT-034, IT-035, IT-036, IT-038 | ✅ |

## Functional Requirement Coverage

| FR | AC | IT | Status |
|----|----|----|--------|
| FR-001 render subcommand | ⊘ RETIRED (§2bis) | IT-001, IT-009, IT-010, IT-017, IT-018 (all retired) | ⊘ |
| FR-002 parse subcommand | AC-1..5 | IT-002, IT-011, IT-012, IT-013, IT-019 (byte-offset round-trip) | ✅ |
| FR-003 extract subcommand | AC-1..5 | IT-004, IT-015, IT-016, IT-020 (determinism rerun), IT-069 (untyped doc → shared `[frontmatter]` diagnostic) | ✅ |
| FR-004 validate subcommand (markdown-only; `--json` removed; composed type+object + `--strict`) | AC-1..12 | IT-047 (md valid), IT-048 (md broken), IT-049 (--archetype), IT-014 (md sweep), IT-056 (no frontmatter), IT-057 (no string `type`), IT-050 (unknown archetype), IT-058 (path-safety arg label), IT-059 (stdin `-` exempt + validated), IT-021 (no stdout), IT-073 (unknown `object:` warns, exit 0), IT-074 (`--strict` escalates warning → exit 1), IT-075 (json warning distinct `kind`/severity), AUDIT-002 (thin boundary) | ✅ |
| FR-010 required-section validation (recast onto FR-032) | AC-1..5 | IT-051 (placeholder), IT-052 (missing), IT-053 (assert), IT-047 (valid exit 0), IT-054 (empty stdout + diagnostics) | ✅ |
| FR-005 path-safety | AC-1..5 | IT-005, IT-006, IT-007, IT-022 (--out reject), IT-023 (stdin bypasses) | ✅ |
| FR-006 IO contract | AC-1..4 | IT-024 (no interleaving), IT-025 (--diagnostics-format=json), IT-011 (stdin) | ✅ |
| FR-007 Exit codes | AC-1..6 | IT-026 (each exit code: 0, 1, 2), IT-027 (no panic on covered inputs) | ✅ |
| FR-008 JSON encoding | AC-1..5 | IT-028 (compact default), IT-029 (--pretty), IT-019 (round-trip), IT-030 (stable field order) | ✅ |
| FR-009 schema subcommand (asserts-based contract) | AC-1..5 | IT-060 (FR schema + asserts), IT-061 (per-section asserts, no template vars), IT-062 (unknown archetype), IT-063 (deterministic stdout), IT-058 (path-safety) | ✅ |
| FR-011 lookup subcommand | AC-1..6 | IT-033, IT-034, IT-035, IT-036, IT-037, IT-038, IT-039 | ✅ |
| FR-012 edit subcommand | AC-1..6 | IT-040, IT-041, IT-042, IT-043, IT-044, IT-045, IT-046 | ✅ |
| FR-013 lint subcommand | AC-1..5 | IT-064 (clean exit 0 silent), IT-065 (warning exit 0 + stderr), IT-066 (error exit 1), IT-067 (--archetype scoping), IT-068 (missing manifest fails fast — also covers the FR-004 CR-note eager-loader behavior for validate/extract/schema) | ✅ |
| FR-014 validate --okf bundle posture (`type` discriminator) | AC-1..9 | IT-069 (untyped → exit 1, `[frontmatter]`), IT-070 (unknown type + broken link → warn, exit 0), IT-071 (index incompleteness → warn, exit 0), IT-072 (defaults to --scope dir), IT-026 (bare `validate` no `--okf` → exit 2, `required_unless_present`), AUDIT-002 (thin boundary) | ✅ |
| FR-015 fix subcommand (unlinked-reference autofix, ADR 0007) | AC-1..6 | IT-076 (dry-run `would-fix` → exit 1, no write), IT-077 (`--write` applies + idempotent re-run exit 0), IT-078 (warn-only never written, no nonzero exit), IT-079 (clean bundle exit 0 empty stdout), IT-080 (`--scope` root + path-safety reject), AUDIT-002 (thin boundary) | 🚧 |

## Non-Functional Requirement Coverage

| NFR | Verification | Trace | Status |
|-----|--------------|-------|--------|
| NFR-001 render p95 ≤ 50 ms | ⊘ RETIRED (§2bis) | BENCH-001 (render bench removed) | ⊘ |
| NFR-002 Static binary | static audit | AUDIT-001 (`ldd` IT verifies no project .so) | ✅ |
| NFR-003 Zero unsafe | static audit | AUDIT-004 (`scripts/check_unsafe_comments.sh` CI gate) | ✅ |
| NFR-004 No network | static + runtime | AUDIT-003 (`cargo deny bans`), IT-008 (strace zero socket()) | ✅ |
| NFR-005 Diagnostic format | unit + IT | IT-031 (each error class parses as Diagnostic JSON) | ✅ |
| NFR-006 CLI stability | snapshot | IT-032 (`quire --help` snapshot pinned) | ✅ |

---

## Test Case Summary

| ID | Title | Type | Priority | Traces To |
|----|-------|------|----------|-----------|
| IT-001 | ⊘ RETIRED (§2bis) — `quire render FR` happy path produces rendered markdown | Integration | P0 | FR-001-AC-1 (retired), US-001-AC-1 (retired) |
| IT-002 | `quire parse` emits valid QuireDocument JSON | Integration | P0 | FR-002-AC-1, US-002-AC-1 |
| IT-003 | ⊘ RETIRED (§2bis) — `quire validate FR --module $ISO --json <obj>` (context mode removed) | Integration | P0 | FR-004-AC-4 (was context mode; removed) |
| IT-004 | `quire extract` emits {extraction, edges} envelope | Integration | P0 | FR-003-AC-1, US-004-AC-1 |
| IT-005 | `--module ../escape` exits 1 with PathSafetyViolation | Integration | P0 | FR-005-AC-1, StR-003-AC-1 |
| IT-006 | Symlink under module to /etc/passwd refused at load | Integration | P0 | FR-005-AC-4, StR-003-AC-4 |
| IT-007 | ⊘ RETIRED (§2bis) — `--data ../../etc/passwd` exits 1 (replaced by IT-055 on the positional doc path) | Integration | P0 | FR-005-AC-3 (retired) |
| IT-008 | No network sockets opened (strace) | Integration | P0 | NFR-004-AC-2, StR-001-AC-4 |
| IT-009 | ⊘ RETIRED (§2bis) — Render byte-parity vs minijinja-cli (FR archetype) | Integration | P0 | FR-001-AC-1 (retired), US-001-AC-2 (retired) |
| IT-010 | ⊘ RETIRED (§2bis) — Schema violation exits 1 before stdout write (render) | Integration | P0 | FR-001-AC-4 (retired), US-001-AC-3 (retired) |
| IT-011 | `parse -` reads stdin | Integration | P1 | FR-002-AC-2, US-002-AC-4 |
| IT-012 | Malformed frontmatter still parses, stderr warns | Integration | P1 | FR-002-AC-3, US-002-AC-3 |
| IT-013 | Empty document → valid empty QuireDocument JSON | Integration | P1 | FR-002-AC-4 |
| IT-014 | Parametric **direct-markdown** validate sweep across 8 ISO archetypes (valid + invalid each; no render-then-validate) | Integration | P0 | FR-004-AC-1..2, US-003-AC-2 |
| IT-015 | Edge dedup by (source, type, target) | Integration | P1 | FR-003-AC-2, US-004-AC-2 |
| IT-016 | Frontmatter sugar field `dependencies:` harvested | Integration | P1 | FR-003-AC-3, US-004-AC-3 |
| IT-017 | ⊘ RETIRED (§2bis) — render `--out` flag writes file, empty stdout (the `--out` write-target path-safety survives on `edit`, IT-041) | Integration | P1 | FR-001-AC-5 (retired) |
| IT-018 | ⊘ RETIRED (§2bis) — 8-archetype render parity sweep | Integration | P0 | FR-001-AC-6 (retired), StR-002 (retired) |
| IT-019 | parse JSON round-trips through QuireDocument deserialize | Integration | P0 | FR-002-AC-5, FR-008-AC-1 |
| IT-020 | extract rerun produces byte-identical stdout | Integration | P1 | FR-003-AC-4 |
| IT-021 | validate writes nothing to stdout on success | Integration | P1 | FR-004-AC-1, FR-006 |
| IT-022 | `edit --out ../escape` rejected (write-target path-safety survives) | Integration | P0 | FR-005-AC-3 note, FR-012 |
| IT-023 | positional `-` (stdin) bypasses path-safety | Integration | P1 | FR-005-AC-5 |
| IT-024 | No stdout/stderr interleaving (chunked write test) | Integration | P1 | FR-006-AC-1..2 |
| IT-025 | `--diagnostics-format=json` produces parseable Diagnostic | Integration | P1 | FR-006-AC-3, NFR-005-AC-2 |
| IT-026 | Each documented exit code is produced by at least one input (incl. bare `validate` with no positional and no `--okf` → exit 2, `cli_errors::it_026_exit_code_2_on_argv_error`) | Integration | P0 | FR-007-AC-1..5, FR-014-AC-7 |
| IT-027 | No panic on randomly malformed inputs (smoke fuzz) | Integration | P1 | FR-007-AC-6 |
| IT-028 | Default JSON output is compact (one line) | Integration | P1 | FR-008-AC-1 |
| IT-029 | `--pretty` produces multi-line indented JSON | Integration | P2 | FR-008-AC-3 |
| IT-030 | JSON field order matches Rust struct order | Integration | P2 | FR-008-AC-4 |
| IT-031 | Each error class's stderr deserializes as Diagnostic when JSON format active | Integration | P1 | NFR-005-AC-1..2 |
| IT-032 | `quire --help` snapshot pinned | Integration | P2 | NFR-006-AC-2 |
| IT-033 | `lookup --heading --level 1` returns the H1 section JSON | Integration | P0 | FR-011-AC-1, US-005-AC-2 |
| IT-034 | `lookup --heading Behavior` uses upstream-style heading normalization | Integration | P0 | US-005-AC-1 |
| IT-035 | `lookup --block-id blk-behavior` returns stable block section JSON | Integration | P0 | FR-011-AC-2, US-005-AC-3 |
| IT-036 | `lookup --id detail-L6` returns parser-derived id section JSON | Integration | P1 | FR-011-AC-3, US-005-AC-4 |
| IT-037 | `lookup --content` emits raw section content only | Integration | P1 | FR-011-AC-4 |
| IT-038 | Missing lookup selector exits 1 with empty stdout | Integration | P1 | FR-011-AC-5, US-005-AC-5 |
| IT-039 | Multiple lookup selectors are rejected by clap as argv error | Integration | P1 | FR-011-AC-5..6 |
| IT-040 | `edit --heading` replaces section body, rest byte-identical | Integration | P0 | FR-012-AC-1 |
| IT-041 | `edit --out <input>` edits the document in place | Integration | P1 | FR-012-AC-3 |
| IT-042 | `edit --block-id` replaces the full stable block | Integration | P0 | FR-012-AC-2 |
| IT-043 | `edit` reads replacement content from a file | Integration | P1 | FR-012-AC-1 |
| IT-044 | `edit` missing section exits 1 without writing the input | Integration | P1 | FR-012-AC-4 |
| IT-045 | `edit` with both/neither selector is rejected | Integration | P1 | FR-012-AC-5 |
| IT-046 | `edit` with `-` for both doc and content is a user error | Integration | P1 | FR-012-AC-6 |
| IT-047 | `quire validate valid-fr.md --module $ISO` exits 0 with no output (markdown default, structure present) | Integration | P0 | FR-004-AC-1, FR-010-AC-4, US-003-AC-1 | ✅ |
| IT-048 | `quire validate broken-fr.md --module $ISO` exits 1; stderr carries a line-numbered diagnostic naming the failing section/assert | Integration | P0 | FR-004-AC-2 | ✅ |
| IT-049 | `quire validate fr.md --module $ISO --archetype FR` overrides frontmatter-derived archetype resolution | Integration | P1 | FR-004-AC-3 | ✅ |
| IT-050 | `quire validate doc.md --module $ISO --archetype NONEXISTENT` exits 1 with `UnknownArchetype` on stderr (re-pointed off the removed `--json` mode) | Integration | P1 | FR-004-AC-6 | ✅ |
| IT-051 | `quire validate rendered-fr.md --module $ISO` exits 1 when `## Specification` is only `TODO`, reason `placeholder` | Integration | P0 | FR-010-AC-1 | ✅ |
| IT-052 | Validate exits 1 when an FR required section is missing (reason `missing`), naming the section (line absent for a fully-missing section — FR-010 CR-003) | Integration | P0 | FR-010-AC-2 | ✅ |
| IT-053 | Validate exits 1 when the Acceptance Criteria table has wrong columns or zero data rows (reason `assert`) | Integration | P0 | FR-010-AC-3 | ✅ |
| IT-054 | Structural validation failure produces empty stdout + non-empty stderr carrying quire-rs diagnostics unchanged | Integration | P0 | FR-010-AC-5 | ✅ |
| IT-055 | `quire validate ../../etc/passwd --module $ISO` exits 1 with PathSafetyViolation naming the positional document arg | Integration | P0 | FR-005-AC-2, StR-003-AC-2, FR-004-AC-7 |
| IT-056 | `quire validate no-frontmatter.md --module $ISO` (no frontmatter, no `--archetype`) exits 1; stderr names missing frontmatter / `--archetype` remedy; empty stdout | Integration | P0 | FR-004-AC-4 |
| IT-057 | `quire validate no-type.md --module $ISO` (frontmatter present, `type` absent or non-string; no `--archetype`) exits 1; stderr names `--archetype`/`type` | Integration | P0 | FR-004-AC-5 |
| IT-058 | path-safety violation diagnostic names the arg label (`document` / `--module`) | Integration | P1 | FR-004-AC-7, FR-009-AC-5 |
| IT-059 | `quire validate - --module $ISO` reads stdin (path-safety-exempt) and still validates structurally | Integration | P1 | FR-004-AC-8 |
| IT-060 | `quire schema FR --module $ISO` exits 0; JSON contains FR frontmatter schema + `body_extraction` asserts | Integration | P0 | FR-009-AC-1 |
| IT-061 | `schema` JSON describes per-section asserts (headings/columns/id-patterns), no template-variable list | Integration | P0 | FR-009-AC-2 |
| IT-062 | `quire schema NONEXISTENT --module $ISO` exits 1 with `UnknownArchetype`, empty stdout | Integration | P1 | FR-009-AC-3 |
| IT-063 | Repeated `quire schema FR` calls produce byte-identical stdout | Integration | P2 | FR-009-AC-4 |
| IT-064 | `quire lint clean.md --module $M` exits 0, silent on both streams | Integration | P0 | FR-013-AC-1 |
| IT-065 | Warning-severity finding: exit 0, stderr `warning: <rule-id>:` + offending value, empty stdout | Integration | P0 | FR-013-AC-2 |
| IT-066 | Error-severity finding: exit 1, stderr `error: <rule-id>:` | Integration | P0 | FR-013-AC-3 |
| IT-067 | `--archetype NFR` suppresses a rule scoped `archetypes: [FR]` | Integration | P1 | FR-013-AC-4 |
| IT-068 | `--module` without manifest.yaml exits 1 naming the missing manifest (eager loader; covers validate/extract/schema too) | Integration | P0 | FR-013-AC-5, FR-004 (CR eager-load) |
| IT-069 | `validate --okf <DIR>` over a bundle with an **untyped** document exits 1; stderr contains `type` + `[frontmatter]` (`cli_okf::okf_untyped_document_is_error`; also covers extract's shared untyped vocabulary) | Integration | P0 | FR-014-AC-1, FR-014-AC-8, FR-003-AC-5 |
| IT-070 | `validate --okf <DIR>` tolerates an **unknown `type`** and a **broken `ix://` link** as warnings: exit 0, stderr `[unknown-type]` + `[dangling-reference]` (`cli_okf::okf_tolerates_unknown_type_and_broken_link`) | Integration | P0 | FR-014-AC-2, FR-014-AC-3 |
| IT-071 | `validate --okf <DIR>` warns on an `index.md` omitting a sibling artifact: exit 0, stderr `[index-incomplete]` naming the missing artifact (root `okf_version` completeness in the same posture) (`cli_okf::okf_index_incompleteness_warns`) | Integration | P0 | FR-014-AC-4, FR-014-AC-5 |
| IT-072 | `validate --okf --scope <DIR>` with no positional validates the `--scope` directory as the bundle root (exit 0, warning-only bundle) (`cli_okf::okf_defaults_to_scope_directory`) | Integration | P1 | FR-014-AC-6 |
| IT-073 | `quire validate <doc with `object:` unknown> --module $M` (no `--strict`) → exit 0, empty stdout; stderr carries a `warning:`-prefixed line naming the unknown object, distinct from any error | Integration | P0 | FR-004-AC-10 |
| IT-074 | Same doc with `--strict` → exit 1; stderr still carries the `warning:` line; empty stdout. A clean doc (no warnings/errors) under `--strict` still exits 0 | Integration | P0 | FR-004-AC-11 |
| IT-075 | Same doc with `--diagnostics-format json --strict` (or no `--strict`) → the warning is a distinct JSON object on stderr carrying a `severity`/`kind` field marking it a warning, separable from an error object | Integration | P1 | FR-004-AC-12 |
| IT-076 | `quire fix <DIR> --module $M` (dry-run) over a bundle with a bare in-bundle reference → exit 1, stderr `would-fix: <path>: <token> -> [<token>](<rel-path>)`, no file modified | Integration | P0 | FR-015-AC-1 |
| IT-077 | `quire fix <DIR> --module $M --write` rewrites the reference to the suggested relative-path link; a second `--write` run changes nothing and exits 0 (idempotence) | Integration | P0 | FR-015-AC-2 |
| IT-078 | A warn-only (unresolved/ambiguous) token is surfaced as `warning: … (<reason>)`, never written even under `--write`, and does not alone cause a nonzero exit | Integration | P0 | FR-015-AC-3 |
| IT-079 | A clean bundle (no auto-fix findings) exits 0 with empty stdout in both dry-run and `--write` | Integration | P1 | FR-015-AC-4 |
| IT-080 | `quire fix --scope <DIR> --module $M` with no positional uses `--scope` as root; a `..`/symlink-escape on root or `--module` is rejected by path-safety before any load | Integration | P0 | FR-015-AC-5, FR-005 |
| BENCH-001 | ⊘ RETIRED (§2bis) — hyperfine render p95 ≤ 50 ms on FR archetype | Benchmark | P0 | NFR-001-AC-1..2 (retired), StR-002 (retired) |
| AUDIT-001 | `ldd` shows only libc + loader (no project .so) | Static | P0 | NFR-002-AC-1 |
| AUDIT-002 | `src/` grep finds no markdown parsing, no structural-validation logic, and **no render/template code** (validation delegated to quire-rs `validate_document` / `validate_bundle_at`; render removed per §2bis) | Static | P1 | StR-004-AC-2, FR-004-AC-9, FR-014-AC-9, FR-015-AC-6 |
| AUDIT-003 | `cargo deny check bans` rejects HTTP client crates | Static | P0 | NFR-004-AC-1 |
| AUDIT-004 | `scripts/check_unsafe_comments.sh` zero unsafe in src/ + tests/ | Static | P0 | NFR-003-AC-1 |

---

## Verification Status

GREEN for the v0.1 surface — every IT / BENCH / AUDIT through IT-046 has landed
and passes `make test` + `make bench` on a Linux dev box (WSL2). `make ci` runs
the full gauntlet locally; CI lanes (rust / licenses / bench) mirror the same
gates. Observed `BENCH-001` p95 is 4.87 ms, well under the 50 ms NFR-001 budget.

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
IT-010, IT-017, IT-018, BENCH-001) and the retired FR-001/US-001/NFR-001/StR-002
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
