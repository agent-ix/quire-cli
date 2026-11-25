---
id: FRR-002
title: "Quire CLI proposed not-high process-risk supplier ratings"
type: FeatureRiskRating
scope: supplier
rating: not_high_process_risk
rated_by: Peter Krenesky
relationships:
  - target: "ix://agent-ix/quire-cli/IU-001"
    type: uses_intended_use
  - target: "ix://agent-ix/quire-cli/FR-001"
    type: rates_feature
  - target: "ix://agent-ix/quire-cli/FR-002"
    type: rates_feature
  - target: "ix://agent-ix/quire-cli/FR-006"
    type: rates_feature
  - target: "ix://agent-ix/quire-cli/FR-008"
    type: rates_feature
  - target: "ix://agent-ix/quire-cli/FR-009"
    type: rates_feature
  - target: "ix://agent-ix/quire-cli/FR-011"
    type: rates_feature
  - target: "ix://agent-ix/quire-cli/FR-013"
    type: rates_feature
  - target: "ix://agent-ix/quire-cli/FR-016"
    type: rates_feature
  - target: "ix://agent-ix/quire-cli/FR-018"
    type: rates_feature
  - target: "ix://agent-ix/quire-cli/FR-035"
    type: rates_feature
  - target: "ix://agent-ix/quire-cli/FR-036"
    type: rates_feature
  - target: "ix://agent-ix/quire-cli/FR-048"
    type: rates_feature
---
# Quire CLI proposed not-high process-risk supplier ratings

## Feature assessments

| Requirement | Failure mode | Detection |
|---|---|---|
| FR-001 | A rendering defect could omit or relocate requirement text in a Markdown document used for review. | Render representative valid and malformed inputs; compare the generated Markdown structure and text with the expected output. |
| FR-002 | Parsed fields could differ from the source document, so a downstream consumer sees altered structured data. | Parse valid, invalid, and boundary documents; compare each JSON field and diagnostic with the source and expected result. |
| FR-006 | A result or diagnostic could be sent on the wrong stream, causing automation to miss or misread an observation. | Exercise stdin, stdout, and stderr cases and compare bytes on each stream with the command contract. |
| FR-008 | JSON escaping or encoding could change text in structured output consumed as a document record. | Round-trip Unicode, control characters, and escaping cases through JSON output and compare the decoded value with the source. |
| FR-009 | The emitted schema could disagree with the accepted input contract and mislead authors about valid records. | Compare the schema subcommand output with the selected module contract and test the documented valid and invalid shapes. |
| FR-011 | Lookup could return a different section or record than requested, leading a reviewer to inspect the wrong content. | Query fixtures by each supported lookup key and compare the returned section or record with the exact requested target. |
| FR-013 | An advisory lint result could be missing or attached to the wrong document content, misleading an author about cleanup. | Run lint fixtures with and without advisory findings; compare finding locations, messages, and non-gating status. |
| FR-016 | An update could fail to install the expected CLI version, leaving users to produce records with an unexpected executable. | Exercise supported install sources and unsupported cases; verify the resulting version/source or explicit refusal. |
| FR-018 | A property-shape classification could be wrong and mislead authors about the kind of check a criterion describes. | Run representative criteria through each property-shape case and compare the classification with the declared rule. |
| FR-035 | A template could render headings or page furniture poorly, making a document harder to review without changing its source facts. | Export fixtures through the default template and inspect title, contents, headers, footers, and pagination output. |
| FR-036 | An organization template could render content awkwardly or obscure it, while leaving source document data unchanged. | Export with representative local templates and verify requested template selection, content retention, and invalid-path refusal. |
| FR-048 | Vulnerability-reporting instructions could be absent or inaccurate, making it harder for a reporter to reach the intended contact. | Check that the repository vulnerability policy names a working private reporting route and avoids publishing sensitive report details. |

## Rationale

This is a proposed supplier-side assessment for IU-001. The `rated_by` field names Peter Krenesky as the human assigned to review and accept the proposed rating. This record is unsigned and unaccepted pending his decision through the signed-commit workflow. No score or product safety classification is assigned.

These features primarily provide document presentation, lookup, encoding, advisory, or installation conveniences. Their failure can create friction, a visible diagnostic, or a representation that users can compare with the source; they do not themselves establish validation, coverage, relation completeness, or approval. The proposed not-high process-risk rating calls for focused checks against the explicit acceptance criteria listed below.

## Assurance activities

| Activity | Acceptance criterion |
|---|---|
| Render representative valid and malformed inputs; compare the generated Markdown structure and text with the expected output. | FR-001-AC-1 through FR-001-AC-6 |
| Parse valid, invalid, and boundary documents; compare each JSON field and diagnostic with the source and expected result. | FR-002-AC-1 through FR-002-AC-5 |
| Exercise stdin, stdout, and stderr cases and compare bytes on each stream with the command contract. | FR-006-AC-1 through FR-006-AC-5 |
| Round-trip Unicode, control characters, and escaping cases through JSON output and compare the decoded value with the source. | FR-008-AC-1 through FR-008-AC-6 |
| Compare the schema subcommand output with the selected module contract and test the documented valid and invalid shapes. | FR-009-AC-1 through FR-009-AC-5 |
| Query fixtures by each supported lookup key and compare the returned section or record with the exact requested target. | FR-011-AC-1 through FR-011-AC-7 |
| Run lint fixtures with and without advisory findings; compare finding locations, messages, and non-gating status. | FR-013-AC-1 through FR-013-AC-5 |
| Exercise supported install sources and unsupported cases; verify the resulting version/source or explicit refusal. | FR-016-AC-1 through FR-016-AC-7 |
| Run representative criteria through each property-shape case and compare the classification with the declared rule. | FR-018-AC-1 through FR-018-AC-10 |
| Export fixtures through the default template and inspect title, contents, headers, footers, and pagination output. | FR-035-AC-1 through FR-035-AC-9 |
| Export with representative local templates and verify requested template selection, content retention, and invalid-path refusal. | FR-036-AC-1 through FR-036-AC-6 |
| Check that the repository vulnerability policy names a working private reporting route and avoids publishing sensitive report details. | FR-048-AC-1 through FR-048-AC-5 |
