---
id: SR-183
title: "PLAT-1177 quire-cli PR159 gap-analysis"
type: SpecReview
analysis: gap-analysis
scope: "agent-ix/quire-cli PR159; tests/, tools/quire-qualify/tests/policy.rs, spec/functional/FR-016-update-subcommand.md, spec/functional/FR-020-assurance-export-subcommand.md, spec/non-functional/NFR-002..005, NFR-009, spec/tests.md"
review_set: subset
---

## Summary

Ticket: PLAT-1177. Planless gap pass scoped to the PR diff. Ran installed `quire matrix --format tsv` (quire 0.36.2, engine 0.50.2) on origin/main and on PR head d80d1f3 in read-only worktrees: untagged 34 -> 24, tagged 188 -> 198, method-without-symbol 23 unchanged. The only rows whose binders changed are the ten claimed rows; no neighbouring row lost or gained a binder. The 24 remaining untagged rows match the PR body's triage table one to one. Each rebinding was read against its criterion text. Plan completion: not assessed.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | medium | NFR-004-M-2 measures socket() calls across all subcommands, but the tag sits on five tests covering 5 of 18 subcommands, which skip-pass when strace is absent; the other strace tests in the same file (IT-143 assurance, IT-158 clauses, update --check) are left untagged | tests/audit_no_network.rs:191 |
| FND-002 | low | FR-016 CR note and spec/tests.md still say AC-3 and AC-7 have no automated trace and AC-4 has no local binder, and keep the FR-016 row at 🚧 until a trace exists; after this PR those statements are false | spec/functional/FR-016-update-subcommand.md:81 |

## Examined scope

```yaml
scope:
- id: FR-020-CON-4
  path: spec/functional/FR-020-assurance-export-subcommand.md
  role: examined
  excerpt: "Unknown or malformed module versions, schema premises, and incomplete module loads SHALL fail closed before any stdout byte."
- id: NFR-002-M-1
  path: spec/non-functional/NFR-002-static-binary.md
  role: examined
  excerpt: "Project-supplied shared libraries linked | 0 | 0 | ldd static audit"
- id: NFR-003-M-1
  path: spec/non-functional/NFR-003-zero-unsafe.md
  role: examined
  excerpt: "Undocumented unsafe blocks in first-party Rust source and tests | 0 | 0 | quire-qualify unsafe-comments AST-backed static audit"
- id: NFR-004-M-1
  path: spec/non-functional/NFR-004-no-network.md
  role: examined
  excerpt: "HTTP/network client crates in dependency tree | 0 | 0 | cargo deny check bans + Cargo.lock audit"
- id: NFR-004-M-2
  path: spec/non-functional/NFR-004-no-network.md
  role: examined
  excerpt: "socket() calls observed across all subcommands | 0 | 0 | strace -e network runtime audit"
- id: NFR-005-M-2
  path: spec/non-functional/NFR-005-diagnostic-format.md
  role: examined
  excerpt: "Known failure modes whose JSON stderr parses as Diagnostic | all | all | Integration test"
- id: NFR-009-M-1
  path: spec/non-functional/NFR-009-native-qualification-boundary.md
  role: examined
  excerpt: "Executable Python or shell qualification files outside fixture data | 0 | 0 | Static inventory (TC-839)"
- id: FR-016-AC-7
  path: spec/functional/FR-016-update-subcommand.md
  role: context_only
  excerpt: "A failing `npm`/`cargo` invocation, or an unreachable registry, exits **4**"
```

## Verdict

Gaps found (medium). Rebindings verified correct: FR-020-CON-4 (it_139/it_140 refuse unknown module/version, mismatched schema premise, malformed premise, missing name/version, unloadable schema, all with empty stdout), NFR-002-M-1 (TC-089 ldd allowlist), NFR-003-M-1 (TC-092 calls the named quire-qualify audit over src/tests/tools and mutation-checks it), NFR-004-M-1 (TC-091 deny.toml + Cargo.lock), NFR-005-M-2 (IT-031 every failure class), NFR-009-M-1 (TC-839, named by the metric). NFR-004-M-2 is a partial claim (FND-001): tag every strace test in the file and record the untraced subcommands, or leave it untagged as (b). Update the FR-016 CR note, the FR-016 coverage row, and the TC-086 row to match the new traces (FND-002). No ledger, snapshot, census, or pin added. No overlap with PR #123 (its audit_no_network.rs hunk is at line 75) or PR #119 (plugin files only).

## Dispositions

| FND | outcome | reason |
| --- | --- | --- |
| FND-001 | fixed | aee9bff: NFR-004-M-2 removed from the five IT-008 strace tests; the row is untagged again and listed as (b) in the PR triage table with the 5-of-18 reason; matrix untagged 25, no neighbouring binder changed |
| FND-002 | fixed | aee9bff: FR-016 CR note, FR-016 coverage row and TC-086 now name cli_update::package_manager_channels as tracing AC-3 --check, AC-4 and AC-7, and list the remaining untraced paths; AC table text unchanged |
