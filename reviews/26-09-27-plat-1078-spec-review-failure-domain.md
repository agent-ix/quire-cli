---
id: SR-066
title: "Failure-domain review of FR-026 quire matrix subcommand"
type: SpecReview
analysis: failure-domain
scope: "agent-ix/quire-cli@72c5c94ef3c9bd6557f98c3edf517b353812c23f; spec/functional/FR-026-matrix-subcommand.md, spec/functional/index.md, spec/tests.md (upstream contract read: agent-ix/quire-rs@af5ec21 spec/functional/FR-050-declarative-coverage-computation.md AC-47..51, FR-051 AC-27/28)"
review_set: subset
---

## Summary

Ticket: PLAT-1078. Examined FR-026's unstated failure modes: zero population,
omission semantics, strict/severity interplay with FR-017, binder identity.
Two medium gaps where an exit code is unspecified or contradicts itself; one
low identity-collapse in binder rendering.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | medium | Present-but-empty coverage_matrix is unhandled and §B overstates FR-050-AC-51; --strict would pass vacuously on it | spec/functional/FR-026-matrix-subcommand.md:59-68, :110-111, :132-135, :174, :176-177 |
| FND-002 | medium | AC-12 says --severity changes nothing in the --strict verdict yet an error promotion fails the run; the hidden failing finding is never surfaced, and FR-017's strict unread-measurement gate is unaddressed | spec/functional/FR-026-matrix-subcommand.md:142-154, :179 |
| FND-003 | low | Binders render as path:line, collapsing distinct same-line binders that upstream keeps distinct by column | spec/functional/FR-026-matrix-subcommand.md:85-86, :172 |

## FND-001 detail

FR-050-AC-51 pins presence when the population is non-empty and omission
"when a module declares no `obligations:` source and so derives none". It
does not state what happens when a source is declared but derives nothing.
FR-026 §B (59-62) asserts "derives none is the one condition, regardless of
which reason"; the upstream text does not say that. If the engine emits
`{"requirements": []}` for that case, §E (108-111) passes it verbatim (key
present), AC-9 (176) says never present-but-empty, and §G's exit-0 condition
(134-135, "every criterion computes tagged or method-without-symbol and the
field is present") is vacuously true: `--strict` exits 0 on a zero population,
the CR-035 hollow pass AC-10 exists to prevent. Fix: make --strict and the
markdown zero-population line key on zero criteria, not key absence, or have
PLAT-1077 pin omission for both reasons in AC-51.

## FND-002 detail

AC-12 (179): "changes nothing in `matrix`'s rendered output or `--strict`
verdict ... an `error`-promoted check with findings elsewhere in the same
computed `CoverageReport` still fails the run". Per FR-017-AC-13 an `error`
promotion exits 1 without `--strict`, and module `grammar_severity` can promote
with no flag at all. Scenario: a repo whose matrix is fully tagged still has a
legacy spec/tests.md unbacked row; `quire matrix --severity
coverage:unbacked-row=error` exits 1, the matrix output shows every criterion
tagged, and no AC requires the failing check and count to be named on stderr.
Separately, FR-017 CR note (lines 21-28) and FR-017-AC-22 make `coverage
--strict` fail on `status-column-matches-nothing`/`hollow-denominator`;
FR-026 neither inherits nor excludes that. Fix: state exactly which non-matrix
report findings affect matrix's exit code and require a stderr diagnostic
naming them when they do.

## FND-003 detail

FR-050-AC-48 insists binders are distinct per `(path, line, column)` so two
same-titled `it(...)` calls are not collapsed. Two registrations on one line
render `a.ts:5, a.ts:5` under FR-026-AC-5. Render `path:line:column` or state
the collapse is accepted.

## Verdict

Two medium findings to fix in this PR; not mergeable with them open.

## Dispositions

Reviewed at agent-ix/quire-cli@370f4b95ec1172ada6b5092a3608406de976a76d.

| FND | Outcome | sha/reason |
| --- | --- | --- |
| FND-001 | fixed | 370f4b9 — §B 68-82 defines zero population as zero criteria in either shape; §G 202-206 and AC-10 require at least one criterion to exit 0 under --strict; IT-173/IT-174 cover both shapes |
| FND-002 | fixed | 370f4b9 — §G 190-223 and AC-12: `--severity` is not accepted (clap exit 2, FR-007-AC-5), grammar_severity promotions have no effect, FR-017-AC-22 explicitly not inherited with a stated reason; IT-176 rewritten to match |
| FND-003 | fixed | 370f4b9 — §D 130-137, §F 180, AC-5, AC-8 render binders as path:line:column; IT-169 asserts no same-line collapse |
