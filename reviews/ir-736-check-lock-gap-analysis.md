---
id: SR-14561
title: gap-analysis review of IR-736 distributed lock gate
type: SpecReview
analysis: gap-analysis
scope: agent-ix/quire-cli; IR-736; src/, tests/, tools/, Cargo.toml, Cargo.lock, README.md,
  complete computed spec matrix
review_set: subset
---

## Summary

Frozen candidate: `2005e457edf9485a0b84f4f30e929a5297f61caa`. Ticket: IR-736. Reviewer-only: no product edits, Cargo, rustc, metadata, build, native tests or aggregate gate executed by this reviewer. Existing producer receipts were inspected. Model identifier is not exposed by this harness and is recorded as unavailable rather than guessed.

## Verdict

FAIL — 51 matrix rows are untagged: 49 inherited rows and two new Demonstration rows. The eight new Test criteria are tagged by real tests; scoped source review finds no new reverse gap or stub. This is not a repository assurance PASS.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | medium | FR-001-AC-1 remains untagged in the computed matrix (inherited baseline, owner PLAT-1177); no bound test symbol establishes this row. | spec/functional/FR-001-render-subcommand.md:72 |
| FND-002 | medium | FR-001-AC-2 remains untagged in the computed matrix (inherited baseline, owner PLAT-1177); no bound test symbol establishes this row. | spec/functional/FR-001-render-subcommand.md:73 |
| FND-003 | medium | FR-001-AC-3 remains untagged in the computed matrix (inherited baseline, owner PLAT-1177); no bound test symbol establishes this row. | spec/functional/FR-001-render-subcommand.md:74 |
| FND-004 | medium | FR-001-AC-4 remains untagged in the computed matrix (inherited baseline, owner PLAT-1177); no bound test symbol establishes this row. | spec/functional/FR-001-render-subcommand.md:75 |
| FND-005 | medium | FR-001-AC-5 remains untagged in the computed matrix (inherited baseline, owner PLAT-1177); no bound test symbol establishes this row. | spec/functional/FR-001-render-subcommand.md:76 |
| FND-006 | medium | FR-001-AC-6 remains untagged in the computed matrix (inherited baseline, owner PLAT-1177); no bound test symbol establishes this row. | spec/functional/FR-001-render-subcommand.md:77 |
| FND-007 | medium | FR-003-AC-3 remains untagged in the computed matrix (inherited baseline, owner PLAT-1177); no bound test symbol establishes this row. | spec/functional/FR-003-extract-subcommand.md:88 |
| FND-008 | medium | FR-004-AC-23 remains untagged in the computed matrix (inherited baseline, owner PLAT-1177); no bound test symbol establishes this row. | spec/functional/FR-004-validate-subcommand.md:183 |
| FND-009 | medium | FR-006-AC-4 remains untagged in the computed matrix (inherited baseline, owner PLAT-1177); no bound test symbol establishes this row. | spec/functional/FR-006-io-contract.md:50 |
| FND-010 | medium | FR-006-AC-5 remains untagged in the computed matrix (inherited baseline, owner PLAT-1177); no bound test symbol establishes this row. | spec/functional/FR-006-io-contract.md:51 |
| FND-011 | medium | FR-016-AC-3 remains untagged in the computed matrix (inherited baseline, owner PLAT-1177); no bound test symbol establishes this row. | spec/functional/FR-016-update-subcommand.md:75 |
| FND-012 | medium | FR-016-AC-4 remains untagged in the computed matrix (inherited baseline, owner PLAT-1177); no bound test symbol establishes this row. | spec/functional/FR-016-update-subcommand.md:76 |
| FND-013 | medium | FR-016-AC-7 remains untagged in the computed matrix (inherited baseline, owner PLAT-1177); no bound test symbol establishes this row. | spec/functional/FR-016-update-subcommand.md:79 |
| FND-014 | medium | FR-017-AC-23 remains untagged in the computed matrix (inherited baseline, owner PLAT-1177); no bound test symbol establishes this row. | spec/functional/FR-017-coverage-subcommand.md:151 |
| FND-015 | medium | FR-018-AC-10 remains untagged in the computed matrix (inherited baseline, owner PLAT-1177); no bound test symbol establishes this row. | spec/functional/FR-018-properties-subcommand.md:73 |
| FND-016 | medium | FR-019-AC-1 remains untagged in the computed matrix (inherited baseline, owner PLAT-1177); no bound test symbol establishes this row. | spec/functional/FR-019-symbols-subcommand.md:76 |
| FND-017 | medium | FR-019-AC-2 remains untagged in the computed matrix (inherited baseline, owner PLAT-1177); no bound test symbol establishes this row. | spec/functional/FR-019-symbols-subcommand.md:77 |
| FND-018 | medium | FR-019-AC-3 remains untagged in the computed matrix (inherited baseline, owner PLAT-1177); no bound test symbol establishes this row. | spec/functional/FR-019-symbols-subcommand.md:78 |
| FND-019 | medium | FR-019-AC-4 remains untagged in the computed matrix (inherited baseline, owner PLAT-1177); no bound test symbol establishes this row. | spec/functional/FR-019-symbols-subcommand.md:79 |
| FND-020 | medium | FR-019-AC-5 remains untagged in the computed matrix (inherited baseline, owner PLAT-1177); no bound test symbol establishes this row. | spec/functional/FR-019-symbols-subcommand.md:80 |
| FND-021 | medium | FR-019-CON-1 remains untagged in the computed matrix (inherited baseline, owner PLAT-1177); no bound test symbol establishes this row. | spec/functional/FR-019-symbols-subcommand.md:94 |
| FND-022 | medium | FR-020-CON-2 remains untagged in the computed matrix (inherited baseline, owner PLAT-1177); no bound test symbol establishes this row. | spec/functional/FR-020-assurance-export-subcommand.md:79 |
| FND-023 | medium | FR-020-CON-4 remains untagged in the computed matrix (inherited baseline, owner PLAT-1177); no bound test symbol establishes this row. | spec/functional/FR-020-assurance-export-subcommand.md:81 |
| FND-024 | medium | FR-021-AC-2 remains untagged in the computed matrix (inherited baseline, owner PLAT-1177); no bound test symbol establishes this row. | spec/functional/FR-021-clauses-subcommand.md:33 |
| FND-025 | medium | FR-021-AC-3 remains untagged in the computed matrix (inherited baseline, owner PLAT-1177); no bound test symbol establishes this row. | spec/functional/FR-021-clauses-subcommand.md:34 |
| FND-026 | medium | FR-021-AC-5 remains untagged in the computed matrix (inherited baseline, owner PLAT-1177); no bound test symbol establishes this row. | spec/functional/FR-021-clauses-subcommand.md:36 |
| FND-027 | medium | FR-021-AC-6 remains untagged in the computed matrix (inherited baseline, owner PLAT-1177); no bound test symbol establishes this row. | spec/functional/FR-021-clauses-subcommand.md:37 |
| FND-028 | medium | FR-027-AC-8 is untagged in the computed matrix; its real external original-AWK versus native-command demonstration is retained outside the repository, rather than bound to a test symbol. | spec/functional/FR-027-check-lock-subcommand.md:123 |
| FND-029 | medium | FR-027-AC-10 is untagged in the computed matrix despite an external exact-head candidate Titan install receipt; published-release and other-host qualification are explicitly unestablished. | spec/functional/FR-027-check-lock-subcommand.md:125 |
| FND-030 | medium | NFR-001-AC-2 remains untagged in the computed matrix (inherited baseline, owner PLAT-1177); no bound test symbol establishes this row. | spec/non-functional/NFR-001-render-latency-budget.md:58 |
| FND-031 | medium | NFR-001-M-1 remains untagged in the computed matrix (inherited baseline, owner PLAT-1177); no bound test symbol establishes this row. | spec/non-functional/NFR-001-render-latency-budget.md:1 |
| FND-032 | medium | NFR-001-M-2 remains untagged in the computed matrix (inherited baseline, owner PLAT-1177); no bound test symbol establishes this row. | spec/non-functional/NFR-001-render-latency-budget.md:1 |
| FND-033 | medium | NFR-001-M-3 remains untagged in the computed matrix (inherited baseline, owner PLAT-1177); no bound test symbol establishes this row. | spec/non-functional/NFR-001-render-latency-budget.md:1 |
| FND-034 | medium | NFR-002-AC-2 remains untagged in the computed matrix (inherited baseline, owner PLAT-1177); no bound test symbol establishes this row. | spec/non-functional/NFR-002-static-binary.md:42 |
| FND-035 | medium | NFR-002-M-1 remains untagged in the computed matrix (inherited baseline, owner PLAT-1177); no bound test symbol establishes this row. | spec/non-functional/NFR-002-static-binary.md:1 |
| FND-036 | medium | NFR-002-M-2 remains untagged in the computed matrix (inherited baseline, owner PLAT-1177); no bound test symbol establishes this row. | spec/non-functional/NFR-002-static-binary.md:1 |
| FND-037 | medium | NFR-003-M-1 remains untagged in the computed matrix (inherited baseline, owner PLAT-1177); no bound test symbol establishes this row. | spec/non-functional/NFR-003-zero-unsafe.md:1 |
| FND-038 | medium | NFR-004-M-1 remains untagged in the computed matrix (inherited baseline, owner PLAT-1177); no bound test symbol establishes this row. | spec/non-functional/NFR-004-no-network.md:1 |
| FND-039 | medium | NFR-004-M-2 remains untagged in the computed matrix (inherited baseline, owner PLAT-1177); no bound test symbol establishes this row. | spec/non-functional/NFR-004-no-network.md:1 |
| FND-040 | medium | NFR-005-M-1 remains untagged in the computed matrix (inherited baseline, owner PLAT-1177); no bound test symbol establishes this row. | spec/non-functional/NFR-005-diagnostic-format.md:1 |
| FND-041 | medium | NFR-005-M-2 remains untagged in the computed matrix (inherited baseline, owner PLAT-1177); no bound test symbol establishes this row. | spec/non-functional/NFR-005-diagnostic-format.md:1 |
| FND-042 | medium | NFR-006-M-1 remains untagged in the computed matrix (inherited baseline, owner PLAT-1177); no bound test symbol establishes this row. | spec/non-functional/NFR-006-cli-stability.md:1 |
| FND-043 | medium | NFR-006-M-2 remains untagged in the computed matrix (inherited baseline, owner PLAT-1177); no bound test symbol establishes this row. | spec/non-functional/NFR-006-cli-stability.md:1 |
| FND-044 | medium | NFR-008-M-1 remains untagged in the computed matrix (inherited baseline, owner PLAT-1177); no bound test symbol establishes this row. | spec/non-functional/NFR-008-native-npm-distribution-tooling.md:1 |
| FND-045 | medium | NFR-008-M-2 remains untagged in the computed matrix (inherited baseline, owner PLAT-1177); no bound test symbol establishes this row. | spec/non-functional/NFR-008-native-npm-distribution-tooling.md:1 |
| FND-046 | medium | NFR-008-M-3 remains untagged in the computed matrix (inherited baseline, owner PLAT-1177); no bound test symbol establishes this row. | spec/non-functional/NFR-008-native-npm-distribution-tooling.md:1 |
| FND-047 | medium | NFR-008-M-4 remains untagged in the computed matrix (inherited baseline, owner PLAT-1177); no bound test symbol establishes this row. | spec/non-functional/NFR-008-native-npm-distribution-tooling.md:1 |
| FND-048 | medium | NFR-009-M-1 remains untagged in the computed matrix (inherited baseline, owner PLAT-1177); no bound test symbol establishes this row. | spec/non-functional/NFR-009-native-qualification-boundary.md:1 |
| FND-049 | medium | NFR-009-M-2 remains untagged in the computed matrix (inherited baseline, owner PLAT-1177); no bound test symbol establishes this row. | spec/non-functional/NFR-009-native-qualification-boundary.md:1 |
| FND-050 | medium | NFR-009-M-3 remains untagged in the computed matrix (inherited baseline, owner PLAT-1177); no bound test symbol establishes this row. | spec/non-functional/NFR-009-native-qualification-boundary.md:1 |
| FND-051 | medium | NFR-009-M-4 remains untagged in the computed matrix (inherited baseline, owner PLAT-1177); no bound test symbol establishes this row. | spec/non-functional/NFR-009-native-qualification-boundary.md:1 |

## Examined Scope

```yaml
- id: FR-001-AC-1
  path: spec/functional/FR-001-render-subcommand.md
  role: examined
  excerpt: (RETIRED) `quire render FR --module $ISO --data ctx.json` against the canonical ISO
    FR archetype produces byte-identical output to `minijinja-cli templates/fr.md.j2 ctx.json`
    for the same context
- id: FR-001-AC-2
  path: spec/functional/FR-001-render-subcommand.md
  role: examined
  excerpt: (RETIRED) `quire render FR --module $ISO --data -` reads context from stdin and produces
    equivalent output
- id: FR-001-AC-3
  path: spec/functional/FR-001-render-subcommand.md
  role: examined
  excerpt: (RETIRED) `quire render NONEXISTENT --module $ISO --data ctx.json` exits 1; stderr
    begins with a `UnknownArchetype` diagnostic; no stdout output
- id: FR-001-AC-4
  path: spec/functional/FR-001-render-subcommand.md
  role: examined
  excerpt: (RETIRED) `quire render FR --module $ISO --data invalid.json` (failing schema) exits
    1; stderr lists violations per FR-017; stdout is empty
- id: FR-001-AC-5
  path: spec/functional/FR-001-render-subcommand.md
  role: examined
  excerpt: (RETIRED) `quire render FR --module $ISO --data ctx.json --out /tmp/fr.md` writes
    the file and produces no stdout
- id: FR-001-AC-6
  path: spec/functional/FR-001-render-subcommand.md
  role: examined
  excerpt: (RETIRED) All 8 ISO archetypes (FR, NFR, StR, US, IT, TC, AC, CON) render byte-identically
    through this subcommand vs. the Python Jinja2 reference (parity matrix vendored from `quire-rs/tests/render_parity/`)
- id: FR-002-AC-1
  path: spec/functional/FR-002-parse-subcommand.md
  role: examined
  excerpt: '`quire parse some-doc.md | jq ''.frontmatter.id''` returns the document''s `id`
    field for an artifact with `id: FR-035` in frontmatter'
- id: FR-002-AC-2
  path: spec/functional/FR-002-parse-subcommand.md
  role: examined
  excerpt: '`cat some-doc.md | quire parse -` produces identical output to `quire parse some-doc.md`'
- id: FR-002-AC-3
  path: spec/functional/FR-002-parse-subcommand.md
  role: examined
  excerpt: A document with malformed frontmatter still produces parseable JSON on stdout; stderr
    contains a non-fatal diagnostic naming the frontmatter line
- id: FR-002-AC-4
  path: spec/functional/FR-002-parse-subcommand.md
  role: examined
  excerpt: Stdout for an empty document is a valid `QuireDocument` JSON with empty `sections[]`
- id: FR-002-AC-5
  path: spec/functional/FR-002-parse-subcommand.md
  role: examined
  excerpt: The byte offsets in the JSON output match the byte slices `quire_rs::parse_document`
    produces in-process (round-trip fixture test)
- id: FR-003-AC-1
  path: spec/functional/FR-003-extract-subcommand.md
  role: examined
  excerpt: '`quire extract sample-fr.md --module $ISO` against a fixture document produces JSON
    with non-empty `extraction` and at least one edge'
- id: FR-003-AC-2
  path: spec/functional/FR-003-extract-subcommand.md
  role: examined
  excerpt: For a document whose frontmatter declares a `type` not present in the module, exit
    3 with `UnknownArchetype` on stderr; stdout is empty
- id: FR-003-AC-3
  path: spec/functional/FR-003-extract-subcommand.md
  role: examined
  excerpt: "\u26D4 RETIRED (CR, 2026-08-20) \u2014 was: for a document with frontmatter sugar\
    \ fields, `.edges` contains a `dependencies`-typed edge. No engine ever harvested sugar\
    \ fields; see the CR note above"
- id: FR-003-AC-4
  path: spec/functional/FR-003-extract-subcommand.md
  role: examined
  excerpt: Re-running extract on the same input produces byte-identical stdout (determinism,
    matches upstream)
- id: FR-003-AC-5
  path: spec/functional/FR-003-extract-subcommand.md
  role: examined
  excerpt: For a document with **no `type`** (the discriminator absent from frontmatter) and
    no `--archetype`, `extract` exits 3 with a `[frontmatter]`-reason `ValidationError`-shaped
    diagnostic on stderr naming the missing `type` (the same vocabulary `validate` uses), not
    a generic anyhow error; stdout is empty
- id: FR-004-AC-1
  path: spec/functional/FR-004-validate-subcommand.md
  role: examined
  excerpt: '`quire validate valid-fr.md --module $ISO` exits 0 with no output (frontmatter valid,
    all required structure present)'
- id: FR-004-AC-2
  path: spec/functional/FR-004-validate-subcommand.md
  role: examined
  excerpt: '`quire validate broken-fr.md --module $ISO` exits 3; stderr contains a line-numbered
    diagnostic naming the failing section/assert'
- id: FR-004-AC-3
  path: spec/functional/FR-004-validate-subcommand.md
  role: examined
  excerpt: '`quire validate fr.md --module $ISO --archetype FR` overrides frontmatter-derived
    archetype resolution'
- id: FR-004-AC-4
  path: spec/functional/FR-004-validate-subcommand.md
  role: examined
  excerpt: A document with **no frontmatter** and no `--archetype` exits 3; stderr names the
    missing frontmatter / `type` and points at `--archetype` as the remedy. No stdout
- id: FR-004-AC-5
  path: spec/functional/FR-004-validate-subcommand.md
  role: examined
  excerpt: A document whose frontmatter is present but has **no string `type`** (key absent,
    or a non-string value) and no `--archetype` exits 3; the diagnostic names `--archetype`
    (or `type`) as the way to resolve the archetype. No stdout
- id: FR-004-AC-6
  path: spec/functional/FR-004-validate-subcommand.md
  role: examined
  excerpt: When the resolved or `--archetype`-overridden name is unknown to the loaded module,
    `validate` exits 3 with quire-rs `UnknownArchetype` on stderr; empty stdout
- id: FR-004-AC-7
  path: spec/functional/FR-004-validate-subcommand.md
  role: examined
  excerpt: A path-safety violation on the document or `--module` exits 3 with a `PathSafetyViolation`
    ([FR-005](./FR-005-path-safety.md)) whose diagnostic names the offending argument label
    (the positional `document`, or `--module`)
- id: FR-004-AC-8
  path: spec/functional/FR-004-validate-subcommand.md
  role: examined
  excerpt: '`quire validate - --module $ISO` reads the document from stdin and is **not** subject
    to path-safety on the document argument (stdin is path-safety-exempt, [FR-005-AC-5](./FR-005-path-safety.md));
    the markdown is still validated structurally'
- id: FR-004-AC-9
  path: spec/functional/FR-004-validate-subcommand.md
  role: examined
  excerpt: All validation logic is delegated to quire-rs; an audit confirms the CLI crate contains
    no structural-validation logic of its own ([StR-004](../stakeholder/StR-004-thin-boundary-over-quire-rs.md)
    thin boundary)
- id: FR-004-AC-10
  path: spec/functional/FR-004-validate-subcommand.md
  role: examined
  excerpt: A document that is otherwise conformant but declares a frontmatter `object:` the
    registry cannot resolve produces a quire-rs **warning**. Without `--strict`, `validate`
    exits **0** and prints the warning to stderr, clearly marked (`warning:` prefix in human
    format) and distinct from any error; stdout stays empty
- id: FR-004-AC-11
  path: spec/functional/FR-004-validate-subcommand.md
  role: examined
  excerpt: 'With `--strict`, the same unknown-`object:` warning becomes exit-failing: `validate`
    exits **3**, the warning still appears on stderr; stdout stays empty. A document with NO
    warnings and no errors still exits 0 under `--strict`'
- id: FR-004-AC-12
  path: spec/functional/FR-004-validate-subcommand.md
  role: examined
  excerpt: Under `--diagnostics-format json`, a warning is emitted as a distinct JSON object
    carrying a `severity`/`kind` field marking it a warning (not an error), so machine consumers
    can tell warnings from errors. An error retains its error `kind`
- id: FR-004-AC-13
  path: spec/functional/FR-004-validate-subcommand.md
  role: examined
  excerpt: 'Scoped validation discovers modules from the default install root `~/.ix/filament/modules`
    and from `IX_FILAMENT_MODULES_PATH` (in addition to `--scope`, `--scope/.ix/modules`, and
    the legacy `IX_SCHEMA_PATH`) with no env var required and no network: a document validates
    against a module provided only via the default-root/env discovery path, and the run opens
    no inet socket'
- id: FR-004-AC-14
  path: spec/functional/FR-004-validate-subcommand.md
  role: examined
  excerpt: When scoped discovery finds zero modules and `quoin` is not available on PATH, `validate`
    exits 3 with an actionable diagnostic naming `quoin plugin ensure-defaults` (and `IX_FILAMENT_MODULES_PATH`);
    empty stdout. When `quoin` IS available, the empty-discovery path shells out to `quoin plugin
    ensure-defaults` once and reloads before validating (the [NFR-004](../non-functional/NFR-004-no-network.md)
    network exception, [ADR-0001](../assets/adr/0001-validate-lazy-init-module-bootstrap.md))
- id: FR-004-AC-15
  path: spec/functional/FR-004-validate-subcommand.md
  role: examined
  excerpt: With `--summary`, a run over a bundle whose documents emit findings from two different
    grammars prints a histogram containing a row for **each** grammar (`[ears:*]` and `[ac:*]`
    both appear), and the exit code is identical to the same run without `--summary`
- id: FR-004-AC-16
  path: spec/functional/FR-004-validate-subcommand.md
  role: examined
  excerpt: '`--severity <grammar>:<check>=off` suppresses that check entirely: the warning disappears
    from stderr AND its row disappears from the `--summary` histogram, while every other check''s
    row is unchanged'
- id: FR-004-AC-17
  path: spec/functional/FR-004-validate-subcommand.md
  role: examined
  excerpt: '`--severity <grammar>:<check>=error` promotes that check: a run that exits 0 without
    the override exits 3 with the override, and the finding is reported as an error rather than
    a warning'
- id: FR-004-AC-18
  path: spec/functional/FR-004-validate-subcommand.md
  role: examined
  excerpt: A malformed `--severity` entry (no `<grammar>:<check>=<level>` shape, an unknown
    level, or an empty grammar/check segment) exits non-zero with a usage diagnostic **before
    any document is read**, never a silently ignored flag
- id: FR-004-AC-19
  path: spec/functional/FR-004-validate-subcommand.md
  role: examined
  excerpt: "A **relative document path** given with `--scope <DIR>` resolves under that scope\
    \ and validates against the module the scope itself carries \u2014 the `--scope`-contains-`manifest.yaml`\
    \ exact-module branch, not the search-root discovery branch, which is AC-13's \u2014 with\
    \ no `--module` argument: a conformant document exits **0** with empty stdout and empty\
    \ stderr"
- id: FR-004-AC-20
  path: spec/functional/FR-004-validate-subcommand.md
  role: examined
  excerpt: A **relative glob** given with `--scope <DIR>` expands under that scope, and a matching
    non-conformant document exits **3** with a line-numbered diagnostic on stderr naming the
    offending file; stdout stays empty
- id: FR-004-AC-23
  path: spec/functional/FR-004-validate-subcommand.md
  role: examined
  excerpt: '`--scope` decides where a relative path or glob resolves **regardless of `--module`**.
    Passing `--module` pins the module set and never moves document resolution to the process
    directory; a run with and without it selects the same documents, and a decoy file of the
    same relative name in the current directory is not read.'
- id: FR-004-AC-21
  path: spec/functional/FR-004-validate-subcommand.md
  role: examined
  excerpt: 'Requirement-**grammar** findings are advisory, distinct from the `object:` warning
    of AC-10: a structurally valid document carrying grammar violations exits **0**, and each
    finding appears on stderr under its `[<grammar>:<check>]` label; stdout stays empty'
- id: FR-004-AC-22
  path: spec/functional/FR-004-validate-subcommand.md
  role: examined
  excerpt: "`--strict` escalates grammar findings the same way it escalates the AC-11 `object:`\
    \ warning \u2014 the same document that exits 0 without it exits **3** with it. This is\
    \ the per-repo promotion lever: a converted repo flips its grammar to blocking in CI without\
    \ a module change"
- id: FR-005-AC-1
  path: spec/functional/FR-005-path-safety.md
  role: examined
  excerpt: '`quire validate doc.md --module ../escape` exits 1 with `PathSafetyViolation` naming
    `--module`'
- id: FR-005-AC-2
  path: spec/functional/FR-005-path-safety.md
  role: examined
  excerpt: '`quire validate ../../etc/passwd --module $ISO` exits 1 with `PathSafetyViolation`
    naming the positional document argument'
- id: FR-005-AC-3
  path: spec/functional/FR-005-path-safety.md
  role: examined
  excerpt: (RETIRED) `quire render FR --module $ISO --data ctx.json --out ../escape.md` exits
    1 with `PathSafetyViolation` naming `--out`. The `--out` rule survives on `edit`'s write
    target (see [FR-012](./FR-012-edit-subcommand.md))
- id: FR-005-AC-4
  path: spec/functional/FR-005-path-safety.md
  role: examined
  excerpt: A symlink inside the module root pointing to `/etc/passwd` is refused at module load
    time with a `PathSafetyViolation`; the offending symlink's relative path is reported
- id: FR-005-AC-5
  path: spec/functional/FR-005-path-safety.md
  role: examined
  excerpt: A positional `-` (stdin) is never subject to path-safety checks; it cannot escape
    the filesystem
- id: FR-006-AC-1
  path: spec/functional/FR-006-io-contract.md
  role: examined
  excerpt: For each subcommand, a failure case produces empty stdout and non-empty stderr
- id: FR-006-AC-2
  path: spec/functional/FR-006-io-contract.md
  role: examined
  excerpt: For each subcommand, a success case produces non-empty stdout (except `validate`)
    and empty stderr (except for non-fatal advisories explicitly allowed by upstream `quire-rs`
    FRs)
- id: FR-006-AC-3
  path: spec/functional/FR-006-io-contract.md
  role: examined
  excerpt: All structured stderr diagnostics are valid `quire-rs::Diagnostic` JSON when the
    `--diagnostics-format=json` flag is set (default is human-readable per upstream FR-017)
- id: FR-006-AC-4
  path: spec/functional/FR-006-io-contract.md
  role: examined
  excerpt: '[FR-001](./FR-001-render-subcommand.md) works in bash (positional `-` stdin handling
    is correct for piped input)'
- id: FR-006-AC-5
  path: spec/functional/FR-006-io-contract.md
  role: examined
  excerpt: "A **census** is a result, not a diagnostic: every human-mode summary figure a subcommand\
    \ computes \u2014 coverage totals and per-document counts, the grammar-clean ratio, the\
    \ criteria-extractable ratio \u2014 is written to stdout and carries no ANSI escape, while\
    \ findings and warnings stay on stderr. `quire <cmd> > out.txt` therefore captures the figures\
    \ a caller redirected for (CR-012)"
- id: FR-007-AC-1
  path: spec/functional/FR-007-exit-codes.md
  role: examined
  excerpt: Successful commands, help and version exit 0.
- id: FR-007-AC-2
  path: spec/functional/FR-007-exit-codes.md
  role: examined
  excerpt: Path traversal and explicit policy refusals exit 2 with diagnostics.
- id: FR-007-AC-3
  path: spec/functional/FR-007-exit-codes.md
  role: examined
  excerpt: Unknown archetypes and invalid or missing input exit 3.
- id: FR-007-AC-4
  path: spec/functional/FR-007-exit-codes.md
  role: examined
  excerpt: Structural validation failures without a primary report exit 3.
- id: FR-007-AC-5
  path: spec/functional/FR-007-exit-codes.md
  role: examined
  excerpt: Invalid argv exits 3 with clap diagnostics.
- id: FR-007-AC-6
  path: spec/functional/FR-007-exit-codes.md
  role: examined
  excerpt: Covered invalid inputs return a classified failure without panicking.
- id: FR-007-AC-7
  path: spec/functional/FR-007-exit-codes.md
  role: examined
  excerpt: Strict coverage/matrix and unresolved property reports exit 1 with the complete requested
    payload.
- id: FR-007-AC-8
  path: spec/functional/FR-007-exit-codes.md
  role: examined
  excerpt: A failed output write exits 4; typed classification is independent of human diagnostic
    wording.
- id: FR-008-AC-1
  path: spec/functional/FR-008-json-output-encoding.md
  role: examined
  excerpt: '`quire parse doc.md` output round-trips through `serde_json::from_str::<QuireDocument>`
    (lib-side smoke test)'
- id: FR-008-AC-2
  path: spec/functional/FR-008-json-output-encoding.md
  role: examined
  excerpt: '`quire extract doc.md --module $MOD` output deserializes into `{ "extraction": ExtractionResult,
    "edges": Vec<HarvestedEdge> }`'
- id: FR-008-AC-3
  path: spec/functional/FR-008-json-output-encoding.md
  role: examined
  excerpt: '`quire parse doc.md --pretty` produces multi-line indented JSON with the same logical
    content as compact form'
- id: FR-008-AC-4
  path: spec/functional/FR-008-json-output-encoding.md
  role: examined
  excerpt: Byte-for-byte output of `parse` is identical across runs against the same input (determinism)
- id: FR-008-AC-5
  path: spec/functional/FR-008-json-output-encoding.md
  role: examined
  excerpt: 'No bare `version`, `schema_version` or `$schema` key appears at any level of JSON
    output: a payload never names the contract revision it claims to conform to (CR-104)'
- id: FR-008-AC-6
  path: spec/functional/FR-008-json-output-encoding.md
  role: examined
  excerpt: "Every JSON payload \u2014 `coverage`, `properties`, `extract` \u2014 carries a top-level\
    \ `engine` object naming the CLI version, the resolved engine version and a capability token\
    \ list; `quire --version` reports both versions distinctly (CR-104)"
- id: FR-009-AC-1
  path: spec/functional/FR-009-schema-subcommand.md
  role: examined
  excerpt: '`quire schema FR --module $ISO` exits 0 and emits JSON containing the FR frontmatter
    schema and the FR `body_extraction` asserts (required headings / columns)'
- id: FR-009-AC-2
  path: spec/functional/FR-009-schema-subcommand.md
  role: examined
  excerpt: "The emitted JSON describes, per required section, the asserts that `validate_document`\
    \ will enforce (heading presence/level, table columns, id-patterns) \u2014 the asserts-based\
    \ input contract, with no template-variable list"
- id: FR-009-AC-3
  path: spec/functional/FR-009-schema-subcommand.md
  role: examined
  excerpt: '`quire schema NONEXISTENT --module $ISO` exits 3 with `UnknownArchetype` on stderr
    and empty stdout'
- id: FR-009-AC-4
  path: spec/functional/FR-009-schema-subcommand.md
  role: examined
  excerpt: Repeated `quire schema FR --module $ISO` calls produce byte-identical stdout
- id: FR-009-AC-5
  path: spec/functional/FR-009-schema-subcommand.md
  role: examined
  excerpt: The command performs module and path-safety checks equivalent to `validate` ([FR-005](./FR-005-path-safety.md))
- id: FR-010-AC-1
  path: spec/functional/FR-010-required-section-validation.md
  role: examined
  excerpt: '`quire validate rendered-fr.md --module $ISO` exits 1 when `## Specification` content
    is only `TODO`, with reason `placeholder`'
- id: FR-010-AC-2
  path: spec/functional/FR-010-required-section-validation.md
  role: examined
  excerpt: It exits 1 when any FR required section is missing (reason `missing`), with a line
    number
- id: FR-010-AC-3
  path: spec/functional/FR-010-required-section-validation.md
  role: examined
  excerpt: It exits 1 when the Acceptance Criteria table has wrong columns or zero data rows
    (reasons `assert`)
- id: FR-010-AC-4
  path: spec/functional/FR-010-required-section-validation.md
  role: examined
  excerpt: It exits 0 when frontmatter is valid and all `body_extraction` asserts hold
- id: FR-010-AC-5
  path: spec/functional/FR-010-required-section-validation.md
  role: examined
  excerpt: Structural failures produce empty stdout and non-empty stderr carrying the quire-rs
    diagnostics unchanged
- id: FR-011-AC-1
  path: spec/functional/FR-011-lookup-subcommand.md
  role: examined
  excerpt: 'Given `# Title`, `quire lookup doc.md --heading Title --level 1` emits a section
    JSON object with `level: 1` and `heading: "Title"`'
- id: FR-011-AC-2
  path: spec/functional/FR-011-lookup-subcommand.md
  role: examined
  excerpt: 'Given `## Behavior {#blk-behavior}`, `quire lookup doc.md --block-id blk-behavior`
    emits a section JSON object with `block_id: "blk-behavior"`'
- id: FR-011-AC-3
  path: spec/functional/FR-011-lookup-subcommand.md
  role: examined
  excerpt: Given `## Behavior` on body line 4, `quire lookup doc.md --id behavior-L4` emits
    that section
- id: FR-011-AC-4
  path: spec/functional/FR-011-lookup-subcommand.md
  role: examined
  excerpt: With `--content`, stdout is the selected section's content only, not a JSON string
- id: FR-011-AC-5
  path: spec/functional/FR-011-lookup-subcommand.md
  role: examined
  excerpt: Passing multiple selectors is an argv error
- id: FR-011-AC-6
  path: spec/functional/FR-011-lookup-subcommand.md
  role: examined
  excerpt: Passing `--level` without `--heading` is an argv error
- id: FR-012-AC-1
  path: spec/functional/FR-012-edit-subcommand.md
  role: examined
  excerpt: '`quire edit doc.md --heading Description --content body.txt` replaces the Description
    body and leaves frontmatter and all other sections byte-identical'
- id: FR-012-AC-2
  path: spec/functional/FR-012-edit-subcommand.md
  role: examined
  excerpt: '`quire edit doc.md --block-id blk-behavior --content block.txt` replaces the full
    `{#blk-behavior}` block'
- id: FR-012-AC-3
  path: spec/functional/FR-012-edit-subcommand.md
  role: examined
  excerpt: '`--out` pointing at the input path edits the document in place'
- id: FR-012-AC-4
  path: spec/functional/FR-012-edit-subcommand.md
  role: examined
  excerpt: A selector that matches no section exits 3 without writing; the input file is unchanged
- id: FR-012-AC-5
  path: spec/functional/FR-012-edit-subcommand.md
  role: examined
  excerpt: Passing both `--heading` and `--block-id` is an argv error; passing neither is a
    user error
- id: FR-012-AC-6
  path: spec/functional/FR-012-edit-subcommand.md
  role: examined
  excerpt: Passing `-` for both `<DOC>` and `--content` is a user error
- id: FR-013-AC-1
  path: spec/functional/FR-013-lint-subcommand.md
  role: examined
  excerpt: '`quire lint clean.md --module $M` against a conforming document exits 0 with empty
    stdout and empty stderr'
- id: FR-013-AC-2
  path: spec/functional/FR-013-lint-subcommand.md
  role: examined
  excerpt: 'Against a document with a `warning`-severity finding, the command exits 0 and stderr
    carries `warning: <rule-id>:` plus the offending value; stdout stays empty'
- id: FR-013-AC-3
  path: spec/functional/FR-013-lint-subcommand.md
  role: examined
  excerpt: 'Against a document with an `error`-severity finding, the command exits 3 and stderr
    carries `error: <rule-id>:`'
- id: FR-013-AC-4
  path: spec/functional/FR-013-lint-subcommand.md
  role: examined
  excerpt: '`--archetype <NAME>` overrides scoping: a rule filtered to `archetypes: [FR]` emits
    nothing when the document is linted as `NFR`'
- id: FR-013-AC-5
  path: spec/functional/FR-013-lint-subcommand.md
  role: examined
  excerpt: A `--module` path without `manifest.yaml` exits 3 naming the missing manifest (shared
    eager-failure loader), not `UnknownArchetype`
- id: FR-014-AC-1
  path: spec/functional/FR-014-validate-okf-bundle.md
  role: examined
  excerpt: '`quire validate --okf <DIR> --module $M` over a bundle whose document has **no `type`**
    exits **3**; stderr carries a `[frontmatter]`-reason diagnostic naming `type` (`type` required
    under OKF)'
- id: FR-014-AC-2
  path: spec/functional/FR-014-validate-okf-bundle.md
  role: examined
  excerpt: 'An **unknown `type`** under `--okf` is a **warning**: the command exits **0** and
    stderr carries an `[unknown-type]` diagnostic'
- id: FR-014-AC-3
  path: spec/functional/FR-014-validate-okf-bundle.md
  role: examined
  excerpt: 'A **broken `ix://` link** under `--okf` is a **warning**: the command exits **0**
    and stderr carries a `[dangling-reference]` diagnostic'
- id: FR-014-AC-4
  path: spec/functional/FR-014-validate-okf-bundle.md
  role: examined
  excerpt: 'An **`index.md` that omits a sibling artifact** under `--okf` is a **warning**:
    the command exits **0** and stderr carries an `[index-incomplete]` diagnostic naming the
    missing artifact'
- id: FR-014-AC-5
  path: spec/functional/FR-014-validate-okf-bundle.md
  role: examined
  excerpt: A root `index.md` **missing `okf_version`** is reported as an `[index-incomplete]`
    warning (exit 0), consistent with AC-4's completeness contract
- id: FR-014-AC-6
  path: spec/functional/FR-014-validate-okf-bundle.md
  role: examined
  excerpt: '`quire validate --okf --scope <DIR> --module $M` with **no positional** validates
    `<DIR>/spec` as the bundle root, with `<DIR>` itself kept as the reference root (exit 0
    for a warning-only bundle); a `<DIR>` holding no `spec/` exits non-zero with a diagnostic
    naming the missing document root, never falling back to walking `<DIR>`'
- id: FR-014-AC-7
  path: spec/functional/FR-014-validate-okf-bundle.md
  role: examined
  excerpt: "`quire validate` with **no positional and no `--okf`** is a clap argv error \u2192\
    \ **exit 3** (`required_unless_present = \"okf\"`), unchanged from [FR-004](./FR-004-validate-subcommand.md)"
- id: FR-014-AC-8
  path: spec/functional/FR-014-validate-okf-bundle.md
  role: examined
  excerpt: "(base concept contract) an untyped document is a hard error (exit 3, `[frontmatter]`\
    \ diagnostic naming `type`) in **both** postures \u2014 strict per-file ([FR-004](./FR-004-validate-subcommand.md))\
    \ and `--okf` bundle \u2014 because quire-rs enforces `type` required + non-empty for every\
    \ validated document"
- id: FR-014-AC-9
  path: spec/functional/FR-014-validate-okf-bundle.md
  role: examined
  excerpt: (thin boundary) all bundle/base-concept validation is delegated to quire-rs (`validate_bundle`,
    base-concept enforcement); the CLI only resolves the root, applies path-safety, and surfaces
    the `BundleReport` ([StR-004](../stakeholder/StR-004-thin-boundary-over-quire-rs.md); TC-090)
- id: FR-015-AC-1
  path: spec/functional/FR-015-fix-subcommand.md
  role: examined
  excerpt: '`quire fix <DIR>` (dry-run) over a bundle with a bare in-bundle reference exits
    **1**, stderr carries `would-fix: <path>: <token> -> [<token>](<rel-path>)`, and no file
    is modified'
- id: FR-015-AC-2
  path: spec/functional/FR-015-fix-subcommand.md
  role: examined
  excerpt: '`quire fix <DIR> --write` rewrites the bare reference to the suggested relative-path
    link in place; a second `--write` run modifies nothing and exits **0** (idempotence, FR-039)'
- id: FR-015-AC-3
  path: spec/functional/FR-015-fix-subcommand.md
  role: examined
  excerpt: "A `WarnOnly` (unresolved/ambiguous) token is surfaced as `warning: \u2026 (<reason>)`,\
    \ is never written even under `--write`, and does not by itself cause a non-zero exit"
- id: FR-015-AC-4
  path: spec/functional/FR-015-fix-subcommand.md
  role: examined
  excerpt: A clean bundle (no `AutoFix` findings) exits **0** with empty stdout in both dry-run
    and `--write`
- id: FR-015-AC-5
  path: spec/functional/FR-015-fix-subcommand.md
  role: examined
  excerpt: '`quire fix --scope <DIR>` with no positional uses `<DIR>/spec` as the bundle root,
    and a `<DIR>` holding no `spec/` exits non-zero with a diagnostic naming the missing document
    root; a `..` or symlink-escape path on the root is rejected by path-safety ([FR-005](./FR-005-path-safety.md))
    before any load'
- id: FR-015-AC-6
  path: spec/functional/FR-015-fix-subcommand.md
  role: examined
  excerpt: (thin boundary) all detection, classification, suggested-link construction, and span
    application are delegated to quire-rs (`unlinked_references`, writeback); the CLI only resolves
    the root, applies path-safety, orders spans, and surfaces results ([StR-004](../stakeholder/StR-004-thin-boundary-over-quire-rs.md))
- id: FR-016-AC-1
  path: spec/functional/FR-016-update-subcommand.md
  role: examined
  excerpt: A binary path under `node_modules/...` classifies as the npm channel; a path under
    `.cargo/...` classifies as the cargo channel; any other path classifies as unknown
- id: FR-016-AC-2
  path: spec/functional/FR-016-update-subcommand.md
  role: examined
  excerpt: On an unknown source, `quire update` (with or without `--check`) prints manual upgrade
    instructions including the npm recipe, the cargo recipe, and the releases URL, exits **0**,
    and performs no install
- id: FR-016-AC-3
  path: spec/functional/FR-016-update-subcommand.md
  role: examined
  excerpt: On the npm channel, `--check` runs `npm view @agent-ix/quire-cli version` and reports
    the latest version without installing
- id: FR-016-AC-4
  path: spec/functional/FR-016-update-subcommand.md
  role: examined
  excerpt: A `--registry <URL>` override for the scoped package is passed as `--@agent-ix:registry=<URL>`
    (scope form), not a bare `--registry`; with no override no registry flag is added
- id: FR-016-AC-5
  path: spec/functional/FR-016-update-subcommand.md
  role: examined
  excerpt: '`update` performs no version comparison against the running binary and shells out
    to npm/cargo for idempotency (no cross-scheme version diff in `src/`)'
- id: FR-016-AC-6
  path: spec/functional/FR-016-update-subcommand.md
  role: examined
  excerpt: '`commands/update.rs` uses the shared `ix-cli-kit::self_update` engine, supplies
    quire''s coordinates through `SelfUpdateConfig`, and contains no parser, renderer, or validator
    logic; quire carries no local copy of the engine'
- id: FR-016-AC-7
  path: spec/functional/FR-016-update-subcommand.md
  role: examined
  excerpt: A failing `npm`/`cargo` invocation, or an unreachable registry, exits **4**
- id: FR-017-AC-1
  path: spec/functional/FR-017-coverage-subcommand.md
  role: examined
  excerpt: "`quire coverage --scope <DIR> --module $M` over a repository whose model matches\
    \ rows exits **0** and renders the human **census** \u2014 the bundle total and the per-document\
    \ counts \u2014 on **stdout**, uncolorized; findings stay on stderr. `--json` and `--format\
    \ tsv` are unaffected: the census is emitted only in human mode, so a machine payload is\
    \ still the only thing on stdout in those modes (amended CR-012)"
- id: FR-017-AC-2
  path: spec/functional/FR-017-coverage-subcommand.md
  role: examined
  excerpt: '`--json` emits the `CoverageReport` payload on stdout, and two runs over identical
    inputs are byte-identical'
- id: FR-017-AC-3
  path: spec/functional/FR-017-coverage-subcommand.md
  role: examined
  excerpt: 'The document root is `<DIR>/spec`: a perfectly typed matrix at the repository root
    or under `plan/` mints nothing, and repo-root `README.md`/`CHANGELOG.md` are never read
    as documents'
- id: FR-017-AC-4
  path: spec/functional/FR-017-coverage-subcommand.md
  role: examined
  excerpt: 'The code walk excludes the document root: a trace-tagged source file under `<DIR>/spec`
    contributes no symbol, while the same file outside it does'
- id: FR-017-AC-5
  path: spec/functional/FR-017-coverage-subcommand.md
  role: examined
  excerpt: A `--scope` holding no `spec/` exits non-zero with a diagnostic naming the missing
    document root by path, carrying machine `kind` `MissingDocumentRoot` under `--diagnostics-format
    json`
- id: FR-017-AC-6
  path: spec/functional/FR-017-coverage-subcommand.md
  role: examined
  excerpt: With no `traceability:` model in scope the command exits non-zero naming the missing
    declaration, and computes nothing
- id: FR-017-AC-7
  path: spec/functional/FR-017-coverage-subcommand.md
  role: examined
  excerpt: '`--strict` exits 1 when a row is unbacked or a status is contradicted, and exits
    0 on the same repository without it'
- id: FR-017-AC-8
  path: spec/functional/FR-017-coverage-subcommand.md
  role: examined
  excerpt: A model matching zero rows fails `--strict` with a diagnostic distinct from the unbacked-rows
    one, rather than reporting full coverage
- id: FR-017-AC-9
  path: spec/functional/FR-017-coverage-subcommand.md
  role: examined
  excerpt: (thin boundary) reconciliation is delegated entirely to quire-rs (`compute_coverage`,
    `extract_tree_scoped`, `trace::bind`); the CLI resolves the two roots, applies path-safety,
    and renders ([StR-004](../stakeholder/StR-004-thin-boundary-over-quire-rs.md))
- id: FR-017-AC-10
  path: spec/functional/FR-017-coverage-subcommand.md
  role: examined
  excerpt: "A row whose status value the module's `traceability.status` classes as nothing is\
    \ reported on **both** surfaces \u2014 as an `undeclared_statuses` entry in `--json` and\
    \ as a rendered line in the default human census \u2014 carrying the authored value verbatim.\
    \ `--strict` does not gate on it: the exit code for a report whose only finding is an undeclared\
    \ status is the same with and without the flag (CR-083)"
- id: FR-017-AC-11
  path: spec/functional/FR-017-coverage-subcommand.md
  role: examined
  excerpt: 'A module-declared `traceability.source_exclude` glob reaches the source walk: a
    tagged file matching one contributes no symbol, a tagged file outside every glob still does,
    and a scope whose module declares none behaves exactly as before (CR-085)'
- id: FR-017-AC-12
  path: spec/functional/FR-017-coverage-subcommand.md
  role: examined
  excerpt: "Each human-census unbacked-row, status-lie and undeclared-status line leads with\
    \ the row's own id when the record carries one (the declaration names a `row_id_column`)\
    \ and keeps the reference kind visible in a bracketed trailer \u2014 `TC-123 (doc.md) has\
    \ no backing symbol [traces-to]` \u2014 so two rows in the same document render distinguishable\
    \ lines; a record without a row id renders the reference kind leading, exactly as before.\
    \ The `--json` payload is unchanged (#51)"
- id: FR-017-AC-13
  path: spec/functional/FR-017-coverage-subcommand.md
  role: examined
  excerpt: "`--severity coverage:<check>=<level>` (checks: `unbacked-row`, `status-lie`, `untracked-symbol`,\
    \ `undeclared-status`) rides the FR-048 severity machinery: entries layer over module `grammar_severity`,\
    \ and a malformed entry \u2014 or a `coverage:` entry naming a check outside the pack's\
    \ four, which FR-048's shape-only key validation would otherwise merge as a silent no-op\
    \ (#57) \u2014 is rejected before any document is read. `off` drops the kind's records from\
    \ **every** output surface (human, `--json`, `--format tsv`), announcing each non-empty\
    \ suppression on stderr with its count. **Totals semantics:** "
- id: FR-017-AC-14
  path: spec/functional/FR-017-coverage-subcommand.md
  role: examined
  excerpt: "`--format tsv` emits one tab-separated record per line on stdout: a header naming\
    \ the nine fixed columns (`kind id document reference status method line targets text`),\
    \ every record carrying every column (empty where the kind has no value), row id leading\
    \ the data cells, id lists flattened with `,`, tab/newline in free text replaced by spaces,\
    \ obligation `parameters` omitted. The `line` column carries the record's 1-based document\
    \ line where the engine provides one (quire-rs FR-050-AC-26) and stays empty where it does\
    \ not \u2014 the arrival the column was reserved for, needing no format change. Ord"
- id: FR-017-AC-15
  path: spec/functional/FR-017-coverage-subcommand.md
  role: examined
  excerpt: '`--json` honours the global `--pretty`: compact single-line by default ([FR-008](./FR-008-json-output-encoding.md)-AC-1),
    indented with the flag, the identical parsed value either way. Byte-identity across runs
    (AC-2) holds in both shapes (#53)'
- id: FR-017-AC-16
  path: spec/functional/FR-017-coverage-subcommand.md
  role: examined
  excerpt: "When a finding record carries the engine's 1-based `line` (quire-rs FR-050-AC-26),\
    \ the human line's parenthesized locus is the clickable `document:line` form \u2014 `TC-123\
    \ (spec/tests.md:9) has no backing symbol [traces-to]` \u2014 for unbacked-row, status-lie,\
    \ undeclared-status and no-symbol-row lines; a record without a line renders the bare document\
    \ exactly as before. The `--json` payload is unchanged (#51 item 3)"
- id: FR-017-AC-17
  path: spec/functional/FR-017-coverage-subcommand.md
  role: examined
  excerpt: "A `no_symbol_rows` record renders in the default human census like every other row-id-carrying\
    \ kind \u2014 row id leading, `document:line` locus, reference kind in the bracketed trailer,\
    \ naming the exempting test-type value: `TC-123 (doc.md:7) is verified by \u2026, which\
    \ mints no source symbol [traces-to]`, with the value rendered verbatim in backticks where\
    \ the ellipsis stands. It was JSON/TSV-only; the record explains an unbacked row the census\
    \ does print, and an explanation only the machine surface carries is one nobody reads (#51,\
    \ the CR-083 argument)"
- id: FR-017-AC-18
  path: spec/functional/FR-017-coverage-subcommand.md
  role: examined
  excerpt: "The subtraction a declared `source_exclude` makes is observable on the human surface:\
    \ a census line `N source file(s) excluded by source_exclude` renders when N > 0 and nothing\
    \ renders at zero, and every `SymbolExtraction` diagnostic \u2014 a refused glob list (quire-rs\
    \ FR-050-AC-25), an unreadable source file \u2014 reaches stderr instead of being computed\
    \ and dropped (#51, quire-rs #215)"
- id: FR-017-AC-19
  path: spec/functional/FR-017-coverage-subcommand.md
  role: examined
  excerpt: "The advisory report lists pass through `--json` unmodified: `shared_trace_ids` (quire-rs\
    \ FR-050-AC-23) carries every status-carrying row id bound by more than one distinct symbol,\
    \ and `vocabulary_coverage` (FR-059-AC-9) serializes through the same wholesale report encoding\
    \ \u2014 both absent when empty, preserving AC-2 byte-identity for conformant corpora. Neither\
    \ has a human rendering in this release; that is a deliberate deferral, not an omission\
    \ (#51 batch note)"
- id: FR-017-AC-20
  path: spec/functional/FR-017-coverage-subcommand.md
  role: examined
  excerpt: '`--module` is repeatable and the declared set is closed and ordered: `quire coverage
    --scope $R --module $A --module $B` reconciles against the union of the two modules'' `traceability:`
    models, in that order, and a module reachable only from `IX_FILAMENT_MODULES_PATH` or `~/.ix/filament/modules/`
    is not consulted. A module named once emits no `DuplicateModuleName`/`DuplicateArchetype`
    diagnostic even when a same-named copy is installed ambiently.'
- id: FR-017-AC-21
  path: spec/functional/FR-017-coverage-subcommand.md
  role: examined
  excerpt: "`quire coverage --help` states the resolution order for `--module` \u2014 that the\
    \ roots are used in the order given and replace ambient discovery rather than adding to\
    \ it \u2014 so a caller can tell an adding flag from a replacing one without running an\
    \ experiment."
- id: FR-017-AC-22
  path: spec/functional/FR-017-coverage-subcommand.md
  role: examined
  excerpt: A full report containing `status-column-matches-nothing` or `hollow-denominator`
    makes `--strict` exit 1 after emitting the structured report, including when severity projection
    hides findings; the same scope without strict remains report-only, and a repaired control
    succeeds.
- id: FR-017-AC-23
  path: spec/functional/FR-017-coverage-subcommand.md
  role: examined
  excerpt: When two selected reference tables use the global status header and an explicit per-reference
    override, the native CLI evaluates both under the declared vocabulary. With backed complete
    rows and both exact headers present, strict succeeds; removing either selected header retains
    its located `status-column-matches-nothing` diagnostic in JSON, reports with exit 0 by default,
    and exits 1 under strict without guessing or borrowing the other table's readable header.
- id: FR-018-AC-1
  path: spec/functional/FR-018-properties-subcommand.md
  role: examined
  excerpt: '`quire properties <DOC> --module $M` over a document with binding criteria exits
    **0** and renders a per-criterion census on **stderr**, leaving stdout for the `--json`
    payload'
- id: FR-018-AC-2
  path: spec/functional/FR-018-properties-subcommand.md
  role: examined
  excerpt: '`--json` emits a `{documents: [{document, archetype, criteria}]}` envelope with
    one record per binding criterion, each carrying `row_id`, `shape`, `property`, `extraction`
    and `extractable`, and two runs over identical inputs are byte-identical'
- id: FR-018-AC-3
  path: spec/functional/FR-018-properties-subcommand.md
  role: examined
  excerpt: A document whose archetype binds no criteria yields an empty record set and still
    exits **0**
- id: FR-018-AC-4
  path: spec/functional/FR-018-properties-subcommand.md
  role: examined
  excerpt: "An unclassifiable or non-extractable criterion is reported in the payload and never\
    \ changes the exit code \u2014 the command has no failure mode driven by classification"
- id: FR-018-AC-5
  path: spec/functional/FR-018-properties-subcommand.md
  role: examined
  excerpt: "Relative document paths resolve under `--scope` \u2014 **regardless of `--module`**,\
    \ which pins the module set and never moves document resolution to the process directory\
    \ (CR-011) \u2014 and a `..` or symlink-escape path is rejected by path-safety ([FR-005](./FR-005-path-safety.md))\
    \ before any load"
- id: FR-018-AC-7
  path: spec/functional/FR-018-properties-subcommand.md
  role: examined
  excerpt: The command passes each document's **scope-relative path** to the engine, so an obligation
    source's `exclude:` globs bind this payload exactly as they bind `coverage --json` (upstream
    FR-053-AC-14); stdin passes no path, having no location a glob could match
- id: FR-018-AC-6
  path: spec/functional/FR-018-properties-subcommand.md
  role: examined
  excerpt: (thin boundary) classification is delegated entirely to quire-rs; the CLI resolves
    paths, loads the module set, and renders ([StR-004](../stakeholder/StR-004-thin-boundary-over-quire-rs.md))
- id: FR-018-AC-8
  path: spec/functional/FR-018-properties-subcommand.md
  role: examined
  excerpt: "The emitted `--json` envelope validates against quire-rs's **published** `schemas/output/properties-v1.schema.json`,\
    \ read from the resolved dependency checkout rather than a vendored copy. The envelope is\
    \ assembled here and nowhere else \u2014 quire-rs publishes the schema but never constructs\
    \ one \u2014 so this is the only place the shape can be gated. The fixture must exercise\
    \ the obligation branch, so conformance is not asserted over an empty payload"
- id: FR-018-AC-9
  path: spec/functional/FR-018-properties-subcommand.md
  role: examined
  excerpt: "Against a module declaring **no obligation source**, every criterion record carries\
    \ `obligation: null` and the payload still conforms to the published schema \u2014 the shape\
    \ a corpus that has not adopted obligations sees"
- id: FR-018-AC-10
  path: spec/functional/FR-018-properties-subcommand.md
  role: examined
  excerpt: "`--criteria` renders one block per criterion on stdout after the census \u2014 the\
    \ row id, a `document:line` locus, the shape and extraction state, and each extraction span\
    \ that was decomposed. A span that was not extracted prints nothing rather than an empty\
    \ label. The default set is the actionable one (`extractable`, specific-shape); `--all`\
    \ includes `example` and `unclassified`, and `--all` without `--criteria` is rejected by\
    \ the parser (CR-012)"
- id: FR-019-AC-1
  path: spec/functional/FR-019-symbols-subcommand.md
  role: examined
  excerpt: '`quire symbols --scope <DIR>` exits **0** with no module supplied, rendering a per-language
    census on **stderr** and leaving stdout for the payload'
- id: FR-019-AC-2
  path: spec/functional/FR-019-symbols-subcommand.md
  role: examined
  excerpt: '`--json` emits one record per extracted symbol carrying `path`, `symbol`, `kind`,
    `language`, `line`, `leading_line`, `end_line`, `container`, `id`, `binds_trace_ids` and
    `carries_implements`; two runs over identical inputs are byte-identical'
- id: FR-019-AC-3
  path: spec/functional/FR-019-symbols-subcommand.md
  role: examined
  excerpt: "With `--module`, each record additionally carries the `trace_ids` it bound; **without**\
    \ one, the human surface states that binding was not asked rather than reporting zero \u2014\
    \ an unbound run and an untagged repository are different facts"
- id: FR-019-AC-4
  path: spec/functional/FR-019-symbols-subcommand.md
  role: examined
  excerpt: '`by_language` carries both `symbols` and `binding_kinds`, so a binding rate is never
    drawn over the population of every symbol'
- id: FR-019-AC-5
  path: spec/functional/FR-019-symbols-subcommand.md
  role: examined
  excerpt: 'Extraction diagnostics reach **stderr** in every output format: a file the extractor
    could not read is indistinguishable from a file with no declarations, and dropping it would
    silently shrink the table'
- id: FR-019-AC-6
  path: spec/functional/FR-019-symbols-subcommand.md
  role: examined
  excerpt: Module resolution is the resolution `coverage` performs, shared rather than restated,
    so the two commands cannot disagree about which module is in scope for one invocation
- id: FR-019-CON-1
  path: spec/functional/FR-019-symbols-subcommand.md
  role: examined
  excerpt: "The command performs no analysis of its own. It reports the extraction and, when\
    \ asked, the binding \u2014 both computed by the engine. A second scanner living in the\
    \ CLI would reintroduce exactly the divergence this command exists to end"
- id: FR-020-AC-1
  path: spec/functional/FR-020-assurance-export-subcommand.md
  role: examined
  excerpt: A pinned fixture containing artifacts, obligations, symbols, resolved and dangling
    corpus relations, `verifies` and `implements` bindings, locators, relation-kind capabilities,
    and `available`, `missing`, `not_applicable`, and `unknown` observations emits a complete
    `quire-assurance` v1 document that validates against `quire_rs::assurance::ASSURANCE_V1_SCHEMA`.
- id: FR-020-AC-2
  path: spec/functional/FR-020-assurance-export-subcommand.md
  role: examined
  excerpt: Two compact runs over identical fixture bytes and arguments produce byte-identical
    stdout; `--pretty` changes only whitespace and is independently byte-identical across runs.
- id: FR-020-AC-3
  path: spec/functional/FR-020-assurance-export-subcommand.md
  role: examined
  excerpt: A mismatched module name/version, missing or extra schema premise, malformed premise
    syntax, unnamed/unversioned module, or unsupported source revision exits non-zero with empty
    stdout and a diagnostic naming the refused premise.
- id: FR-020-AC-4
  path: spec/functional/FR-020-assurance-export-subcommand.md
  role: examined
  excerpt: A valid corpus with zero artifacts, obligations, symbols, or relations still emits
    the complete successful envelope and exits zero. A module without a `traceability:` model
    exports its static artifacts and symbols with empty obligations and no `verifies`/`implements`
    relations. A document the corpus walker cannot read remains a successful export with an
    `unknown` relation observation and non-empty reason as quire-rs FR-068 requires; a missing
    root, invalid module/source premise, or export-wide upstream error exits non-zero with empty
    stdout.
- id: FR-020-AC-5
  path: spec/functional/FR-020-assurance-export-subcommand.md
  role: examined
  excerpt: Module-loader and symbol-extraction diagnostics are emitted on stderr in human or
    JSON diagnostic form and never enter the assurance payload.
- id: FR-020-AC-6
  path: spec/functional/FR-020-assurance-export-subcommand.md
  role: examined
  excerpt: The command delegates construction to `build_assurance_export`, validation to `read_assurance_export`,
    corpus loading to `Spec`, extraction to `extract_tree_scoped`, and binding to `trace::bind`;
    a static boundary audit rejects a second graph, schema, or direct parser in the CLI.
- id: FR-020-AC-7
  path: spec/functional/FR-020-assurance-export-subcommand.md
  role: examined
  excerpt: The command opens no network socket and spawns no child process on success or refusal
    paths.
- id: FR-020-AC-8
  path: spec/functional/FR-020-assurance-export-subcommand.md
  role: examined
  excerpt: A checked-in golden JSON fixture validates against the upstream schema in Rust, is
    consumed from the exact same bytes by required Node/TypeScript and Python compatibility
    probes without normalization, and pins every v1 field and state token. A missing probe runtime
    fails the gate.
- id: FR-020-CON-2
  path: spec/functional/FR-020-assurance-export-subcommand.md
  role: examined
  excerpt: The command SHALL execute no test, proof, solver, consumer, package-manager, Git,
    or network command. It performs parsing and static source extraction only.
- id: FR-020-CON-3
  path: spec/functional/FR-020-assurance-export-subcommand.md
  role: examined
  excerpt: The CLI SHALL add no verdict, execution result, evidence freshness claim, generic
    evidence envelope, or tool-provenance field to the closed upstream payload.
- id: FR-020-CON-4
  path: spec/functional/FR-020-assurance-export-subcommand.md
  role: examined
  excerpt: Unknown or malformed module versions, schema premises, source revisions, and incomplete
    module loads SHALL fail closed before any stdout byte.
- id: FR-021-AC-1
  path: spec/functional/FR-021-clauses-subcommand.md
  role: examined
  excerpt: Both commands path-check and strictly load one explicit module and select authority,
    id, and version exactly; a missing version or invalid referenced set is a user error, never
    a nearest-version guess.
- id: FR-021-AC-2
  path: spec/functional/FR-021-clauses-subcommand.md
  role: examined
  excerpt: '`evaluate` accepts repeatable `KEY=VALUE` context dimensions and delegates applicability
    to the engine. Duplicate or malformed context is rejected.'
- id: FR-021-AC-3
  path: spec/functional/FR-021-clauses-subcommand.md
  role: examined
  excerpt: Evaluation preserves `binding`, `not_binding`, and `unresolved`; missing context
    remains unresolved with its reason.
- id: FR-021-AC-4
  path: spec/functional/FR-021-clauses-subcommand.md
  role: examined
  excerpt: '`diff` delegates to the engine and reports added, removed, and changed clauses between
    two exact versions of the same authority and set id.'
- id: FR-021-AC-5
  path: spec/functional/FR-021-clauses-subcommand.md
  role: examined
  excerpt: Human results and deterministic TSV are emitted on stdout; TSV escapes structural
    characters so one clause remains one row.
- id: FR-021-AC-6
  path: spec/functional/FR-021-clauses-subcommand.md
  role: examined
  excerpt: JSON carries tool provenance including the `clause_sets` capability and conforms
    to the pinned engine's published binding or diff schema.
- id: FR-021-AC-7
  path: spec/functional/FR-021-clauses-subcommand.md
  role: examined
  excerpt: The CLI contains only transport and rendering policy. Clause content, rights validation,
    applicability, and comparison remain engine/module concerns.
- id: FR-022-AC-1
  path: spec/functional/FR-022-npm-launcher-process-contract.md
  role: examined
  excerpt: The installed `quire` entry selects exactly the optional package for the current
    platform and architecture and resolves only its fixed `bin/quire[.exe]` member.
- id: FR-022-AC-2
  path: spec/functional/FR-022-npm-launcher-process-contract.md
  role: examined
  excerpt: The launcher forwards the ordered host argument strings without adding, removing,
    or rewriting an element, inherits all three standard streams, and returns the native process's
    normal exit status.
- id: FR-022-AC-3
  path: spec/functional/FR-022-npm-launcher-process-contract.md
  role: examined
  excerpt: The launcher mirrors a native process's terminating signal rather than translating
    it into an unrelated normal exit status.
- id: FR-022-AC-4
  path: spec/functional/FR-022-npm-launcher-process-contract.md
  role: examined
  excerpt: An unsupported platform/architecture exits 1, lists the generated supported set,
    names the source-build alternative, and launches nothing.
- id: FR-022-AC-5
  path: spec/functional/FR-022-npm-launcher-process-contract.md
  role: examined
  excerpt: A supported but absent optional package exits 1, names that exact package, explains
    how optional dependencies can be restored, and launches nothing.
- id: FR-022-AC-6
  path: spec/functional/FR-022-npm-launcher-process-contract.md
  role: examined
  excerpt: A chmod refusal is tolerated, while an actual spawn error exits 1 and reports `failed
    to launch binary` with the host error.
- id: FR-022-AC-7
  path: spec/functional/FR-022-npm-launcher-process-contract.md
  role: examined
  excerpt: The host file contains no version policy, package generation, binary-format validation,
    checksum logic, document handling, extraction, clause grammar, or source semantics.
- id: FR-023-AC-1
  path: spec/functional/FR-023-rust-npm-package-assembly.md
  role: examined
  excerpt: One typed Rust catalog is the production authority for Rust target, npm platform,
    npm architecture, package name, binary name, and expected binary format for all four targets.
- id: FR-023-AC-2
  path: spec/functional/FR-023-rust-npm-package-assembly.md
  role: examined
  excerpt: "Before changing the output tree, the tool requires one input binary for each artifact\
    \ target present \u2014 a non-empty subset of the catalog \u2014 and rejects an artifact\
    \ directory outside the catalog, an empty artifact set, or an incorrectly named target artifact."
- id: FR-023-AC-3
  path: spec/functional/FR-023-rust-npm-package-assembly.md
  role: examined
  excerpt: Before changing the output tree, the tool verifies that every input has the target's
    expected ELF machine, Mach-O CPU type, or PE/COFF machine and rejects a binary copied from
    another target.
- id: FR-023-AC-4
  path: spec/functional/FR-023-rust-npm-package-assembly.md
  role: examined
  excerpt: "A successful run replaces the output with exactly one scoped platform package per\
    \ artifact target present \u2014 never a package for a platform this run did not build \u2014\
    \ containing only `bin/quire[.exe]`, `LICENSE`, and `package.json`; non-Windows binaries\
    \ are mode `0755`."
- id: FR-023-AC-5
  path: spec/functional/FR-023-rust-npm-package-assembly.md
  role: examined
  excerpt: Every generated manifest declares the exact release version, `AGPL-3.0-or-later`,
    the matching singleton `os` and `cpu`, the public npm registry/access policy, and the repository
    metadata.
- id: FR-023-AC-6
  path: spec/functional/FR-023-rust-npm-package-assembly.md
  role: examined
  excerpt: "The launcher manifest receives the same release version and exactly one same-version\
    \ optional dependency for each artifact target present in this run \u2014 derived from the\
    \ artifacts, never carried over from a prior run's committed manifest \u2014 with no dependency\
    \ for a platform outside the catalog or absent from this run."
- id: FR-023-AC-7
  path: spec/functional/FR-023-rust-npm-package-assembly.md
  role: examined
  excerpt: Repeated generation from identical inputs is byte-identical, and any preflight refusal
    leaves the prior output and launcher files byte-identical.
- id: FR-023-AC-8
  path: spec/functional/FR-023-rust-npm-package-assembly.md
  role: examined
  excerpt: Local `npm pack` inspection and an offline clean consumer installation prove the
    generated file membership, dependency resolution, license presence, and native executable
    selection without publishing or network access.
- id: FR-023-AC-9
  path: spec/functional/FR-023-rust-npm-package-assembly.md
  role: examined
  excerpt: "After publish, a separate command fetches the launcher package as actually published\
    \ at the release version \u2014 from the supplied npm registry, never from the local working\
    \ tree \u2014 and asserts both that the launcher itself resolves there and that every platform\
    \ package its *published* `optionalDependencies` names also resolves at its declared version.\
    \ It fails on any package/version that does not resolve, on a registry it cannot reach,\
    \ and on an empty or absent `optionalDependencies` map (never a zero-iteration pass). It\
    \ queries the registry through the package's own npm scope (`--@agent-ix:regis"
- id: FR-024-AC-1
  path: spec/functional/FR-024-rust-release-version-contract.md
  role: examined
  excerpt: The version command accepts a complete SemVer value and rejects missing or invalid
    input before changing any file.
- id: FR-024-AC-2
  path: spec/functional/FR-024-rust-release-version-contract.md
  role: examined
  excerpt: A successful update changes only the root Cargo package version, launcher package
    version, and every catalog-derived launcher optional-dependency version to the identical
    supplied value.
- id: FR-024-AC-3
  path: spec/functional/FR-024-rust-release-version-contract.md
  role: examined
  excerpt: "The command validates the complete proposed Cargo and launcher state before its\
    \ first replacement; a malformed launcher manifest, an empty optional-dependency set, or\
    \ a dependency naming a platform outside the catalog leaves every governed file byte-identical.\
    \ A launcher declaring a non-empty proper subset of the catalog is a valid input, not a\
    \ mutation to reject \u2014 it is bumped like any other."
- id: FR-024-AC-4
  path: spec/functional/FR-024-rust-release-version-contract.md
  role: examined
  excerpt: The release verification command rejects disagreement among the requested version,
    root Cargo package, launcher package, launcher optional dependencies, and native binary
    `--version` output.
- id: FR-024-AC-5
  path: spec/functional/FR-024-rust-release-version-contract.md
  role: examined
  excerpt: The manual release workflow invokes the Rust tool for npm package generation and
    version assertions and contains no executable MJS, Perl, embedded Node mutation, or independently
    maintained npm target/version assertion.
- id: FR-024-AC-6
  path: spec/functional/FR-024-rust-release-version-contract.md
  role: examined
  excerpt: Publication remains opt-in on a manually dispatched workflow; local qualification
    neither dispatches the workflow nor publishes a GitHub, Cargo, or npm artifact.
- id: FR-025-AC-1
  path: spec/functional/FR-025-rust-qualification-tooling.md
  role: examined
  excerpt: A complete backed assurance report passes, while each missing, unbacked, wrongly
    promoted, unmatched, lying, or untracked case fails naming the condition
- id: FR-025-AC-2
  path: spec/functional/FR-025-rust-qualification-tooling.md
  role: examined
  excerpt: Missing fields, wrong field types, malformed JSON, empty benchmark results, and non-finite
    or negative thresholds fail before a success observation is emitted
- id: FR-025-AC-3
  path: spec/functional/FR-025-rust-qualification-tooling.md
  role: examined
  excerpt: Nearest-rank p95 is computed deterministically and values on either side of the configured
    threshold produce opposite verdicts
- id: FR-025-AC-4
  path: spec/functional/FR-025-rust-qualification-tooling.md
  role: examined
  excerpt: The thin-boundary gate accepts admitted dispatch and rejects forbidden direct, aliased,
    and nested-group references with a file and syntax locus
- id: FR-025-AC-5
  path: spec/functional/FR-025-rust-qualification-tooling.md
  role: examined
  excerpt: The unsafe gate accepts a documented block, rejects an undocumented block, honours
    only exact baseline loci, rejects stale exemptions, and changes the baseline only through
    the explicit update operation
- id: FR-025-AC-6
  path: spec/functional/FR-025-rust-qualification-tooling.md
  role: examined
  excerpt: Repository Make targets invoke Rust-owned gates without inline report parsing or
    source classification
- id: FR-025-AC-7
  path: spec/functional/FR-025-rust-qualification-tooling.md
  role: examined
  excerpt: No executable Python or shell qualification file remains, while the npm launcher
    and non-executable fixture/data formats remain
- id: FR-025-CON-1
  path: spec/functional/FR-025-rust-qualification-tooling.md
  role: examined
  excerpt: The Rust-source gates SHALL NOT classify syntax with grep, regular expressions alone,
    or a hand-written lexer
- id: FR-025-CON-2
  path: spec/functional/FR-025-rust-qualification-tooling.md
  role: examined
  excerpt: Make targets SHALL contain orchestration only
- id: FR-025-CON-3
  path: spec/functional/FR-025-rust-qualification-tooling.md
  role: examined
  excerpt: Make targets SHALL NOT interpret coverage, benchmark, or Rust-source content
- id: FR-025-CON-4
  path: spec/functional/FR-025-rust-qualification-tooling.md
  role: examined
  excerpt: '`npm/quire-cli/bin/quire.js` SHALL remain the admitted npm distribution launcher'
- id: FR-025-CON-5
  path: spec/functional/FR-025-rust-qualification-tooling.md
  role: examined
  excerpt: '`npm/quire-cli/bin/quire.js` SHALL NOT acquire qualification semantics'
- id: FR-026-AC-1
  path: spec/functional/FR-026-matrix-subcommand.md
  role: examined
  excerpt: '`quire matrix --scope <DIR> --module $M` over a repository whose module declares
    an `obligations:` source that derives a non-empty population exits 0 and renders every requirement''s
    criteria in every format'
- id: FR-026-AC-2
  path: spec/functional/FR-026-matrix-subcommand.md
  role: examined
  excerpt: '`matrix` shares `coverage`''s refusals verbatim: no `spec/` under `--scope` exits
    3 as `MissingDocumentRoot` (FR-017-AC-5); no `traceability:` model in scope exits 3 naming
    the missing declaration (FR-017-AC-6); `--module` is repeatable and closed exactly as FR-017-AC-20/21
    specify, and `quire matrix --help` states the identical `--module` resolution-order text
    FR-017-AC-21 requires of `coverage --help`'
- id: FR-026-AC-3
  path: spec/functional/FR-026-matrix-subcommand.md
  role: examined
  excerpt: Markdown requirement tables render in the engine's own `requirements[]` order (sorted
    by `document`); the CLI performs no additional sort, group or alphabetization
- id: FR-026-AC-4
  path: spec/functional/FR-026-matrix-subcommand.md
  role: examined
  excerpt: Each requirement renders as the exact heading `## <document>` with the requirement's
    own `document` value verbatim (no link, no backticks), followed by one table with header
    `Criterion | Statement | Binders | Status`, rows in the engine's own per-document criteria
    order
- id: FR-026-AC-5
  path: spec/functional/FR-026-matrix-subcommand.md
  role: examined
  excerpt: "A criterion's `Binders` cell lists each `(path, line, column)` binder as `path:line:column`,\
    \ separated by `, `, in the engine's own binder order \u2014 never collapsing two same-line\
    \ binders that differ only by column; a binder the engine marks `ignored` carries a trailing\
    \ ` (ignored)`; a criterion with zero binders renders the literal `(none)`"
- id: FR-026-AC-6
  path: spec/functional/FR-026-matrix-subcommand.md
  role: examined
  excerpt: "A `Statement` cell is the criterion's own `statement` field (never re-derived) with\
    \ every tab/newline/CR replaced by a space, truncated first \u2014 left intact at \u2264\
    80 Unicode scalar values or cut to 77 scalar values plus a trailing `...` above that \u2014\
    \ and only then pipe-escaped (`|`), so escaping never splits the cut and never counts toward\
    \ the 80-character budget; `--format json`/`--format tsv` carry the untruncated, unescaped\
    \ text for the identical criterion"
- id: FR-026-AC-7
  path: spec/functional/FR-026-matrix-subcommand.md
  role: examined
  excerpt: "`--format json` emits `{\"coverage_matrix\": <value>, \"engine\": {cli, engine,\
    \ capabilities}}` on stdout; the `coverage_matrix` key carries the engine's own value unmodified\
    \ \u2014 a bare array of requirement entries, never re-wrapped in an object \u2014 and is\
    \ absent when the engine omits it (zero population), never present-but-empty; two runs over\
    \ identical inputs are byte-identical"
- id: FR-026-AC-8
  path: spec/functional/FR-026-matrix-subcommand.md
  role: examined
  excerpt: "`--format tsv` emits header `document\\tcriterion\\tstatus\\tbinders\\tstatement`,\
    \ one untruncated, unescaped record per criterion in the JSON/markdown order, `binders`\
    \ comma-separated as `path:line:column` with the same `(ignored)` marker and empty (not\
    \ `(none)`) at zero; a zero-criteria report (\xA7B) emits the header alone"
- id: FR-026-AC-9
  path: spec/functional/FR-026-matrix-subcommand.md
  role: examined
  excerpt: "The zero-population state \u2014 `coverage_matrix` absent, whether the module declares\
    \ no `obligations:` source or declares one that derives nothing \u2014 renders in markdown\
    \ as the single line `No obligations matched this scope.` (no heading, no table) and in\
    \ TSV as the header alone; `--format json` omits the key, as the engine does"
- id: FR-026-AC-10
  path: spec/functional/FR-026-matrix-subcommand.md
  role: examined
  excerpt: "`--strict`'s pass condition requires **at least one** criterion, all `tagged`/`method-without-symbol`:\
    \ it exits 1 on the zero-population state (\xA7B) exactly as it exits 1 on an `untagged`/`tagged-by-ignored-test`\
    \ criterion \u2014 a report measuring nothing must not exit 0 (FR-050-AC-14/CR-035 argument,\
    \ applied to the derived-obligation population)"
- id: FR-026-AC-11
  path: spec/functional/FR-026-matrix-subcommand.md
  role: examined
  excerpt: "A criterion computing `method-without-symbol` never fails `--strict` on its own,\
    \ regardless of its binder count, and renders in every format exactly like any other criterion\
    \ \u2014 its own `Status`/`status` cell/field states `method-without-symbol` verbatim"
- id: FR-026-AC-12
  path: spec/functional/FR-026-matrix-subcommand.md
  role: examined
  excerpt: '`matrix` does not accept `--severity` (an attempt is an ordinary clap argv error,
    exit 3, FR-007-AC-5) and runs none of the `unbacked-row`/`status-lie`/`untracked-symbol`/`undeclared-status`
    checks; a module''s `grammar_severity` promotion of any of those checks has no effect on
    `matrix`''s exit code, because `coverage_matrix` carries none of their record kinds. `matrix`
    also does not inherit `coverage`''s FR-017-AC-22 unread-measurement gate (`status-column-matches-nothing`/`hollow-denominator`):
    `coverage_matrix` reads no status column, so there is no unread-column state for it to gate
    on'
- id: FR-026-AC-13
  path: spec/functional/FR-026-matrix-subcommand.md
  role: examined
  excerpt: "`matrix` uses only the FR-007 exit codes \u2014 0, 1, 2, 3, 4 \u2014 across every\
    \ case above, including an unrecognized `--severity` flag (exit 3); the shared taxonomy\
    \ is authoritative"
- id: FR-026-AC-14
  path: spec/functional/FR-026-matrix-subcommand.md
  role: examined
  excerpt: "`matrix` never opens a file for writing: no flag names an output path, and running\
    \ it in every format against a fixture repository leaves every file anywhere under `<scope>`\
    \ \u2014 not only under `<scope>/spec` \u2014 byte-identical before and after the run"
- id: FR-027-AC-1
  path: spec/functional/FR-027-check-lock-subcommand.md
  role: examined
  excerpt: A lock with two or more distinct package names from one agent-ix repository at the
    same resolved commit passes; empty and registry/path-only locks also pass.
- id: FR-027-AC-2
  path: spec/functional/FR-027-check-lock-subcommand.md
  role: examined
  excerpt: Distinct package names from one canonical repository at different resolved commits
    fail with exit 2 and diagnostics naming the lock, repository, all affected names, and revisions.
- id: FR-027-AC-3
  path: spec/functional/FR-027-check-lock-subcommand.md
  role: examined
  excerpt: Duplicate agent-ix Git package names fail even with identical sources or different
    repositories; legitimate repeated registry and non-agent-ix package names pass.
- id: FR-027-AC-4
  path: spec/functional/FR-027-check-lock-subcommand.md
  role: examined
  excerpt: 'Branch/tag/rev/no-query and terminal .git/slash variants normalize to one repository:
    aligned fragments pass and split fragments fail, including GitHub case and transport variations.'
- id: FR-027-AC-5
  path: spec/functional/FR-027-check-lock-subcommand.md
  role: examined
  excerpt: Missing, empty, shortened, and non-hex agent-ix resolved fragments, malformed URLs,
    invalid package field types, and malformed TOML fail as invalid input rather than clean
    results.
- id: FR-027-AC-6
  path: spec/functional/FR-027-check-lock-subcommand.md
  role: examined
  excerpt: Multiple independently aligned locks may use different revisions; any failing lock
    fails the invocation and its diagnostic names that lock. A missing or unreadable path cannot
    yield a successful report.
- id: FR-027-AC-7
  path: spec/functional/FR-027-check-lock-subcommand.md
  role: examined
  excerpt: Success and failure streams, global diagnostic formatting, missing arguments, stdin
    rejection, path traversal, and typed I/O outcomes conform to the existing CLI contract;
    repeated invocations produce identical observations.
- id: FR-027-AC-8
  path: spec/functional/FR-027-check-lock-subcommand.md
  role: examined
  excerpt: A split-revision fixture passes the original consumer nine-line AWK checker and fails
    this command. Qualification executes the existing original checker externally without adding
    an executable copy to quire-cli.
- id: FR-027-AC-9
  path: spec/functional/FR-027-check-lock-subcommand.md
  role: examined
  excerpt: Execution leaves all inputs unchanged and performs no network, Cargo, Git, or shell
    invocation.
- id: FR-027-AC-10
  path: spec/functional/FR-027-check-lock-subcommand.md
  role: examined
  excerpt: The normal quire-cli release/install route supplies this same command; installation
    and representative aligned/split invocation are qualified separately on each IR/CG/RT gate
    host, with unavailable hosts explicitly unqualified.
- id: FR-027-CON-1
  path: spec/functional/FR-027-check-lock-subcommand.md
  role: examined
  excerpt: Coherence logic SHALL have one Rust owner in quire-cli; consumer Makefiles invoke
    the released binary without copied executable checker logic.
- id: FR-027-CON-2
  path: spec/functional/FR-027-check-lock-subcommand.md
  role: examined
  excerpt: The command SHALL remain a developer gate without introducing runtime dependency
    edges into IR, CG, or RT.
- id: FR-027-CON-3
  path: spec/functional/FR-027-check-lock-subcommand.md
  role: examined
  excerpt: Local qualification SHALL NOT publish a release or claim remote-host installation
    from a local receipt.
- id: NFR-001-AC-1
  path: spec/non-functional/NFR-001-render-latency-budget.md
  role: examined
  excerpt: "(RETIRED) `make bench` runs `hyperfine --warmup 3 'target/release/quire render FR\
    \ --module $ISO --data ctx.json'` and reports p95 \u2264 50 ms."
- id: NFR-001-AC-2
  path: spec/non-functional/NFR-001-render-latency-budget.md
  role: examined
  excerpt: (RETIRED) CI runs the same hyperfine harness on each push; merge is blocked if p95
    regresses by > 20 % from the prior release.
- id: NFR-001-AC-3
  path: spec/non-functional/NFR-001-render-latency-budget.md
  role: examined
  excerpt: (RETIRED) Cargo release profile uses `lto = "thin"`, `codegen-units = 1`, matching
    `quire-rs/Cargo.toml`.
- id: NFR-001-M-1
  path: spec/non-functional/NFR-001-render-latency-budget.md
  role: examined
  excerpt: Render p50 latency (cold-start)
- id: NFR-001-M-2
  path: spec/non-functional/NFR-001-render-latency-budget.md
  role: examined
  excerpt: Render p95 latency (cold-start)
- id: NFR-001-M-3
  path: spec/non-functional/NFR-001-render-latency-budget.md
  role: examined
  excerpt: Render p99 latency (cold-start)
- id: NFR-002-AC-1
  path: spec/non-functional/NFR-002-static-binary.md
  role: examined
  excerpt: 'On Linux x86_64, `ldd target/release/quire` lists at most: `linux-vdso.so.1`, `libgcc_s.so.1`
    (if used), `libc.so.6`, `libpthread.so.0`, `libdl.so.2`, `libm.so.6`, `/lib64/ld-linux-x86-64.so.2`.
    No `libquire_rs.so`, no `libssl`, no other project-supplied `.so`'
- id: NFR-002-AC-2
  path: spec/non-functional/NFR-002-static-binary.md
  role: examined
  excerpt: '`cargo install --git https://github.com/agent-ix/quire-cli quire-cli` produces a
    working binary on a fresh Linux machine with only `rustc` installed'
- id: NFR-002-M-1
  path: spec/non-functional/NFR-002-static-binary.md
  role: examined
  excerpt: Project-supplied shared libraries linked
- id: NFR-002-M-2
  path: spec/non-functional/NFR-002-static-binary.md
  role: examined
  excerpt: '`cargo install` on a fresh Linux host (only `rustc`) yields a runnable binary'
- id: NFR-003-AC-1
  path: spec/non-functional/NFR-003-zero-unsafe.md
  role: examined
  excerpt: The native AST-backed gate reports zero undocumented `unsafe` blocks in first-party
    Rust source and tests
- id: NFR-003-AC-2
  path: spec/non-functional/NFR-003-zero-unsafe.md
  role: examined
  excerpt: Local qualification fails if an `unsafe` block lacks a nearby `// SAFETY:` comment
    and its exact locus is not in the reviewed baseline
- id: NFR-003-M-1
  path: spec/non-functional/NFR-003-zero-unsafe.md
  role: examined
  excerpt: Undocumented `unsafe` blocks in first-party Rust source and tests
- id: NFR-004-AC-1
  path: spec/non-functional/NFR-004-no-network.md
  role: examined
  excerpt: '`cargo deny check bans` rejects `reqwest`, `hyper`, `tonic`, `surf`, `ureq`'
- id: NFR-004-AC-2
  path: spec/non-functional/NFR-004-no-network.md
  role: examined
  excerpt: "An IT runs each subcommand under `strace -fe network` (or equivalent) on its happy\
    \ path (registry present \u2014 including scoped discovery that finds modules) and asserts\
    \ zero `socket()` calls in quire's own process and descendants. The scoped-`validate` empty-discovery\
    \ lazy-init (which spawns `quoin`) is the documented exception ([ADR-0001](../assets/adr/0001-validate-lazy-init-module-bootstrap.md))\
    \ and is out of this happy-path scope"
- id: NFR-004-AC-3
  path: spec/non-functional/NFR-004-no-network.md
  role: examined
  excerpt: '`Cargo.lock` audit (CI gate) lists no HTTP client crate'
- id: NFR-004-M-1
  path: spec/non-functional/NFR-004-no-network.md
  role: examined
  excerpt: HTTP/network client crates in dependency tree
- id: NFR-004-M-2
  path: spec/non-functional/NFR-004-no-network.md
  role: examined
  excerpt: '`socket()` calls observed across all subcommands'
- id: NFR-005-AC-1
  path: spec/non-functional/NFR-005-diagnostic-format.md
  role: examined
  excerpt: Every stderr emission in `src/` is produced by calling `quire-rs::format_violation`
    or `serde_json::to_string(&Diagnostic { ... })`
- id: NFR-005-AC-2
  path: spec/non-functional/NFR-005-diagnostic-format.md
  role: examined
  excerpt: An IT collects stderr from each known failure mode and confirms parseability as `Diagnostic`
    JSON when `--diagnostics-format=json` is set
- id: NFR-005-M-1
  path: spec/non-functional/NFR-005-diagnostic-format.md
  role: examined
  excerpt: stderr emissions in `src/` not produced via `quire-rs::Diagnostic`
- id: NFR-005-M-2
  path: spec/non-functional/NFR-005-diagnostic-format.md
  role: examined
  excerpt: Known failure modes whose JSON stderr parses as `Diagnostic`
- id: NFR-006-AC-1
  path: spec/non-functional/NFR-006-cli-stability.md
  role: examined
  excerpt: A CHANGELOG entry documents every flag change, subcommand change, or exit-code change
    with the corresponding SemVer label
- id: NFR-006-AC-2
  path: spec/non-functional/NFR-006-cli-stability.md
  role: examined
  excerpt: A snapshot test pins `quire --help` output; updates require explicit reviewer approval
- id: NFR-006-M-1
  path: spec/non-functional/NFR-006-cli-stability.md
  role: examined
  excerpt: Surface changes (flag/subcommand/exit-code) released without a SemVer-labelled CHANGELOG
    entry
- id: NFR-006-M-2
  path: spec/non-functional/NFR-006-cli-stability.md
  role: examined
  excerpt: Unreviewed `quire --help` snapshot drift merged
- id: NFR-008-AC-1
  path: spec/non-functional/NFR-008-native-npm-distribution-tooling.md
  role: examined
  excerpt: The only repository-owned JavaScript executed in npm distribution is `npm/quire-cli/bin/quire.js`,
    and it remains within every ADR-0002 boundary.
- id: NFR-008-AC-2
  path: spec/non-functional/NFR-008-native-npm-distribution-tooling.md
  role: examined
  excerpt: All distribution test or audit verdicts are asserted by Rust tests carrying `ix-trace-rs`
    links to the owning acceptance criteria.
- id: NFR-008-AC-3
  path: spec/non-functional/NFR-008-native-npm-distribution-tooling.md
  role: examined
  excerpt: The distribution tool and launcher contain no document, extraction, clause, profile,
    grammar, temporal, protocol, or source-semantic implementation and the Rust tool has no
    `quire-rs` dependency.
- id: NFR-008-AC-4
  path: spec/non-functional/NFR-008-native-npm-distribution-tooling.md
  role: examined
  excerpt: Local qualification uses `CARGO_BUILD_JOBS=2` and no hosted-CI dispatch or publication.
- id: NFR-008-AC-5
  path: spec/non-functional/NFR-008-native-npm-distribution-tooling.md
  role: examined
  excerpt: The retained Node/npm qualification reliance is limited to packing, clean offline
    installation, dependency resolution, and exercising the launcher process seam; the Rust
    tests decide every result.
- id: NFR-008-AC-6
  path: spec/non-functional/NFR-008-native-npm-distribution-tooling.md
  role: examined
  excerpt: Rust distribution dependencies pass the repository's license/source policy, the launcher
    imports only Node built-ins, and every emitted npm package declares and carries the repository's
    `AGPL-3.0-or-later` license.
- id: NFR-008-M-1
  path: spec/non-functional/NFR-008-native-npm-distribution-tooling.md
  role: examined
  excerpt: First-party npm decision/generator/assertion paths outside Rust
- id: NFR-008-M-2
  path: spec/non-functional/NFR-008-native-npm-distribution-tooling.md
  role: examined
  excerpt: Foreign production files beyond the approved Node launcher
- id: NFR-008-M-3
  path: spec/non-functional/NFR-008-native-npm-distribution-tooling.md
  role: examined
  excerpt: Unreviewed target catalogs
- id: NFR-008-M-4
  path: spec/non-functional/NFR-008-native-npm-distribution-tooling.md
  role: examined
  excerpt: Distribution dependencies on Quire parsing/extraction/profile crates
- id: NFR-009-M-1
  path: spec/non-functional/NFR-009-native-qualification-boundary.md
  role: examined
  excerpt: Executable Python or shell qualification files outside fixture data
- id: NFR-009-M-2
  path: spec/non-functional/NFR-009-native-qualification-boundary.md
  role: examined
  excerpt: Shell processes spawned by Rust qualification tests
- id: NFR-009-M-3
  path: spec/non-functional/NFR-009-native-qualification-boundary.md
  role: examined
  excerpt: Implemented FR-025 matrix rows lacking canonical `ix-trace-rs` bindings
- id: NFR-009-M-4
  path: spec/non-functional/NFR-009-native-qualification-boundary.md
  role: examined
  excerpt: Hosted CI runs dispatched by this slice
```

## Coverage

The maintained TOML and URL libraries own parsing; this module owns only the explicitly scoped Git coherence policy. Native command, shared typed failures, path guard, shared diagnostic renderer, sorted repository/package/revision sets, nine integration tests, two data fixtures and help snapshot were examined. All ten changed files were inspected; workflows are unchanged. No copied checker, catalog, checksum ledger, bypass, new unsafe, panic path, async/lock surface or duplicated helper was found. Source inventory of src/, tests/ and private tools maps existing CLI commands to FR-002..FR-027 and shared I/O/path/numeric/distribution requirements; stub scan finds only deliberate test-fixture TODO strings. This inventory is not a new semantic audit of every existing implementation.

Actual focused receipts: format, targeted locked Clippy, nine cli_check_lock tests and one help test pass at 0bd9457. Source/tests/manifests/lock bytes at the frozen candidate are identical to that receipt head (only README changed). Scope actual MemoryMax 16 GiB, MemorySwapMax 0; focused peak 411299840 bytes; zero memory events. Repository-wide aggregate remains the root lead's later gate, not asserted here.

Actual current-head external controls: original IR/CG/RT AWK checkers each return 0 on the same retained split input; native candidate returns 2 on split and 0 on aligned. Input files, originals and production source/lock remained byte-identical. Receipt external-controls-qualified.json succeeds at frozen head, peak 15753216 bytes. No checker code was copied into quire-cli.

Exact-head normal Git Cargo release installation PASS is now verified: install-titan-final-head.log/.json show resolved candidate 2005e457, actual 16 GiB/Swap0, 41.636 seconds, peak 630816768 bytes, no memory events. Cargo .crates2.json independently records that same full revision and release profile. Actual installed PATH invocation returns aligned 0 and split 2; installed traces each show exactly one execve and zero internet sockets with unchanged inputs. Only Titan candidate is qualified; release publication, remote gate hosts and IR-641 consumer adoption remain explicitly unqualified. The original workspace-ambiguous install failure and later stale branch-cache 0bd9457 receipt remain distinct preserved results, neither presented as final-head installation. README explicitly selects the existing quire-cli package; no new installer or product revision pin is introduced.

No AssuranceProfile applies (no AssuranceProfile artifact under spec/). Spec-review and its sub-analyses are not run: this candidate has no spec/ or plan/ changes; FR-027/US-008 were reviewed and merged in prerequisite PR132. This pass reads those contracts as implementation inputs. Full unsupported old CLI coverage remains owned separately by PLAT-1177.

Reconciliation: quire matrix --scope /home/peter/dev/worktrees/ir736-check-lock --format json (CLI 0.36.2, engine 0.50.2); no run evidence read. Criteria: 199 tagged, 51 untagged, 21 method-without-symbol; total 271. No ignored-only binder reported. The 21 inspection/method-only rows are distinguished from the 51 untagged rows.

Plan completion: not assessed

Optional semantic review: skipped; ordinary code-review spec/code/test alignment is assessed in SR-14560.

## Bindings

```yaml
- test_id: aligned_empty_path_registry_and_other_owner_locks_pass
  ac_id: FR-027-AC-1
  trace: correct
- test_id: every_split_lock_and_affected_package_is_reported_in_order
  ac_id: FR-027-AC-2
  trace: correct
- test_id: aligned_empty_path_registry_and_other_owner_locks_pass
  ac_id: FR-027-AC-3
  trace: correct
- test_id: duplicate_agent_names_are_refused_even_with_identical_or_different_repositories
  ac_id: FR-027-AC-3
  trace: correct
- test_id: transport_case_query_and_terminal_variants_share_repository_identity
  ac_id: FR-027-AC-4
  trace: correct
- test_id: malformed_fragments_urls_toml_and_typed_package_fields_are_invalid
  ac_id: FR-027-AC-5
  trace: correct
- test_id: every_split_lock_and_affected_package_is_reported_in_order
  ac_id: FR-027-AC-6
  trace: correct
- test_id: independent_locks_are_not_compared_and_fatal_paths_never_report_success
  ac_id: FR-027-AC-6
  trace: correct
- test_id: input_permission_refusal_is_distinct_from_output_failure
  ac_id: FR-027-AC-6
  trace: correct
- test_id: every_split_lock_and_affected_package_is_reported_in_order
  ac_id: FR-027-AC-7
  trace: correct
- test_id: independent_locks_are_not_compared_and_fatal_paths_never_report_success
  ac_id: FR-027-AC-7
  trace: correct
- test_id: global_format_color_and_pretty_use_existing_stream_contract
  ac_id: FR-027-AC-7
  trace: correct
- test_id: input_permission_refusal_is_distinct_from_output_failure
  ac_id: FR-027-AC-7
  trace: correct
- test_id: gate_leaves_inputs_unchanged_and_opens_no_network_or_child_process
  ac_id: FR-027-AC-9
  trace: correct
```

## Dispositions

Round 1: unchanged candidate 2005e457; no product fix was requested or landed. Independently inspected tracker handoffs and raw external demonstration/install/trace receipts. The original finding text is unchanged. All 51 outcomes are explicit: 49 deferred to existing PLAT-1177 and two accepted-no-change by the code lead. The repository verdict remains FAIL and the computed matrix remains 199 tagged / 51 untagged / 21 method-without-symbol.

| FND | outcome | reason |
| --- | --- | --- |
| FND-001 | deferred | Existing owner PLAT-1177 retains the inherited FR-001-AC-1 row, explicitly reaffirmed by owner-ticket comment 98d39fad-9510-4d8b-8413-0fbdf4d79ca8. Triage and meaningful evidence repair remain that ticket's scope; this PR introduces no change to that criterion or binding and does not claim it fixed. |
| FND-002 | deferred | Existing owner PLAT-1177 retains the inherited FR-001-AC-2 row, explicitly reaffirmed by owner-ticket comment 98d39fad-9510-4d8b-8413-0fbdf4d79ca8. Triage and meaningful evidence repair remain that ticket's scope; this PR introduces no change to that criterion or binding and does not claim it fixed. |
| FND-003 | deferred | Existing owner PLAT-1177 retains the inherited FR-001-AC-3 row, explicitly reaffirmed by owner-ticket comment 98d39fad-9510-4d8b-8413-0fbdf4d79ca8. Triage and meaningful evidence repair remain that ticket's scope; this PR introduces no change to that criterion or binding and does not claim it fixed. |
| FND-004 | deferred | Existing owner PLAT-1177 retains the inherited FR-001-AC-4 row, explicitly reaffirmed by owner-ticket comment 98d39fad-9510-4d8b-8413-0fbdf4d79ca8. Triage and meaningful evidence repair remain that ticket's scope; this PR introduces no change to that criterion or binding and does not claim it fixed. |
| FND-005 | deferred | Existing owner PLAT-1177 retains the inherited FR-001-AC-5 row, explicitly reaffirmed by owner-ticket comment 98d39fad-9510-4d8b-8413-0fbdf4d79ca8. Triage and meaningful evidence repair remain that ticket's scope; this PR introduces no change to that criterion or binding and does not claim it fixed. |
| FND-006 | deferred | Existing owner PLAT-1177 retains the inherited FR-001-AC-6 row, explicitly reaffirmed by owner-ticket comment 98d39fad-9510-4d8b-8413-0fbdf4d79ca8. Triage and meaningful evidence repair remain that ticket's scope; this PR introduces no change to that criterion or binding and does not claim it fixed. |
| FND-007 | deferred | Existing owner PLAT-1177 retains the inherited FR-003-AC-3 row, explicitly reaffirmed by owner-ticket comment 98d39fad-9510-4d8b-8413-0fbdf4d79ca8. Triage and meaningful evidence repair remain that ticket's scope; this PR introduces no change to that criterion or binding and does not claim it fixed. |
| FND-008 | deferred | Existing owner PLAT-1177 retains the inherited FR-004-AC-23 row, explicitly reaffirmed by owner-ticket comment 98d39fad-9510-4d8b-8413-0fbdf4d79ca8. Triage and meaningful evidence repair remain that ticket's scope; this PR introduces no change to that criterion or binding and does not claim it fixed. |
| FND-009 | deferred | Existing owner PLAT-1177 retains the inherited FR-006-AC-4 row, explicitly reaffirmed by owner-ticket comment 98d39fad-9510-4d8b-8413-0fbdf4d79ca8. Triage and meaningful evidence repair remain that ticket's scope; this PR introduces no change to that criterion or binding and does not claim it fixed. |
| FND-010 | deferred | Existing owner PLAT-1177 retains the inherited FR-006-AC-5 row, explicitly reaffirmed by owner-ticket comment 98d39fad-9510-4d8b-8413-0fbdf4d79ca8. Triage and meaningful evidence repair remain that ticket's scope; this PR introduces no change to that criterion or binding and does not claim it fixed. |
| FND-011 | deferred | Existing owner PLAT-1177 retains the inherited FR-016-AC-3 row, explicitly reaffirmed by owner-ticket comment 98d39fad-9510-4d8b-8413-0fbdf4d79ca8. Triage and meaningful evidence repair remain that ticket's scope; this PR introduces no change to that criterion or binding and does not claim it fixed. |
| FND-012 | deferred | Existing owner PLAT-1177 retains the inherited FR-016-AC-4 row, explicitly reaffirmed by owner-ticket comment 98d39fad-9510-4d8b-8413-0fbdf4d79ca8. Triage and meaningful evidence repair remain that ticket's scope; this PR introduces no change to that criterion or binding and does not claim it fixed. |
| FND-013 | deferred | Existing owner PLAT-1177 retains the inherited FR-016-AC-7 row, explicitly reaffirmed by owner-ticket comment 98d39fad-9510-4d8b-8413-0fbdf4d79ca8. Triage and meaningful evidence repair remain that ticket's scope; this PR introduces no change to that criterion or binding and does not claim it fixed. |
| FND-014 | deferred | Existing owner PLAT-1177 retains the inherited FR-017-AC-23 row, explicitly reaffirmed by owner-ticket comment 98d39fad-9510-4d8b-8413-0fbdf4d79ca8. Triage and meaningful evidence repair remain that ticket's scope; this PR introduces no change to that criterion or binding and does not claim it fixed. |
| FND-015 | deferred | Existing owner PLAT-1177 retains the inherited FR-018-AC-10 row, explicitly reaffirmed by owner-ticket comment 98d39fad-9510-4d8b-8413-0fbdf4d79ca8. Triage and meaningful evidence repair remain that ticket's scope; this PR introduces no change to that criterion or binding and does not claim it fixed. |
| FND-016 | deferred | Existing owner PLAT-1177 retains the inherited FR-019-AC-1 row, explicitly reaffirmed by owner-ticket comment 98d39fad-9510-4d8b-8413-0fbdf4d79ca8. Triage and meaningful evidence repair remain that ticket's scope; this PR introduces no change to that criterion or binding and does not claim it fixed. |
| FND-017 | deferred | Existing owner PLAT-1177 retains the inherited FR-019-AC-2 row, explicitly reaffirmed by owner-ticket comment 98d39fad-9510-4d8b-8413-0fbdf4d79ca8. Triage and meaningful evidence repair remain that ticket's scope; this PR introduces no change to that criterion or binding and does not claim it fixed. |
| FND-018 | deferred | Existing owner PLAT-1177 retains the inherited FR-019-AC-3 row, explicitly reaffirmed by owner-ticket comment 98d39fad-9510-4d8b-8413-0fbdf4d79ca8. Triage and meaningful evidence repair remain that ticket's scope; this PR introduces no change to that criterion or binding and does not claim it fixed. |
| FND-019 | deferred | Existing owner PLAT-1177 retains the inherited FR-019-AC-4 row, explicitly reaffirmed by owner-ticket comment 98d39fad-9510-4d8b-8413-0fbdf4d79ca8. Triage and meaningful evidence repair remain that ticket's scope; this PR introduces no change to that criterion or binding and does not claim it fixed. |
| FND-020 | deferred | Existing owner PLAT-1177 retains the inherited FR-019-AC-5 row, explicitly reaffirmed by owner-ticket comment 98d39fad-9510-4d8b-8413-0fbdf4d79ca8. Triage and meaningful evidence repair remain that ticket's scope; this PR introduces no change to that criterion or binding and does not claim it fixed. |
| FND-021 | deferred | Existing owner PLAT-1177 retains the inherited FR-019-CON-1 row, explicitly reaffirmed by owner-ticket comment 98d39fad-9510-4d8b-8413-0fbdf4d79ca8. Triage and meaningful evidence repair remain that ticket's scope; this PR introduces no change to that criterion or binding and does not claim it fixed. |
| FND-022 | deferred | Existing owner PLAT-1177 retains the inherited FR-020-CON-2 row, explicitly reaffirmed by owner-ticket comment 98d39fad-9510-4d8b-8413-0fbdf4d79ca8. Triage and meaningful evidence repair remain that ticket's scope; this PR introduces no change to that criterion or binding and does not claim it fixed. |
| FND-023 | deferred | Existing owner PLAT-1177 retains the inherited FR-020-CON-4 row, explicitly reaffirmed by owner-ticket comment 98d39fad-9510-4d8b-8413-0fbdf4d79ca8. Triage and meaningful evidence repair remain that ticket's scope; this PR introduces no change to that criterion or binding and does not claim it fixed. |
| FND-024 | deferred | Existing owner PLAT-1177 retains the inherited FR-021-AC-2 row, explicitly reaffirmed by owner-ticket comment 98d39fad-9510-4d8b-8413-0fbdf4d79ca8. Triage and meaningful evidence repair remain that ticket's scope; this PR introduces no change to that criterion or binding and does not claim it fixed. |
| FND-025 | deferred | Existing owner PLAT-1177 retains the inherited FR-021-AC-3 row, explicitly reaffirmed by owner-ticket comment 98d39fad-9510-4d8b-8413-0fbdf4d79ca8. Triage and meaningful evidence repair remain that ticket's scope; this PR introduces no change to that criterion or binding and does not claim it fixed. |
| FND-026 | deferred | Existing owner PLAT-1177 retains the inherited FR-021-AC-5 row, explicitly reaffirmed by owner-ticket comment 98d39fad-9510-4d8b-8413-0fbdf4d79ca8. Triage and meaningful evidence repair remain that ticket's scope; this PR introduces no change to that criterion or binding and does not claim it fixed. |
| FND-027 | deferred | Existing owner PLAT-1177 retains the inherited FR-021-AC-6 row, explicitly reaffirmed by owner-ticket comment 98d39fad-9510-4d8b-8413-0fbdf4d79ca8. Triage and meaningful evidence repair remain that ticket's scope; this PR introduces no change to that criterion or binding and does not claim it fixed. |
| FND-028 | accepted-no-change | Code-lead decision IR-736 comment d5200d62-0cab-42fd-8bb4-6a2ee8e35b58 accepts the remaining computed-symbol gap. Independent receipt inspection confirms all three original consumer AWK checkers exited 0 while the actual native command exited 2 on the same unchanged split input. The normative verification method is Demonstration and explicitly requires external execution without copying its checker. No fabricated test binding is added; the row remains untagged. |
| FND-029 | accepted-no-change | Code-lead decision IR-736 comment d5200d62-0cab-42fd-8bb4-6a2ee8e35b58 accepts the remaining computed-symbol gap. Independent normal Git Cargo release-install log, .crates2.json and PATH receipts identify exact candidate 2005e457 and aligned/split exit 0/2 on Titan, the actual local IR/CG/RT gate host. Installed traces each contain one execve and no internet sockets. This is candidate installation only; published releases and unavailable other hosts remain explicitly unqualified. The normative verification method remains Demonstration; the row remains untagged. |
| FND-030 | deferred | Existing owner PLAT-1177 retains the inherited NFR-001-AC-2 row, explicitly reaffirmed by owner-ticket comment 98d39fad-9510-4d8b-8413-0fbdf4d79ca8. Triage and meaningful evidence repair remain that ticket's scope; this PR introduces no change to that criterion or binding and does not claim it fixed. |
| FND-031 | deferred | Existing owner PLAT-1177 retains the inherited NFR-001-M-1 row, explicitly reaffirmed by owner-ticket comment 98d39fad-9510-4d8b-8413-0fbdf4d79ca8. Triage and meaningful evidence repair remain that ticket's scope; this PR introduces no change to that criterion or binding and does not claim it fixed. |
| FND-032 | deferred | Existing owner PLAT-1177 retains the inherited NFR-001-M-2 row, explicitly reaffirmed by owner-ticket comment 98d39fad-9510-4d8b-8413-0fbdf4d79ca8. Triage and meaningful evidence repair remain that ticket's scope; this PR introduces no change to that criterion or binding and does not claim it fixed. |
| FND-033 | deferred | Existing owner PLAT-1177 retains the inherited NFR-001-M-3 row, explicitly reaffirmed by owner-ticket comment 98d39fad-9510-4d8b-8413-0fbdf4d79ca8. Triage and meaningful evidence repair remain that ticket's scope; this PR introduces no change to that criterion or binding and does not claim it fixed. |
| FND-034 | deferred | Existing owner PLAT-1177 retains the inherited NFR-002-AC-2 row, explicitly reaffirmed by owner-ticket comment 98d39fad-9510-4d8b-8413-0fbdf4d79ca8. Triage and meaningful evidence repair remain that ticket's scope; this PR introduces no change to that criterion or binding and does not claim it fixed. |
| FND-035 | deferred | Existing owner PLAT-1177 retains the inherited NFR-002-M-1 row, explicitly reaffirmed by owner-ticket comment 98d39fad-9510-4d8b-8413-0fbdf4d79ca8. Triage and meaningful evidence repair remain that ticket's scope; this PR introduces no change to that criterion or binding and does not claim it fixed. |
| FND-036 | deferred | Existing owner PLAT-1177 retains the inherited NFR-002-M-2 row, explicitly reaffirmed by owner-ticket comment 98d39fad-9510-4d8b-8413-0fbdf4d79ca8. Triage and meaningful evidence repair remain that ticket's scope; this PR introduces no change to that criterion or binding and does not claim it fixed. |
| FND-037 | deferred | Existing owner PLAT-1177 retains the inherited NFR-003-M-1 row, explicitly reaffirmed by owner-ticket comment 98d39fad-9510-4d8b-8413-0fbdf4d79ca8. Triage and meaningful evidence repair remain that ticket's scope; this PR introduces no change to that criterion or binding and does not claim it fixed. |
| FND-038 | deferred | Existing owner PLAT-1177 retains the inherited NFR-004-M-1 row, explicitly reaffirmed by owner-ticket comment 98d39fad-9510-4d8b-8413-0fbdf4d79ca8. Triage and meaningful evidence repair remain that ticket's scope; this PR introduces no change to that criterion or binding and does not claim it fixed. |
| FND-039 | deferred | Existing owner PLAT-1177 retains the inherited NFR-004-M-2 row, explicitly reaffirmed by owner-ticket comment 98d39fad-9510-4d8b-8413-0fbdf4d79ca8. Triage and meaningful evidence repair remain that ticket's scope; this PR introduces no change to that criterion or binding and does not claim it fixed. |
| FND-040 | deferred | Existing owner PLAT-1177 retains the inherited NFR-005-M-1 row, explicitly reaffirmed by owner-ticket comment 98d39fad-9510-4d8b-8413-0fbdf4d79ca8. Triage and meaningful evidence repair remain that ticket's scope; this PR introduces no change to that criterion or binding and does not claim it fixed. |
| FND-041 | deferred | Existing owner PLAT-1177 retains the inherited NFR-005-M-2 row, explicitly reaffirmed by owner-ticket comment 98d39fad-9510-4d8b-8413-0fbdf4d79ca8. Triage and meaningful evidence repair remain that ticket's scope; this PR introduces no change to that criterion or binding and does not claim it fixed. |
| FND-042 | deferred | Existing owner PLAT-1177 retains the inherited NFR-006-M-1 row, explicitly reaffirmed by owner-ticket comment 98d39fad-9510-4d8b-8413-0fbdf4d79ca8. Triage and meaningful evidence repair remain that ticket's scope; this PR introduces no change to that criterion or binding and does not claim it fixed. |
| FND-043 | deferred | Existing owner PLAT-1177 retains the inherited NFR-006-M-2 row, explicitly reaffirmed by owner-ticket comment 98d39fad-9510-4d8b-8413-0fbdf4d79ca8. Triage and meaningful evidence repair remain that ticket's scope; this PR introduces no change to that criterion or binding and does not claim it fixed. |
| FND-044 | deferred | Existing owner PLAT-1177 retains the inherited NFR-008-M-1 row, explicitly reaffirmed by owner-ticket comment 98d39fad-9510-4d8b-8413-0fbdf4d79ca8. Triage and meaningful evidence repair remain that ticket's scope; this PR introduces no change to that criterion or binding and does not claim it fixed. |
| FND-045 | deferred | Existing owner PLAT-1177 retains the inherited NFR-008-M-2 row, explicitly reaffirmed by owner-ticket comment 98d39fad-9510-4d8b-8413-0fbdf4d79ca8. Triage and meaningful evidence repair remain that ticket's scope; this PR introduces no change to that criterion or binding and does not claim it fixed. |
| FND-046 | deferred | Existing owner PLAT-1177 retains the inherited NFR-008-M-3 row, explicitly reaffirmed by owner-ticket comment 98d39fad-9510-4d8b-8413-0fbdf4d79ca8. Triage and meaningful evidence repair remain that ticket's scope; this PR introduces no change to that criterion or binding and does not claim it fixed. |
| FND-047 | deferred | Existing owner PLAT-1177 retains the inherited NFR-008-M-4 row, explicitly reaffirmed by owner-ticket comment 98d39fad-9510-4d8b-8413-0fbdf4d79ca8. Triage and meaningful evidence repair remain that ticket's scope; this PR introduces no change to that criterion or binding and does not claim it fixed. |
| FND-048 | deferred | Existing owner PLAT-1177 retains the inherited NFR-009-M-1 row, explicitly reaffirmed by owner-ticket comment 98d39fad-9510-4d8b-8413-0fbdf4d79ca8. Triage and meaningful evidence repair remain that ticket's scope; this PR introduces no change to that criterion or binding and does not claim it fixed. |
| FND-049 | deferred | Existing owner PLAT-1177 retains the inherited NFR-009-M-2 row, explicitly reaffirmed by owner-ticket comment 98d39fad-9510-4d8b-8413-0fbdf4d79ca8. Triage and meaningful evidence repair remain that ticket's scope; this PR introduces no change to that criterion or binding and does not claim it fixed. |
| FND-050 | deferred | Existing owner PLAT-1177 retains the inherited NFR-009-M-3 row, explicitly reaffirmed by owner-ticket comment 98d39fad-9510-4d8b-8413-0fbdf4d79ca8. Triage and meaningful evidence repair remain that ticket's scope; this PR introduces no change to that criterion or binding and does not claim it fixed. |
| FND-051 | deferred | Existing owner PLAT-1177 retains the inherited NFR-009-M-4 row, explicitly reaffirmed by owner-ticket comment 98d39fad-9510-4d8b-8413-0fbdf4d79ca8. Triage and meaningful evidence repair remain that ticket's scope; this PR introduces no change to that criterion or binding and does not claim it fixed. |
