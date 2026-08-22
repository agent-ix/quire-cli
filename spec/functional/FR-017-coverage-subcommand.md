---
id: FR-017
title: "quire coverage subcommand"
type: FR
relationships:
  - target: "ix://agent-ix/quire-cli/spec/stakeholder/StR-004"
    type: "implements"
    cardinality: "1:1"
---

## Description

The CLI SHALL provide a `coverage` subcommand that surfaces the quire-rs
declarative coverage rollup (upstream [FR-050](ix://agent-ix/quire-rs/FR-050))
over a repository, reconciling the trace ids a module's `traceability:` model
declares against the source symbols that carry trace tags.

```
quire coverage [--scope <DIR>] [--module <PATH>] [--json | --format <human|json|tsv>]
               [--strict] [--severity coverage:<check>=<level>]...
```

This command shipped in v0.13.0 and was authored as a requirement only in
2026-08-16 (see the CR note below). Its criteria are **read off working code**,
not proposed: it is the most behavior-visible surface this CLI has added in
three releases, and it had no FR, no acceptance criteria and no matrix rows of
its own.

## Behavior

### §A — Two roots from one scope

`--scope` names the repository root. Two roots derive from it and are never
interchanged (quire-rs FR-050-AC-17, CR-045):

- the **document root** `<scope>/spec`, which the corpus is walked from;
- the **code root** `<scope>`, walked for source symbols with the document
  root excluded.

`<scope>` stays the relativization base for every path in the report, so a
compliant repository's output is byte-identical to a pre-split run. A `--scope`
holding no `spec/` is a named error (`MissingDocumentRoot`), never a silent
fallback to walking the scope. `spec/` is convention, not configuration.

### §B — The model is module data

The `traceability:` model comes from the discovered module set, or from
`--module`. With no model in scope the command **refuses**, naming the missing
declaration — it does not guess, since guessing is exactly the agent-grep
behavior the command replaces.

### §C — Report, not verdict

Default output is a human census on **stderr**; `--json` emits the
`CoverageReport` payload on **stdout**, which is the **stable interface** (the
human form may change). The split is deliberate: stdout carries the payload and
nothing else, so `quire coverage --json | jq` is safe with the census still
visible on a terminal. `--strict` exits 1 when any row is unbacked or any status is
contradicted; it is **off by default**, because whether a gap blocks is the
consuming workflow's policy, not this command's
([FR-050-CON-1](ix://agent-ix/quire-rs/FR-050)).

A model matching **zero** rows is reported as itself and fails `--strict`
separately from a clean run — "found nothing" and "all covered" are opposite
states (quire-rs FR-050-AC-14, CR-035).

### §D — Agent-sized projection (#53)

The primary consumer of this command is an agent, and the full `--json`
payload on a real corpus (~310k tokens) exceeds what one can read. Two levers,
both reusing shapes that already exist:

- **The `coverage` severity pack** — `--severity coverage:<check>=<level>`
  over the checks `unbacked-row`, `status-lie`, `untracked-symbol`,
  `undeclared-status`, riding the same FR-048 machinery (module
  `grammar_severity` ∪ CLI flag, reject-before-read) as `validate
  --severity`. `off` is the projection lever; `error` a per-check gate.
- **`--format tsv`** — the same records, one tab-separated line each on
  stdout (~36% of the JSON size measured), row id leading the data cells and
  a `line` column carrying the engine's 1-based line where the record has one
  (quire-rs v0.42.0, FR-050-AC-26; the column predates the data, so its
  arrival needed no format change). `--json` stays exactly as it is: the
  stable program contract.

Projection moves the payload cliff; it does not remove it — a large enough
corpus still needs a file or NDJSON, which is deliberately out of scope until
a consumer needs the whole rollup.

## Acceptance Criteria

| ID | Criteria | Verification |
|----|----------|--------------|
| FR-017-AC-1 | `quire coverage --scope <DIR> --module $M` over a repository whose model matches rows exits **0** and renders the human **census** — the bundle total and the per-document counts — on **stdout**, uncolorized; findings stay on stderr. `--json` and `--format tsv` are unaffected: the census is emitted only in human mode, so a machine payload is still the only thing on stdout in those modes (amended CR-012) | Test (IT-089, IT-117) |
| FR-017-AC-2 | `--json` emits the `CoverageReport` payload on stdout, and two runs over identical inputs are byte-identical | Test (TC-740) |
| FR-017-AC-3 | The document root is `<DIR>/spec`: a perfectly typed matrix at the repository root or under `plan/` mints nothing, and repo-root `README.md`/`CHANGELOG.md` are never read as documents | Test (TC-810) |
| FR-017-AC-4 | The code walk excludes the document root: a trace-tagged source file under `<DIR>/spec` contributes no symbol, while the same file outside it does | Test (IT-087) |
| FR-017-AC-5 | A `--scope` holding no `spec/` exits non-zero with a diagnostic naming the missing document root by path, carrying machine `kind` `MissingDocumentRoot` under `--diagnostics-format json` | Test (TC-811, IT-086) |
| FR-017-AC-6 | With no `traceability:` model in scope the command exits non-zero naming the missing declaration, and computes nothing | Test (TC-740) |
| FR-017-AC-7 | `--strict` exits 1 when a row is unbacked or a status is contradicted, and exits 0 on the same repository without it | Test (IT-092) |
| FR-017-AC-8 | A model matching zero rows fails `--strict` with a diagnostic distinct from the unbacked-rows one, rather than reporting full coverage | Test (TC-797) |
| FR-017-AC-9 | (thin boundary) reconciliation is delegated entirely to quire-rs (`compute_coverage`, `extract_tree_scoped`, `trace::bind`); the CLI resolves the two roots, applies path-safety, and renders ([StR-004](../stakeholder/StR-004-thin-boundary-over-quire-rs.md)) | Inspection (TC-090) |
| FR-017-AC-10 | A row whose status value the module's `traceability.status` classes as nothing is reported on **both** surfaces — as an `undeclared_statuses` entry in `--json` and as a rendered line in the default human census — carrying the authored value verbatim. `--strict` does not gate on it: the exit code for a report whose only finding is an undeclared status is the same with and without the flag (CR-083) | Test (IT-107) |
| FR-017-AC-11 | A module-declared `traceability.source_exclude` glob reaches the source walk: a tagged file matching one contributes no symbol, a tagged file outside every glob still does, and a scope whose module declares none behaves exactly as before (CR-085) | Test (IT-108) |
| FR-017-AC-12 | Each human-census unbacked-row, status-lie and undeclared-status line leads with the row's own id when the record carries one (the declaration names a `row_id_column`) and keeps the reference kind visible in a bracketed trailer — `TC-123 (doc.md) has no backing symbol [traces-to]` — so two rows in the same document render distinguishable lines; a record without a row id renders the reference kind leading, exactly as before. The `--json` payload is unchanged (#51) | Test (IT-109, IT-107) |
| FR-017-AC-13 | `--severity coverage:<check>=<level>` (checks: `unbacked-row`, `status-lie`, `untracked-symbol`, `undeclared-status`) rides the FR-048 severity machinery: entries layer over module `grammar_severity`, and a malformed entry — or a `coverage:` entry naming a check outside the pack's four, which FR-048's shape-only key validation would otherwise merge as a silent no-op (#57) — is rejected before any document is read. `off` drops the kind's records from **every** output surface (human, `--json`, `--format tsv`), announcing each non-empty suppression on stderr with its count. **Totals semantics:** `totals` and `groups` always describe the full reconciliation, computed before projection, and `--strict` gates on the full computation — projection changes what is rendered, never what is judged. `error` exits 1 when the kind has findings, without `--strict` (#53) | Test (IT-110) |
| FR-017-AC-14 | `--format tsv` emits one tab-separated record per line on stdout: a header naming the nine fixed columns (`kind id document reference status method line targets text`), every record carrying every column (empty where the kind has no value), row id leading the data cells, id lists flattened with `,`, tab/newline in free text replaced by spaces, obligation `parameters` omitted. The `line` column carries the record's 1-based document line where the engine provides one (quire-rs v0.42.0, FR-050-AC-26) and stays empty where it does not — the arrival the column was reserved for, needing no format change. Ordering mirrors the JSON arrays; output is byte-identical across runs (#53) | Test (IT-111, TC-812) |
| FR-017-AC-15 | `--json` honours the global `--pretty`: compact single-line by default ([FR-008](./FR-008-json-output-encoding.md)-AC-1), indented with the flag, the identical parsed value either way. Byte-identity across runs (AC-2) holds in both shapes (#53) | Test (IT-119) |
| FR-017-AC-16 | When a finding record carries the engine's 1-based `line` (quire-rs v0.42.0, FR-050-AC-26), the human line's parenthesized locus is the clickable `document:line` form — `TC-123 (spec/tests.md:9) has no backing symbol [traces-to]` — for unbacked-row, status-lie, undeclared-status and no-symbol-row lines; a record without a line renders the bare document exactly as before. The `--json` payload is unchanged (#51 item 3) | Test (IT-113, IT-107) |
| FR-017-AC-17 | A `no_symbol_rows` record renders in the default human census like every other row-id-carrying kind — row id leading, `document:line` locus, reference kind in the bracketed trailer, naming the exempting test-type value: `TC-123 (doc.md:7) is verified by …, which mints no source symbol [traces-to]`, with the value rendered verbatim in backticks where the ellipsis stands. It was JSON/TSV-only; the record explains an unbacked row the census does print, and an explanation only the machine surface carries is one nobody reads (#51, the CR-083 argument) | Test (IT-114) |
| FR-017-AC-18 | The subtraction a declared `source_exclude` makes is observable on the human surface: a census line `N source file(s) excluded by source_exclude` renders when N > 0 and nothing renders at zero, and every `SymbolExtraction` diagnostic — a refused glob list (quire-rs FR-050-AC-25), an unreadable source file — reaches stderr instead of being computed and dropped (#51, quire-rs #215) | Test (IT-115) |
| FR-017-AC-19 | The v0.42.0 advisory report lists pass through `--json` unmodified: `shared_trace_ids` (quire-rs FR-050-AC-23) carries every status-carrying row id bound by more than one distinct symbol, and `vocabulary_coverage` (FR-059-AC-9) serializes through the same wholesale report encoding — both absent when empty, preserving AC-2 byte-identity for conformant corpora. Neither has a human rendering in this release; that is a deliberate deferral, not an omission (#51 batch note) | Test (IT-116); Inspection (`vocabulary_coverage` — the CLI serializes the whole `CoverageReport`, and the severity projection does not touch either list) |

> **CR note (authored after the fact, 2026-08-16):** this document did not
> exist while the command shipped, changed its default root (PR #27) and
> changed what it parses (PR #29). The gap was found by
> agent-ix/quire-cli#31: `coverage` and `properties` had no owning requirement
> anywhere in this repo's spec, so three releases of behavior change had
> nothing to be measured against and no matrix row to contradict. The criteria
> above are read off the shipped command and its existing tests rather than
> proposed, on the FR-019/FR-020/FR-022 backfill pattern from quire-rs CR-042.

## Dependencies

- **Upstream**: [StR-004](../stakeholder/StR-004-thin-boundary-over-quire-rs.md) thin boundary over quire-rs; quire-rs [FR-050](ix://agent-ix/quire-rs/FR-050) (declarative coverage computation), [FR-051](ix://agent-ix/quire-rs/FR-051) (source-symbol extraction).
- **Downstream**: the `gap-analysis` workflow, which consumes the `--json` payload as data and owns the verdict policy.


> **CR-012 note (2026-08-22):** AC-1 is **amended** — the census moves to
> stdout. `agent-ix/quire-cli#59`, `agent-ix/quire-cli#60`; epic
> `agent-ix/quoin#197`.
>
> **This corrects a contradiction, it does not introduce a new contract.**
> [FR-006](./FR-006-io-contract.md) has said since v0.1 that every subcommand
> puts the *primary result on stdout, all diagnostics on stderr*, and
> FR-006-AC-2 requires a success case to produce **non-empty stdout** (except
> `validate`). AC-1 said the opposite for the human surface, and the
> implementation followed AC-1: `write_diagnostic_human` is `eprintln!` wrapped
> in `RED`, and every human line went through it.
>
> **What that cost, measured** over `agent-ix/filament-ide-rs` under
> `quire 0.29.0`:
>
> ```text
> quire coverage --scope . > out.txt                  # 0 bytes; 90,462 to stderr
> quire validate --scope . "spec/**/*.md" > out.txt   # 0 bytes; 62,256 to stderr
> quire properties --scope . 'spec/**/*.md' > out.txt # 0 bytes
> ```
>
> Three consequences: the obvious command silently produced an empty file; a
> census — `Coverage: 1238/2390 rows backed (51%)`, `755/797 docs grammar-clean
> (94%)` — rendered in the **same red as every finding**; and a caller could not
> pipe findings without the summary interleaved.
>
> **The #51 WONTFIX is not overturned.** That decision declined *"findings to
> stdout"* and routed it to `--format tsv`, on the grounds that stdout should
> carry only a machine payload so `--json | jq` keeps working. Both halves still
> hold: **findings stay on stderr**, and the census is emitted only in the human
> branch — `--json` and `--format tsv` put nothing else on stdout, so `| jq` is
> untouched. What moves is the census, which is a *result*, and which that
> decision did not consider.
>
> **Never colorized.** A number is not a severity, and a census rendered in
> error red was half the defect.

> **CR-011 note (2026-08-20):** AC-10 and AC-11 are new — the CLI carries the
> two capabilities quire-rs v0.41.0 added, in the same release rather than the
> next one.
>
> Both are pure reachability. `undeclared_statuses` (quire-rs CR-083) and
> `source_exclude` (CR-085) are **inert** until this crate renders the one and
> passes the other: an engine field nothing prints is a finding nobody reads,
> and a manifest key nothing forwards is a declaration with no effect whatsoever.
>
> That is the defect the previous programme phase kept finding — `quire-cli` five
> releases behind, so FR-057..061 were unreachable from any command line;
> `vocabulary_coverage` shipped and declared by no module. The mechanical tell it
> recommends is **grep for the symbol from the surface a user actually invokes**,
> and IT-107/IT-108 are that grep written down.
>
> `--strict` deliberately does **not** gate on an undeclared status. Shipping it
> as a gate would flip repositories red on an engine bump for a condition nobody
> has been told about; promotion is a separate, measured, user-gated decision
> once the corpus is clean. IT-107 asserts the exit code is unchanged, over a
> fixture whose rows are all backed so the undeclared status is the only finding
> — otherwise the assertion would pass on unbacked rows, which `--strict` gates
> on legitimately.

> **CR note (row-id-leading findings, 2026-08-21, #51):** AC-12 is new. The
> human renderer printed the reference **kind** per finding line (`traces-to
> (spec/tests.md) has no backing symbol`, four identical lines for four rows),
> while `UnbackedRow`, `StatusLie` and `UndeclaredStatus` all already carry
> `row_id: Option<String>` — the id was computed, serialized to `--json`, and
> never rendered. That omission is why downstream consumers parsed the
> megabyte-scale JSON payload to answer questions the human output had
> already computed. Of the row-id-carrying record kinds, `no_symbol_rows`
> still had no human renderer at all at this point (JSON-only), and the
> `path:line` prefix waited on the engine — both closed by the v0.42.0
> batch below (AC-16/AC-17). The stdout/stderr split of AC-1 is
> deliberately untouched here.

> **CR note (#51 close-out, 2026-08-21, quire-rs v0.42.0 batch):** AC-16..19
> are new; AC-14's `line` column is now populated. Three decisions recorded:
>
> 1. **`no_symbol_rows` renders (AC-17).** The consistent move: it is the
>    fourth row-id-carrying kind, and the record exists to *explain* an
>    unbacked row the census already prints — leaving the explanation
>    JSON-only re-creates exactly the finding-nobody-reads defect AC-10
>    closed for `undeclared_statuses`. It is deliberately outside the AC-13
>    severity pack: the pack's checks mirror engine-gateable kinds, and a
>    no-symbol row is an exemption note, not a gateable finding.
> 2. **#51 item (2) — findings to stdout — is WONTFIX, superseded by
>    `--format tsv` (AC-14).** TSV puts full, row-id-carrying,
>    line-carrying records on stdout in grep/cut-able form, which is what
>    the item was for, without breaking AC-1's invariant that stdout carries
>    only a machine payload (`quire coverage --json | jq` with the census
>    still visible). AC-1 stands unamended, deliberately.
> 3. **`shared_trace_ids` / `vocabulary_coverage` (AC-19) get no human
>    rendering this release.** Both are advisory-first engine lists
>    (quire-rs CR-087/CR-091); they pass through `--json` wholesale, and a
>    human/TSV rendering is deferred until a consumer asks — the same
>    promotion-is-a-measured-decision posture as AC-10's `--strict` stance.

> **CR note (pre-release review fixes, 2026-08-21, #57 / SR-007):** two
> AC-13/AC-14 hardenings out of the v0.29.0 review gate. (1) AC-13's check
> vocabulary is **closed**: FR-048 validates only a `--severity` key's shape
> (deliberate for `validate`, where grammars are module-declared and open),
> so a typo'd coverage check merged, matched nothing, and silently did not
> project — the entry is now rejected before any document is read, naming
> the four valid checks. Entries for other grammars keep the FR-048 open
> posture unchanged. (2) AC-14's tab/newline-replacement clause gains a
> pinning test (TC-812): the #53 measurement found 0/1,107 statements
> carrying a structural character, so no corpus fixture could exercise the
> guard and mutating it to the identity left the suite green.

> **CR note (agent-first output, 2026-08-21, #53):** AC-13..15 are new (§D).
> One deliberate change to an existing surface rides with them: the `--json`
> payload was unconditionally pretty-printed (`to_string_pretty`) while the
> global `--pretty` was silently ignored; AC-15 brings `coverage` onto the
> FR-008-AC-1 posture every other JSON surface has. Whitespace-only — the
> payload parses identically, `--pretty` reproduces the previous byte shape,
> and AC-2's cross-run byte-identity holds in both. `--format tsv` is also
> the first rendered surface `no_symbol_rows`, `diagnostics` (where
> `uncatalogued-verification-method` reports), `obligations` and
> `implements` have ever had.

