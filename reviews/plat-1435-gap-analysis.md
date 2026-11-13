---
id: SR-20261112234824
title: "PLAT-1435 quire-cli PR183 gap-analysis"
type: SpecReview
analysis: gap-analysis
scope: "agent-ix/quire-cli PR183 head 6da415a; spec/functional/FR-031-export-subcommand.md, spec/functional/FR-032-export-converter-execution.md, spec/functional/FR-033-export-errors.md, spec/non-functional/NFR-010-export-single-converter-single-path.md, src/commands/export*, tests/cli_export.rs"
review_set: subset
---

## Summary

Ticket: PLAT-1435. Plan completion: not assessed, because the analysis was planless.

The matrix source was `quire matrix --format json`, built from this branch. `quoin matrix --json` (quoin 0.30.1) reported 360 of 392 criteria repo-wide as `untagged`, including every criterion bound by a `#[trace]` marker in this PR. That is a tool discrepancy, so it was not used as evidence.

FR-031 to FR-033 criteria status:
- **Tagged:** FR-031-AC-2, 3, 6, 7 and 8; FR-032-AC-1, 2, 4, 5 and 6; FR-033-AC-1 to AC-10.
- **Tagged only by an ignored test:** FR-031-AC-1 and FR-032-AC-7.
- **Untagged:** FR-031-AC-4, 5, 9, 10 and 11, which are the JSON tables owned by PLAT-1603, and NFR-010-AC-3.
- **Inspection (method without symbol):** FR-032-AC-3, NFR-010-AC-1 and NFR-010-AC-2.

Ticket ACs:

| Ticket AC | Coverage |
| --- | --- |
| 1. PDF and DOCX from a fixture; headings, lists, tables and code survive | The test exists but is `#[ignore]`. It has not been executed here, because pandoc, typst and pdftotext are absent on this host. It also asserts no list item or table cell. See FND-001. |
| 2. Runs through `producer-execute`, no new execution path | Met. The only launch site is `producer-execute`. The requests validate against `producer-execute --schema request`. |
| 3. Missing converter is a clear error; converter failure exits non-zero with stderr | Met for missing pandoc, typst and engineering-assurance (exit 3) and for a non-zero pandoc with stderr (exit 4). |

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | medium | FR-031-AC-1 and FR-032-AC-7 (ticket AC 1) are bound only to `real_converters_keep_the_fixture_headings_and_code`. That test is `#[ignore]`, and no Makefile target runs it with `--ignored`, so nothing ever executes it and the "fails naming the missing tool" clause can never fire. The test also checks only headings and the code identifier. Ticket AC 1 requires lists and tables to survive, and the fixture has both (`first fixture item`, `FR-001-AC-1`), but neither is asserted. | tests/cli_export.rs:648-670 |
| FND-002 | medium | FR-033-AC-4 has a second half: "a fixture that outlives the request timeout exits 4 naming `timed_out`". No test covers it. `pandoc_failure_exits_4_with_its_stderr_and_no_typst_request` carries the FR-033-AC-4 tag but tests only the non-zero exit. A stand-in executor that returns `{"state":{"kind":"timed_out"}}` would cover it without waiting 120 s. | tests/cli_export.rs:338-357 |
| FND-003 | low | The FR-033-AC-8 unsupported-host branch is untested. `interpret` takes the non-Linux arm only through `cfg!(target_os = "linux")`, so on Linux CI the test covers only the generic containment_failure message. A host predicate passed as a parameter would make the message unit-testable. | src/commands/export/producer.rs:359-362, tests/cli_export.rs:410-437 |
| FND-004 | low | FR-032-AC-2 requires every request to "equal the field table above". The tests assert most fields, but they do not assert `containment`, `response`, the `budget` members other than `timeoutMillis`, or the `caller.version`. Schema validity does not pin those values. | tests/cli_export.rs:199-245 |
| FND-005 | low | FR-032 section B requires the request on stdin (`--request -`). The test rig's `engineering-assurance` wrapper ignores the argv that quire passes, and re-invokes the real executor with the request inline. A regression to another argv form would therefore pass every test. | tests/cli_export.rs:77-83 |
| FND-006 | low | NFR-010-AC-3 ("adding the export adds no network client crate", Test) is untagged. The existing NFR-004 dependency audit in tests/audit_no_network.rs covers the property but does not carry the NFR-010-AC-3 id. | tests/audit_no_network.rs |
| FND-007 | low | The spec cannot be satisfied as written. The FR-033 "Containment failure" row requires the diagnostic to name "the executor's reason". The published result schema gives `containment_failure` only `kind` (`additionalProperties: false`), so the result carries no reason, and the implementation correctly names only the state. The row should be amended to drop the reason, or to say where the reason comes from. | spec/functional/FR-033-export-errors.md |

## Examined scope

```yaml
scope:
- {id: FR-031-AC-1, path: spec/functional/FR-031-export-subcommand.md, role: examined, excerpt: "Exporting a committed fixture FR artifact containing headings, a list, a table and a code block to pdf and to docx writes both files, and the text extracted from each contains every fixture heading and the code block text."}
- {id: FR-031-AC-2, path: spec/functional/FR-031-export-subcommand.md, role: examined, excerpt: "fails validation against the --module set exits 3, writes no output file and starts no converter"}
- {id: FR-031-AC-3, path: spec/functional/FR-031-export-subcommand.md, role: examined, excerpt: "A JSON input without --kind, a Markdown input with --kind, and a JSON input with --module each exit 3 with a diagnostic naming the flag."}
- {id: FR-031-AC-6, path: spec/functional/FR-031-export-subcommand.md, role: examined, excerpt: "the converter input in/document.md equals the bytes of the committed body after the frontmatter"}
- {id: FR-031-AC-8, path: spec/functional/FR-031-export-subcommand.md, role: examined, excerpt: "After an export, the working directory, the input directory and the output directory hold no file other than the input and --output."}
- {id: FR-032-AC-1, path: spec/functional/FR-032-export-converter-execution.md, role: examined, excerpt: "--to docx produces exactly one producer-execute request, and --to pdf produces a pandoc request followed by a typst request"}
- {id: FR-032-AC-2, path: spec/functional/FR-032-export-converter-execution.md, role: examined, excerpt: "Every request used by the tests validates against the schema ... names a canonical producer.executable, and equals the field table above."}
- {id: FR-032-AC-4, path: spec/functional/FR-032-export-converter-execution.md, role: examined, excerpt: "When the pandoc request does not complete, no typst request is made."}
- {id: FR-032-AC-6, path: spec/functional/FR-032-export-converter-execution.md, role: examined, excerpt: "After success and after each failure of FR-033, the one temporary capability root no longer exists."}
- {id: FR-033-AC-1, path: spec/functional/FR-033-export-errors.md, role: examined, excerpt: "With pandoc absent, --to docx exits 3, names pandoc and --pandoc, and writes no file."}
- {id: FR-033-AC-4, path: spec/functional/FR-033-export-errors.md, role: examined, excerpt: "A fixture pandoc that exits non-zero with a stderr message makes the export exit 4 and print that message, and a fixture that outlives the request timeout exits 4 naming timed_out."}
- {id: FR-033-AC-7, path: spec/functional/FR-033-export-errors.md, role: examined, excerpt: "After a failed export over a pre-existing --output, that file's bytes are unchanged."}
- {id: FR-033-AC-8, path: spec/functional/FR-033-export-errors.md, role: examined, excerpt: "a stand-in result containment_failure on a non-Linux host makes the export exit 4 with the unsupported-host message"}
- {id: FR-033-AC-9, path: spec/functional/FR-033-export-errors.md, role: examined, excerpt: "An input without read permission exits 2; --output equal to the input exits 2 ...; a missing output directory and an --output that is a directory each exit 4; none starts a converter."}
- {id: FR-033-AC-10, path: spec/functional/FR-033-export-errors.md, role: examined, excerpt: "A fixture whose first heading reads --output=/tmp/x exports with that text in the document and no change to the arguments of any request."}
- {id: NFR-010-AC-3, path: spec/non-functional/NFR-010-export-single-converter-single-path.md, role: examined, excerpt: "Adding the export adds no network client crate to the dependency list."}
- {id: FR-031-AC-4, path: spec/functional/FR-031-export-subcommand.md, role: context_only, excerpt: "--kind matrix fixture ... (JSON tables, owned by PLAT-1603)"}
```

## Verdict

Changes requested. FND-001 and FND-002 are medium and should be fixed in this PR. For FND-001, add a Makefile target that runs the ignored test, and add list and table assertions. The ticket AC 1 box must stay unchecked until the test has passed once on a host with real pandoc, typst and pdftotext. FND-003 to FND-007 are low.

The tests drive the real binary and the real `producer-execute` over generated stand-in scripts. No test asserts against its own builder: the request is checked against the published EA schema and the FR-032 literal table, and the body against an independent split of the fixture. Every in-scope tag binds to a test that asserts its AC, apart from the partial FR-033-AC-4 and FR-032-AC-2 noted above.

**Criteria in FR-031 to FR-033 not met and not owned by a later ticket.** These are non-blocking as a list, but each is filed as a finding:
- The output-writability pre-check (code-review SR-20261112234823 FND-001).
- The canonical path in a refused diagnostic (code-review FND-002).
- The timed_out test (FND-002 here).

**Deferred to later tickets, correctly.**
- The JSON tables (PLAT-1603) are refused with exit 3, "not available in this build".
- `--template`, `--reference-doc` and `in/front.md` belong to FR-034 and FR-035.
- `SOURCE_DATE_EPOCH` belongs to FR-037.
- The title page (FR-031-AC-7, real-converter half) belongs to FR-034 and FR-035.
- Mermaid with no image (FR-031-AC-6, real-converter half) belongs to PLAT-1604.

## Dispositions

Round 1, at head 67afa73. The test results are as recorded in SR-20261112234823's dispositions.

| FND | outcome | reason |
| --- | --- | --- |
| FND-001 | still-open | `make test-export-real` (`cargo test --locked --test cli_export -- --ignored`) is added, and the PR body states that ticket AC 1 stays unchecked until it passes on a host with the tools. That part is accepted as a deferred, non-blocking execution. However, `real_converters_keep_the_fixture_headings_and_code` is unchanged: it still asserts no list item and no table cell, although ticket AC 1 requires both and the fixture has both. Adding `"first fixture item"` and `"FR-001-AC-1"` to its needles closes this finding. |
| FND-002 | fixed | The new test `a_timed_out_converter_exits_4_naming_the_state` uses a stand-in result `timed_out` and asserts exit 4, `timed_out` and no output. |
| FND-003 | fixed | `interpret` takes a `linux: bool`. The unit tests `a_non_linux_host_reports_the_unsupported_host_message` and `containment_failure_message_depends_on_the_host` cover both arms. |
| FND-004 | still-open | The fix commit adds no assertions on `containment`, `response`, the remaining `budget` members or `caller.version`. The new FR-032-AC-2-tagged tests cover the refused diagnostic and the UTF-8 refusal, not field-table equality. Comparing the whole request against a literal `json!` of the FR-032 table, with `capabilityRoot` substituted, would close it. |
| FND-005 | still-open | The rig now appends `$*` to `$LOG/argv`, but no test reads that file, so `producer-execute --request -` is still unverified. An assertion such as `assert_eq!(read(log/argv), "producer-execute --request -\n")` in the docx test would close it. |
| FND-006 | fixed | The new test `the_export_adds_no_network_client_crate`, tagged NFR-010-AC-3, checks the `[dependencies]` table for network client crates. |
| FND-007 | fixed | The FR-033 row now reads "Names the state `containment_failure`; the result carries no further reason." The Linux code message is "containment_failure: producer-execute could not contain the converter", which names the state, so the spec and the code agree. The spec file passes `quire validate`. |
