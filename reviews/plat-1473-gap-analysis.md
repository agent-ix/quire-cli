---
id: SR-20261112205303
title: "PLAT-1473 quire-cli PR176 gap-analysis"
type: SpecReview
analysis: gap-analysis
scope: "agent-ix/quire-cli PR176 head 4681c37; tests/, spec/tests.md, spec/functional/FR-027-check-lock-subcommand.md, PLAT-1473 (as data)"
review_set: subset
---

## Summary

Ticket: PLAT-1473. This gap analysis checks the PR against the ticket's fix, which is read as data: delete the copied fixtures and the refresh target. It also checks the owner's rule that a vendored copy is deleted and nothing replaces it. The PR deletes the copy and the target, and adds no replacement copy. `quire matrix --scope .` (quire 0.43.0, engine 0.58.0) was used to check the ticket's FR-027 note.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | high | Several AC-to-test bindings that spec/tests.md marks as passing now point at tests that fail at the PR head, because they still read the deleted module through `iso_module()`. The affected tests trace IT-014, IT-023 / FR-005-AC-5, IT-055 / FR-005-AC-2 / FR-004-AC-7, IT-056 / FR-004-AC-4, IT-057 / FR-004-AC-5, IT-059 / FR-004-AC-8, IT-062 / FR-009-AC-3, IT-099 / FR-003-AC-2, IT-105 / FR-004-AC-21 / FR-004-AC-15, and FR-007-AC-8. The ticket's outcome, "tests reference ... instead of a committed copy", is met for 3 tests and missed for 10. | tests/common/mod.rs:18-20, spec/tests.md |
| FND-002 | low | The ticket also notes that every FR-027 AC shows as untagged in `quire matrix`. That is measured true at this head: FR-027-AC-1..10 are `untagged`, and FR-027-CON-1..3 are `method-without-symbol`. The PR does not address it. The ticket calls it a separate small cleanup, so fix it here or close it on the ticket explicitly. | spec/functional/FR-027-check-lock-subcommand.md |

## Examined scope

```yaml
scope:
- id: IT-008
  path: tests/audit_no_network.rs
  role: examined
  excerpt: "validate_does_not_open_inet_socket: validate valid-fr.md --module validate-mod under strace"
- id: IT-081
  path: tests/audit_no_network.rs
  role: examined
  excerpt: "scoped_env_discovery_validates_without_inet_socket: module reachable only via IX_FILAMENT_MODULES_PATH"
- id: FR-004-AC-21
  path: tests/cli_stream_contract.rs
  role: examined
  excerpt: "human_census_figures_stay_on_stdout_without_color: [ears:vague-response] on stderr, census on stdout"
- id: FR-027-AC-1
  path: spec/functional/FR-027-check-lock-subcommand.md
  role: examined
  excerpt: "A lock with two or more distinct package names from one agent-ix repository a..."
```

## Verdict

Changes requested. The vendored copy and its refresh target are gone, and nothing replaces them. The three tests the coder re-pointed still assert the same properties. FND-001 blocks the merge.
