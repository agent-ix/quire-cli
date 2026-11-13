---
id: SR-20261112234812
title: "PLAT-1434 quire-cli PR181 spec-review/base"
type: SpecReview
analysis: base
scope: "agent-ix/quire-cli PR181; spec/functional/FR-030..FR-032, spec/non-functional/NFR-010, spec/usecase/US-009"
review_set: subset
---

## Summary

Ticket: PLAT-1434. Base checklist over the PR's US-009, FR-030..FR-032 and NFR-010 against the PLAT-1434 acceptance criteria.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | high | FR-030 implements StR-004 but puts new rendering logic in quire-cli: assembling an intermediate Markdown document, dropping frontmatter and promoting `title`, captioning Mermaid blocks, and turning JSON reports into Markdown tables. StR-004 limits the CLI to argv parsing, path safety, stream wiring and calls into quire-rs, and StR-004-AC-2 forbids markdown parsing and template rendering in src/. FR-030 neither names an upstream quire-rs FR for this behaviour nor adds a stated StR-004 exception as FR-027 does. As written, the two requirements contradict each other. Either author the assembly/table rendering upstream, or add an explicit, bounded StR-004 exception for export. | spec/functional/FR-030-export-subcommand.md:6, spec/stakeholder/StR-004-thin-boundary-over-quire-rs.md |
| FND-002 | medium | `--kind soup` names its source only as 'the SOUP report document (PLAT-1518)' with 'the fields the report names'. No spec artifact defines that report's JSON shape, and PLAT-1518 is not yet merged. So the soup half of FR-030-AC-9, and the shape-mismatch refusal FR-032 requires for each kind, have no defined oracle. Specify the accepted soup shape here, or reference the owning spec artifact once it exists. Alternatively, drop `soup` until it does; `--kind table` covers the generic case. | spec/functional/FR-030-export-subcommand.md:76, :121 |
| FND-003 | medium | FR-030 requires 'the same structural validation as `quire validate`', but `export` has no `--module` or `--scope` option. FR-004 says Markdown validation always needs a module registry to resolve the archetype. The spec does not say how export finds the module (discovery from the input's repository, a required `--module` flag, or `--scope`), so two implementers could validate against different module sets. FR-030-AC-2's 'fails `quire validate`' has no fixed module to fail against. | spec/functional/FR-030-export-subcommand.md:61 |
| FND-004 | low | FR-030 §D and FR-031 §C ('Hooks for later specs'), and FR-031's reason 'Typst is the engine of the later template spec', describe planned future features in this public repo: document identity, title page, organisation templates, approval and signature blocks, bundles, and template injection points. The ticket's standing instruction is no roadmap in public repos. Reduce each to a one-line scope exclusion that cites the owning ticket. Drop the design of how later specs will extend the request, and the template-spec reason for choosing Typst. The self-contained-executable reason stands on its own. | spec/functional/FR-030-export-subcommand.md:97, spec/functional/FR-031-export-converter-execution.md:39, :79 |
| FND-005 | low | FR-031-AC-3 says the typst request carries '`compile` only', but the statement it verifies requires `compile` plus an input under `in/` and an output under `out/`. Read literally, the AC contradicts the statement. Reword it to 'carries exactly `compile`, one input under `in/` and one output under `out/`'. | spec/functional/FR-031-export-converter-execution.md:91 |
| FND-006 | low | FR-030-AC-1/4/6/7 assert on 'text extracted' from real PDF and DOCX output, so they need real pandoc and typst plus a PDF/DOCX text extractor. Only FR-031-AC-7 states how real converters reach the test (env vars, fail if unset). Neither the extractor nor how these ACs are gated is specified, so the tests could quietly become stand-in-only or skip. | spec/functional/FR-030-export-subcommand.md:113 |

## Examined scope

```yaml
scope:
- id: FR-030
  path: spec/functional/FR-030-export-subcommand.md
  role: examined
  excerpt: 'The CLI SHALL provide an `export` subcommand that renders one Markdown artifact or one JSON report as a PDF or DOCX file. `export` lives in quire-cli because quire-cli already owns the Markdown artifact boundary and `quire matrix`; the Quoin and SOUP reports reach it as files, so it needs no dependency on Quoin. It is the only export command: Quoin and Engineering Assurance add none.'
- id: FR-031
  path: spec/functional/FR-031-export-converter-execution.md
  role: examined
  excerpt: '`quire export` SHALL run every converter as one request of the `engineering-assurance.producer-execution-request/v1` protocol, executed by `engineering-assurance producer-execute` (EA FR-029), so a converter is a declared external tool and `quire` launches no converter process itself.'
- id: FR-032
  path: spec/functional/FR-032-export-errors.md
  role: examined
  excerpt: '`quire export` SHALL classify each failure into one of three kinds, name the cause on stderr, use the exit codes of FR-007, and leave no file at `--output`.'
- id: NFR-010
  path: spec/non-functional/NFR-010-export-single-converter-single-path.md
  role: examined
  excerpt: Export SHALL use pandoc and typst as its only converters, launched only through `engineering-assurance producer-execute`, with no converter binary, template or reference document placed in any repository or package.
- id: US-009
  path: spec/usecase/US-009-export-document-to-pdf-docx.md
  role: examined
  excerpt: As a reviewer or approver who works in office tools rather than a repository I want a Quire Markdown artifact, or a JSON report the toolchain wrote, as a PDF or DOCX file So that I can read, circulate and sign it without a Markdown viewer or a JSON reader.
- id: FR-030-AC-1
  path: spec/functional/FR-030-export-subcommand.md
  role: examined
  excerpt: Exporting a fixture FR artifact containing headings, a list, a table and a code block to `pdf` and to `docx` writes both files, and the text extracted from each contains every fixture heading and the code block text.
- id: FR-030-AC-2
  path: spec/functional/FR-030-export-subcommand.md
  role: examined
  excerpt: Exporting a Markdown file that fails `quire validate` exits 3, writes no output file and starts no converter.
- id: FR-030-AC-3
  path: spec/functional/FR-030-export-subcommand.md
  role: examined
  excerpt: A JSON input without `--kind`, and a Markdown input with `--kind`, each exit 3 with a diagnostic naming the flag.
- id: FR-030-AC-4
  path: spec/functional/FR-030-export-subcommand.md
  role: examined
  excerpt: Exporting a `quire matrix --format json` fixture with `--kind matrix` to `docx` yields a table whose rows hold every criterion id of the fixture.
- id: FR-030-AC-5
  path: spec/functional/FR-030-export-subcommand.md
  role: examined
  excerpt: A `--kind table` fixture whose rows carry a nested object yields a cell holding that object as compact JSON, and no field of any row is missing from the output.
- id: FR-030-AC-6
  path: spec/functional/FR-030-export-subcommand.md
  role: examined
  excerpt: A fixture with a `mermaid` block exports with the block text and the caption `Diagram source (Mermaid)` present, no image in the output, and a stderr warning naming the block.
- id: FR-030-AC-7
  path: spec/functional/FR-030-export-subcommand.md
  role: examined
  excerpt: A fixture with frontmatter exports with the `title` as the document title and no `type:` frontmatter line in the extracted text.
- id: FR-030-AC-8
  path: spec/functional/FR-030-export-subcommand.md
  role: examined
  excerpt: Exporting the same input twice never writes a file other than `--output`, and the working directory and input directory hold no new file.
- id: FR-030-AC-9
  path: spec/functional/FR-030-export-subcommand.md
  role: examined
  excerpt: The `--kind rmm` and `--kind soup` fixtures export with one table row per clause and per SOUP record respectively.
- id: FR-031-AC-1
  path: spec/functional/FR-031-export-converter-execution.md
  role: examined
  excerpt: With fixture executables standing in for pandoc and typst, `--to docx` produces exactly one producer-execute request, and `--to pdf` produces a pandoc request followed by a typst request.
- id: FR-031-AC-2
  path: spec/functional/FR-031-export-converter-execution.md
  role: examined
  excerpt: Every request used by the tests validates against the schema published by `engineering-assurance producer-execute --schema request` and names an absolute `producer.executable`.
- id: FR-031-AC-3
  path: spec/functional/FR-031-export-converter-execution.md
  role: examined
  excerpt: The pandoc request carries `--sandbox`, `--standalone`, `--from markdown` and one `-o` under `out/`, and the typst request carries `compile` only.
- id: FR-031-AC-4
  path: spec/functional/FR-031-export-converter-execution.md
  role: examined
  excerpt: When the pandoc request does not complete, no typst request is made.
- id: FR-031-AC-5
  path: spec/functional/FR-031-export-converter-execution.md
  role: examined
  excerpt: During a run, the only child process `quire` itself starts is `engineering-assurance`.
- id: FR-031-AC-6
  path: spec/functional/FR-031-export-converter-execution.md
  role: examined
  excerpt: After success and after each failure of FR-032, the temporary capability root no longer exists.
- id: FR-031-AC-7
  path: spec/functional/FR-031-export-converter-execution.md
  role: examined
  excerpt: With real pandoc and typst available (set through environment variables naming their paths; the test fails naming the variable if unset), a fixture exports to a PDF and a DOCX that open as a zip (DOCX) and start with `%PDF` (PDF).
- id: FR-032-AC-1
  path: spec/functional/FR-032-export-errors.md
  role: examined
  excerpt: With `pandoc` absent, `--to docx` exits 3, names `pandoc` and `--pandoc`, and writes no file.
- id: FR-032-AC-2
  path: spec/functional/FR-032-export-errors.md
  role: examined
  excerpt: With `typst` absent and `pandoc` present, `--to pdf` exits 3 naming `typst`, and no pandoc request was made.
- id: FR-032-AC-3
  path: spec/functional/FR-032-export-errors.md
  role: examined
  excerpt: With `engineering-assurance` absent, the export exits 3 naming `engineering-assurance`.
- id: FR-032-AC-4
  path: spec/functional/FR-032-export-errors.md
  role: examined
  excerpt: A fixture pandoc that exits non-zero with a stderr message makes the export exit 4 and print that message, and a fixture that outlives the request timeout exits 4 naming `timed_out`.
- id: FR-032-AC-5
  path: spec/functional/FR-032-export-errors.md
  role: examined
  excerpt: A fixture pandoc that exits 0 without writing the output exits 4.
- id: FR-032-AC-6
  path: spec/functional/FR-032-export-errors.md
  role: examined
  excerpt: Each of a missing input, a `.txt` input, a non-UTF-8 input, malformed JSON, and a JSON input whose shape does not match `--kind matrix` exits 3 naming the file, with no converter started.
- id: FR-032-AC-7
  path: spec/functional/FR-032-export-errors.md
  role: examined
  excerpt: After a failed export over a pre-existing `--output`, that file's bytes are unchanged.
- id: NFR-010-AC-1
  path: spec/non-functional/NFR-010-export-single-converter-single-path.md
  role: examined
  excerpt: Review of the export code finds no process launch other than the one of `engineering-assurance producer-execute`, and no converter other than pandoc and typst.
- id: NFR-010-AC-2
  path: spec/non-functional/NFR-010-export-single-converter-single-path.md
  role: examined
  excerpt: The published npm package file list and the repository tree hold no pandoc or typst binary, template (`.typ`, `.docx`, `.dotx`) other than inline test strings, or reference document.
- id: NFR-010-AC-3
  path: spec/non-functional/NFR-010-export-single-converter-single-path.md
  role: examined
  excerpt: Adding the export adds no network client crate to the dependency list.
- id: StR-004
  path: spec/stakeholder/StR-004-thin-boundary-over-quire-rs.md
  role: context_only
  excerpt: 'For artifact operations, the CLI SHALL be a thin process boundary over `quire-rs`. ... StR-004-AC-2: `src/` contains no markdown parsing, template rendering, graph construction, assurance projection, or JSON Schema definition/validation logic -- only argv/premise parsing, path-safety checks, stdin/stdout wiring, calls into `quire-rs`, and the explicitly owned FR-027 Cargo.lock developer gate.'
- id: FR-004
  path: spec/functional/FR-004-validate-subcommand.md
  role: context_only
  excerpt: 'CR note (`--module` is REQUIRED): Markdown validation always needs a module registry to resolve the archetype and its `body_extraction` asserts, so `--module` is mandatory (not bracketed/optional) in the implementation.'
```

## Verdict

Changes required. Ticket ACs that the spec meets: pandoc chosen with a reason, typst used only as the PDF engine; producer-execute as the only launch path; command `quire export` lives in quire-cli; Mermaid is carried as captioned source with a warning; the three error kinds are specified; nothing is vendored (NFR-010); `quire validate` passes with no warnings on the new files. No paywalled standard text, no compatibility layer or fallback (fallback is explicitly forbidden), and no pins, SHAs or digests (`producer.version` is a required EA request field, not a pin). Open: the StR-004 contradiction (FND-001), the undefined soup input (FND-002), and the unspecified module resolution for validation (FND-003).

## New findings (disposition pass 1)

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-007 | medium | This was introduced by the fix. To insert the Mermaid caption 'by text edit ... before the opening fence of each such block', quire-cli has to locate fenced code blocks in the Markdown source: backtick and tilde fences, fences nested in lists or block quotes, and `mermaid` text inside another fence or an indented code block. That is Markdown parsing in src/. The new StR-004 exception text says 'Markdown artifacts are still parsed ... only by quire-rs', and FR-030 says '`export` parses no Markdown'. Either state that the block positions come from quire-rs (for example its parse output's code-block spans), or leave the block to the converter and drop the caption. | spec/functional/FR-030-export-subcommand.md:37, :119; spec/stakeholder/StR-004-thin-boundary-over-quire-rs.md:24 |
| FND-008 | low | `--kind rmm` accepts only a top-level `rows` array. That matches the single-set `quoin discharge --rmm --json` (quoin RequirementsMapping: schemaVersion, clauseSet, asOf, rows, unusedFacts). `quoin discharge --rmm --pack` (quoin FR-140) emits `index` plus `matrices` of those documents, and `--kind rmm` would refuse it. State that the kind is single-set only, or accept the pack form. | spec/functional/FR-030-export-subcommand.md:96 |

## New findings (disposition pass 2)

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-009 | medium | This was introduced by the fix. The new table-content rules drop report members that Quoin's own rendering shows. For `rmm`, `unusedFacts` is never rendered (quoin rmm.rs renders the facts not spent after the table, per FR-135-AC-4), and neither is `asOf`. For `soup`, `unreadable_scans` is never rendered (quoin render.rs prints 'unreadable scan file, not read: <path>' for each), and neither is `as_of`. The exported PDF/DOCX would hide unspent evidence and unread scans that the source report states, so the export looks more complete than the source. Render these members too, for example as a further table or line. | spec/functional/FR-030-export-subcommand.md:96-97 |

## Dispositions

| FND | outcome | reason |
| --- | --- | --- |
| FND-001 | fixed | - |
| FND-002 | still-open | The soup shape is now defined as 'a top-level object with a `records` array of objects, each with string members `name` and `version`', and the spec says producers 'align to these members'. That shape does not match the real SOUP report. quoin#844 (PLAT-1518, open; rust/crates/quoin-cli/src/soup/report/render.rs json()) emits a top-level `rows` array whose rows carry `purl`, `id`, `path`, `version`, `serves`, `known_anomalies`, ... and has no `records` and no `name`. As written, `--kind soup` refuses every SOUP report the toolchain writes, and the ticket AC (SOUP report rendered as tables) is not met. Bind the shape to the producer's actual members (`rows`, keyed by `purl`), or cite quoin FR-144 as the shape owner. |
| FND-003 | fixed | - |
| FND-004 | fixed | - |
| FND-005 | fixed | - |
| FND-006 | fixed | - |
| FND-002 | fixed | - |
| FND-007 | fixed | - |
| FND-008 | fixed | - |
