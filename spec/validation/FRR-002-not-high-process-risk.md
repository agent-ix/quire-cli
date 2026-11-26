---
id: FRR-002
title: "Quire CLI proposed not-high process-risk supplier ratings"
type: FeatureRiskRating
scope: supplier
rating: not_high_process_risk
status: proposed
relationships:
  - target: "ix://agent-ix/quire-cli/IU-001"
    type: uses_intended_use
  - target: "ix://agent-ix/quire-cli/FR-001"
    type: rates_feature
  - target: "ix://agent-ix/quire-cli/FR-002"
    type: rates_feature
  - target: "ix://agent-ix/quire-cli/FR-003"
    type: rates_feature
  - target: "ix://agent-ix/quire-cli/FR-005"
    type: rates_feature
  - target: "ix://agent-ix/quire-cli/FR-006"
    type: rates_feature
  - target: "ix://agent-ix/quire-cli/FR-008"
    type: rates_feature
  - target: "ix://agent-ix/quire-cli/FR-009"
    type: rates_feature
  - target: "ix://agent-ix/quire-cli/FR-011"
    type: rates_feature
  - target: "ix://agent-ix/quire-cli/FR-012"
    type: rates_feature
  - target: "ix://agent-ix/quire-cli/FR-013"
    type: rates_feature
  - target: "ix://agent-ix/quire-cli/FR-015"
    type: rates_feature
  - target: "ix://agent-ix/quire-cli/FR-016"
    type: rates_feature
  - target: "ix://agent-ix/quire-cli/FR-018"
    type: rates_feature
  - target: "ix://agent-ix/quire-cli/FR-020"
    type: rates_feature
  - target: "ix://agent-ix/quire-cli/FR-021"
    type: rates_feature
  - target: "ix://agent-ix/quire-cli/FR-022"
    type: rates_feature
  - target: "ix://agent-ix/quire-cli/FR-025"
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
  - target: "ix://agent-ix/quire-cli/FR-035"
    type: rates_feature
  - target: "ix://agent-ix/quire-cli/FR-036"
    type: rates_feature
  - target: "ix://agent-ix/quire-cli/FR-037"
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
  - target: "ix://agent-ix/quire-cli/FR-046"
    type: rates_feature
  - target: "ix://agent-ix/quire-cli/FR-048"
    type: rates_feature
---
# Quire CLI proposed not-high process-risk supplier ratings

## Rationale

This is a proposed supplier-side assessment for IU-001. It is unsigned and unaccepted pending a human decision through the signed-commit workflow. No score or product safety classification is assigned.

Under IU-001, these features do not themselves report verification, trace, discharge, or approval as successful when their supporting evidence does not warrant that result. Their outcomes can be checked against source data and the acceptance criteria below. This is a proposed human assessment, not a computed score or product-safety classification.

## Assurance activities

| Activity | Acceptance criterion |
|---|---|
| Render representative valid and malformed inputs; compare the generated Markdown structure and text with the expected output. | FR-001-AC-1 through FR-001-AC-6 |
| Parse valid, invalid, and boundary documents; compare each JSON field and diagnostic with the source and expected result. | FR-002-AC-1 through FR-002-AC-5 |
| Extract fixture documents with known records and links; compare emitted records and edges with the fixture contents. | FR-003-AC-1 through FR-003-AC-6 |
| Exercise traversal and boundary paths for module, document, and write targets; confirm refusal and no out-of-scope file change. | FR-005-AC-1 through FR-005-AC-5 |
| Exercise stdin, stdout, and stderr cases and compare bytes on each stream with the command contract. | FR-006-AC-1 through FR-006-AC-5 |
| Round-trip Unicode, control characters, and escaping cases through JSON output and compare the decoded value with the source. | FR-008-AC-1 through FR-008-AC-6 |
| Compare the schema subcommand output with the selected module contract and test the documented valid and invalid shapes. | FR-009-AC-1 through FR-009-AC-5 |
| Query fixtures by each supported lookup key and compare the returned section or record with the exact requested target. | FR-011-AC-1 through FR-011-AC-7 |
| Edit fixture sections and blocks, including refusal cases; compare the full file bytes outside the intended edit. | FR-012-AC-1 through FR-012-AC-6 |
| Run lint fixtures with and without advisory findings; compare finding locations, messages, and non-gating status. | FR-013-AC-1 through FR-013-AC-5 |
| Run link-fix fixtures with unique, ambiguous, and absent targets; inspect proposed changes and verify only intended bytes change. | FR-015-AC-1 through FR-015-AC-6 |
| Exercise supported install sources and unsupported cases; verify the resulting version/source or explicit refusal. | FR-016-AC-1 through FR-016-AC-7 |
| Run representative criteria through each property-shape case and compare the classification with the declared rule. | FR-018-AC-1 through FR-018-AC-10 |
| Run assurance fixtures with known source facts and compare every emitted fact, relation, and refusal with the source. | FR-020-AC-1 through FR-020-AC-10 |
| Compare known clause sets, including restricted-content and mismatch fixtures; verify each reported difference and refusal. | FR-021-AC-1 through FR-021-AC-10 |
| Inspect release workflow and package fixtures; verify supported targets, installed version, and publication gates. | FR-022-AC-1 through FR-022-AC-9 |
| Run backed and deliberately incomplete qualification fixtures; assert the exact rejected target, status, and boundary behavior. | FR-025-AC-1 through FR-025-AC-7 |
| Run aligned, split, malformed, duplicate, unreadable, and multi-lock fixtures; compare results and verify inputs remain unchanged. | FR-027-AC-1 through FR-027-AC-10 |
| Run requirements with present and absent criticality and compare each rendered value with the source declaration. | FR-028-AC-1 through FR-028-AC-11 |
| Run allowed and restricted clause fixtures and assert the publication refusal names the restricted content. | FR-029-AC-1 through FR-029-AC-8 |
| Run relation fixtures with present, absent, duplicate, and malformed edges; compare the report with the module declarations. | FR-030-AC-1 through FR-030-AC-28 |
| Export representative single documents and compare extracted output content, metadata, and selected format. | FR-031-AC-1 through FR-031-AC-13 |
| Use controlled converter executables to record arguments, order, inputs, and outputs for each supported export path. | FR-032-AC-1 through FR-032-AC-8 |
| Exercise missing tools, invalid inputs, converter errors, and write failures; verify non-success results and no false completed output. | FR-033-AC-1 through FR-033-AC-11 |
| Use repositories with known source revisions and identity metadata; compare the exported identity and refusal cases. | FR-034-AC-1 through FR-034-AC-20 |
| Export fixtures through the default template and inspect title, contents, headers, footers, and pagination output. | FR-035-AC-1 through FR-035-AC-9 |
| Export with representative local templates and verify requested template selection, content retention, and invalid-path refusal. | FR-036-AC-1 through FR-036-AC-6 |
| Export identical inputs repeatedly under varied clocks and host metadata; compare output bytes for stability. | FR-037-AC-1 through FR-037-AC-6 |
| Export ordered multi-document fixtures; compare every included document, order, bundle identity, and refusal case. | FR-039-AC-1 through FR-039-AC-8 |
| Run invalid identity, template, approval, and bundle fixtures; assert precise refusals and absence of completed output. | FR-040-AC-1 through FR-040-AC-8 |
| Run design-relation fixtures with complete, missing, and rejected relations; compare reported design status with source edges. | FR-041-AC-1 through FR-041-AC-11 |
| Export diagrams with known Mermaid inputs and compare generated images and their placement with the source blocks. | FR-042-AC-1 through FR-042-AC-15 |
| Run design matrix fixtures across formats, empty populations, errors, and strict mode; compare design cells and verdicts. | FR-043-AC-1 through FR-043-AC-27 |
| Run flow fixtures with known nodes, edges, and gaps; compare emitted flow relations and findings with source declarations. | FR-044-AC-1 through FR-044-AC-16 |
| Run health-data-flow fixtures with complete and incomplete relations; compare each reported flow and gap with declarations. | FR-046-AC-1 through FR-046-AC-10 |
| Check that the repository vulnerability policy names a working private reporting route and avoids publishing sensitive report details. | FR-048-AC-1 through FR-048-AC-5 |
