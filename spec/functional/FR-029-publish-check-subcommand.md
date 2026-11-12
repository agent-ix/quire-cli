---
id: FR-029
title: "quire publish-check subcommand"
type: FR
relationships:
  - target: "ix://agent-ix/quire-cli/spec/stakeholder/StR-004"
    type: "implements"
    cardinality: "1:1"
  - target: "ix://agent-ix/quire-cli/spec/functional/FR-021"
    type: "references"
  - target: "ix://agent-ix/quire-rs/spec/functional/FR-083"
    type: "references"
---
# FR-029: quire publish-check subcommand

## Description

The CLI SHALL expose the engine's publishability check
([quire-rs FR-083](ix://agent-ix/quire-rs/FR-083)) so a publish step can refuse
a module that carries licence-restricted clause text.

```text
quire publish-check --module <PATH> [--format human|json]
```

The command path-checks and strictly loads one module, calls the engine's
`check_publishable`, and renders the result. It is a check only: it publishes,
packs and installs nothing, and writes no file. The CLI holds no rights policy;
which clause sets are restricted is decided entirely by the engine.

No existing publish flow for modules exists in this repository (FR-022 covers
only the CLI's own distribution), so this command is the hook point for any
module publish step.

## Acceptance Criteria

| ID | Criteria | Verification |
|----|----------|--------------|
| FR-029-AC-1 | For a synthetic module whose clause sets are all publishable, the command exits 0, writes the line `no restricted clause sets` to stdout, and writes nothing to stderr. | Test |
| FR-029-AC-2 | For a synthetic `example.invalid` module with restricted sets, the command exits 2 (`Outcome::Refused`) and stderr names each restricted set as `authority/id@version`, one per line, in the engine's order. Two restricted sets are both named. | Test |
| FR-029-AC-3 | `--format json` writes to stdout an object with a `restricted` array (empty when clean) whose entries carry `authority`, `id`, `version` and `text_clauses`, plus tool provenance. Under `--format json` stderr carries only the refusal diagnostic naming the sets (nothing when clean). The exit code is unchanged by the format. | Test |
| FR-029-AC-4 | Neither stdout nor stderr contains clause text from the fixture in any format; a test searches for the fixture's text sentinel and fails if found. | Test |
| FR-029-AC-5 | `..` in `--module` exits 2 (FR-005 path refusal). A missing module path, or a module whose clause set fails strict load, exits 3; a `clause_sets` symlink that resolves outside the module root is an engine strict-load failure and exits 3. None of these prints the clean verdict. | Test |
| FR-029-AC-6 | The module directory is byte-identical after any run, and the command opens no network socket, as verified for every subcommand by NFR-004-AC-2. | Test |
| FR-029-AC-7 | The exit code is a function of the engine verdict only: given a hand-built `Vec<RestrictedClauseSet>`, an empty vector maps to exit 0 and a non-empty vector maps to exit 2. | Test |
| FR-029-AC-8 | The publish-check module source contains none of the tokens `citation_only`, `clearance_ref`, `rights` or `licensed`, so no rights policy lives in the CLI (FR-021-AC-7). | Test, Review |

## Constraints

The repository and its fixtures SHALL contain only original synthetic clause
content. Restricted source material is not a CLI fixture.

## Dependencies

- **Upstream:** [quire-rs FR-083](ix://agent-ix/quire-rs/FR-083) owns the
  restricted-versus-publishable rule and `check_publishable`.
- **Local boundary:** [StR-004](../stakeholder/StR-004-thin-boundary-over-quire-rs.md);
  [FR-005](./FR-005-path-safety.md), [FR-006](./FR-006-io-contract.md),
  [FR-007](./FR-007-exit-codes.md) govern path checks, streams and outcomes.
- **Ticket:** PLAT-1419 (spec), PLAT-1421 (implementation).
