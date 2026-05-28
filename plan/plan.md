# Implementation Plan: quire-cli

Generated from `~/dev/quire-cli/spec/` via `/spec-to-plan`. Derived from 4 StR + 4 US + 8 FR + 6 NFR + 32 IT/BENCH/AUDIT cases (see `spec/tests.md` — 100 % AC coverage).

This is a **thin process boundary** over `quire-rs`. The plan is correspondingly small: argv parsing, sandbox guards, I/O wiring, packaging — every parse/render/extract/validate behavior delegates to the upstream crate.

## Requirements Summary

### Stakeholder Requirements
- [ ] **StR-001** Single static binary serves the agent hot path (`render`, `parse`, `extract`, `validate` in one cold-start process each).
- [ ] **StR-002** p95 ≤ 50 ms end-to-end render budget.
- [ ] **StR-003** Sandbox: reject `..` in path args, refuse symlinks escaping `--module`, inherit template-side guarantees from `quire-rs`.
- [ ] **StR-004** Thin boundary — every FR cites the upstream `quire-rs` FR it surfaces; no parser/renderer/validator logic in this crate.

### User Stories
- [ ] **US-001** Agent renders FR via `quire render` with sub-50 ms wall time.
- [ ] **US-002** Human parses an artifact for debugging via `quire parse`.
- [ ] **US-003** CI validates committed artifacts via `quire validate`.
- [ ] **US-004** Batch indexer extracts cross-references via `quire extract`.

### Functional Requirements

**CLI surface (one per subcommand):**
- [ ] **FR-001** `quire render` — consumes upstream `quire-rs` FR-001, FR-002, FR-014.
- [ ] **FR-002** `quire parse` — consumes upstream `quire-rs` FR-005, FR-006, FR-008.
- [ ] **FR-003** `quire extract` — consumes upstream `quire-rs` FR-011, FR-015.
- [ ] **FR-004** `quire validate` — consumes upstream `quire-rs` FR-002, FR-017.

**Cross-cutting CLI infrastructure:**
- [ ] **FR-005** Path-safety guard (canonicalize + `..` reject + symlink-escape reject).
- [ ] **FR-006** Stdin/stdout/stderr contract (no interleaving; stderr-only diagnostics).
- [ ] **FR-007** Exit code contract (0 success / 1 user error / 2 argv / 134 panic).
- [ ] **FR-008** JSON output encoding (compact default + `--pretty`; stable field order).

### Non-Functional Requirements
- [ ] **NFR-001** p95 ≤ 50 ms (hyperfine harness in `make bench`, CI-gated).
- [ ] **NFR-002** Static binary (`ldd` lists only libc + loader).
- [ ] **NFR-003** Zero unsafe in this crate (`check_unsafe_comments.sh`).
- [ ] **NFR-004** No network deps (`cargo deny bans` HTTP clients; strace IT-008 on Linux).
- [ ] **NFR-005** Stderr diagnostics expressible as `quire-rs::Diagnostic`.
- [ ] **NFR-006** SemVer on subcommand surface, exit codes, JSON output schemas.

---

## Dependency Graph

### Internal edges

```
FR-005 (path-safety)  ──┐
FR-006 (I/O contract) ──┼──→ FR-001 render
FR-007 (exit codes)   ──┤    FR-002 parse
FR-008 (JSON encoding) ─┘    FR-003 extract
                             FR-004 validate
```

FR-005..008 are **cross-cutting infrastructure** that every subcommand depends on. They land first (Track A), then subcommands land in parallel (Track B).

### Upstream dependency

The entire CLI depends on `quire-rs ≥ 0.1.0` (path dep during dev; crates.io pin once `quire-rs` publishes a stable). Every subcommand's logic is `quire_rs::<api>` plus stdin/stdout glue.

### Cross-cutting NFRs

- **NFR-002 (static binary)**: enforced by release-profile config + `cargo install`-style build; no special task.
- **NFR-003 (zero unsafe)**: inherited from `rust-lib-cookiecutter`; CI gate already wired.
- **NFR-004 (no network)**: `cargo deny bans` (already in `deny.toml`); add specific bans for `reqwest`/`hyper`/`tonic`/`surf`/`ureq` if not present.
- **NFR-005 (diagnostic format)**: applies to FR-005 (path-safety) and every stderr emission site.
- **NFR-001 (latency budget)**: gates FR-001 specifically; `make bench` runs the hyperfine harness.

---

## Execution Tracks

### Track A — Cross-cutting infrastructure (foundation)

Sequential. Every other task depends on these.

1. **T-001 Cargo.toml + binary scaffold**
   - Convert lib crate to binary; add `[[bin]] name = "quire"`; `src/main.rs` + `src/cli.rs`.
   - Deps: `quire-rs` (path: `../quire-rs` for dev), `clap` (derive feature), `serde_json`, `anyhow`, `serde`.
   - Release profile: `lto = "thin"`, `codegen-units = 1`.
2. **T-002 Path-safety module** (FR-005) — `src/safety.rs`:
   - `validate_module_path`, `validate_data_path`, `validate_out_path`.
   - Reject `..` segments; canonicalize; check symlink escape.
   - `PathSafetyViolation` mapped to `quire_rs::Diagnostic`.
3. **T-003 I/O wiring** (FR-006, FR-007, FR-008):
   - `src/io.rs`: `read_data(arg) -> serde_json::Value` (file vs `-`), `write_stdout`, `write_diagnostics`.
   - Exit-code helpers; `--diagnostics-format=json` flag plumbing.
   - JSON output encoder (compact / `--pretty`).
4. **T-004 Cargo.deny + unsafe baseline**:
   - Confirm `deny.toml` bans HTTP client crates.
   - Run `scripts/check_unsafe_comments.sh`; baseline empty.

### Track B — Subcommand implementations (parallel after Track A)

Each task is a separate `src/commands/<name>.rs` module wired into `src/main.rs::dispatch`.

5. **T-005 `render` command** (FR-001) — `src/commands/render.rs`
   - Argv: `<archetype>`, `--module`, `--data <file|->`, `--out`.
   - Calls `Registry::load_from(module)`, `render_by_name(&registry, archetype, &data)`.
   - Writes output to stdout or `--out`.
6. **T-006 `parse` command** (FR-002) — `src/commands/parse.rs`
   - Argv: `<doc|->`.
   - `quire_rs::parse_document(text) -> QuireDocument`.
   - Serialize as JSON; respect `--pretty`.
7. **T-007 `extract` command** (FR-003) — `src/commands/extract.rs`
   - Argv: `<doc|->`, `--module`.
   - Load registry; parse; look up archetype by `artifact_type` frontmatter; run `extract` + `harvest_edges`.
   - Emit `{extraction, edges}` JSON envelope.
   - Per review F-5: does NOT auto-validate.
8. **T-008 `validate` command** (FR-004) — `src/commands/validate.rs`
   - Argv: `<archetype>`, `--module`, `--data <file|->`.
   - Load registry; resolve archetype; `quire_rs::validate(&compiled, &data)`.
   - Exit 0/1 with stderr diagnostics on failure; no stdout.

### Track C — Tests + benches (parallel after Track B starts)

Each IT/BENCH/AUDIT in `spec/tests.md` becomes one entry in `tests/` or `benches/` or CI workflow.

9. **T-009 Vendor fixtures** (review F-2)
   - Copy `quire-rs/tests/render_parity/modules/iso/` → `tests/fixtures/iso/`.
   - Copy a handful of context JSON files for each archetype.
   - Add `make refresh-fixtures` target that re-syncs from upstream.
10. **T-010 Happy-path ITs** (IT-001..004, IT-009, IT-014, IT-017, IT-018, IT-019, IT-021)
    - `tests/cli_render.rs`, `tests/cli_parse.rs`, `tests/cli_extract.rs`, `tests/cli_validate.rs`.
    - `assert_cmd` + `predicates`.
    - IT-018 parametrizes over all 8 ISO archetypes.
11. **T-011 Sandbox ITs** (IT-005, IT-006, IT-007, IT-022, IT-023)
    - `tests/cli_sandbox.rs`. Tempdir fixtures with `..` patterns and symlinks.
12. **T-012 Error-path ITs** (IT-010, IT-012, IT-013, IT-026, IT-027)
    - `tests/cli_errors.rs`. Cover every exit code and each documented failure mode.
13. **T-013 I/O contract ITs** (IT-011, IT-024, IT-025, IT-028, IT-029, IT-030)
    - `tests/cli_io.rs`. Stdin handling, no-interleave, `--pretty`, field-order snapshot.
14. **T-014 Static audits**:
    - **AUDIT-001** (`ldd` shape): `tests/audit_ldd.rs` — Linux-only, gated by `#[cfg(target_os = "linux")]`.
    - **AUDIT-002** (no parser/render logic in `src/`): a small shell script in `scripts/check_thin_boundary.sh` greps for `parse_document(`, `render(`, `validate(` outside `commands/` dispatch sites.
    - **AUDIT-003** wired through `deny.toml`.
    - **AUDIT-004** wired through `check_unsafe_comments.sh`.
15. **T-015 Network audit** (IT-008): `tests/audit_no_network.rs` — `strace -fe network` wrapper, `#[cfg(target_os = "linux")]` per review F-3.
16. **T-016 Benchmarks**:
    - **BENCH-001** in `benches/render.rs` (criterion) AND a `make bench` target invoking `hyperfine` against the release binary.
    - CI runs `make bench` and asserts p95 ≤ 50 ms.
17. **T-017 `--help` snapshot** (IT-032, NFR-006-AC-2)
    - Per review F-4: snapshot lives at `tests/snapshots/help.txt`. Use `insta` or a hand-rolled byte-compare.

### Track D — Distribution + docs

18. **T-018 `README.md`**: install instructions (`cargo install --git ...`), usage examples per subcommand, pointer to `quire-rs` for engine docs.
19. **T-019 CI workflow updates** (`.github/workflows/ci.yml`):
    - Add `make bench` job (Linux only) gating on p95.
    - Add `make audit-thin-boundary` step.
    - Keep existing fmt/clippy/test/deny lanes from cookiecutter.
20. **T-020 Release tag + publish**:
    - Cut `v0.1.0` once Track B + Track C green.
    - `cargo publish` to crates.io (or org registry) once `quire-rs` is published.

---

## Quality Gates

Each gate is a hard merge-block until the corresponding tests are green.

| Gate | Trigger | Owner |
|------|---------|-------|
| **G1 Track A scaffold compiles** | T-001..004 done; `make build` green; `cargo deny check` green; zero unsafe | Track A |
| **G2 All four subcommands implemented** | T-005..008 done; trivial smoke run for each succeeds | Track B |
| **G3 Happy + sandbox ITs green** | T-009..011 done; `make test` green; all P0 ITs pass | Track C |
| **G4 Error + I/O ITs green** | T-012..013 done; coverage of every exit code in FR-007 | Track C |
| **G5 Static audits + network audit green** | T-014..015 done; AUDIT-001..004, IT-008 pass | Track C |
| **G6 Benchmark gate green** | T-016 done; `hyperfine` reports p95 ≤ 50 ms | Track C |
| **G7 Help snapshot pinned** | T-017 done; snapshot committed | Track C |
| **G8 Ready to tag** | G1..G7 all green; README + CI updated | Track D |

---

## Risks & Open Decisions

| ID | Item | Resolution |
|----|------|------------|
| R-1 | `quire-rs` not yet on crates.io; dep is path-based during dev. | Tag `quire-rs v0.1.0` and publish before `quire-cli v0.1.0` (matches parent plan's Q1 → Q2 → ... order). Until then, `cargo install --git` is the only install vector. |
| R-2 | Vendored fixture drift from upstream | `make refresh-fixtures` + a CI lane that compares vendored copies to upstream and warns on drift. |
| R-3 | `strace` Linux-only (IT-008) | Mark IT linux-only per F-3; rely on `cargo deny bans` cross-platform. |
| R-4 | Single `quire-rs::Diagnostic` shape may not cover every CLI-originated case | Extend upstream `Diagnostic` enum if a new variant is needed; never invent a parallel shape (NFR-005). |
| R-5 | Extract auto-validate left off in v0.1 | Documented in F-5. Re-evaluate as `--validate` flag in v0.2 if agents end up calling extract then validate back-to-back. |

---

## Test Plan Summary

100 % AC coverage; see `spec/tests.md` for the full traceability table. Counts:

- 32 IT/BENCH/AUDIT entries
- 22 ITs (process-level `assert_cmd` integration)
- 1 BENCH (`hyperfine` p95 gate)
- 4 AUDITs (ldd, thin-boundary grep, deny bans, unsafe comments)
- 5 P0 sandbox + happy-path ITs gate v0.1.0

---

## Estimated Order of Execution

1. T-001 (scaffold) → T-002 (safety) → T-003 (I/O) → T-004 (audits wired) — sequential.
2. T-005..T-008 — parallel.
3. T-009 → (T-010, T-011, T-012, T-013) — fixtures first, then ITs in parallel.
4. T-014, T-015, T-016, T-017 — parallel after subcommands compile.
5. T-018, T-019, T-020 — final.

A single focused implementer can complete the plan in 1-2 days. A team of two can parallelize Tracks B and C and finish in well under a day.
