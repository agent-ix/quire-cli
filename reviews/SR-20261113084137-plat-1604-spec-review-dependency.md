---
id: "SR-20261113084137"
title: "PLAT-1604 quire-cli PR189 spec-review/dependency"
type: SpecReview
analysis: dependency
scope: "agent-ix/quire-cli PR189 head 1a40cc4; spec/functional/FR-007, FR-031, FR-032, FR-033, FR-042, spec/non-functional/NFR-010, NFR-011, spec/stakeholder/StR-004, spec/usecase/US-009, spec/spec.md"
review_set: subset
---

## Summary

Ticket: PLAT-1604. New relationship edges and the stated upstream requirements. Checked against quire-rs origin/main fb0a09b src/query.rs (`DiagramBlock`, `extract_diagrams`, `scan_fenced_blocks_with_lines`) and EA origin/main FR-019/FR-029. The EA dependency holds: an mmdr request fits the closed request fields, its budgets are inside the FR-019 limits, the response adapter accepts exit 0, and retained outputs stay in the capability root for pandoc to read. Confirmed: `DiagramBlock` has `index`, `language`, `source`, `section` and `tag`, and no line field. Four findings: the upstream ask misdescribes `index` and asks for an ordinal that already exists; the ask is not filed anywhere; it leaves out fence-length matching, which AC-8 and the identity data depend on; and the edge type conflicts with StR-004-AC-1.

## Findings

| ID | Severity | Summary | Refs | Escape Cause |
| --- | --- | --- | --- | --- |
| FND-001 | medium | The upstream statement says `extract_diagrams` returns "a 0-based index among all fenced blocks" and asks quire-rs to add "the language-filtered ordinal". In quire-rs `push_block`, the function returns early for a non-matching language before `*index += 1`, so with `Some("mermaid")` the `index` is already the 0-based ordinal among mermaid blocks. Only the doc comment says otherwise, and that is a quire-rs doc bug. The 0-based opening-fence line also already exists inside quire-rs (crate-private `scan_fenced_blocks_with_lines`, relative to the body with frontmatter stripped). Fix: restate the ask as "expose the opening-fence line on `DiagramBlock`, 1-based, in full-file coordinates", drop the ordinal request, and note the doc-comment correction. | spec/functional/FR-042-export-mermaid-diagrams.md:199 | wrong-requirement |
| FND-002 | medium | FR-042 "depends on that addition", but no quire-rs requirement or ticket for it exists (a Linear search for `extract_diagrams` finds none), and the `consumes` edge points at FR-010, which does not have the field. The spec cannot be implemented as written until quire-rs changes. Fix: file the quire-rs requirement, cite its id in Dependencies, and make the implementation ticket blocked-by it in Linear. | spec/functional/FR-042-export-mermaid-diagrams.md:199 | wrong-requirement |
| FND-003 | medium | The upstream text keeps quire-rs's current fence model ("backtick and tilde, matching close"), which has no fence-length rule. In that model a 3-backtick line closes a 4-backtick fence, and a ```` ````mermaid ```` opener yields language "". FR-042-AC-8 (a mermaid block inside a longer backtick fence) passes only by accident under that model. Headings and lines after such a fence are misattributed, so the identity data in diagnostics can be wrong. Fix: the upstream ask should require CommonMark fence rules (a close fence of the same character and at least the opening length; the info word after any fence length). | spec/functional/FR-042-export-mermaid-diagrams.md:190 | wrong-requirement |
| FND-004 | low | FR-042 links quire-rs FR-010 with `type: consumes`. StR-004-AC-1 requires an FR that wraps a quire-rs API to declare the upstream FR with `type: implements`. FR-031 is mixed precedent. Fix: use `implements`, or amend StR-004-AC-1 to accept `consumes`. | spec/functional/FR-042-export-mermaid-diagrams.md:19-20 | wrong-requirement |

## Examined scope

```yaml
scope:
- id: FR-042#dependencies-quire-rs
  path: spec/functional/FR-042-export-mermaid-diagrams.md
  role: examined
  excerpt: '**Upstream, quire-rs requirement needed**: `extract_diagrams` of [quire-rs FR-010](ix://agent-ix/quire-rs/FR-010)
    returns a 0-based index among all fenced blocks, the language, the source and the section heading,
    but no line position. quire-rs must add to each fenced-block record the 1-based line of the opening
    fence in the full file (frontmatter included) and the language-filtered ordinal, with the fence recognition
    of its FR-007 and FR-011 CR-004 (backtick and tilde, matching close). This FR depends on that addition;
    quire-cli SHALL NOT compute positions itself.'
- id: FR-042#A-find
  path: spec/functional/FR-042-export-mermaid-diagrams.md
  role: examined
  excerpt: '`quire` SHALL find the blocks with the fenced-block query of quire-rs ([quire-rs FR-010](ix://agent-ix/quire-rs/FR-010)
    `extract_diagrams`, filtered to the language `mermaid`), over the committed content that [FR-034](./FR-034-document-identity.md)
    section A validates and converts. It SHALL NOT scan the Markdown text for fences itself, because Markdown
    parsing stays in quire-rs ([StR-004](../stakeholder/StR-004-thin-boundary-over-quire-rs.md)).'
- id: FR-042#A-identity
  path: spec/functional/FR-042-export-mermaid-diagrams.md
  role: examined
  excerpt: Each block is identified by the input path, its ordinal among the `mermaid` blocks of that
    document (1-based, in document order), its first line number in the committed file (1-based, counting
    the frontmatter), and the heading of the section that holds it, when there is one. The line number
    and the section come from the quire-rs record; the upstream requirement is stated in Dependencies.
- id: FR-042#relationships
  path: spec/functional/FR-042-export-mermaid-diagrams.md
  role: examined
  excerpt: "- target: \"ix://agent-ix/quire-rs/FR-010\"\n    type: \"consumes\""
- id: FR-042-AC-8
  path: spec/functional/FR-042-export-mermaid-diagrams.md
  role: examined
  excerpt: A fixture holding a `mermaid` block inside a longer backtick fence of another language, and
    a `~~~mermaid` block, export with images for the second only and the first printed as code.
- id: FR-042#B-request
  path: spec/functional/FR-042-export-mermaid-diagrams.md
  role: examined
  excerpt: 'Each `mmdr` request SHALL be the complete field set of the request table in [FR-032](./FR-032-export-converter-execution.md)
    with these definitions, and the fields of that table not named here apply unchanged: `producer.name`
    `mmdr`; `arguments` `-i`, `in/diagrams/<k>.mmd`, `-o`, the image path, `-e`, `png`, in that order,
    where the image path is `img/<k>.png` for DOCX and `typ/img/<k>.png` for PDF; `outputs` one entry,
    role `diagram`, that same path, `required` true; `environment` as FR-032; `budget` `timeoutMillis`
    60000, `maxOutputArtifacts` 1, `maxConcurrency` 1 and the other bounds of FR-'
- id: FR-032#A-mmdr
  path: spec/functional/FR-032-export-converter-execution.md
  role: examined
  excerpt: A Markdown input with Mermaid blocks SHALL first make the `mmdr` requests of [FR-042](./FR-042-export-mermaid-diagrams.md),
    which add `--lua-filter` and the diagram files to the pandoc request there defined. The pandoc request
    is made only if all of them completed.
- id: StR-004#bounded-exception
  path: spec/stakeholder/StR-004-thin-boundary-over-quire-rs.md
  role: examined
  excerpt: and the generation of the diagram files and pandoc filter of [FR-042](../functional/FR-042-export-mermaid-diagrams.md),
    which locates Mermaid blocks only through the quire-rs fenced-block query and writes their text as
    renderer input, with no Markdown scan of its own.
```

## Verdict

FAIL (medium). The quire-rs ask must be restated precisely (FND-001, FND-003) and filed (FND-002) before the implementation ticket can start. The EA producer-execute dependency is sound.

## New findings (disposition pass 1)

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-005 | low | This was introduced by the fix. FR-042 states the ask as including "the info word read after the whole fence", which FR-042-AC-8's four-backtick `mermaid` block depends on. The acceptance criteria of PLAT-1682 list fence length, indentation and fence characters, but not the info word after a fence longer than three characters. Fix: add that criterion to PLAT-1682, so the ticket and FR-042 state the same requirement. | spec/functional/FR-042-export-mermaid-diagrams.md:243; Linear PLAT-1682 |

## Dispositions

| FND | outcome | reason |
| --- | --- | --- |
| FND-001 | fixed | - |
| FND-002 | fixed | - |
| FND-003 | fixed | - |
| FND-004 | fixed | - |
