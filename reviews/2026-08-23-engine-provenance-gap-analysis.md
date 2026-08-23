---
id: SR-056
title: "Gap analysis — engine provenance, quire-cli#68 (EPIC quire-rs#264 Wave 0)"
type: SpecReview
analysis: gap-analysis
scope: "spec/functional/FR-008-json-output-encoding.md, spec/tests.md, src/engine.rs, src/lockfile.rs, build.rs, src/commands/, tests/cli_provenance.rs, tests/output_contract.rs; and agent-ix/quire-rs FR-055"
review_set: subset
---

# SR-056: Gap analysis — engine provenance (`quire-cli#68`)

## Summary

Post-implementation gate on `agent-ix/quire-cli#68`, Wave 0 of EPIC
`agent-ix/quire-rs#264`, across both repositories. There is no plan bundle —
the unit of work is a GitHub ticket with five acceptance checkboxes — so Step 1
was run against the ticket's own AC list instead.

Matrix verification was run with the engine under test (`quire coverage --json`
from this branch's build, not the installed 0.29.0 binary — which is the defect
this ticket exists to make visible). **Every row this change added binds to a
real tagged test in both repositories.**

One real gap was found against the ticket's own acceptance criteria and closed
before this document was written: `#68` AC-4 asks for a golden snapshot covering
the envelope, extending `#60`, and none existed — `tests/snapshots/` held only
`help.txt`. `IT-129` and `tests/snapshots/extract-envelope.json` now cover it.

One deviation from the ticket's wording is deliberate and recorded below rather
than fixed.

## Verdict

**CONDITIONAL** — no unbacked new rows and no incomplete acceptance criterion,
with one documented scope deviation (`validate` carries no provenance, because
it emits no JSON payload) and one pre-existing matrix debt this change did not
introduce and does not close.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | medium | `#68` AC-4 ("golden snapshot covers the envelope") had no implementation; found by this analysis and closed | tests/snapshots/extract-envelope.json:1 |
| FND-002 | low | `validate` carries no provenance — it emits no JSON payload, so the ticket's four-command list is unsatisfiable as written | src/commands/validate.rs:1 |
| FND-003 | low | `parse`, `lookup` and `schema` deliberately carry no provenance: their stdout IS an engine value, not a CLI envelope | src/commands/parse.rs:21 |
| FND-004 | low | 23 pre-existing unbacked rows in quire-cli and 280 in quire-rs, none introduced or closed here | spec/tests.md:1 |

## Coverage

### The ticket's acceptance criteria

| `#68` AC | State | Evidence |
|---|---|---|
| `quire --version` reports CLI and engine distinctly | done | `IT-123` — asserts the engine version **by value**, read from the lockfile at the process boundary |
| Every JSON payload carries `engine.{cli,engine,capabilities}` | done, with FND-002/003 | `IT-124` extract, `IT-125` properties, `IT-126` coverage |
| A `-<n>-g<sha>` suffix is reported verbatim, never rounded | done | 11 `lockfile` unit tests incl. tag/rev/branch/describe/sha forms; quire-rs `TC-1010` accepts the describe form |
| Golden snapshot covers the envelope (extends `#60`) | **done by this analysis** | `IT-129` + `tests/snapshots/extract-envelope.json` |
| Consumers can assert a capability token without parsing a version | done | `IT-124`/`IT-125`/`IT-126` each assert `binding_census ∈ capabilities`; the published schemas leave the vocabulary unenumerated |

### Matrix verification

Run with this branch's binary, both repositories:

| | quire-cli | quire-rs |
|---|---|---|
| rows backed / total | 262 / 304 | 938 / 1288 |
| binder read rate | rust 154 / 189 | rust 597 / 889, python 18 / 79 |
| untracked symbols | 0 | — |
| **rows added here, unbacked** | **0** | **0** |

`IT-100`, `IT-123`…`IT-129` and `TC-1010` all resolve to real tagged tests.
`FR-008-AC-5`, `FR-008-AC-6` and `FR-055-AC-8` each appear in the authoritative
AC→TC audit tables — the last of those only after the code review caught its
absence (SR-055 FND-005).

### Underspecified code

Every module added carries an owning requirement:

- `src/engine.rs`, `src/commands/*` wiring — `FR-008-AC-6`.
- `src/lockfile.rs`, `build.rs` — `FR-008-AC-6` names the lockfile as the source
  explicitly, so the mechanism is specified rather than incidental.
- quire-rs `schemas/output/*` `EngineProvenance` — `FR-055-AC-8`.

No stub, `todo!()`, or placeholder return was found in the change.

### Deviations, stated rather than silently absorbed

`#68` says "every JSON payload (`coverage`, `properties`, `validate`,
`extract`)". **`validate` emits no JSON payload** — it writes diagnostics to
stderr and nothing to stdout (FR-004, CR-012) — so there is nowhere to attach
provenance, and the ticket's list is unsatisfiable as written rather than
partially implemented.

`parse`, `lookup` and `schema` are excluded on a principle worth recording,
because the next person to add a payload will face it: their stdout **is** an
engine value — a `QuireDocument`, a section, an archetype schema — not an
envelope the CLI assembled. Attaching a key would modify a value the engine
owns, which FR-008 behaviour rule 5 forbids, and would break `FR-008-AC-1`'s
round-trip through `from_str::<QuireDocument>`. Provenance goes on envelopes the
CLI builds; it never annotates the engine's own values.

### Semantic review

**Not run.** Steps 1–3 are complete; the optional intent↔test↔code pass was not
requested. The adversarial equivalent for this change was run separately and is
recorded in SR-055, which found one Critical and five High defects that every
green gate had passed.
