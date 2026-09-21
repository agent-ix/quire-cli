---
name: trace
description: Use when an agent needs to answer "what backs this requirement" or "what does this file/symbol verify" over a repository's declared traceability model with quire-cli. Covers quire trace forward (--id) and inverse (--symbol/--file) lookup, the claims-vs-citations distinction, --prefix, --exclude-path, and when NOT to reach for it.
metadata:
  short-description: Structural trace lookup
---

# Trace

Use this skill instead of hand-rolling `grep`/`rg`/`jq` for "what backs FR-047" or
"what does this file verify." `quire trace` is the default path for both
questions — reach for it first, and only fall back to a manual pipeline when
you can say why `trace` cannot answer the question (see "When NOT to use
this" below).

## Forward lookup: what backs this id

```bash
quire trace --id FR-047 --module path/to/module --json
```

Exactly one of `--id` / `--symbol` / `--file` is required. `--module` is not
a required *flag* — omit it and `trace` falls back to a `manifest.yaml` at
`--scope`, then to ambient module discovery, the same resolution `quire
coverage` uses — but a declared `traceability:` model is required for the
graph to compute at all, from wherever it resolves. Pass `--module`
explicitly whenever the ambient/ambient-manifest resolution might not be the
module you mean. A scope with no model anywhere fails loudly rather than
guessing.

## Inverse lookup: what does this symbol or file verify

```bash
# One symbol: {path}#{qualified_name}
quire trace --symbol src/foo.rs#Foo::bar --module path/to/module --json

# A bare, unqualified name — every symbol sharing it. Ambiguous names list
# every candidate in `ambiguous_matches`; the tool never picks one for you.
quire trace --symbol bar --module path/to/module --json

# Every claim/citation in one file.
quire trace --file src/foo.rs --module path/to/module --json
```

## Claims vs citations — read this before you script against the JSON

**The whole reason this tool exists.** The JSON payload has two separate top-level
keys:

- `claims.verifies` / `claims.implements` — trace tags a module's declared
  grammar actually recognized. This is real evidence: a `verifies` claim is a
  test that would fail if the requirement broke.
- `citations` — everything else: near-miss tags, orphaned legacy forms, and
  plain id-shaped text sitting in a comment, doc comment, or string literal.
  **A citation is NOT verification evidence.** It is a place the id is
  *mentioned*, nothing more.

Never write `jq` that merges `.claims` and `.citations` into one list, and
never treat a nonzero `.citations` count as "this is covered." The human
output enforces the same split visually: a `Claims (N)` section, then a
`Citations (N) — NOT verification evidence` section, always in that order,
always visually separated.

```bash
quire trace --id FR-047 --module path/to/module --json | jq '.claims'
quire trace --id FR-047 --module path/to/module --json | jq '.citations'
```

`--json` (or `--format json`) is required for `jq` to have anything to parse
— the default output is the human form, and piping it into `jq` fails with a
parse error, not a quiet no-op.

## Every record's confidence is reported, not assumed

Each claim/citation record carries `language` and `confidence`
(`structural` | `line_heuristic`), read off the engine's own extraction for
that specific record — never a fixed "Rust is trustworthy, Python isn't"
table. Check `confidence` per record rather than assuming by file extension;
which languages are `structural` changes as quire-rs ports more extractors.
The human form prints a one-line caveat on stderr whenever a result set
contains any `line_heuristic` record — treat those the way you would treat a
grep hit, not an AST-grounded fact.

## `--prefix` — the FR/AC/TC family, explicitly

`--id` matches **exactly** by default: `--id FR-047` never pulls in
`FR-047-AC-1`. Add `--prefix` to match the whole family on a separator
boundary:

```bash
quire trace --id FR-047 --prefix --module path/to/module --json
```

`FR-047` with `--prefix` matches `FR-047-AC-1` and `FR-047-CON-2`, but never
`FR-0470` — the boundary is enforced, not a loose substring match.

## `--exclude-path` — silence known-noisy paths

The id-shaped pattern this tool's citation channel uses is generic and will
match things that are not real citations — most commonly an SPDX license
header (`AGPL-3.0-or-later` reads as an id-shaped `AGPL-3` token). If a
particular path in your repo is a known source of this noise (a fixture tree,
generated code, vendored files), drop its citations explicitly:

```bash
quire trace --id FR-047 --module path/to/module --exclude-path 'fuzz/**' --json
```

This only ever removes `citations`. It never removes a `claims` record —
those come from a declared trace-tag form, not the generic pattern, so there
is no equivalent false-positive class to filter there.

## Zero matches is a normal result, not an error

```bash
quire trace --id FR-999-NOWHERE --module path/to/module --json
# exit 0, {"resolved": false, "claims": {...empty...}, "citations": [], ...}
```

Exit code 0 always means "the query ran"; check `.resolved`, not the exit
code, to learn whether anything matched. A nonzero exit means the query
itself was malformed (bad flags) or the module has no declared model — not
"nothing was found."

## Grouped output for dense ids

Some ids are cited hundreds of times in one repository (common placeholder
ids like `FR-001`, or a repo-wide false-positive class like `AGPL-3`). The
human form automatically groups citations by file with counts once the count
gets large; `--format json` is never grouped or truncated — always request
JSON when you need every individual citation.

## When NOT to use this tool

- **Semantic or fuzzy search.** `trace` answers structural questions about a
  *declared* traceability model — exact id matches, exact/bare symbol
  matches, whole-file listings. It has no notion of "documents related to
  billing" or "code that looks like it's about X." Use your normal
  code-search tools for that.
- **No module declares a `traceability:` model.** `trace` refuses rather than
  guessing (the same refusal `quire coverage` gives). If the repository has
  no such module, this tool has nothing to compute over.
- **You need FR/AC/TC hierarchy walked automatically.** The engine matches
  ids exactly and knows nothing of that hierarchy by design; `--prefix` gets
  you the family, but only on request — `trace` will never silently widen a
  query.
- **You need coverage verdicts (backed/unbacked, status lies).** That is
  `quire coverage`, a different reconciliation over the same graph. `trace`
  only answers "what claims or cites this," not "is this requirement
  satisfied."

## Reporting

Report findings with:

- the exact `quire trace` invocation used
- `resolved` (from JSON) or the presence/absence of `Claims (0)`/`Citations
  (0)` in human output
- whether every hit was a `claims` record (real evidence) or included
  `citations` (mentions only) — say which, explicitly, rather than a bare hit
  count
- any `line_heuristic` records in the result, since those carry lower
  confidence than a `structural` one
