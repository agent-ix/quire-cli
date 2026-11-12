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
  - target: "ix://agent-ix/quire-rs/spec/functional/FR-038"
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
> [FR-007](./FR-007-exit-codes.md), spec.md) — see each artifact's CR note.

## Description

The CLI SHALL add an `--okf` flag to the `validate` subcommand that validates a
directory wholesale as an OKF bundle under the permissive
(`BundlePosture::Okf`) posture by default, or under `BundlePosture::Strict`
when `--strict` is also given (§A), plus enforce the base-concept contract (`type`
required + non-empty) in both postures, delegating all bundle and concept
validation to `quire-rs`. The behavioral surface is specified below.

## Behavior

### §A — `--okf` bundle validation

The CLI SHALL accept an `--okf` flag on the `validate` subcommand:

```
quire validate [<DIR>] --okf [--strict] [--scope <DIR>] [--module <PATH>]
```

With `--okf`, the **bundle root** is the positional argument when one is given,
and `<scope>/spec` when none is (CR note below). It is validated wholesale as an
OKF bundle under the permissive posture (`BundlePosture::Strict` when `--strict` is given, FR-014-AC-11) via `quire_rs::validate_bundle(&spec,
&registry, posture, document_root, reference_root)`, where `posture` is `BundlePosture::Okf` unless `--strict` is given, in which case it is `BundlePosture::Strict` — two roots, so
the module's path-bound declarations keep resolving against the repository scope
(quire-rs FR-049-AC-9). The returned `BundleReport` is surfaced on stderr:
warnings (non-fatal) first, then errors, in the shared quire-rs diagnostic shape
(`<path>: <message> [<reason>]`). A finding the engine locates (`BundleFinding.line`,
quire-rs FR-049-AC-19..22) carries its line exactly as a located document diagnostic does
(`<path>: line <N>: <message> [<reason>]`, FR-014-AC-12) and its JSON record gains a numeric
`line` field with its `path` and `reason` (FR-014-AC-13). Record order is FR-014-AC-16.

Posture semantics (permissive):
- `type` is still **required + non-empty** on every document — an untyped
  document is a **hard error** (exit 3).
- An **unknown type** (a `type` not registered in the loaded module) is a
  **WARNING** (`[unknown-type]`), not an error.
- A **broken `ix://` link** (dangling cross-reference) is a **WARNING**
  (`[dangling-reference]`), not an error.
- An **`index.md` completeness gap** — a sibling artifact not listed in its
  directory `index.md`, or a root `index.md` missing `okf_version` — is a
  **WARNING** (`[index-incomplete]`), not an error.

Exit code: **1** when the report carries any hard error (e.g. an untyped
document); otherwise **0**, even when warnings were emitted. With `--strict`, any finding exits 3 (FR-014-AC-11). `--okf` writes
nothing to stdout.

The bundle root is the positional directory; when no positional is given it is
`--scope`. Path-safety ([FR-005](./FR-005-path-safety.md)) applies to the resolved bundle root.

### §B — Base concept contract (both postures)

The base concept contract is enforced **upstream in quire-rs for every validated
document**, surfaced through the existing `validate` diagnostics:
- `type` is **required and non-empty** (an untyped document is a hard error,
  exit 3, in both the strict per-file path of [FR-004](./FR-004-validate-subcommand.md) and the `--okf` bundle path);
- optional `description` / `tags` are typed when present.

### §C — `documents` arg becomes `required_unless_present = "okf"`

The positional `documents` argument is now `required_unless_present = "okf"`:
- `quire validate` with no positional **and** no `--okf` remains a clap argv
  error → **exit 3** (unchanged; the strict per-file path needs a document).
- `quire validate --okf` with no positional is **valid**: the bundle root is
  `<scope>/spec`. A scope with no `spec/` directory is a **named error**, never
  a silent fallback to walking the scope (CR note below).

## Acceptance Criteria

| ID | Criteria | Verification |
|----|----------|--------------|
| FR-014-AC-1 | `quire validate --okf <DIR> --module $M` over a bundle whose document has **no `type`** exits **3**; stderr carries a `[frontmatter]`-reason diagnostic naming `type` (`type` required under OKF) | Test |
| FR-014-AC-2 | An **unknown `type`** under `--okf` is a **warning**: the command exits **0** and stderr carries an `[unknown-type]` diagnostic | Test |
| FR-014-AC-3 | A **broken `ix://` link** under the bundle's own authority (or with no package declared, per FR-014-AC-10) under `--okf` is a **warning**: the command exits **0** and stderr carries a `[dangling-reference]` diagnostic | Test |
| FR-014-AC-4 | An **`index.md` that omits a sibling artifact** under `--okf` is a **warning**: the command exits **0** and stderr carries an `[index-incomplete]` diagnostic naming the missing artifact | Test |
| FR-014-AC-5 | A root `index.md` **missing `okf_version`** is reported as an `[index-incomplete]` warning (exit 0), consistent with AC-4's completeness contract | Test |
| FR-014-AC-6 | `quire validate --okf --scope <DIR> --module $M` with **no positional** validates `<DIR>/spec` as the bundle root, with `<DIR>` itself kept as the reference root (exit 0 for a warning-only bundle); a `<DIR>` holding no `spec/` exits non-zero with a diagnostic naming the missing document root, never falling back to walking `<DIR>` | Test |
| FR-014-AC-7 | `quire validate` with **no positional and no `--okf`** is a clap argv error → **exit 3** (`required_unless_present = "okf"`), unchanged from [FR-004](./FR-004-validate-subcommand.md) | Test |
| FR-014-AC-8 | (base concept contract) an untyped document is a hard error (exit 3, `[frontmatter]` diagnostic naming `type`) in **both** postures — strict per-file ([FR-004](./FR-004-validate-subcommand.md)) and `--okf` bundle — because quire-rs enforces `type` required + non-empty for every validated document | Test |
| FR-014-AC-9 | (thin boundary) all bundle/base-concept validation is delegated to quire-rs (`validate_bundle`, base-concept enforcement); the CLI only resolves the root, applies path-safety, and surfaces the `BundleReport` ([StR-004](../stakeholder/StR-004-thin-boundary-over-quire-rs.md)) | Inspection |
| FR-014-AC-10 | Under `--okf` the bundle package (`<org>/<repo>`, lowercased) is declared from the `origin` remote of the checkout containing each bundle's document root — not the caller's cwd or scope (quire-rs FR-076). A dangling `ix://<other-org>/<other-repo>/ID` link under a **foreign** authority is skipped (no diagnostic); a miss under the bundle's **own** authority still warns `[dangling-reference]`; with **no** package declared (no git checkout or no `origin`) every miss warns | Test |
| FR-014-AC-11 | `quire validate --okf --strict <DIR>` runs the bundle under quire-rs `BundlePosture::Strict` and also applies FR-004's `--strict` escalation (FR-004 step 5, FR-004-AC-11, FR-004-AC-22): every finding Strict turns into an error (`unknown-type`, `index-incomplete`, `index-okf-version`, archetype body errors, `dangling-reference`) and every finding that remains a warning (for example the FR-040-AC-10 edge-target checks) exits **3**, so any finding at all exits 3 and a finding-free bundle exits 0. A `dangling-reference` whose target is a bare id or sits under the bundle's own authority (FR-014-AC-10) therefore exits 3, and stderr prints the source document **path**, the edge type and the target as harvested (quire-rs FR-038-AC-11). A target under a foreign authority is skipped (quire-rs FR-038-AC-9, FR-014-AC-10) and contributes no finding, so a bundle that is otherwise Strict-conformant and has only foreign-authority targets exits 0. A miss with no package declared, and a two-segment `ix://<org>/<id>` link to an absent id, are dangling and exit 3 (FR-014-AC-10, quire-rs FR-038-AC-9). Without `--strict`, FR-014-AC-3 is unchanged. | Test |
| FR-014-AC-12 | A bundle finding whose `BundleFinding.line` is `Some(N)` — a `dangling-trace-reference` or `reference-kind-mismatch` finding (quire-rs FR-049-AC-19/20) at its referencing row, or a bridged document error/warning (quire-rs FR-049-AC-22), under `--strict` — is printed in the located form `<path>: line <N>: <message> [<reason>]`, the same `line <N>: ` prefix located document diagnostics use (FR-004-AC-2) | Test |
| FR-014-AC-13 | With `--diagnostics-format json`, the record of a finding with `Some(N)` carries a numeric `line` field equal to `N`, paired with `path` (the finding's document path) and `reason` (its reason token) as every other located JSON diagnostic does (FR-004 document validation); a finding with `line` `None` carries none of `line`, `path` or `reason` | Test |
| FR-014-AC-14 | A bridged document error under `--okf --strict` carries the line of its document-validation result: the line printed by single-document `quire validate` for the same document (FR-004-AC-2) is the line printed by the bundle form | Test |
| FR-014-AC-15 | A finding with `line` `None` renders byte-identically to the pre-FR-014-AC-12 form `<path>: <message> [<reason>]` in human output and as the same JSON record, with no `line`, `path` or `reason` key | Test |
| FR-014-AC-16 | Bundle diagnostics are emitted warnings first, then errors, each in the engine's order, in both human and JSON output, whether or not they carry a line | Test |

> **CR note (two roots from one scope, 2026-08-15):** AC-6 previously read
> "validates the `--scope` directory as the bundle root". That is no longer what
> ships, and the behavior changed in PR #27 without a spec edit — which this
> note corrects rather than the other way round. Upstream quire-rs CR-045 split
> the single conflated root into a **document root** (`<scope>/spec`, what the
> corpus is walked from) and a **reference root** (`<scope>`, what a module's
> `document:`/`exclude:` declarations resolve against), because a corpus walked
> from `<scope>/spec` with one conflated root silently un-minted every
> path-bound trace target — 123 new `dangling-trace-reference` findings on
> quire-rs's own spec. The same split removed the repository-wide crawl that had
> been reading `README.md`, `CHANGELOG.md` and `plan/*.md` as documents; 9,172
> `required 'type' is missing` errors across 223 repos went with it.
>
> This is a **breaking traversal change** for any repository that pointed
> `--scope` directly at a bundle directory: `quire validate --okf --scope
> path/to/bundle` now errors unless `path/to/bundle/spec` exists. Pass the
> bundle as the positional argument instead — an explicit positional is honored
> as given and stays self-contained. `spec/` is convention, not configuration:
> there is no manifest key and no flag, deliberately, since a configurable
> document root reintroduces the ambiguity the split removed.

## Dependencies

- **Upstream**: [US-003](../usecase/US-003-ci-validates-archetype-conformance.md) CI validates archetype conformance; [FR-004](./FR-004-validate-subcommand.md) validate (extends); quire-rs [FR-038](ix://agent-ix/quire-rs/spec/functional/FR-038) FR-038-AC-11 (open in quire-rs#646), quire-rs [FR-032](ix://agent-ix/quire-rs/FR-032) (`validate_bundle_at`).
- **Downstream**: OKF bundle CI gates consuming the permissive posture.
