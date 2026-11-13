---
id: SR-14573
title: "PLAT-1610 quire-cli PR193 code-review (Rust lane)"
type: SpecReview
analysis: code-review
scope: "agent-ix/quire-cli PR193 head 5a05572; src/commands/export.rs, src/commands/export/approval.rs, src/commands/export/bundle.rs, src/commands/export/json.rs, src/commands/export/producer.rs, tests/cli_export.rs"
review_set: subset
---

## Summary

Ticket: PLAT-1610. Code review with the Rust lane (rust-review, rust-style baseline) of `git diff origin/main...HEAD` at 5a05572.

The verdict fixtures were checked against the real result shape. engineering-assurance origin/main (7c6a73b) was built in a scratch target, and six real `approval-verify` verdicts were produced from SSH-signed tags made at review time with throwaway keys: `approved`, `untrusted_signer`, `wrong_record`, `meaning_missing`, `unsigned` (a lightweight tag) and `unverifiable` (a tag that does not exist). The PR's `quire export` then ran over all six, with stand-in pandoc and typst and the real `producer-execute`. It exited 0, and `in/approval.md` held six rows in order with the expected cells. The member names the code reads all exist with the types it expects: `approver`, `meaning`, `signed_at`, `signed_at_source`, `signed_at_missing`, `signature.signer.principal`, `signature.signed_off_meaning.values`, `status`, `reason` (top level, `unverifiable` only), `tag`, `record_id` and `record_digest`.

Gates run: `cargo test --test cli_export approval` (4 passed, 1 ignored real-converter test), the remaining approval tests, and the two unit tests in approval.rs. Real pandoc and typst are not installed, so the ignored test did not run.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | medium | The `claimed()` fixture used for `wrong_record` (and `meaning_missing`) has `signature.status: signed_untrusted` with an unmapped signer, and the test asserts Person `unmapped` for `wrong_record`. The real verifier never emits that combination: `wrong_record` and `meaning_missing` need `signed_valid` with a mapped `signature.signer.principal`, and the real export shows that principal in the Person cell. The `meaning_missing` fixture also has `presence: present, values: []`, where the real verdict has `presence: missing` and no values. As a result the FR-038 Person fallback (`signature.signer.principal` when `approver` is absent) has no test: deleting the `.or_else(...)` in `row()` passes every test. | tests/cli_export.rs:3213-3226, tests/cli_export.rs:3361-3366, src/commands/export/approval.rs:92-97 |
| FND-002 | low | An `--approval` file is read with `read_utf8` (`fs::read`) and has no size bound. A FIFO or `/dev/zero` given as `--approval` blocks or exhausts memory before any diagnostic. The `--template` input of the same command is bounded at 16 MiB (custom.rs MAX_TEMPLATE_BYTES); rust-style safety requires resource bounds on caller-controlled input. | src/commands/export/approval.rs:69, src/commands/export.rs:232-238 |
| FND-003 | low | `format_instant` checks only the shape (digits and separators). `2026-13-45T99:99:99Z` is accepted and rendered as a UTC instant, although the column claims to show a UTC instant. The real verifier never emits such a value; only a hand-made verdict reaches it. | src/commands/export/approval.rs:141-152 |
| FND-004 | low | `cell()` maps only `char::is_control` (Cc) and U+2028/U+2029 to a space. Unicode bidi controls (U+202A-U+202E, U+2066-U+2069) and other Cf characters in signer-typed meaning text pass through, so a signer can visually reorder the text of the Meaning cell in the PDF or DOCX. They cannot reorder the separate Verdict cell or the `claimed:` prefix, which comes before them. FR-034 C does not require this, so the finding is low. | src/commands/export/approval.rs:155-167 |

## Examined scope

```yaml
scope:
- {id: src/commands/export/approval.rs, path: src/commands/export/approval.rs, role: examined, excerpt: "load/parse/row/format_instant/cell/markdown: the block built from verdict files alone"}
- {id: src/commands/export.rs, path: src/commands/export.rs, role: examined, excerpt: "--approval flag, JSON refusal in check_flags, approval::load before identity::resolve"}
- {id: src/commands/export/bundle.rs, path: src/commands/export/bundle.rs, role: examined, excerpt: "approval::load before resolve_sources; approval passed once to Generated"}
- {id: src/commands/export/producer.rs, path: src/commands/export/producer.rs, role: examined, excerpt: "in/approval.md written and passed last in pandoc arguments"}
- {id: src/commands/export/json.rs, path: src/commands/export/json.rs, role: examined, excerpt: "approval: None for JSON reports"}
- {id: tests/cli_export.rs, path: tests/cli_export.rs, role: examined, excerpt: "approval block tests (FR-038, FR-039, FR-040)"}
```

## Verdict

Mergeable after FND-001 is fixed. Rebuild the `wrong_record` and `meaning_missing` fixtures to the real shape (`signed_valid`, mapped `signature.signer.principal`, and `presence: missing` for `meaning_missing`), and assert that Person shows that principal. The low findings may be fixed in the same round or accepted.

Checked and correct:

- Escaping: every verdict-derived cell (person, time, meaning, verdict status and reason, record id, digest and tag) and the column titles go through `cell()`, then `escape_markdown`. Hostile signer text arrives as backslash-escaped Str text, so it cannot add a typst raw block, OpenXML, a link, emphasis or a table row (the real verdict with a `|*x*` principal and a `{=typst}` meaning confirms this).
- In a bundle the block appears once, after the last document.
- A JSON input with `--approval` is refused with exit 3.
- Every approval error is raised before identity resolution and before any converter.
- Exit 3 for a missing file, invalid JSON, a wrong protocol and a missing `status`.
- Only `status == "approved"` gets a time, and only it gets an unprefixed meaning.
- The code makes no git, network or verification call.
- Nothing is vendored, and there are no pins and no SHA ceremony.
- Not adding `deny_unknown_fields` to `Verdict` is justified: the struct reads a subset of a producer's payload that carries `trust`, `binding` and other members.

Rebase risk: PR #192 (PLAT-1609) edits the same `Generated`/`Plan` structs in producer.rs, the same `Generated` literals in export.rs, bundle.rs and json.rs, and appends tests at the same point of tests/cli_export.rs (after line 3175). A textual conflict is certain, and resolving it is mechanical.

## Dispositions

Round 1, checked against fix commit 723ed9d. The fixed head was also run end to end over real `approval-verify` verdicts.

| FND | Outcome | Reason |
| --- | --- | --- |
| FND-001 | fixed | The new `mapped()` fixture (signed_valid, mapped `signature.signer.principal`, `presence: missing` with no values for meaning_missing) matches the real wrong_record and meaning_missing verdicts. The test asserts Person `Wren Mapped` and `Maya Mapped`, so deleting the `.or_else` now fails it. |
| FND-002 | fixed | `--approval` is read through `read_bounded`, shared with `--template`: regular file only (checked before open), at most 16 MiB, otherwise exit 3. A FIFO and an oversized file are tested. |
| FND-003 | fixed | `format_instant` now validates month, day of month (leap years by the Gregorian rule), hour, minute and second. A unit test covers 2026-13-45, 02-30, a non-leap 02-29, month 00, hour 24, minute 60 and second 60, and accepts 2024-02-29. |
| FND-004 | fixed | `is_format_control` maps U+061C, U+200E, U+200F, U+202A-U+202E and U+2066-U+2069, plus U+2028 and U+2029, to a space. A unit test covers this, and a real verdict carrying U+202E was flattened. |
