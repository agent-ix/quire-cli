---
id: SR-20261113103139
title: "PLAT-1604 quire-cli PR194 code-review (Rust lane)"
type: SpecReview
analysis: code-review
scope: "agent-ix/quire-cli PR194 head cd1ce4b; Cargo.lock, CHANGELOG.md, src/commands/export.rs, src/commands/export/bundle.rs, src/commands/export/diagrams.rs, src/commands/export/json.rs, src/commands/export/producer.rs, src/commands/validate.rs, tests/cli_export.rs, tests/fixtures/export/export-fr.md"
review_set: subset
---

## Summary

Ticket: PLAT-1604. This is a code review with the Rust lane (rust-review and the rust-style baseline) of `git diff origin/main...HEAD` at cd1ce4b. No AssuranceProfile applies.

What was checked:

- **The mmdr contract.** The flags `-i <file> -o <file> -e png` were checked against the real CLI source of mermaid-rs-renderer 0.3.1 (`src/cli.rs` in the local cargo registry). `-e` is `outputFormat`, a clap ValueEnum that takes `png`. A `.mmd` input is read as one diagram, not as Markdown. `png` is a default feature. A failure is `error: <msg>` on stderr with exit 1. The `mmdr` binary itself is not installed.
- **The Lua filter.** It was read line by line against the pandoc Lua API. It has never run, because pandoc and Lua are not installed. Global `CodeBlock` and `Pandoc` functions are picked up under the typewise traversal, so the count check runs after every block. These calls are correct: `classes:includes`, `io.open`/`read('a')` (Lua 5.4), `pandoc.mediabag.insert(name, mime, contents)`, `pandoc.Image(inlines, src)`, `pandoc.Para`, and the `FORMAT == 'docx'` gate. `--sandbox` does not restrict filter IO. The only values interpolated into the source are `COUNT` (a usize) and `IMAGE_DIR` (a `&'static str`). Document text reaches the filter only through `in/diagrams/<k>.mmd` and `.alt`, whose names come from the running number. Pandoc escapes the alt inlines for Typst and OpenXML, so a hostile heading cannot inject.
- **Block identity.** quire-rs `extract_diagrams` at 9baf155 keeps the raw lines between the fences and does not dedent list items. The filter compares texts (LF-normalised) and then the count. That makes a mis-swap impossible: blocks with equal text get identical images, and any other disagreement fails the export, provided the filter is present (FND-001).
- **The quire-rs bump (77403eb to 9baf155).** `with_bundle_package` now returns `Result`, and its only error is `IdentityFixed` for a corpus that does not have exactly one member. `Spec::from_path` always builds one member, so the `?` in validate.rs is unreachable in practice. The adaptation is minimal and correct.
- **The thin-boundary placement.** quire-qualify's `is_admitted_dispatch` admits engine primitives only in `src/main.rs` and `src/commands/*.rs`. The other commands (lint, parse, lookup, edit) follow the same pattern. Putting `parse_document` in export.rs therefore follows the audit's design; it is not an evasion of it. `extract_diagrams` is not on the forbidden list, and no fence scan exists in quire-cli. The StR-004 bound "diagram files and filter generation from the quire-rs fenced-block query" holds.
- **The fixture edit.** Removing the mermaid block from `tests/fixtures/export/export-fr.md` loses no coverage. The only assertion that used it (`contains("```mermaid")`, the retired FR-031 section C rule) now asserts the rust block, which still covers FR-031-AC-6's other-language half. No test references the removed block.
- **Exit codes and ordering.** A missing `mmdr` is exit 3, raised only when a block exists, and it fails before any request with the other tools. A failed request, a missing PNG or an empty PNG is exit 4, prefixed `<doc>: mermaid diagram <n> (line <L>, section '<H>')` and keeping the state, the exit status and stderr. Rendering stops at the first failure. The capability root is removed by the TempDir guard on every path. Bundle `k` runs across documents while the alt and diagnostic ordinals stay per document. A JSON input passes `false` and an empty list.
- **No ceremony.** Nothing is vendored. No pin, SHA or digest is added, and no network is used.

Gates: see Verdict.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | high | A document whose only Mermaid-classed block is one quire-rs does not find exports with the diagram left as source code and exits 0. Examples are an attribute-style ```` ```{.mermaid} ```` fence (quire-rs `re_info_word` `^\s*(\w*)` yields `""`) or a block in a block quote. In that case `generated.diagrams` is empty, so no `in/diagrams.lua` is written and no `--lua-filter` is passed, pandoc renders the block as a code block, and the export succeeds. This breaks the ticket AC "never ... left as source" and FR-042-AC-7 ("the output of no run holds the source text as a code block"). The PR's own ignored test `real_pandoc_blocks_that_quire_rs_does_not_find_fail_the_export` builds exactly this fixture (one `{.mermaid}` block, nothing else) and asserts exit 4, so it can never pass. The spec conflicts with itself here: FR-042 section A bullet 3 and FR-032-AC-8 require no `--lua-filter` without blocks, while AC-7 requires failure. The fix needs a ruling. Either (a) always pass the filter for a Markdown input, with `COUNT = 0` so pandoc's count check fails any block quire-rs missed, and amend FR-032-AC-8 and FR-042 section A; or (b) keep the conditional filter, give the AC-7 and AC-15 fixtures one quire-visible block as well, and amend AC-7 and the ticket wording to admit that an attribute-only document exports as code. Option (a) is the one that meets the ticket. | src/commands/export/producer.rs:237-243, src/commands/export/producer.rs:104-106, tests/cli_export.rs:4818-4861 |
| FND-002 | medium | Pandoc's Markdown reader converts tabs to spaces before parsing unless `--preserve-tabs` is given, and the request does not pass it. quire-rs keeps the tab bytes in `source`, which is written to `in/diagrams/<k>.mmd`. The filter's `lf(source) ~= lf(block.text)` comparison therefore fails for any valid diagram that contains a tab, such as tab-indented Mermaid, and the export ends in exit 4 with "the block differs" for a correct document. Either add `--preserve-tabs` to the pandoc arguments (an FR-032 argument-table change) or expand tabs in the comparison, and add a stand-in or real test with a tab-indented block. Confidence is medium: this follows pandoc's documented default and was not run. | src/commands/export/diagrams.rs:128, src/commands/export/producer.rs:86-99 |
| FND-003 | low | `a_hostile_heading_changes_no_generated_filter_and_is_only_alternative_text` asserts that no `work/pwned` or `<rig>/pwned` file exists, but that assertion cannot fail. The stand-in pandoc never runs the Lua filter, and real pandoc's working directory is the capability root, which is deleted before the check. The byte-equal `in/diagrams.lua` assertion is the real guard and is sound. Either drop the `pwned` assertions, or have the stand-in executor log the capability root listing and assert that it has no `pwned`. | tests/cli_export.rs:4740-4758 |

## Verdict

**FAIL**. FND-001 is high: a quire-invisible Mermaid block is left as source, and the PR's AC-7 test cannot pass as written. That needs a spec ruling, preferably to always pass the filter for Markdown inputs.

Everything else is sound:

- the request envelope refactor (`envelope`, and `tool.name` replacing `Step::producer`, which gives identical names)
- the per-block `render_diagram` path, including the extra empty-PNG check
- the filter's data-file design and the generated-name discipline
- the mmdr flags
- the bundle numbering
- the quire-rs adaptation
- the thin-boundary placement

Gates run at cd1ce4b with the target removed afterwards:

- `cargo test --locked --all-features` (full suite): all green. cli_export had 88 passed and 17 ignored real-converter tests. Every other binary passed, including cli_validate (25), cli_okf (15) and cli_matrix (23). No quire-rs behaviour change from the 77403eb..9baf155 bump surfaced.
- `make fmt-check`: ok.
- `make lint` (clippy all-targets, all-features, `-D warnings`): ok.
- `make audit-thin-boundary`: "thin-boundary audit ok".

The real-converter tests (FR-042-AC-1, AC-7 and AC-15) did not run, because pandoc, typst and mmdr are not installed.
