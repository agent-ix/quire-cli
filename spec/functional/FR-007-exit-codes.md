---
id: FR-007
title: "Exit code contract"
type: FR
object_type: cli_command
relationships:
  - target: "ix://agent-ix/quire-cli/spec/stakeholder/StR-001"
    type: "implements"
    cardinality: "1:1"
---

## Description

The CLI SHALL use ix-cli-kit outcomes uniformly across subcommands, so callers
can branch on typed outcomes without parsing diagnostics.

## Behavior

| Code | Meaning |
| --- | --- |
| 0 | Successful completion, including help and version requests. |
| 1 | Partial result: a complete requested report was emitted, but strict coverage/matrix or property resolution found incomplete or failing records. |
| 2 | Refusal: path traversal or an explicit domain policy prevented the operation. |
| 3 | Invalid request: invalid argv, missing or incorrectly typed input paths, malformed input, unknown archetypes, unresolved selection, module load or structural validation failures. |
| 4 | Internal failure: unexpected implementation, serialization or output I/O failure. |

The CLI SHALL derive outcomes from typed errors and explicit domain verdicts,
never diagnostic text. Input permission refusals SHALL use 2; missing input
SHALL use 3. Output and unclassified I/O failures SHALL use 4. A validation or
lint failure that emits diagnostics without a complete primary report SHALL
use 3. Empty strict matrix reports SHALL use 1 because the requested complete
zero-population report was emitted. Existing diagnostics and payload schemas
SHALL remain unchanged. No compatibility mapping SHALL be provided.

## Acceptance Criteria

| ID | Criteria | Verification |
| --- | --- | --- |
| FR-007-AC-1 | Successful commands, help and version exit 0. | Test |
| FR-007-AC-2 | Path traversal and explicit policy refusals exit 2 with diagnostics. | Test |
| FR-007-AC-3 | Unknown archetypes and invalid or missing input exit 3. | Test |
| FR-007-AC-4 | Structural validation failures without a primary report exit 3. | Test |
| FR-007-AC-5 | Invalid argv exits 3 with clap diagnostics. | Test |
| FR-007-AC-6 | Covered invalid inputs return a classified failure without panicking. | Test |
| FR-007-AC-7 | Strict coverage/matrix and unresolved property reports exit 1 with the complete requested payload. | Test |
| FR-007-AC-8 | A failed output write exits 4; typed classification is independent of human diagnostic wording. | Test |

> CR-001 (2026-10-02, PLAT-111): Peter explicitly approved the shared
> outcomes in this workspace. Domain policy remains downstream; this replaces
> the former generic user-error code and clap's default argv code.

## Dependencies

- **Upstream**: [StR-001](../stakeholder/StR-001-static-binary-hot-path.md) single-binary hot path.
- **Downstream**: every subcommand and [FR-006](./FR-006-io-contract.md) I/O contract rely on these exit codes.
