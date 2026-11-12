---
id: SR-20261110162004
title: "Integrity analysis of validate --okf --strict dangling references (PR #167)"
type: SpecReview
analysis: integrity
scope: "agent-ix/quire-cli; spec/functional/FR-014-validate-okf-bundle.md, spec/tests.md"
review_set: subset
---
# Integrity analysis of validate --okf --strict dangling references (PR #167)

## Summary

Ticket: PLAT-1407. This checks that the new FR-014-AC-11 row is consistent with
the rest of FR-014, with the coverage matrix in spec/tests.md, and with the
dependency graph. `quire validate --scope . spec/functional/FR-014-validate-okf-bundle.md`
exits 0. `quire validate --okf --scope .` exits 0 with warnings only, all of them
already on main.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | medium | FR-014's Description and §A still say `--okf` validates under the permissive `BundlePosture::Okf` posture. The §A synopsis `quire validate [<DIR>] --okf [--scope <DIR>] [--module <PATH>]` has no `--strict`. AC-11 adds a behavior that has no Behavior text, and the statement now contradicts it | spec/functional/FR-014-validate-okf-bundle.md:34-56 |
| FND-002 | low | The FR-014 row in spec/tests.md still reads "AC-1..10" and is marked ✅. AC-11 has no open entry, so the matrix row claims FR-014 is fully backed | spec/tests.md:74 |
| FND-003 | low | AC-11 depends on quire-rs FR-038-AC-11 (open, quire-rs#646), but FR-014's `relationships` and `## Dependencies` record only quire-rs FR-032. The dependency is not in the graph | spec/functional/FR-014-validate-okf-bundle.md:1-16,133-135 |

## Verdict

PASS with findings. Fix FND-001 in this PR: add `--strict` to the §A synopsis,
and add a short "with `--strict`" paragraph under §A that names the Strict
posture and its exit semantics. Extend the tests.md FR-014 row to AC-11 marked
open, and add the quire-rs FR-038 dependency edge.

## Dispositions

| FND | outcome | reason |
| --- | --- | --- |
| FND-001 | still-open | f031077 fixed the Description and the §A synopsis. §A's exit-code paragraph still reads "otherwise **0**, even when warnings were emitted" with no `--strict` qualifier, which contradicts AC-11 (any finding exits 3). The inline call still passes `BundlePosture::Okf` unconditionally. Add one sentence: with `--strict`, any finding exits 3 (FR-014-AC-11) |
| FND-002 | fixed | f031077: the FR-014 row in spec/tests.md now carries "AC-11 🚧 open: `--okf --strict` has no IT yet; depends on quire-rs FR-038-AC-11" |
| FND-003 | fixed | f031077 adds a `consumes` relationship to `ix://agent-ix/quire-rs/spec/functional/FR-038` and names FR-038-AC-11 (quire-rs#646) under Dependencies |
