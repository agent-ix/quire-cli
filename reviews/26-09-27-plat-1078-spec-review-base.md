---
id: SR-065
title: "Base review of FR-026 quire matrix subcommand"
type: SpecReview
analysis: base
scope: "agent-ix/quire-cli@72c5c94ef3c9bd6557f98c3edf517b353812c23f; spec/functional/FR-026-matrix-subcommand.md, spec/functional/index.md, spec/tests.md (upstream contract read: agent-ix/quire-rs@af5ec21 spec/functional/FR-050-declarative-coverage-computation.md AC-47..51, FR-051 AC-27/28)"
review_set: subset
---

## Summary

Ticket: PLAT-1078 (PR agent-ix/quire-cli#103). Base checklist review of the new
FR-026 `quire matrix` spec against the upstream quire-rs FR-050-AC-47..51 /
FR-051-AC-27/28 contract (merged quire-rs#494). ID formats, index entry and
FR frontmatter are sound; `quire validate` over the three touched files exits 0
with no grammar findings on FR-026. One contract defect blocks: the markdown
`Statement` column and the TSV `statement` field render a field the upstream
`coverage_matrix` criterion does not carry.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | high | Statement column/field has no upstream source: FR-050-AC-48 criterion carries only id, method, binders, status; rendering a statement forces the CLI to re-derive obligations, contradicting §A and StR-004 | spec/functional/FR-026-matrix-subcommand.md:94-104, :173, :175, :119-128 |
| FND-002 | medium | Markdown exact outputs are not pinned: zero-population line text, level-2 heading form, and pipe escaping of statement text (FR-026-AC-4's own text contains a pipe) | spec/functional/FR-026-matrix-subcommand.md:78-80, :171, :176 |
| FND-003 | low | JSON section calls the omitted value an empty array; upstream coverage_matrix is an object carrying requirements[] | spec/functional/FR-026-matrix-subcommand.md:108-112, :174 |
| FND-004 | low | "writes only to stdout" contradicts stderr refusal diagnostics; "a fourth, non-error state" names no three others | spec/functional/FR-026-matrix-subcommand.md:36, :63 |

## FND-001 detail

Upstream FR-050-AC-48 (quire-rs@af5ec21 line 190): "A criterion entry carries
the obligation's own `id`, its declared `method` when the obligation states
one, a `binders` list, and the computed `status`". No `statement`. The FR-053
`Obligation` record has `statement`, but it is not on the matrix criterion.
FR-026 §D (94-104), AC-6 (173), §F (119-128) and AC-8 (175) render a
statement. Failure scenario: the implementer either joins `obligation::derive`
output or re-reads documents to fetch the text (a second computation the §A
text at 51-55 forbids, and the brief's "CLI must not re-derive anything
upstream owns"), or IT-170/IT-172 are unimplementable against the upstream
field. Fix: get PLAT-1077 to add `statement` to the criterion (AC-48) before
it ships, or drop the Statement column, AC-6 and the TSV statement field.

## FND-002 detail

AC-9 requires "a stated zero-population line" with no text; AC-4 says the
heading "names" the document path without its exact form (plain, backticked,
linked). Neither is falsifiable to an exact byte output, which IT-168/IT-173
need. A statement containing `|` (FR-026-AC-4 itself contains `\|`) splits
the table row into extra columns; the ACs do not say whether pipes are
escaped, and whether the escape counts toward the 80 scalar values or can be
cut in half by the 77-char truncation (leaving a dangling backslash).

## Verdict

**NOT MERGEABLE** until FND-001 is resolved (upstream field added, or the
statement surface removed). FND-002 should be fixed in the same round.

## New findings (disposition pass 1)

Reviewed at agent-ix/quire-cli@370f4b95ec1172ada6b5092a3608406de976a76d.

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-005 | low | Worked example states its statement is 63 characters; it is 61 scalar values, so a test asserting the pinned figure fails | spec/functional/FR-026-matrix-subcommand.md:116-117 |
| FND-006 | low | §A calls the upstream statement "already-normalized"; FR-053 defines Obligation.statement as the cell "verbatim and untruncated" (normalization applies only to statement_hash). The CLI's own scrub makes it harmless, but the upstream contract is misdescribed | spec/functional/FR-026-matrix-subcommand.md:62-63 |

## Dispositions

| FND | Outcome | sha/reason |
| --- | --- | --- |
| FND-001 | fixed | 370f4b9 — §A (59-66), §D (144), AC-6 read `statement` from the criterion as amended by PLAT-1077 and forbid re-derivation; citation accepted per leader ruling (upstream amendment not yet landed) |
| FND-002 | fixed | 370f4b9 — exact `## <document>` heading (§C 92-97, AC-4), literal zero line `No obligations matched this scope.` (99-104, AC-9), `\|` escaping after truncation (108-126, AC-6) |
| FND-003 | fixed | 370f4b9 — §E 165-167 and AC-7 describe an object carrying `requirements[]`, never a bare array |
| FND-004 | fixed | 370f4b9 — Description 22-24 states stdout result / stderr diagnostics (FR-006); "fourth, non-error state" removed |

### Dispositions — round 2

Reviewed at agent-ix/quire-cli@587f609eadc9e17d8fd72eb84673310dd3895226. Regression check of the 370f4b9..587f609 diff: only FR-026 lines 59-65 and 119 changed in spec; no new defect.

| FND | Outcome | sha/reason |
| --- | --- | --- |
| FND-005 | fixed | 587f609 — worked example now states 61 characters, matching the statement's scalar-value count |
| FND-006 | fixed | 587f609 — §A now says the statement is carried verbatim and untruncated, citing FR-053 |
