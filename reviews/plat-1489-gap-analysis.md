---
id: SR-20261112034127
title: "PLAT-1489 quire-cli PR173 gap-analysis"
type: SpecReview
analysis: gap-analysis
scope: "agent-ix/quire-cli PR173 head 01136b3; spec/functional/FR-014-validate-okf-bundle.md, spec/tests.md, src/commands/validate.rs, tests/cli_bundle_line.rs; engine reference quire-rs v0.58.0 spec/functional/FR-049, FR-069, src/corpus/declared_tables.rs"
review_set: subset
---

## Summary

Ticket: PLAT-1489. Planless gap analysis of FR-014-AC-12..15 against IT-258..IT-261 and `surface_bundle`. The engine contract it checks is quire-rs v0.58.0 FR-049-AC-19..22 and FR-069. Plan completion: not assessed.

Every new AC is tagged by at least one test. The computed coverage (`quire coverage --json` plus `quire-qualify assurance-traceability`) reports no unbacked FR-014 row and 20/20 required targets backed. Mutations that break line rendering are caught. A mutation that emits errors before warnings is not caught by any of the 53 focused tests.

The coder reported that a kind mismatch on a second table under the same section produced no finding. This was investigated and reproduced as a quire-rs engine behaviour (FND-003).

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | medium | FR-014-AC-13's clause "Record order (warnings first, then errors, engine order) is unchanged" has no test that can fail. Swapping the warning and error loops in `surface_bundle` passes all of cli_bundle_line, cli_okf, cli_validate, output_contract and cli_stream_contract. No IT fixture mixes warnings and errors in one bundle and asserts their order. | spec/functional/FR-014-validate-okf-bundle.md:119, src/commands/validate.rs:424-435 |
| FND-002 | low | The IT-259 row says the mismatch row sits "below another table of the document". The fixture's `links.md` has one table, so the case is not exercised. FR-049-AC-20's "row below another table or section reports its own line" is only half-tested. A table in a preceding section works: a probe with `## Other` holding a table before `## Links` reports `line 18` for the mismatch row. | spec/tests.md:307, tests/cli_bundle_line.rs:162-184 |
| FND-003 | high | Engine (quire-rs v0.58.0, also origin/main 03033ae), not a PR173 defect. A document-reference declaration scans only the FIRST table of each matched section. `declared_tables::table_in` calls `query::parse_table_with_lines(&sec.content)`, which returns one table. Every row of a second table under the same declared heading is silently skipped: no `dangling-trace-reference`, no `reference-kind-mismatch`, no `kind-column-matches-nothing`, no scan diagnostic. Repro: a `## Links` section with table 1 `K-1 north L-1`, a prose line, then table 2 with `K-2 north R-1` (a mismatch) and `K-3 north L-7` (dangling). `quire validate --okf` exits 0 with empty stderr. FR-049 says the engine SHALL "extract every referenced id from the declared cells". FR-076 by contrast reports a second table explicitly. This is a false-clean result for an assurance check and belongs to quire-rs. | quire-rs src/corpus/declared_tables.rs:894-898, quire-rs spec/functional/FR-049-verification-reference-integrity.md:33-37 |

## Examined scope

```yaml
scope:
- id: FR-014-AC-12
  path: spec/functional/FR-014-validate-okf-bundle.md
  role: examined
  excerpt: "A bundle finding whose BundleFinding.line is Some(N) ... is printed in the located form <path>: line <N>: <message> [<reason>], the same line <N>: prefix located document diagnostics use (FR-004-AC-2)"
- id: FR-014-AC-13
  path: spec/functional/FR-014-validate-okf-bundle.md
  role: examined
  excerpt: "With --diagnostics-format json, the record of a finding with Some(N) carries a numeric line field equal to N; a finding with line None carries no line key. Record order (warnings first, then errors, engine order) is unchanged"
- id: FR-014-AC-14
  path: spec/functional/FR-014-validate-okf-bundle.md
  role: examined
  excerpt: "A bridged document error under --okf --strict carries the line of its document-validation result: the line printed by single-document quire validate for the same document (FR-004-AC-2) is the line printed by the bundle form"
- id: FR-014-AC-15
  path: spec/functional/FR-014-validate-okf-bundle.md
  role: examined
  excerpt: "A finding with line None renders byte-identically to the pre-FR-014-AC-12 form <path>: <message> [<reason>] in human output and as the same JSON record, with no line key"
- id: FR-049-AC-19
  path: quire-rs spec/functional/FR-049-verification-reference-integrity.md
  role: context_only
  excerpt: "BundleFinding has an optional line; a dangling-trace-reference finding carries Some(line) ... every unresolved id of a multi-id cell carries the same row line"
- id: FR-049-AC-20
  path: quire-rs spec/functional/FR-049-verification-reference-integrity.md
  role: context_only
  excerpt: "A reference-kind-mismatch finding carries the line of the referencing row ... a row below another table or section of the same document reports its own line"
- id: FR-049-AC-22
  path: quire-rs spec/functional/FR-049-verification-reference-integrity.md
  role: context_only
  excerpt: "A document-level error or warning bridged into the bundle report keeps its line ... a semantic.record-invalid finding of FR-069 reaches a validate_bundle caller at the same line"
```

## Verdict

AC-12, AC-14 and AC-15 are backed by tests that fail on mutation. The IT-261 no-line form was compared byte for byte against an origin/main build on the same fixture: human and JSON, Okf and Strict, all identical. FND-001 should be fixed in-PR with an ordering test. FND-002 is low. FND-003 is an engine defect for quire-rs; it does not block PR173.

## Dispositions

| FND | outcome | reason |
| --- | --- | --- |
| FND-001 | fixed | b1ccfb3 adds FR-014-AC-16 and IT-266 (`it266_warnings_precede_errors`). Swapping the warning and error loops in `surface_bundle` fails IT-266 (verified). |
| FND-002 | fixed | b1ccfb3: the IT-263 row no longer claims "below another table of the document". |
| FND-003 | deferred | This is a quire-rs engine defect (`declared_tables::table_in` reads only the first table of a section), not a PR173 defect. quire-rs owns it; route it to quire-rs triage. It does not block PR173. |
