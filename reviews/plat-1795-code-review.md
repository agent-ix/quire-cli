---
id: SR-20261116045011
title: "PLAT-1795 quire-cli code review"
type: SpecReview
analysis: code-review
scope: "agent-ix/quire-cli; tests/assurance_cross_language.rs, tools/quire-qualify/src/coverage.rs, tools/quire-qualify/src/error.rs, tools/quire-qualify/tests/coverage.rs; FR-020 and FR-025"
review_set: subset
relationships:
  - { target: "ix://agent-ix/quire-cli/FR-020", type: references }
  - { target: "ix://agent-ix/quire-cli/FR-025", type: references }
---

## Summary

Reviewed quire-cli PR #226 at `8f6ecb4481c33d67c37bc10614a6ae421ceb951e`, including the strict qualification decoder and shared assurance corpus. The decoder check is sound; the Node path still checks JSON syntax only, so it does not provide the semantic acceptance/refusal evidence required by PLAT-1795.

## Verdict

**CONDITIONAL** — one medium coverage finding remains for the Node reader boundary; PLAT-1795 is incomplete until that boundary is resolved.

## Scope examined

| Unit | Path | Disposition |
| --- | --- | --- |
| FR-025-AC-2 | spec/functional/FR-025-rust-qualification-tooling.md | examined |
| FR-020-AC-8 | spec/functional/FR-020-assurance-export-subcommand.md | examined |
| Decoder implementation and regression | tools/quire-qualify/src/coverage.rs; tools/quire-qualify/src/error.rs; tools/quire-qualify/tests/coverage.rs | examined |
| Cross-language case corpus | tests/assurance_cross_language.rs | examined |

## Review checks

The exact trailing-whitespace, trailing-garbage, and concatenated-document cases use the production `check_bytes` decoder, and the new error path is a distinct typed variant. The Rust and Python paths run the same case corpus through `read_assurance_export` and Python `jsonschema`; the Node block only calls `JSON.parse` and compares `json_syntax_valid`, so semantic refusals for unsupported versions, duplicate identities, unknown fields, and inconsistent module premises are not exercised in Node. The PR body records this limitation and does not claim Node semantic acceptance.

Focused checks run by this reviewer:

- `/home/peter/dev/worktrees/locked-build.sh /tmp/plat-1795-qcli-review-coverage-target cargo test --locked -p quire-qualify --test coverage coverage_decoder_requires_one_complete_json_document -- --nocapture` — 1 passed, 5 filtered.
- `/home/peter/dev/worktrees/locked-build.sh /tmp/plat-1795-qcli-review-cross-target cargo test --locked --test assurance_cross_language it_144_languages_receive_one_acceptance_refusal_corpus -- --nocapture` — 1 passed, 1 filtered; Python jsonschema and Node were available.
- `/home/peter/dev/worktrees/locked-build.sh /tmp/plat-1795-qcli-review-fmt-target cargo fmt --all -- --check` — passed.
- `/home/peter/dev/worktrees/locked-build.sh /tmp/plat-1795-qcli-review-clippy-coverage-target cargo clippy --locked -p quire-qualify --test coverage -- -D warnings` — passed.
- `/home/peter/dev/worktrees/locked-build.sh /tmp/plat-1795-qcli-review-clippy-cross-target cargo clippy --locked --test assurance_cross_language -- -D warnings` — passed.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | medium | The Node corpus step validates JSON syntax only; it does not test the expected schema/version/identity acceptance and refusal outcomes, leaving the Node reader requirement in PLAT-1795 unverified. | tests/assurance_cross_language.rs:236-255 |

## Dispositions

| FND | outcome | reason |
| --- | --- | --- |
| FND-001 | deferred | No supported Node assurance reader exists in this repository scope. The existing JSON.parse probe supplies syntax-only evidence and does not satisfy the Node acceptance/refusal obligation; PLAT-1795 remains blocked pending an actual supported reader. |
