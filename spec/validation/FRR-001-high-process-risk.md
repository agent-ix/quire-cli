---
id: FRR-001
title: "Quire CLI proposed high process-risk supplier ratings"
type: FeatureRiskRating
scope: supplier
rating: high_process_risk
status: proposed
relationships:
  - target: "ix://agent-ix/quire-cli/IU-001"
    type: uses_intended_use
  - target: "ix://agent-ix/quire-cli/FR-004"
    type: rates_feature
  - target: "ix://agent-ix/quire-cli/FR-007"
    type: rates_feature
  - target: "ix://agent-ix/quire-cli/FR-010"
    type: rates_feature
  - target: "ix://agent-ix/quire-cli/FR-014"
    type: rates_feature
  - target: "ix://agent-ix/quire-cli/FR-017"
    type: rates_feature
  - target: "ix://agent-ix/quire-cli/FR-019"
    type: rates_feature
  - target: "ix://agent-ix/quire-cli/FR-021"
    type: rates_feature
  - target: "ix://agent-ix/quire-cli/FR-026"
    type: rates_feature
  - target: "ix://agent-ix/quire-cli/FR-038"
    type: rates_feature
  - target: "ix://agent-ix/quire-cli/FR-045"
    type: rates_feature
  - target: "ix://agent-ix/quire-cli/FR-047"
    type: rates_feature
---
# Quire CLI proposed high process-risk supplier ratings

## Rationale

This is a proposed supplier-side assessment for IU-001. It is unsigned and unaccepted pending a human decision through the signed-commit workflow. No score or product safety classification is assigned.

These features can affect the correctness or interpretation of validation, traceability, export, identity, rights, approval, or self-test records used for the intended use. A failure could make an engineering record appear complete, verified, correctly attributed, or approved when the evidence does not support that result. The proposed high process-risk rating calls for assurance against the explicit acceptance criteria listed below.

## Feature-specific reasons

| Feature | Proposed false-claim reason |
|---|---|
| FR-004 | The validate command can report invalid specification input as valid. |
| FR-007 | An incorrect exit code can signal success for a failed verification command. |
| FR-010 | Required-section validation can accept a document missing a required section. |
| FR-014 | OKF bundle validation can report an invalid bundle as valid. |
| FR-017 | Coverage output can claim a relation or criterion covered without a valid trace. |
| FR-019 | Symbols output can attribute a test or implementation to the wrong requirement. |
| FR-021 | Clause-set evaluation can report an incorrect evaluation result. |
| FR-026 | Matrix output can report requirement evidence or coverage that underlying runs do not support. |
| FR-038 | The approval block can present an unverified signed-approval verdict as approval. |
| FR-045 | Trace output can claim a formalizes relation that is not present. |
| FR-047 | Self-test execution can report a selected installed-binary check passed when it failed. |

## Assurance activities

| Activity | Acceptance criterion |
|---|---|
| Run valid and invalid document fixtures and assert findings, severity, and exit status for every validation case. | FR-004-AC-1 through FR-004-AC-23 |
| Run passing and failing command cases and assert each result maps to the specified exit code. | FR-007-AC-1 through FR-007-AC-9 |
| Render valid and invalid examples, then validate each output and assert the required structure is detected. | FR-010-AC-1 through FR-010-AC-5 |
| Validate conformant and nonconformant bundles and assert the expected finding set and posture for each case. | FR-014-AC-1 through FR-014-AC-17 |
| Run fixtures with backed, unbacked, unmatched, and lying coverage rows; compare coverage facts and strict verdicts. | FR-017-AC-1 through FR-017-AC-24 |
| Run documents containing known symbols and references; compare the emitted symbol inventory with the source locations. | FR-019-AC-1 through FR-019-AC-6 |
| Compare known clause sets, including restricted-content and mismatch fixtures; verify each reported difference and refusal. | FR-021-AC-1 through FR-021-AC-10 |
| Run fixtures across Markdown, JSON, TSV, strict, empty, and error states; compare rows, binders, order, and verdicts. | FR-026-AC-1 through FR-026-AC-17 |
| Verify approval blocks from controlled valid, invalid, unsigned, and untrusted verdict fixtures against signer and verdict source data. | FR-038-AC-1 through FR-038-AC-21 |
| Run trace and symbol fixtures with known formalizes edges and compare forward and inverse reports with the source. | FR-045-AC-1 through FR-045-AC-10 |
| Run known passing and failing embedded cases; seed denied writes and verify the run record reports the correct case status and path. | FR-047-AC-1 through FR-047-AC-10 |
