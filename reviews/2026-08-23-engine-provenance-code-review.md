---
id: SR-055
title: "Code review — engine provenance on --version and every payload (#68)"
type: SpecReview
analysis: code-review
scope: "build.rs, src/lockfile.rs, src/engine.rs, src/main.rs, src/commands/{coverage,properties,extract}.rs, tests/cli_provenance.rs, tests/cli_extract.rs, tests/output_contract.rs, spec/functional/FR-008, spec/tests.md; and agent-ix/quire-rs schemas/output/, spec/functional/FR-055, tests/output_contract.rs"
review_set: subset
---

# SR-055: Code review — engine provenance on `--version` and every payload (#68)

## Summary

Reviewed `agent-ix/quire-cli#69` and its sibling `agent-ix/quire-rs#281` — Wave 0
of EPIC `agent-ix/quire-rs#264` — under the `rust-review` lane, both repositories
in one pass because the change is one contract split across a seam. Every gate
was green on both sides at review time (`fmt --check`, `clippy -D warnings`, 25
CLI test targets, 873 engine tests, `cargo deny`, every static audit,
`make validate`), and **one Critical and five High defects survived them**.

The Critical one is the shape of every finding here: `coverage --json` was
silently re-sorting every key at every depth, output stayed byte-identical
*across runs*, so the determinism property everybody checks (FR-050-AC-7) still
held and no test in either repository noticed.

All 21 findings are fixed in `quire-cli@3c8bcb9` and `quire-rs@bf62bc4`. Each
fix was verified by mutation — reintroducing the defect and confirming the new
assertion fails — not by argument.

## Verdict

**FAIL** — six findings at `high` or above on first submission. All are now
fixed and mutation-verified; the verdict records the state reviewed, not the
state after remediation.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | high | `serde_json::Value` round-trip alphabetised every key at every depth in `coverage`/`extract` payloads | src/engine.rs:104 |
| FND-002 | high | `coverage --json` had no schema conformance test on either side of the seam, so the CLI's added key was ungated | tests/output_contract.rs:1 |
| FND-003 | high | `[[patch.unused]]` stanzas hijacked lockfile parsing — a patch Cargo discarded was reported as the linked engine | src/lockfile.rs:41 |
| FND-004 | high | A git dep with no tag/rev/branch fell back to the stale `0.33.0` manifest constant, restating the defect being fixed | src/lockfile.rs:52 |
| FND-005 | high | FR-055-AC-8 was absent from quire-rs's authoritative AC→TC audit table, dropping the repo below 100% coverage | quire-rs spec/tests.md:1462 |
| FND-006 | high | TC-860 gated a payload instance, not the contract: a declared `engine.schema_version` passed every assertion | quire-rs tests/output_contract.rs:449 |
| FND-007 | medium | TC-856 exists to carry every optional key and carried 8 of 13; four keys had drifted out across earlier changes | quire-rs tests/output_contract.rs:93 |
| FND-008 | medium | Two `quire-rs` lockfile stanzas took the first, and Cargo sorts by version — deterministically the lowest | src/lockfile.rs:41 |
| FND-009 | medium | `--version` assembled in two places; `engine::version_line()` had no production caller and nothing bound them | src/main.rs:21 |
| FND-010 | medium | IT-123 asserted only that the word "engine" appeared, so a `--version` naming no engine passed | tests/cli_provenance.rs:78 |
| FND-011 | medium | IT-127 never checked exit status — vacuous on any crash, since empty stdout carries no banned key | tests/cli_provenance.rs:229 |
| FND-012 | medium | IT-127 ran only `extract` while its matrix row claimed "no payload" carries a bare version key | spec/tests.md:IT-127 |
| FND-013 | medium | `CAPABILITIES` documented itself as "compile-checked by construction" with nothing enforcing it | src/engine.rs:35 |
| FND-014 | medium | Three doc comments claimed the inner shape was "emitted unmodified"; FND-001 made all three false | src/engine.rs:99 |
| FND-015 | medium | quire-rs's `engine` description disavowed "a constant" while describing the lockfile field that IS one | quire-rs schemas/output/coverage-v1.schema.json:68 |
| FND-016 | low | `git_pin` iterated params outer and keys inner, so `?branch=main&tag=v0.45.0` reported `main` | src/lockfile.rs:83 |
| FND-017 | low | `build.rs` assumed the lockfile sits beside the manifest; a workspace build shipped `engine unknown` | build.rs:17 |
| FND-018 | low | `assert_ne!(ENGINE_VERSION, CLI_VERSION)` encodes a coincidence that breaks when the CLI reaches 0.45.0 | src/engine.rs:141 |
| FND-019 | low | `assert!(line.contains(ENGINE_VERSION))` is unconditionally true when the version is empty | src/engine.rs:150 |
| FND-020 | low | `uniqueItems` on `capabilities` was un-gated: deleting it kept the suite green | quire-rs schemas/output/coverage-v1.schema.json:72 |
| FND-021 | low | The two hand-authored `EngineProvenance` definitions had no agreement test | quire-rs schemas/output/properties-v1.schema.json:97 |

## The Critical finding, in full

`attach` took the payload as a `serde_json::Value`, inserted `engine`, and
re-encoded. `serde_json::Map` is a `BTreeMap` unless the `preserve_order`
feature is enabled, and it is not — so the round-trip rewrote **every key at
every nesting level** into alphabetical order.

Measured, main binary against branch binary, identical input:

| | before | after |
|---|---|---|
| root | `unbacked_rows, status_lies, untracked_symbols, groups, criteria, metrics, totals` | alphabetical, `criteria` first |
| `metrics[0]` | `name, unit, method, shape, state, value, population, examined, matched` | alphabetical |

**Why nothing caught it.** The determinism property everyone checks is
*byte-identity across runs over identical input* (quire-rs FR-050-AC-7), and a
`BTreeMap` is perfectly deterministic — five consecutive runs produced one md5.
The damage is cross-version: every checked-in `coverage.json` in the ecosystem
would have shown a 100%-changed diff carrying no content change, and quire-cli
FR-008-AC-4 ("field order SHALL match the public Rust struct declaration order")
would have become quietly false with no AC or test updated.

Fixed with `#[serde(flatten)]`, which streams the inner value's fields in its own
`Serialize` order and appends `engine` after them.

**The regression guard needed fixing twice.** The first assertion compared the
key order of the *parsed* `Value` — which sorts, so it measured serde's map type
rather than the emitter, and failed against correct output. The assertions now
read the emitted bytes.

## Notes

- **The reviewers confirmed several things I had assumed rather than verified**,
  and those are recorded here as non-findings so a later reader does not re-open
  them: `include!`ing `src/lockfile.rs` into both `build.rs` and the library is
  sound (a build script is a separate crate, so no duplicate symbols);
  `strip_kv`/`git_pin` do not mis-fire on prefix collisions (`version_req`,
  `tagline`) because both require `=` immediately after the key;
  `skip_serializing_if` survived the round-trip; and `properties --json` was
  unaffected by FND-001 because it already went through `json!`.
- **FND-007 predates this change by four releases.** `implements` (CR-080),
  `binding_census` (CR-093), `metrics` (CR-094) and `suspicions` (CR-100) each
  arrived as an optional key, each landed green, and none was added to the
  payload whose stated purpose is to carry every optional key. A drift guard now
  derives the optional set from the schema and fails naming any key the payload
  omits, so the fifth occurrence is the last.
- **FND-005 and FND-006 are both gates that did not gate what they claimed**,
  which is the same failure class the EPIC exists to fix one layer up. Neither
  was catchable by any existing automation: `make validate` is grammar-only, and
  there is no AC→TC completeness check in the Makefile or CI.
