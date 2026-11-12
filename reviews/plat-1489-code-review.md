---
id: SR-20261112034126
title: "PLAT-1489 quire-cli PR173 code-review"
type: SpecReview
analysis: code-review
scope: "agent-ix/quire-cli PR173 head 01136b3; src/commands/validate.rs, tests/cli_bundle_line.rs, CHANGELOG.md"
review_set: subset
---

## Summary

Ticket: PLAT-1489. Code review with the rust-review lane folded in, scoped to `git diff origin/main...HEAD` (PR173 head 01136b3). The PR renders `BundleFinding.line` in `validate --okf`. The human form reuses `line_prefix`, so it matches located document diagnostics byte for byte. JSON gains a `line` field only when the engine located the finding.

Focused checks at head: `cargo test --locked` for cli_bundle_line (4), cli_okf (15), cli_validate (26), output_contract (3) and cli_stream_contract (5) all pass. `cargo clippy --all-targets --all-features -D warnings` and `cargo fmt --check` are clean.

Mutation checks: dropping the line from the human form fails IT-258, IT-259 and IT-260. Dropping the JSON `line` field fails the same three. Dropping the line for bridged findings only fails IT-260. The no-line output is byte-identical to origin/main for human and JSON, with and without `--strict`, on the IT-261 fixture.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | medium | A located bundle JSON record carries `line` but no `path` or `reason`. Every other located JSON diagnostic in this repo carries `path` with `line`: document validation (`surface_result`) and both matrix diagnostics. A consumer gets `"line":19` with no structured file, so it must parse the message prose to tell which document the line belongs to. That parsing is the coupling `DiagnosticFields` exists to remove. Emit `path` (and `reason`) with `line`, at least for located findings, so FR-014-AC-15's unlocated record is unchanged. | src/commands/validate.rs:452-463 |
| FND-002 | low | `located()` branches on `Some`/`None`, but `DiagnosticFields.line` is `skip_serializing_if = "Option::is_none"` (ix-cli-kit streams.rs:249). `record.with_fields(DiagnosticFields { line, ..Default::default() })` already emits no `line` key for `None`, so the helper and its doc reason are redundant. | src/commands/validate.rs:450-463 |

## Examined scope

```yaml
scope:
- id: src/commands/validate.rs#surface_bundle
  path: src/commands/validate.rs
  role: examined
  excerpt: "let message = bundle_message(w); let record = ix_cli_kit::streams::Record::warning(\"ValidationWarning\", &message); located(record, w.line).emit(ctx.diagnostics);"
- id: src/commands/validate.rs#bundle_message
  path: src/commands/validate.rs
  role: examined
  excerpt: "format!(\"{}: {}{} [{}]\", f.path.display(), line_prefix(f.line), f.message, f.reason)"
- id: src/commands/validate.rs#surface_result
  path: src/commands/validate.rs
  role: context_only
  excerpt: "DiagnosticFields { reason: Some(error.reason.as_str()), path: Some(label), line: error.line, subject, change_target, remedy }"
- id: src/commands/matrix.rs
  path: src/commands/matrix.rs
  role: context_only
  excerpt: "DiagnosticFields { reason: Some(&diagnostic.reason), path: diagnostic.path.as_deref(), line: diagnostic.line, .. }"
- id: tests/cli_bundle_line.rs
  path: tests/cli_bundle_line.rs
  role: examined
  excerpt: "IT-258..IT-261; module and bundle built in tempdirs; IT-260/261 copy existing quire-cli-owned tests/fixtures/validate-mod docs into a tempdir"
- id: CHANGELOG.md
  path: CHANGELOG.md
  role: examined
  excerpt: "validate --okf renders BundleFinding.line (PLAT-1489) ... unlocated findings are unchanged (FR-014-AC-12..15)."
```

## Verdict

Approve with changes. The rendering is correct and the human form shares `line_prefix` with document diagnostics. FND-001 is a JSON-shape inconsistency and should be fixed in-PR. FND-002 is optional. The PR adds no vendored files: `tests/fixtures/validate-mod` is quire-cli's own fixture, not a quire-rs copy.
