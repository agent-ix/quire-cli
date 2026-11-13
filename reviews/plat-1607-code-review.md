---
id: SR-20261113085035
title: "PLAT-1607 quire-cli PR190 code-review"
type: SpecReview
analysis: code-review
scope: "agent-ix/quire-cli PR190 head 3919a56; Makefile, src/commands/export.rs, src/commands/export/custom.rs, tests/cli_export.rs"
review_set: subset
---

## Summary

Ticket: PLAT-1607. Code review with the rust-review lane folded in, scoped to `git diff origin/main...HEAD` (head 3919a56). The PR adds `quire export --template <PATH>` (FR-036). A new module `custom.rs` checks the extension against `--to`, canonicalizes the path, opens it, refuses a non-regular file, and reads at most 16 MiB + 1 bytes from that same descriptor. `export_markdown` loads the template before identity resolution, validation and converter resolution, and passes the bytes in place of the generated default. JSON input refuses the flag in `check_flags`.

The design is sound. The regular-file check and the size limit apply to the open descriptor, so a swap between check and read cannot slip a different file through. The template bytes live only in memory and in `in/template.*` under the capability root. Nothing falls back to the default template on failure. The path never reaches a converter argument. There is no new process launch, nothing is vendored, and no template binary is committed (`git ls-files` has no `.typ`, `.docx` or `.dotx`). Gates: `cargo fmt --check` passed, all-target/all-feature clippy with `-D warnings` passed, and the focused tests passed (7 integration tests, 2 custom unit tests, 1 real-converter test ignored).

Probed with the built binary: a FIFO named `fifo.typ` hangs the export (killed by `timeout 5`, rc 124). `file.typ/x.typ` (ENOTDIR) and a symlink loop `loop.typ` (ELOOP) exit 4. A symlink to a directory exits 3, as specified.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | medium | `File::open` runs before the regular-file check, and opening a FIFO for reading blocks until a writer appears. `mkfifo t.typ; quire export in.md --to pdf --output o.pdf --template t.typ` hangs forever where FR-040 requires exit 3 "not a regular file". Fix: check `fs::metadata(&resolved)?.is_file()` before opening (keep the fd-level check after opening), or open with `O_NONBLOCK` via `OpenOptionsExt::custom_flags`. Add a FIFO case to the invalid-template test. | src/commands/export/custom.rs:30-35 |
| FND-002 | medium | Path errors other than NotFound and PermissionDenied go through `failure::input`, which maps them to `Outcome::Internal` (exit 4). A path through a regular file (`file.typ/x.typ`, ENOTDIR) and a symlink loop (ELOOP) both exit 4 ("cannot be read"), but FR-040 says a template path that is missing or not a regular file is exit 3. Fix: in `io_error`, keep PermissionDenied as exit 2 and classify every other path-resolution error from `canonicalize`/`open` as invalid (exit 3), with a test for ENOTDIR. | src/commands/export/custom.rs:30,52-57 |
| FND-003 | low | `an_unreadable_template_exits_2_naming_the_path` returns early, and passes, when the process can read a mode-000 file (root, or CAP_DAC_OVERRIDE in a container CI). The exit-2 half of FR-040-AC-2 then has no evidence anywhere, and the test reports a pass. Judgement: the skip should be explicit, not silent. Add a unit test in `custom.rs` that passes `io::Error::from(ErrorKind::PermissionDenied)` through `io_error` and asserts `failure::outcome` is `Refused`, so the mapping runs everywhere. Make the integration test announce the skip (`eprintln!` naming the reason) instead of returning silently. | tests/cli_export.rs:2423-2440 |
| FND-004 | low | Doc comments still describe only the default template. `IdentityParts::template` says "The default template: Typst source (PDF) or a DOCX reference document", and `Generated::identity_parts` says "Front matter and the default template", but the field now also carries the organisation template. | src/commands/export/producer.rs:113,121 |
| FND-005 | low | The ignored DOCX real test calls `Command::new("python3")...expect("python3 runs")` without `require_tool`, so a missing python3 panics without the "needs the real tool" message the suite promises (FR-032-AC-7 convention). The ignore reason says "pandoc, unzip and python3" but `real_export` also requires typst. The Makefile `test-export-real` comment does not list python3. | tests/cli_export.rs:2494-2510, Makefile:45-46 |

## Examined scope

```yaml
scope:
- id: src/commands/export/custom.rs
  path: src/commands/export/custom.rs
  role: examined
  excerpt: "let resolved = fs::canonicalize(path)...; let file = File::open(&resolved)...; if !metadata.is_file() ...; file.take(MAX_TEMPLATE_BYTES + 1).read_to_end(&mut bytes)"
- id: src/commands/export.rs
  path: src/commands/export.rs
  role: examined
  excerpt: "InputClass::Json if args.template.is_some() => invalid_request!(\"--template is refused for a JSON input\"); ... (Some(bytes), _) => bytes,"
- id: src/commands/export/producer.rs
  path: src/commands/export/producer.rs
  role: context_only
  excerpt: "files.push((template_name, parts.template.as_slice()));"
- id: src/commands/failure.rs
  path: src/commands/failure.rs
  role: context_only
  excerpt: "std::io::ErrorKind::PermissionDenied => Outcome::Refused, _ => Outcome::Internal,"
- id: tests/cli_export.rs
  path: tests/cli_export.rs
  role: examined
  excerpt: "a_template_is_copied_unchanged_into_the_capability_root_only; an_invalid_template_exits_3_naming_the_path_and_starts_no_converter; an_unreadable_template_exits_2_naming_the_path; a_template_is_refused_for_a_json_input_naming_the_flag; real_pdf_/real_docx_ ignored tests"
- id: Makefile
  path: Makefile
  role: examined
  excerpt: "# Real-converter export tests (FR-031-AC-1, FR-032-AC-7, FR-034-AC-10, FR-035, FR-036)."
```

## Verdict

Changes requested. FND-001 (FIFO hang) and FND-002 (exit 4 instead of 3) are small, local fixes in `custom.rs` and should land in this PR with tests. FND-003 to FND-005 are low and also belong in the same fix round. Clean: TOCTOU handling (one descriptor), the size bound on bytes actually read, the extension check, the JSON refusal, no fallback to the default, template bytes only in the capability root, no template path in any argument, no new execution path, and no vendoring, pins or committed template files.
