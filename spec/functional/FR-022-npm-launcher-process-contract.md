---
id: FR-022
title: "npm launcher process contract"
type: FR
object_type: cli_command
relationships:
  - target: "ix://agent-ix/quire-cli/spec/stakeholder/StR-001"
    type: "implements"
    cardinality: "1:1"
---

# FR-022: npm launcher process contract

## Description

The `@agent-ix/quire-cli` npm package SHALL expose `quire` through the minimal
host launcher decided by ADR-0002. The launcher SHALL select only a platform
package declared in its generated `optionalDependencies` and SHALL otherwise
behave as a transparent process boundary around the packaged native binary.

## Behavior

1. The launcher derives `<platform>-<arch>` from the Node host and derives the
   package name `@agent-ix/quire-cli-<platform>-<arch>`.
2. The derived package is supported only when it is an own property of the
   launcher's generated `optionalDependencies`. The launcher does not carry a
   second independently maintained target allowlist.
3. The executable is `bin/quire.exe` for `win32` and `bin/quire` otherwise.
4. The launcher passes every argument after its own executable path in the
   original order and inherits stdin, stdout, and stderr.
5. A normal child exit returns the identical status. A signal-terminated child
   causes the launcher to terminate with the identical signal.
6. On a non-Windows host, the launcher makes one best-effort attempt to restore
   executable mode `0755`; refusal is not itself fatal because the file may
   already be executable. Any subsequent launch failure is fatal.

## Acceptance Criteria

| ID | Criteria | Verification |
|----|----------|--------------|
| FR-022-AC-1 | The installed `quire` entry selects exactly the optional package for the current platform and architecture and resolves only its fixed `bin/quire[.exe]` member. | Test (IT-159, TC-815) |
| FR-022-AC-2 | The launcher forwards the ordered host argument strings without adding, removing, or rewriting an element, inherits all three standard streams, and returns the native process's normal exit status. | Test (IT-159) |
| FR-022-AC-3 | The launcher mirrors a native process's terminating signal rather than translating it into an unrelated normal exit status. | Test (IT-162) |
| FR-022-AC-4 | An unsupported platform/architecture exits 1, lists the generated supported set, names the source-build alternative, and launches nothing. | Test (IT-160) |
| FR-022-AC-5 | A supported but absent optional package exits 1, names that exact package, explains how optional dependencies can be restored, and launches nothing. | Test (IT-160) |
| FR-022-AC-6 | A chmod refusal is tolerated, while an actual spawn error exits 1 and reports `failed to launch binary` with the host error. | Test (IT-161) |
| FR-022-AC-7 | The host file contains no version policy, package generation, binary-format validation, checksum logic, document handling, extraction, clause grammar, or source semantics. | Static test (TC-815, TC-820) |

## Dependencies

- **Upstream**: ADR-0002 owner disposition for the minimal npm host boundary.
- **Downstream**: FR-023 generates the launcher manifest and its closed optional
  dependency set.
