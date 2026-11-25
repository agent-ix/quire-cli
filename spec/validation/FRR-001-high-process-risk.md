---
id: FRR-001
title: "Quire CLI proposed high process-risk supplier ratings"
type: FeatureRiskRating
scope: supplier
rating: high_process_risk
rated_by: Peter Krenesky
relationships:
  - target: "ix://agent-ix/quire-cli/IU-001"
    type: uses_intended_use
  - target: "ix://agent-ix/quire-cli/FR-003"
    type: rates_feature
  - target: "ix://agent-ix/quire-cli/FR-004"
    type: rates_feature
  - target: "ix://agent-ix/quire-cli/FR-005"
    type: rates_feature
  - target: "ix://agent-ix/quire-cli/FR-007"
    type: rates_feature
  - target: "ix://agent-ix/quire-cli/FR-010"
    type: rates_feature
  - target: "ix://agent-ix/quire-cli/FR-012"
    type: rates_feature
  - target: "ix://agent-ix/quire-cli/FR-014"
    type: rates_feature
  - target: "ix://agent-ix/quire-cli/FR-015"
    type: rates_feature
  - target: "ix://agent-ix/quire-cli/FR-017"
    type: rates_feature
  - target: "ix://agent-ix/quire-cli/FR-019"
    type: rates_feature
  - target: "ix://agent-ix/quire-cli/FR-020"
    type: rates_feature
  - target: "ix://agent-ix/quire-cli/FR-021"
    type: rates_feature
  - target: "ix://agent-ix/quire-cli/FR-022"
    type: rates_feature
  - target: "ix://agent-ix/quire-cli/FR-025"
    type: rates_feature
  - target: "ix://agent-ix/quire-cli/FR-026"
    type: rates_feature
  - target: "ix://agent-ix/quire-cli/FR-027"
    type: rates_feature
  - target: "ix://agent-ix/quire-cli/FR-028"
    type: rates_feature
  - target: "ix://agent-ix/quire-cli/FR-029"
    type: rates_feature
  - target: "ix://agent-ix/quire-cli/FR-030"
    type: rates_feature
  - target: "ix://agent-ix/quire-cli/FR-031"
    type: rates_feature
  - target: "ix://agent-ix/quire-cli/FR-032"
    type: rates_feature
  - target: "ix://agent-ix/quire-cli/FR-033"
    type: rates_feature
  - target: "ix://agent-ix/quire-cli/FR-034"
    type: rates_feature
  - target: "ix://agent-ix/quire-cli/FR-037"
    type: rates_feature
  - target: "ix://agent-ix/quire-cli/FR-038"
    type: rates_feature
  - target: "ix://agent-ix/quire-cli/FR-039"
    type: rates_feature
  - target: "ix://agent-ix/quire-cli/FR-040"
    type: rates_feature
  - target: "ix://agent-ix/quire-cli/FR-041"
    type: rates_feature
  - target: "ix://agent-ix/quire-cli/FR-042"
    type: rates_feature
  - target: "ix://agent-ix/quire-cli/FR-043"
    type: rates_feature
  - target: "ix://agent-ix/quire-cli/FR-044"
    type: rates_feature
  - target: "ix://agent-ix/quire-cli/FR-045"
    type: rates_feature
  - target: "ix://agent-ix/quire-cli/FR-046"
    type: rates_feature
  - target: "ix://agent-ix/quire-cli/FR-047"
    type: rates_feature
---
# Quire CLI proposed high process-risk supplier ratings

## Feature assessments

| Requirement | Failure mode | Detection |
|---|---|---|
| FR-003 | Extracted records or edges could omit or change a document relationship used in trace review. | Extract fixture documents with known records and links; compare emitted records and edges with the fixture contents. |
| FR-004 | An invalid document could be reported valid, or a valid document invalid, changing the validation observation. | Run valid and invalid document fixtures and assert findings, severity, and exit status for every validation case. |
| FR-005 | A path boundary defect could read an unintended source or write an edit outside the requested document. | Exercise traversal and boundary paths for module, document, and write targets; confirm refusal and no out-of-scope file change. |
| FR-007 | A failing validation or gate could exit successfully, allowing automation to accept an invalid record. | Run passing and failing command cases and assert each result maps to the specified exit code. |
| FR-010 | A rendered document missing required structure could pass the structural check and appear ready for use. | Render valid and invalid examples, then validate each output and assert the required structure is detected. |
| FR-012 | An edit could change the wrong section or corrupt bytes in the source document used as a record. | Edit fixture sections and blocks, including refusal cases; compare the full file bytes outside the intended edit. |
| FR-014 | An invalid document or broken reference in a bundle could pass, making the corpus appear structurally sound. | Validate conformant and nonconformant bundles and assert the expected finding set and posture for each case. |
| FR-015 | An automatic link fix could point a reference at the wrong target and change the meaning of a document relation. | Run link-fix fixtures with unique, ambiguous, and absent targets; inspect proposed changes and verify only intended bytes change. |
| FR-017 | Coverage could report criteria as backed when no matching tests support them, creating a false assurance record. | Run fixtures with backed, unbacked, unmatched, and lying coverage rows; compare coverage facts and strict verdicts. |
| FR-019 | A symbol could be omitted or misidentified, changing symbol and traceability observations used in review. | Run documents containing known symbols and references; compare the emitted symbol inventory with the source locations. |
| FR-020 | Assurance facts could be omitted, altered, or attached to the wrong source requirement in the exported observation. | Run assurance fixtures with known source facts and compare every emitted fact, relation, and refusal with the source. |
| FR-021 | A clause set could be compared incorrectly, causing restricted or conflicting module content to be misreported. | Compare known clause sets, including restricted-content and mismatch fixtures; verify each reported difference and refusal. |
| FR-022 | A release could install an incomplete or mismatched executable, so subsequent records come from an unexpected CLI build. | Inspect release workflow and package fixtures; verify supported targets, installed version, and publication gates. |
| FR-025 | A qualification check could accept incomplete or unsupported evidence and report a gate as passing. | Run backed and deliberately incomplete qualification fixtures; assert the exact rejected target, status, and boundary behavior. |
| FR-026 | A matrix cell, binder, or strict verdict could be wrong, making coverage or design status appear complete when it is not. | Run fixtures across Markdown, JSON, TSV, strict, empty, and error states; compare rows, binders, order, and verdicts. |
| FR-027 | A split-revision lock could pass, allowing records to be produced with incoherent versions of related packages. | Run aligned, split, malformed, duplicate, unreadable, and multi-lock fixtures; compare results and verify inputs remain unchanged. |
| FR-028 | A matrix could show the wrong declared criticality beside a requirement and misstate the source record. | Run requirements with present and absent criticality and compare each rendered value with the source declaration. |
| FR-029 | Restricted clause text could pass the publication check and be carried into an artifact intended for distribution. | Run allowed and restricted clause fixtures and assert the publication refusal names the restricted content. |
| FR-030 | A required relation could be reported present when absent, or missing when present, distorting a completeness report. | Run relation fixtures with present, absent, duplicate, and malformed edges; compare the report with the module declarations. |
| FR-031 | An export could omit or alter source content and produce an inaccurate review document. | Export representative single documents and compare extracted output content, metadata, and selected format. |
| FR-032 | The converter process could receive incorrect inputs or sequencing and produce an incomplete exported record. | Use controlled converter executables to record arguments, order, inputs, and outputs for each supported export path. |
| FR-033 | An export failure could be reported as success or leave a partial file that appears to be a completed record. | Exercise missing tools, invalid inputs, converter errors, and write failures; verify non-success results and no false completed output. |
| FR-034 | An export could identify the wrong document source, revision, or producer and misattribute the record. | Use repositories with known source revisions and identity metadata; compare the exported identity and refusal cases. |
| FR-037 | Host- or clock-dependent output could make equivalent inputs produce differing exported records. | Export identical inputs repeatedly under varied clocks and host metadata; compare output bytes for stability. |
| FR-038 | An approval block could misstate the signer, verdict, meaning, or time and create a false impression of approval. | Verify approval blocks from controlled valid, invalid, unsigned, and untrusted verdict fixtures against signer and verdict source data. |
| FR-039 | A bundle could omit, duplicate, or reorder included documents, so the combined export differs from the requested set. | Export ordered multi-document fixtures; compare every included document, order, bundle identity, and refusal case. |
| FR-040 | Invalid identity, template, approval, or bundle input could be accepted and yield a misleading export. | Run invalid identity, template, approval, and bundle fixtures; assert precise refusals and absence of completed output. |
| FR-041 | Design relations could be reported incorrectly, changing the design evidence shown to a reviewer. | Run design-relation fixtures with complete, missing, and rejected relations; compare reported design status with source edges. |
| FR-042 | A rendered diagram could misrepresent a relation from the source document and mislead a review of the export. | Export diagrams with known Mermaid inputs and compare generated images and their placement with the source blocks. |
| FR-043 | Design criteria, binders, or verdicts could be wrong in the matrix and make design completeness appear different from the source. | Run design matrix fixtures across formats, empty populations, errors, and strict mode; compare design cells and verdicts. |
| FR-044 | A flow relation could be omitted or misdirected in the report, giving an inaccurate view of the documented flow. | Run flow fixtures with known nodes, edges, and gaps; compare emitted flow relations and findings with source declarations. |
| FR-045 | A formalizes relation could be omitted or attached to the wrong criterion in the trace and symbol reports. | Run trace and symbol fixtures with known formalizes edges and compare forward and inverse reports with the source. |
| FR-046 | A health-data flow relation could be misreported, creating an inaccurate record of documented data movement. | Run health-data-flow fixtures with complete and incomplete relations; compare each reported flow and gap with declarations. |
| FR-047 | A failing or sandbox-violating self-test case could be reported as passing, overstating what the installed binary demonstrated. | Run known passing and failing embedded cases; seed denied writes and verify the run record reports the correct case status and path. |

## Rationale

This is a proposed supplier-side assessment for IU-001. The `rated_by` field names Peter Krenesky as the human assigned to review and accept the proposed rating. This record is unsigned and unaccepted pending his decision through the signed-commit workflow. No score or product safety classification is assigned.

These features can affect the correctness or interpretation of validation, traceability, export, identity, rights, approval, or self-test records used for the intended use. A failure could make an engineering record appear complete, verified, correctly attributed, or approved when the evidence does not support that result. The proposed high process-risk rating calls for assurance against the explicit acceptance criteria listed below.

## Assurance activities

| Activity | Acceptance criterion |
|---|---|
| Extract fixture documents with known records and links; compare emitted records and edges with the fixture contents. | FR-003-AC-1 through FR-003-AC-6 |
| Run valid and invalid document fixtures and assert findings, severity, and exit status for every validation case. | FR-004-AC-1 through FR-004-AC-23 |
| Exercise traversal and boundary paths for module, document, and write targets; confirm refusal and no out-of-scope file change. | FR-005-AC-1 through FR-005-AC-5 |
| Run passing and failing command cases and assert each result maps to the specified exit code. | FR-007-AC-1 through FR-007-AC-9 |
| Render valid and invalid examples, then validate each output and assert the required structure is detected. | FR-010-AC-1 through FR-010-AC-5 |
| Edit fixture sections and blocks, including refusal cases; compare the full file bytes outside the intended edit. | FR-012-AC-1 through FR-012-AC-6 |
| Validate conformant and nonconformant bundles and assert the expected finding set and posture for each case. | FR-014-AC-1 through FR-014-AC-17 |
| Run link-fix fixtures with unique, ambiguous, and absent targets; inspect proposed changes and verify only intended bytes change. | FR-015-AC-1 through FR-015-AC-6 |
| Run fixtures with backed, unbacked, unmatched, and lying coverage rows; compare coverage facts and strict verdicts. | FR-017-AC-1 through FR-017-AC-24 |
| Run documents containing known symbols and references; compare the emitted symbol inventory with the source locations. | FR-019-AC-1 through FR-019-AC-6 |
| Run assurance fixtures with known source facts and compare every emitted fact, relation, and refusal with the source. | FR-020-AC-1 through FR-020-AC-10 |
| Compare known clause sets, including restricted-content and mismatch fixtures; verify each reported difference and refusal. | FR-021-AC-1 through FR-021-AC-10 |
| Inspect release workflow and package fixtures; verify supported targets, installed version, and publication gates. | FR-022-AC-1 through FR-022-AC-9 |
| Run backed and deliberately incomplete qualification fixtures; assert the exact rejected target, status, and boundary behavior. | FR-025-AC-1 through FR-025-AC-7 |
| Run fixtures across Markdown, JSON, TSV, strict, empty, and error states; compare rows, binders, order, and verdicts. | FR-026-AC-1 through FR-026-AC-17 |
| Run aligned, split, malformed, duplicate, unreadable, and multi-lock fixtures; compare results and verify inputs remain unchanged. | FR-027-AC-1 through FR-027-AC-10 |
| Run requirements with present and absent criticality and compare each rendered value with the source declaration. | FR-028-AC-1 through FR-028-AC-11 |
| Run allowed and restricted clause fixtures and assert the publication refusal names the restricted content. | FR-029-AC-1 through FR-029-AC-8 |
| Run relation fixtures with present, absent, duplicate, and malformed edges; compare the report with the module declarations. | FR-030-AC-1 through FR-030-AC-28 |
| Export representative single documents and compare extracted output content, metadata, and selected format. | FR-031-AC-1 through FR-031-AC-13 |
| Use controlled converter executables to record arguments, order, inputs, and outputs for each supported export path. | FR-032-AC-1 through FR-032-AC-8 |
| Exercise missing tools, invalid inputs, converter errors, and write failures; verify non-success results and no false completed output. | FR-033-AC-1 through FR-033-AC-11 |
| Use repositories with known source revisions and identity metadata; compare the exported identity and refusal cases. | FR-034-AC-1 through FR-034-AC-20 |
| Export identical inputs repeatedly under varied clocks and host metadata; compare output bytes for stability. | FR-037-AC-1 through FR-037-AC-6 |
| Verify approval blocks from controlled valid, invalid, unsigned, and untrusted verdict fixtures against signer and verdict source data. | FR-038-AC-1 through FR-038-AC-21 |
| Export ordered multi-document fixtures; compare every included document, order, bundle identity, and refusal case. | FR-039-AC-1 through FR-039-AC-8 |
| Run invalid identity, template, approval, and bundle fixtures; assert precise refusals and absence of completed output. | FR-040-AC-1 through FR-040-AC-8 |
| Run design-relation fixtures with complete, missing, and rejected relations; compare reported design status with source edges. | FR-041-AC-1 through FR-041-AC-11 |
| Export diagrams with known Mermaid inputs and compare generated images and their placement with the source blocks. | FR-042-AC-1 through FR-042-AC-15 |
| Run design matrix fixtures across formats, empty populations, errors, and strict mode; compare design cells and verdicts. | FR-043-AC-1 through FR-043-AC-27 |
| Run flow fixtures with known nodes, edges, and gaps; compare emitted flow relations and findings with source declarations. | FR-044-AC-1 through FR-044-AC-16 |
| Run trace and symbol fixtures with known formalizes edges and compare forward and inverse reports with the source. | FR-045-AC-1 through FR-045-AC-10 |
| Run health-data-flow fixtures with complete and incomplete relations; compare each reported flow and gap with declarations. | FR-046-AC-1 through FR-046-AC-10 |
| Run known passing and failing embedded cases; seed denied writes and verify the run record reports the correct case status and path. | FR-047-AC-1 through FR-047-AC-10 |
