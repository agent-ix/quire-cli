---
id: FR-005
title: "Path-safety guard on --module, positional doc paths, and edit write targets"
type: FR
object_type: middleware
relationships:
  - target: "ix://agent-ix/quire-cli/spec/stakeholder/StR-003"
    type: "implements"
    cardinality: "1:1"
---

> **CR note (render removal — 2026-06-04):** `--data`/`--out` belonged to the
> removed `render` subcommand (mirrors quire-rs render retirement, commit 500a3d3).
> The path-safety rule itself is unchanged and generic; it now applies to the
> surviving path arguments — positional `<DOC>` and `--module` (`validate`/`parse`/
> `extract`/`lookup`/`schema`) and `--out`/`--in-place` targets of `edit`. AC
> examples are rephrased onto `validate`; the `--data` AC is retired. The
> path-safety semantics (reject `..`, canonicalize, no symlink escape, stdin `-`
> exempt) are preserved verbatim.

## Description

Before any I/O, the CLI SHALL canonicalize and validate every user-supplied
filesystem path argument, rejecting `..` segments and symlink escapes from the
`--module` root, so that a malicious or malformed path argument cannot escape the
intended module root. The behavioral surface is specified below.

## Behavior

Before any I/O, the CLI SHALL canonicalize and validate every filesystem path argument:

1. **Reject `..`** literal segments in any user-supplied path argument (`--module`, positional `<DOC>`, and `edit`'s `--out`/`--in-place` target).
2. **Reject paths that, after `std::fs::canonicalize`, escape the parent of the user-supplied `--module` root.** (Specifically: document and write-target paths MAY be anywhere on the filesystem the invoking user can read/write; `--module` is canonicalized once and the engine operates only on paths within it.)
3. **Reject symlinks under `--module`** that resolve outside the canonicalized module root. The Registry loader SHALL refuse to follow such links.
4. Path-safety violations exit 1 with a `PathSafetyViolation` diagnostic on stderr identifying the offending argument and the violated rule. No partial I/O is performed; no module load is attempted.

The check SHALL run **before** any file is opened.

## Acceptance Criteria

| ID | Criteria | Verification |
|----|----------|--------------|
| FR-005-AC-1 | `quire validate doc.md --module ../escape` exits 1 with `PathSafetyViolation` naming `--module` | Test |
| FR-005-AC-2 | `quire validate ../../etc/passwd --module $ISO` exits 1 with `PathSafetyViolation` naming the positional document argument | Test |
| FR-005-AC-3 | (RETIRED) `quire render FR --module $ISO --data ctx.json --out ../escape.md` exits 1 with `PathSafetyViolation` naming `--out`. The `--out` rule survives on `edit`'s write target (see [FR-012](./FR-012-edit-subcommand.md)) | Test (TC-022) |
| FR-005-AC-4 | A symlink inside the module root pointing to `/etc/passwd` is refused at module load time with a `PathSafetyViolation`; the offending symlink's relative path is reported | Test |
| FR-005-AC-5 | A positional `-` (stdin) is never subject to path-safety checks; it cannot escape the filesystem | Test |

## Dependencies

- **Upstream**: [StR-003](../stakeholder/StR-003-sandbox-inheritance.md) sandbox inheritance; FR-035 manifest contract.
- **Downstream**: [FR-004](./FR-004-validate-subcommand.md) validate, [FR-003](./FR-003-extract-subcommand.md) extract, [FR-009](./FR-009-schema-subcommand.md) schema, [FR-011](./FR-011-lookup-subcommand.md) lookup, [FR-012](./FR-012-edit-subcommand.md) edit (all apply path-safety).
