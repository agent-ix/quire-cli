---
id: FR-005
title: "Path-safety guard on --module, --data, --out, and positional doc paths"
artifact_type: FR
object_type: middleware
relationships:
  - target: "ix://agent-ix/quire-cli/spec/stakeholder/StR-003"
    type: "implements"
    cardinality: "1:1"
---

## Behavior

Before any I/O, the CLI SHALL canonicalize and validate every filesystem path argument:

1. **Reject `..`** literal segments in any user-supplied path argument (`--module`, `--data`, `--out`, positional `<DOC>`).
2. **Reject paths that, after `std::fs::canonicalize`, escape the parent of the user-supplied `--module` root.** (Specifically: `--data` and `--out` paths MAY be anywhere on the filesystem the invoking user can read/write; `--module` is canonicalized once and the engine operates only on paths within it.)
3. **Reject symlinks under `--module`** that resolve outside the canonicalized module root. The Registry loader SHALL refuse to follow such links.
4. Path-safety violations exit 1 with a `PathSafetyViolation` diagnostic on stderr identifying the offending argument and the violated rule. No partial I/O is performed; no module load is attempted.

The check SHALL run **before** any file is opened.

## Acceptance

- **FR-005-AC-1**: `quire render FR --module ../escape --data ctx.json` exits 1 with `PathSafetyViolation` naming `--module`.
- **FR-005-AC-2**: `quire render FR --module $ISO --data ../../etc/passwd` exits 1 with `PathSafetyViolation` naming `--data`.
- **FR-005-AC-3**: `quire render FR --module $ISO --data ctx.json --out ../escape.md` exits 1 with `PathSafetyViolation` naming `--out`.
- **FR-005-AC-4**: A symlink inside the module root pointing to `/etc/passwd` is refused at module load time with a `PathSafetyViolation`; the offending symlink's relative path is reported.
- **FR-005-AC-5**: `--data -` (stdin) is never subject to path-safety checks; it cannot escape the filesystem.
