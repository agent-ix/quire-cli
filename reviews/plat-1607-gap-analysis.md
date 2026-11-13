---
id: SR-20261113085036
title: "PLAT-1607 quire-cli PR190 gap-analysis"
type: SpecReview
analysis: gap-analysis
scope: "agent-ix/quire-cli PR190 head 3919a56; spec FR-036, FR-040, FR-035, FR-032, NFR-010 vs tests/cli_export.rs, src/commands/export/custom.rs, src/commands/export.rs"
review_set: subset
---

## Summary

Ticket: PLAT-1607. Planless gap analysis of FR-036, with FR-040-AC-2/6/7, FR-035-AC-6, FR-032 (request table) and NFR-010, against the PR's tests and code. Plan completion: not assessed. Trace bindings come from `quire trace --id`. `quoin matrix` (0.30.1) reports every Test AC in this repository as `untagged`, including ones on main that carry `#[trace]` markers, so this review does not use it. That is a tooling observation, not a finding against this PR.

Every FR-036 AC has a bound test. AC-2, AC-3, AC-4, AC-5 and AC-6 run with stand-in converters. The stand-ins assert real request arguments (`in/template.typ` / `in/template.docx`, and no `org.` in any argument), the copied bytes (byte-equal to the user's file), that the capability root is removed, that no stray copy exists in `work/` or the repository, and that the `quire-export` metadata members are present. AC-1 is covered only by two ignored real-converter tests: pandoc, typst and pdftotext are not installed here, and the tests are explicit about what they need. All fixtures are created at test time. Ticket ACs: the fixture template for PDF and DOCX is AC-1 (ignored lane plus stand-in AC-2). An invalid or missing template is refused (AC-5, FR-040-AC-2). The default is used when no template is given (an existing test at tests/cli_export.rs:942).

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | medium | The FR-036-AC-1 PDF test asserts `second.contains("FR-001")` to prove the header shows the `document-id` member. Page 2 is the contents page, and it lists the fixture heading `[FR-001] Ember Overview`, so this assertion passes even if the template never receives the id. Assert on the header text itself, for example `second.contains("ORG-HEADER-MARKER FR-001")`, or use a marker id that does not occur in the body. | tests/cli_export.rs:2464-2490 |
| FND-002 | low | The stand-in custom-template test checks only the template flag and its value. It does not assert that the whole pandoc argument list equals the FR-032 table, although `pandoc_arguments(to, out)` exists for exactly that. A custom template that dropped `in/front.md` (the generated title page and contents, which FR-036 requires "whatever the template") or reordered arguments would pass the non-ignored lane. Assert `literals(pandoc) == pandoc_arguments(to, out)`. | tests/cli_export.rs:2311-2330 |
| FND-003 | low | FR-040-AC-6 (a pre-existing `--output` is unchanged) is not exercised for template failures. The invalid-template and failing-template tests only assert that `--output` does not exist. Pre-create `--output` with known bytes in at least the invalid-template case, assert that it is unchanged, and add the FR-040-AC-6 trace. | tests/cli_export.rs:2368-2421 |
| FND-004 | low | `without_a_template_the_default_one_is_used` is a strict subset of the existing `the_default_template_is_generated_into_the_capability_root`, which already asserts `paper: "a4"` and more. It is traced to FR-035-AC-6, an Inspection AC about the repository tree that the test does not check. Delete it. The existing test covers the ticket's "default when none is given" criterion. | tests/cli_export.rs:2355-2364 |

## Examined scope

```yaml
scope:
- id: FR-036-AC-1
  path: spec/functional/FR-036-organisation-export-template.md
  role: examined
  excerpt: "With a fixture .typ template whose header prints a fixed marker and the quire-export document-id member, a PDF export holds the marker and the id on pages after the first, and the generated title page. With a fixture .docx reference document whose footer holds a marker, the DOCX footer part holds that marker and the document body holds the generated title page."
- id: FR-036-AC-2
  path: spec/functional/FR-036-organisation-export-template.md
  role: examined
  excerpt: "The template path passed to the request is under the capability root as in/template.typ or in/template.docx, its bytes equal the user's file, and after the export the capability root is gone and no copy exists beside --output, the input or the repository."
- id: FR-036-AC-3
  path: spec/functional/FR-036-organisation-export-template.md
  role: examined
  excerpt: "A stand-in pandoc that records in/metadata.yaml sees the quire-export map with title, document-id, commit, date and the member version or release."
- id: FR-036-AC-4
  path: spec/functional/FR-036-organisation-export-template.md
  role: examined
  excerpt: "A template that fails to compile makes the export exit 4 with the converter's message (FR-033), writes no output file and does not retry with the default template."
- id: FR-036-AC-5
  path: spec/functional/FR-036-organisation-export-template.md
  role: examined
  excerpt: "A .docx template with --to pdf, a .typ template with --to docx, a missing path, a directory, and a file above 16 MiB each exit 3 naming the path (FR-040) with no converter started."
- id: FR-036-AC-6
  path: spec/functional/FR-036-organisation-export-template.md
  role: examined
  excerpt: "--template with a JSON input exits 3 naming the flag."
- id: FR-040-AC-2
  path: spec/functional/FR-040-export-template-and-identity-errors.md
  role: examined
  excerpt: "A missing template path, a directory, an extension that does not match --to, and a template above 16 MiB each exit 3 naming the path; an unreadable template exits 2."
- id: FR-040-AC-6
  path: spec/functional/FR-040-export-template-and-identity-errors.md
  role: examined
  excerpt: "In every case above no converter or engineering-assurance process is started, --output is absent, and a pre-existing --output is unchanged."
- id: FR-040-AC-7
  path: spec/functional/FR-040-export-template-and-identity-errors.md
  role: examined
  excerpt: "No failure above falls back to the default template, the current date or HEAD: the diagnostic is the only result."
- id: FR-035-AC-6
  path: spec/functional/FR-035-default-export-template.md
  role: examined
  excerpt: "The repository tree and the package file list hold no .typ, .docx or .dotx file for the default template or the generated parts, and these reach the converter only through the capability root."
- id: FR-032
  path: spec/functional/FR-032-export-converter-execution.md
  role: context_only
  excerpt: "arguments: Literal entries in order. Pandoc: --sandbox, --standalone, --from, markdown-yaml_metadata_block, --to, ..., then --template, in/template.typ (PDF) or --reference-doc, in/template.docx (DOCX), then the input files in order: in/front.md, then in/document.md"
- id: NFR-010-AC-1
  path: spec/non-functional/NFR-010-export-single-converter-single-path.md
  role: examined
  excerpt: "Review of the export code finds no process launch other than the one of engineering-assurance producer-execute, and no converter other than pandoc and typst."
- id: NFR-010-AC-2
  path: spec/non-functional/NFR-010-export-single-converter-single-path.md
  role: examined
  excerpt: "The published npm package file list and the repository tree hold no pandoc or typst binary, template file (.typ, .docx, .dotx) other than inline test strings, or reference document."
```

## Verdict

Changes requested. FND-001 makes half of FR-036-AC-1 unfalsifiable, and is a one-line fix. FND-002 to FND-004 are low. NFR-010-AC-1 and AC-2 are clean: no new launch site in the export code, and no committed template file. FR-036-AC-2 to AC-6 are verified by tests that can fail.
