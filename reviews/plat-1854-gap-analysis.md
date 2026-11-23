---
id: SR-20261116045014
title: "PLAT-1854 quire-cli PR230 gap-analysis"
type: SpecReview
analysis: gap-analysis
scope: "agent-ix/quire-cli PR230 head e4ede7f; src/commands/{mod,coverage,matrix,trace,relations,symbols,assurance}.rs, tests/cli_declaration_drop.rs, spec/functional/"
review_set: subset
---

## Summary

Ticket: PLAT-1854. This is a planless gap pass scoped to the diff. Plan completion: not assessed. I ran `quire coverage --scope . --json` from this head's binary and the repo's `quire-qualify assurance-traceability` over its output. Behaviour owner: quire-rs FR-058-AC-47.

quire-cli's spec has no FR-058 and no local criterion for the declaration-drop refusal. The closest local requirements are FR-017 (coverage), FR-026 (matrix) and FR-007 (exit codes). The new tests are therefore tagged to an id that does not exist in this corpus.

The reverse gap is also open. Commands that compute over the merged model (`relations`, `symbols --module`) are not wired to the refusal, so the upstream AC's "every consumer" clause is only partly met here.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | medium | No local acceptance criterion owns the CLI-side behaviour: refusing with exit 3 and naming each dropped declaration. The tests tag the upstream quire-rs id FR-058-AC-47, which the local corpus cannot bind, and this produces 4 untracked symbols (see code-review SR FND-001). A quire-cli AC under FR-017/FR-007 is needed, but in a follow-up and not in this PR. | spec/functional/FR-017-coverage-subcommand.md, spec/functional/FR-007-exit-codes.md |
| FND-002 | medium | Reverse gap: `relations` (all modes) and `symbols --module` compute over the merged model with no `require_complete_model()` call and no test. The code-review SR FND-002/FND-003 has the details. | src/commands/relations.rs:124-165, src/commands/symbols.rs:94-125 |

## Examined scope

```yaml
scope:
- id: FR-058-AC-47
  path: "ix://agent-ix/quire-rs/FR-058"
  role: context_only
  excerpt: "A declaration that cannot be loaded drops only itself; every consumer computing over the merged model exits with the FR-050 Invalid exit and names each dropped declaration on stderr, including a whole unreadable traceability: section."
- id: tests/cli_declaration_drop.rs
  path: tests/cli_declaration_drop.rs
  role: examined
  excerpt: "#[trace(\"FR-058-AC-47\")] on 4 tests"
- id: spec/functional
  path: spec/functional
  role: examined
  excerpt: "FR-001..FR-046; no FR-058"
```

## Verdict

The tests are real behavioural tests: all four fail when `src/commands/mod.rs` is reverted. They bind to no local criterion, though, and two consumers are uncovered. Spec work for a local AC is deferred, as the brief says. The untracked-symbol gate break and the two uncovered consumers belong in this PR.
