---
id: FR-014
title: "quire validate --okf bundle posture"
type: FR
object_type: cli_command
relationships:
  - target: "ix://agent-ix/quire-cli/spec/usecase/US-003"
    type: "implements"
    cardinality: "1:1"
  - target: "ix://agent-ix/quire-cli/spec/functional/FR-004"
    type: "extends"
    cardinality: "1:1"
  - target: "ix://agent-ix/quire-rs/spec/functional/FR-032"
    type: "consumes"
    cardinality: "1:1"
---

> **CR note (OKF bundle posture, 2026-06-16):** new `--okf` flag on the existing
> `validate` subcommand ([FR-004](./FR-004-validate-subcommand.md)). It validates a **directory as an OKF bundle**
> under the permissive (`BundlePosture::Okf`) posture by delegating to the new
> quire-rs `validate_bundle_at(root, &registry, BundlePosture::Okf)` API — distinct
> from the per-file strict path of [FR-004](./FR-004-validate-subcommand.md), which is unchanged when `--okf` is
> absent (backward compatible). The bundle-validation logic lives entirely in
> quire-rs; the CLI surfaces the returned `BundleReport` warnings/errors on stderr
> ([StR-004](../stakeholder/StR-004-thin-boundary-over-quire-rs.md) thin boundary). Verified by `tests/cli_okf.rs`.

> **CR note (`type` is the discriminator, not `artifact_type`, 2026-06-16):** OKF
> adoption renamed the archetype discriminator frontmatter key to `type`. This FR
> and its ACs use `type` throughout. The base-concept contract (see Behavior §B)
> requires `type` non-empty in **both** postures; the rename backsync corrected
> remaining `artifact_type` prose elsewhere in the spec ([FR-003](./FR-003-extract-subcommand.md), [FR-004](./FR-004-validate-subcommand.md), [FR-013](./FR-013-lint-subcommand.md),
> [FR-007](./FR-007-exit-codes.md), spec.md) — see each artifact's CR note and `spec/log.md`.

## Description

The CLI SHALL add an `--okf` flag to the `validate` subcommand that validates a
directory wholesale as an OKF bundle under the permissive
(`BundlePosture::Okf`) posture, plus enforce the base-concept contract (`type`
required + non-empty) in both postures, delegating all bundle and concept
validation to `quire-rs`. The behavioral surface is specified below.

## Behavior

### §A — `--okf` bundle validation

The CLI SHALL accept an `--okf` flag on the `validate` subcommand:

```
quire validate [<DIR>] --okf [--scope <DIR>] [--module <PATH>]
```

With `--okf`, the positional argument (or `--scope` when no positional is given)
names the **bundle root directory**, validated wholesale as an OKF bundle under
the permissive posture via `quire_rs::validate_bundle_at(root, &registry,
BundlePosture::Okf)`. The returned `BundleReport` is surfaced on stderr:
warnings (non-fatal) first, then errors, in the shared quire-rs diagnostic shape
(`<path>: <message> [<reason>]`).

Posture semantics (permissive):
- `type` is still **required + non-empty** on every document — an untyped
  document is a **hard error** (exit 1).
- An **unknown type** (a `type` not registered in the loaded module) is a
  **WARNING** (`[unknown-type]`), not an error.
- A **broken `ix://` link** (dangling cross-reference) is a **WARNING**
  (`[dangling-reference]`), not an error.
- An **`index.md` completeness gap** — a sibling artifact not listed in its
  directory `index.md`, or a root `index.md` missing `okf_version` — is a
  **WARNING** (`[index-incomplete]`), not an error.

Exit code: **1** when the report carries any hard error (e.g. an untyped
document); otherwise **0**, even when warnings were emitted. `--okf` writes
nothing to stdout.

The bundle root is the positional directory; when no positional is given it is
`--scope`. Path-safety ([FR-005](./FR-005-path-safety.md)) applies to the resolved bundle root.

### §B — Base concept contract (both postures)

The base concept contract is enforced **upstream in quire-rs for every validated
document**, surfaced through the existing `validate` diagnostics:
- `type` is **required and non-empty** (an untyped document is a hard error,
  exit 1, in both the strict per-file path of [FR-004](./FR-004-validate-subcommand.md) and the `--okf` bundle path);
- optional `description` / `tags` are typed when present.

### §C — `documents` arg becomes `required_unless_present = "okf"`

The positional `documents` argument is now `required_unless_present = "okf"`:
- `quire validate` with no positional **and** no `--okf` remains a clap argv
  error → **exit 2** (unchanged; the strict per-file path needs a document).
- `quire validate --okf` with no positional is **valid**: the bundle root
  defaults to `--scope`.

## Acceptance Criteria

| ID | Criteria | Verification |
|----|----------|--------------|
| FR-014-AC-1 | `quire validate --okf <DIR> --module $M` over a bundle whose document has **no `type`** exits **1**; stderr carries a `[frontmatter]`-reason diagnostic naming `type` (`type` required under OKF) | Test |
| FR-014-AC-2 | An **unknown `type`** under `--okf` is a **warning**: the command exits **0** and stderr carries an `[unknown-type]` diagnostic | Test |
| FR-014-AC-3 | A **broken `ix://` link** under `--okf` is a **warning**: the command exits **0** and stderr carries a `[dangling-reference]` diagnostic | Test |
| FR-014-AC-4 | An **`index.md` that omits a sibling artifact** under `--okf` is a **warning**: the command exits **0** and stderr carries an `[index-incomplete]` diagnostic naming the missing artifact | Test |
| FR-014-AC-5 | A root `index.md` **missing `okf_version`** is reported as an `[index-incomplete]` warning (exit 0), consistent with AC-4's completeness contract | Test |
| FR-014-AC-6 | `quire validate --okf --scope <DIR> --module $M` with **no positional** validates the `--scope` directory as the bundle root (exit 0 for a warning-only bundle) | Test |
| FR-014-AC-7 | `quire validate` with **no positional and no `--okf`** is a clap argv error → **exit 2** (`required_unless_present = "okf"`), unchanged from [FR-004](./FR-004-validate-subcommand.md) | Test |
| FR-014-AC-8 | (base concept contract) an untyped document is a hard error (exit 1, `[frontmatter]` diagnostic naming `type`) in **both** postures — strict per-file ([FR-004](./FR-004-validate-subcommand.md)) and `--okf` bundle — because quire-rs enforces `type` required + non-empty for every validated document | Test |
| FR-014-AC-9 | (thin boundary) all bundle/base-concept validation is delegated to quire-rs (`validate_bundle_at`, base-concept enforcement); the CLI only resolves the root, applies path-safety, and surfaces the `BundleReport` ([StR-004](../stakeholder/StR-004-thin-boundary-over-quire-rs.md); AUDIT-002) | Inspection |

## Dependencies

- **Upstream**: [US-003](../usecase/US-003-ci-validates-archetype-conformance.md) CI validates archetype conformance; [FR-004](./FR-004-validate-subcommand.md) validate (extends); quire-rs FR-032 (`validate_bundle_at`).
- **Downstream**: OKF bundle CI gates consuming the permissive posture.
