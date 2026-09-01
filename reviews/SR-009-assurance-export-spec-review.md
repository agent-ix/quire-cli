---
id: SR-009
title: "Assurance-export CLI specification review"
type: SpecReview
analysis: scope-boundary
scope: "spec/usecase/US-006-export-assurance-facts.md, spec/functional/FR-020-assurance-export-subcommand.md, spec/stakeholder/StR-004-thin-boundary-over-quire-rs.md, spec/tests.md, agent-ix/quire-cli#74, agent-ix/quire-rs#386/#389, agent-ix/engineering-assurance#5/#7"
review_set: subset
---

## Summary

Preimplementation review of the Quire CLI #74 contract against the accepted
Engineering Assurance ownership boundary and the merged quire-rs
`assurance-v1` implementation. The reviewed boundary is a CLI adapter only:
Quire owns static artifacts, obligations, symbols, relations, and availability
facts; native tools execute verification; Quoin owns evidence retention,
integrity, audit, attestations, receipts, and verdicts.

## Verdict

**PASS after specification fixes.** The command is implementable entirely by
composing public quire-rs APIs. It introduces no second graph, schema, evidence
envelope, runner, or verdict policy. Every issue acceptance criterion has a
planned process-level or static test, and failure is observably distinct from a
successful export whose arrays are empty.

## Findings

| ID | Severity | Summary | Refs |
|----|----------|---------|------|
| FND-001 | high | **FIXED in FR-020 before implementation.** Merely passing emitted tuples to `read_assurance_export` would reject missing expectations but accept unused extra expected tuples. FR-020 now requires equality between the caller's complete accepted set and the engine-emitted set before stdout, with missing, extra, and wrong-digest cases in IT-139. | FR-020 Behavior 3; FR-020-AC-3; IT-139 |
| FND-002 | high | **FIXED in FR-020 before implementation.** Reusing environment/default discovery would make the active module set depend on workstation state and could silently activate sibling modules. The command now requires one exact `--module` directory and performs no discovery or lazy installation. | FR-020 Inputs; FR-020 Behavior 1 |
| FND-003 | medium | **FIXED in FR-020 before implementation.** Existing JSON commands append a CLI provenance object, but `assurance-v1` is a closed upstream envelope with `additionalProperties: false`. FR-020-CON-3 forbids appending that object; compatibility is instead bound by the exact Cargo revision and capability token. | FR-020-CON-1; FR-020-CON-3; IT-145 |
| FND-004 | medium | **FIXED in FR-020 before implementation.** “Does not execute tests” was too narrow: Git revision discovery, package-manager bootstrap, a consumer validation command, or a solver would also violate the campaign ownership boundary. FR-020-CON-2 and IT-143 prohibit every child process and network syscall on success and refusal paths. | FR-020-CON-2; FR-020-AC-7; IT-143 |
| FND-005 | low | **ACCEPTED and explicit.** The caller supplies repository and full revision identities; the CLI validates their shape through quire-rs but does not prove that the working tree matches them. This is upstream FR-067's deliberate caller-selected identity contract, not a CLI trust claim. | FR-020 Inputs; quire-rs FR-067-CON-2 |

## Coverage and dependency checks

- Upstream API and schema owner: quire-rs FR-067/FR-068, merged by PR #389 at
  `e3352a0644abcfd5f0ebad348bc7aca235925ecc`, manifest version 0.46.0.
- Ownership/type-fit prerequisite: Engineering Assurance #5 is merged.
- Governance prerequisite: quire-contract-ir #38 is merged.
- Matrix: FR-020 AC-1..9 map to IT-136..145 and TC-814; US-006 AC-1..4 map
  to the same executable/static evidence.
- Hosted CI remains manual-only and is not part of this review or plan.

## Implementation authorization

Proceed only after committing this reviewed specification as its own baseline.
Implementation must keep the upstream JSON bytes load-bearing, prove the exact
premise set rather than a subset, and open a PR for external review before any
administrative merge.
