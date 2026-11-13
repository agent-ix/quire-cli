---
id: SR-20261113045747
title: "PLAT-1554 quire-cli PR184 gap-analysis"
type: SpecReview
analysis: gap-analysis
scope: "agent-ix/quire-cli PR184 head b429ebb; spec/functional/FR-030-relations-subcommand.md, src/commands/{corpus_input,relations,validate}.rs, tests/cli_relations.rs"
review_set: subset
---

## Summary

Ticket: PLAT-1554. Planless gap analysis of FR-030 against the PR's tests and code. `quire matrix --scope . --format json` (quire 0.44.0, engine 0.59.1) shows all 23 Test-method FR-030 criteria `tagged`. FR-030-AC-22 (Inspection) is `method-without-symbol`; it was checked by inspection: the CLI diff holds no hazard, threat or classification name, and the report comes from `quire_rs::relation_report`. Reverse gap: none. Every behaviour added (the `[member]` finding prefix, the `--repo` module-discovery root, the `with_bundle_package` Result mapping) traces to FR-030 or FR-014. No stubs and no coverage inflation. Semantic review: skipped (opt-in, not requested), although each tagged test was read against its AC.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | medium | FR-030-AC-24 specifies its fixture as a module declaring a `document: spec/tests.md` target. quire-rs deleted that form, and a `document:` key is now an unknown field that the loader refuses (quire-rs src/traceability.rs:840, :2494). The AC cannot be run as written. The test substitutes an `exclude: ['spec/fixtures/**']` glob authored against the repository root, which checks the same intent: declarations resolve against `<DIR>`, not `<DIR>/spec`, per member. Ruling: amend the AC text to the exclude-glob form the test uses. | spec/functional/FR-030-relations-subcommand.md:102, tests/cli_relations.rs:701-738 |
| FND-002 | low | The FR-030-AC-20 test checks the resolved half (both members) only by `gold-rule-idle` being absent from stderr. It does not assert the exit status or that validation ran. A run that fails before validating, for example on a member load error, passes this half by accident. Assert success (or exit 0) before the absence check. | tests/cli_relations.rs:652-657 |
| FND-003 | low | The FR-030-AC-17 test asserts only that stderr contains `o/a` and `no document root`. It does not check the `MissingDocumentRoot` kind or the member's root path, both of which the AC requires. The output does carry both (checked by hand under `--diagnostics-format json`), so this is a test gap only. | tests/cli_relations.rs:539-547 |

## Coverage

- FR-030 criteria: 24; Test-method criteria tagged: 23 of 23; Inspection (AC-22): checked, clean.
- Untagged criteria: 0. Criteria tagged only by ignored tests: 0.
- Focused run: `cargo test --test cli_relations` 14 passed, 0 failed (quire-rs path patch to main+#666).
- Semantic review: skipped (not requested).
- Plan completion: not assessed

## Verdict

CONDITIONAL: one medium finding (the AC-24 spec text) and two low test-assertion gaps. No untagged criterion and no high finding.

## Examined scope

```yaml
scope:
- {id: FR-030-AC-1, path: spec/functional/FR-030-relations-subcommand.md, role: examined, excerpt: "quire relations --scope <DIR> --module $M --json over a fixture corpus with an unmitigated risk, a mitigated-but-untested risk and a mitigated-and-tested risk emits rows whose state and verification are respectively uncovered, covered/unverified, covered/verified ..."}
- {id: FR-030-AC-13, path: spec/functional/FR-030-relations-subcommand.md, role: examined, excerpt: "With --strict, a selection producing zero rows exits 1 with an empty-population diagnostic, and a corpus whose every row is not-tracked exits 1 with a nothing-measured diagnostic; both exit 0 without --strict."}
- {id: FR-030-AC-17, path: spec/functional/FR-030-relations-subcommand.md, role: examined, excerpt: "A --repo whose root holds no spec/ exits 3 with MissingDocumentRoot naming that root and the member identity."}
- {id: FR-030-AC-20, path: spec/functional/FR-030-relations-subcommand.md, role: examined, excerpt: "quire validate --okf --repo o/a=<A> --repo o/b=<B> --module $M over a fixture where a classified requirement in o/a mitigates a hazard in o/b carries no finding bearing the fixture relation's own check token for it, and with the o/b flag removed carries that finding, naming the member identity o/a."}
- {id: FR-030-AC-22, path: spec/functional/FR-030-relations-subcommand.md, role: examined, excerpt: "The CLI source holds no hazard, threat or safety-classification name, and delegates the report to quire-rs."}
- {id: FR-030-AC-24, path: spec/functional/FR-030-relations-subcommand.md, role: examined, excerpt: "quire validate --okf --repo o/a=<A> --repo o/b=<B> --module $M over a fixture where member o/a holds <A>/spec/tests.md and the module declares a document: spec/tests.md target: the target is minted for o/a and not for o/b, which lacks the file, so each member's declarations resolve against its own <DIR>, not <DIR>/spec."}
- {id: FR-030-AC-2..AC-12,AC-14..AC-16,AC-18,AC-19,AC-21,AC-23, path: spec/functional/FR-030-relations-subcommand.md, role: examined, excerpt: "remaining Test-method criteria; each tagged test read against its AC; no defect"}
```

## Dispositions

| FND | outcome | reason |
| --- | --- | --- |
| FND-001 | still-open | No fix landed. spec/functional/FR-030-relations-subcommand.md FR-030-AC-24 still names the `document: spec/tests.md` target form; the test still uses the exclude-glob form. |
| FND-002 | still-open | No fix landed. tests/cli_relations.rs:652-657 still checks the both-members half only by absence of `gold-rule-idle` on stderr, with no exit-status assertion. |
| FND-003 | still-open | No fix landed. tests/cli_relations.rs:539-547 still asserts only `o/a` and `no document root`; neither the `MissingDocumentRoot` kind nor the root path is asserted (the output carries both, re-checked at 603492f). |

## New findings (disposition pass 1)

Re-review at head 603492f. `quire matrix --scope .` (quire 0.44.0, engine 0.59.1): all 23 Test-method FR-030 criteria `tagged`; FR-030-AC-22 `method-without-symbol`, re-inspected: the diff's `src/` holds no hazard, threat, risk, mitigation or classification name. Mutation spot-checks, each rebuilt and run against `cargo test --test cli_relations`: dropping the `[member]` finding prefix killed AC-20 and AC-24 tests; dropping `Verification::Partial` from the strict check killed AC-11; disabling the `required_relations` pre-check killed AC-15; passing `<DIR>/spec` as the reference root killed AC-24; loading only the first `--repo` member in `validate --okf` killed AC-20 and AC-24; changing the untyped kind-mismatch text survived (FND-004).

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-004 | medium | The kind-mismatch branch for an untyped target (`KindMismatchEntry.archetype == None`, rendered `is an untyped document`) has no test. It is reachable: a gold rule with an `addresses` edge to a document without `type:` prints `kind mismatch: o/a:UNTYPED-1 is an untyped document (via addresses), accepted: Concern`, and its JSON entry omits `archetype`. Replacing the text with `MUTATED` left all 14 cli_relations tests green. Add an AC-7 case with an untyped target, asserting both the human line and the absent `archetype` key. | src/commands/relations.rs:228, tests/cli_relations.rs:345-367 |
| FND-005 | low | The AC-20 and AC-24 member-identity checks are substring matches for `o/a` / `o/b` on stderr lines that also carry the absolute tempdir path (`<tmp>/a/spec/...`). A tempdir name ending in `o` makes `o/a` match without the `[o/a]` prefix, so the prefix mutation is killed only by chance of the random name. The AC-24 excluded half also asserts only absence, with no exit status (as FND-002 for AC-20). Assert the `[o/a] ` / `[o/b] ` prefix and the exit status. | tests/cli_relations.rs:660-665, tests/cli_relations.rs:726-737 |
