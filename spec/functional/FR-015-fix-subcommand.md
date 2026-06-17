---
id: FR-015
title: "quire fix subcommand (unlinked-reference autofix)"
type: FR
object_type: cli_command
relationships:
  - target: "ix://agent-ix/quire-cli/spec/usecase/US-004"
    type: "implements"
    cardinality: "1:1"
  - target: "ix://agent-ix/quire-cli/spec/functional/FR-005"
    type: "requires"
    cardinality: "1:1"
  - target: "ix://agent-ix/quire-rs/spec/functional/FR-039"
    type: "consumes"
    cardinality: "1:1"
---

## Description

The CLI SHALL expose a `fix` subcommand that surfaces and, on request, applies
quire-rs unlinked-reference suggestions (upstream FR-039): bare artifact-id
tokens in a bundle's prose that should be relative-path links (ADR 0007). All
detection and classification live in quire-rs; the CLI resolves the bundle root,
applies path-safety ([FR-005](./FR-005-path-safety.md)), and surfaces / writes the engine-provided
suggestions. The behavioral surface is specified below.

## Behavior

```
quire fix [<DIR>] [--scope <DIR>] [--write] [--diagnostics-format <fmt>]
```

No `--module` is required: unlinked-reference detection (FR-039) operates over
the loaded `Spec` corpus alone (id index + path→id), with no archetype registry.

1. Path-safety ([FR-005](./FR-005-path-safety.md)) on the resolved bundle root (the positional `DIR`, or
   `--scope` when no positional is given).
2. Load the bundle root into a `Spec` (quire-rs FR-025) and call
   `unlinked_references(&spec)` (FR-039). Findings are partitioned by the engine
   into `AutoFix { suggested_link }` and `WarnOnly { reason }`.
3. **Dry-run (default, no `--write`)** — apply nothing. Emit on **stderr**:
   - each `AutoFix` as `would-fix: <path>: <token> -> <suggested_link>`;
   - each `WarnOnly` as `warning: <path>: <token> (<reason>)`, where `Unresolved`
     notes the token resolves to no in-bundle artifact and may need a manual
     `ix://` reference.
   NEVER writes stdout and NEVER mutates files.
4. **Apply (`--write`)** — for every `AutoFix`, splice `suggested_link` over the
   finding's `byte_span` using the quire-rs byte-exact writeback primitives
   ([FR-008](./FR-008-json-output-encoding.md)), applying a file's spans in descending start order so earlier offsets
   stay valid, and rewrite each touched file in place. Any fix whose span
   **overlaps** one already applied is **skipped** (defensive guard — the engine
   does not emit overlapping fixes, but applying two splices to the same region
   would corrupt the file). `WarnOnly` findings are left untouched (the engine
   supplies no suggestion for them). Report `fixed <N> reference(s) in <path>`
   per touched file on stderr, where `<N>` counts only the applied fixes.
5. Exit codes:
   - **dry-run**: exit **0** when there are zero `AutoFix` findings; exit **1**
     when one or more `AutoFix` findings remain (an actionable CI gate —
     "specs carry no unlinked references"). `WarnOnly` findings are advisory and
     do NOT affect the exit code.
   - **`--write`**: exit **0** after a successful apply (and on a clean re-run,
     since applying every suggestion is idempotent per FR-039); exit **1** only
     on an I/O / path-safety failure.

The bundle / detection / suggestion logic is wholly upstream (quire-rs FR-039 +
writeback [FR-008](./FR-008-json-output-encoding.md)); the CLI is a thin process boundary ([StR-004](../stakeholder/StR-004-thin-boundary-over-quire-rs.md)).

## Acceptance Criteria

| ID | Criteria | Verification |
|----|----------|--------------|
| FR-015-AC-1 | `quire fix <DIR>` (dry-run) over a bundle with a bare in-bundle reference exits **1**, stderr carries `would-fix: <path>: <token> -> [<token>](<rel-path>)`, and no file is modified | Test |
| FR-015-AC-2 | `quire fix <DIR> --write` rewrites the bare reference to the suggested relative-path link in place; a second `--write` run modifies nothing and exits **0** (idempotence, FR-039) | Test |
| FR-015-AC-3 | A `WarnOnly` (unresolved/ambiguous) token is surfaced as `warning: … (<reason>)`, is never written even under `--write`, and does not by itself cause a non-zero exit | Test |
| FR-015-AC-4 | A clean bundle (no `AutoFix` findings) exits **0** with empty stdout in both dry-run and `--write` | Test |
| FR-015-AC-5 | `quire fix --scope <DIR>` with no positional uses `--scope` as the bundle root; a `..` or symlink-escape path on the root is rejected by path-safety ([FR-005](./FR-005-path-safety.md)) before any load | Test |
| FR-015-AC-6 | (thin boundary) all detection, classification, suggested-link construction, and span application are delegated to quire-rs (`unlinked_references`, writeback); the CLI only resolves the root, applies path-safety, orders spans, and surfaces results ([StR-004](../stakeholder/StR-004-thin-boundary-over-quire-rs.md)) | Inspection |

## Dependencies

- **Upstream**: quire-rs FR-039 (unlinked-reference detection) + [FR-008](./FR-008-json-output-encoding.md) (byte-exact writeback); [FR-005](./FR-005-path-safety.md) path-safety; [US-004](../usecase/US-004-cross-reference-extraction.md) cross-reference extraction.
- **Downstream**: OKF migration tooling / CI gates that keep specs link-complete.
