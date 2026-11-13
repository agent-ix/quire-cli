---
id: SR-20261112234823
title: "PLAT-1435 quire-cli PR183 code-review"
type: SpecReview
analysis: code-review
scope: "agent-ix/quire-cli PR183 head 6da415a; src/commands/export.rs, src/commands/export/producer.rs, src/commands/export/tool.rs, src/commands/mod.rs, src/main.rs, tests/cli_export.rs, tests/fixtures/export/export-fr.md, Cargo.toml, CHANGELOG.md"
review_set: subset
---

## Summary

Ticket: PLAT-1435. This is a code review with the rust-review lane folded in (rust-style loaded). It is scoped to `git diff origin/main...HEAD` (one commit, 6da415a). It was checked against FR-031, FR-032, FR-033, NFR-010, FR-007 and StR-004. The later-ticket scope (FR-034 to FR-040, the JSON tables of PLAT-1603, and Mermaid of PLAT-1604) was judged only for whether the base implementation is consistent with it.

The PR adds `quire export <INPUT>.md --to pdf|docx --output <PATH>`. It validates the input through `validate::run`, takes the body boundary from `quire_rs::extract_frontmatter`, and writes `in/document.md` and `in/metadata.yaml` under a canonical `tempfile` capability root. It launches only `engineering-assurance producer-execute --request -`, sends one or two FR-032 requests, maps the result states to the FR-033 exit codes, and copies the output through a sibling `NamedTempFile` and `persist`.

Measured on this host:
- `cargo test --test cli_export` gives 15 passed and 1 ignored.
- `cargo clippy --all-targets --all-features -D warnings` is clean.
- A manual repro of a read-only output directory confirmed FND-001.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | medium | FR-033 "Output unwritable" must be checked before any conversion. `check_output_target` checks only that `--output` is not a directory and that its parent exists, not that the parent is writable. With `--output` in a mode-555 directory, the stand-in pandoc ran through producer-execute, and the export then failed at the copy with exit 4 (repro on this host). The full conversion runs before an unwritable output is detected. | src/commands/export.rs:168-179, src/commands/export.rs:221-233 |
| FND-002 | medium | FR-032 section A requires that "if the executor still refuses the request, the diagnostic names the canonical path it passed". `interpret` renders `refused` through the generic arm as `<tool> did not complete: refused (<reason>)`. That message names neither `tool.path` nor the capability root, so a `refused (executable)` or `refused (capability_root)` gives the user no path. No test covers the refused state. | src/commands/export/producer.rs:350-375 |
| FND-003 | low | The `--output` is-a-directory and missing-directory refusals use bare `anyhow::bail!`. They reach exit 4 only through the unclassified `Outcome::Internal` fallback in `failure::outcome`, not through an explicit classification. The behaviour is correct today, but FR-033 names this kind, and the rest of the module classifies explicitly (`invalid_request!`, `policy_refusal!`, `converter_failure`). | src/commands/export.rs:168-179 |
| FND-004 | low | `request()` serialises `capabilityRoot`, `HOME`, `TMPDIR` and `producer.executable` with `Path::display().to_string()`. That form is lossy for non-UTF-8 paths. A non-UTF-8 tool path or temporary directory is silently replaced, the executor then reports `unavailable` or `refused`, and the diagnostic is misleading. Converting with `to_str()` and refusing a non-UTF-8 path as invalid input would fail clearly. | src/commands/export/producer.rs:171-178 |
| FND-005 | low | The two `#[cfg(unix)]` / `#[cfg(not(unix))]` declarations of `is_executable` produce a new `symbol-id-collision` diagnostic on every `quire matrix` run over this repository: "2 declarations of function `is_executable` share one symbol id". One function with the cfg split inside its body avoids it. | src/commands/export/tool.rs:51-60 |

## Examined scope

```yaml
scope:
- {id: FR-032, path: spec/functional/FR-032-export-converter-execution.md, role: examined, excerpt: "quire SHALL launch only engineering-assurance producer-execute, with the request on stdin (--request -), and read its single result line. It SHALL NOT call a converter, a shell or any other executable."}
- {id: FR-033, path: spec/functional/FR-033-export-errors.md, role: examined, excerpt: "Output unwritable | The --output directory is missing or unwritable, or --output is a directory. | 4 | Names the path; checked before any conversion and again at the copy."}
- {id: FR-031, path: spec/functional/FR-031-export-subcommand.md, role: examined, excerpt: "The command SHALL pass the body of the artifact, the bytes after its YAML frontmatter block unchanged, to the converter as the Markdown source (in/document.md)"}
- {id: NFR-010, path: spec/non-functional/NFR-010-export-single-converter-single-path.md, role: examined, excerpt: "Export SHALL use pandoc and typst as its only converters, launched only through engineering-assurance producer-execute, with no converter binary, template file or reference document placed in this repository or its package."}
- {id: FR-007, path: spec/functional/FR-007-exit-codes.md, role: examined, excerpt: "3 Invalid request ... a required external tool that is missing or not executable; 4 Internal failure ... a failure reported by an external tool that a subcommand runs"}
- {id: StR-004, path: spec/stakeholder/StR-004-thin-boundary-over-quire-rs.md, role: context_only, excerpt: "thin boundary over quire-rs"}
- {id: Cargo.toml, path: Cargo.toml, role: examined, excerpt: "tempfile = \"3\" moved from [dev-dependencies] to [dependencies]"}
```

## Verdict

Changes requested: fix FND-001 and FND-002 in this PR. FND-003 to FND-005 are low.

These parts were checked and are correct:
- **One launch site.** The only `Command::new` in the export code is `engineering-assurance producer-execute --request -`, and no shell is involved (NFR-010-AC-1).
- **Canonical paths.** Every tool is resolved with `fs::canonicalize` and then a regular-file check with an executable bit. A link to a directory is a missing converter (exit 3).
- **Capability root.** It is created under a canonicalised `temp_dir()` and canonicalised again. Its removal is owned by a `TempDir` guard held in `RenderedFile`, so `?` early returns drop it, and it is dropped before `run` returns. No panicking call sits on the export path, so the release profile's `panic = "abort"` does not bypass the guard in practice.
- **Output.** It goes through a sibling `NamedTempFile` and `persist`, which is an atomic rename, so a pre-existing `--output` survives a failure.
- **Input as data.** Input text reaches the converter only as file content. The arguments are fixed literals (FR-033-AC-10).
- **Result parsing.** The `State` and `Stream` parsing matches the published `producer-execute --schema result`: `refused.reason` is a string enum, `failed.reason` has a `kind`, and stderr is a byte array decoded lossily.
- **Exit codes.** The mapping (3 for a missing converter or invalid input, 2 for permission or output-equals-input, 4 for a converter, executor or containment failure) matches FR-007 and FR-033.
- **Dependencies.** No network crate was added, and `tempfile` was correctly moved to `[dependencies]`.
- **Clean of ceremony.** Nothing is vendored, there are no pins or SHA ceremony, and there is no Markdown parsing beyond `extract_frontmatter`.

## Dispositions

Round 1, at head 67afa73 (fix commit "PLAT-1435: fix review round 1"). These gates pass at this head:
- `cargo fmt --check`;
- `cargo clippy --all-targets --all-features -D warnings`;
- `cargo test --test cli_export`: 20 passed, 1 ignored;
- the export unit tests: 5 passed.

| FND | outcome | reason |
| --- | --- | --- |
| FND-001 | fixed | `check_output_target` now probes the output directory with `NamedTempFile::new_in(parent)` and classifies a failure as exit 4. The new test `an_unwritable_output_directory_is_refused_before_any_conversion` asserts exit 4, "not writable" and zero requests. |
| FND-002 | fixed | The `refused` arm now appends `executable '<canonical path>', capability root '<root>'`. The new test `a_refused_request_names_the_canonical_executable_and_the_capability_root` covers it. |
| FND-003 | fixed | Output-target refusals go through `output_error`, which is explicitly `Outcome::Internal` (exit 4). |
| FND-004 | still-open | Only the executable half is fixed: `tool::resolve` now refuses a non-UTF-8 tool path with exit 3, and a test covers it. `request()` still writes `root.to_str().unwrap_or_default()`, so a non-UTF-8 temporary directory becomes an empty `capabilityRoot`, with `HOME=/out/home` and `TMPDIR=/out/tmp`. The executor's rejection then reads as an engineering-assurance mismatch. The fix is to refuse a non-UTF-8 capability root in `convert` before building any request. The `unwrap_or_default()` on `executable` is now unreachable and can become an `expect` or a typed `String` on `Tool`. |
| FND-005 | fixed | `is_executable` is now one function, with the cfg split inside `has_execute_permission`. `quire matrix` no longer reports a `symbol-id-collision` for src/commands/export/tool.rs. |
