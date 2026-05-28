# quire-cli

Static binary CLI wrapping [`quire-rs`](https://github.com/agent-ix/quire-rs):
`render`, `parse`, `extract`, `validate` — one cold-start process per call,
NFR-001 p95 ≤ 50 ms.

This crate is a **thin process boundary**. Every subcommand surfaces an
upstream `quire-rs` API; no markdown parsing, template rendering, or JSON-
schema validation lives here.

## Install

```bash
cargo install --git https://github.com/agent-ix/quire-cli
```

The release binary is statically linked against only baseline system libs
(libc, libm, ld-linux); see `tests/audit_ldd.rs` (NFR-002).

## Usage

All subcommands share two global flags:

| Flag | Default | Purpose |
|------|---------|---------|
| `--diagnostics-format <human\|json>` | `human` | stderr diagnostic encoding |
| `--pretty` | off | indented JSON output (parse, extract) |

### `quire render <archetype> --module <PATH> --data <FILE\|->`

Render a registered archetype to markdown.

```bash
quire render FR \
    --module ~/.ix/schemas/spec-artifacts-iso \
    --data ./FR-001.json
# or read context from stdin:
cat FR-001.json | quire render FR --module ./iso --data -
# or write to a file (empty stdout):
quire render FR --module ./iso --data ctx.json --out FR-001.md
```

### `quire parse <DOC\|->`

Parse a markdown document and emit a `QuireDocument` JSON envelope.

```bash
quire parse spec/functional/FR-001.md
# or via stdin:
cat FR-001.md | quire parse - | jq '.frontmatter.id'
# pretty:
quire --pretty parse FR-001.md
```

### `quire extract <DOC\|-> --module <PATH> [--archetype <NAME>]`

Run the `body_extraction` DSL declared on the document's `object_type`,
plus edge harvest (frontmatter relationships + body `ix://` links).
Emits `{extraction, edges}`.

```bash
quire extract spec/objects/EX-001.md --module ./extract-mod
quire extract - --module ./mod --archetype ExtractSample < EX-001.md
```

Per the spec, `extract` does NOT auto-validate; run `validate` separately
if you want schema enforcement.

### `quire validate <archetype> --module <PATH> --data <FILE\|->`

Validate a JSON context against an archetype's frontmatter schema. Exit
0 on success, exit 1 on schema violation. Writes nothing to stdout.

```bash
quire validate FR --module ./iso --data ./FR-001.json
echo "exit=$?"
```

## Exit codes (FR-007)

| Code | Meaning |
|------|---------|
| 0 | success |
| 1 | user error (sandbox refusal, parse failure, schema violation, unknown archetype, I/O error) |
| 2 | argv error (missing required flag, unknown flag) |
| 134 | panic (never expected — file an issue) |

## Sandbox guarantees (FR-005, StR-003)

- `--module`, `--data`, `--out`: `..` segments are rejected as
  `Diagnostic::PathTraversal` with reason `DotDotSegment`.
- Symlinks that escape the canonicalized path are rejected with reason
  `SymlinkEscape`.
- `--data -` (stdin) bypasses path-safety by design.

## Development

```bash
make build              # release build
make test               # cargo test (unit + ITs + audits)
make lint               # clippy -D warnings
make fmt-check          # rustfmt --check
make deny               # cargo deny check licenses
make deny-bans          # cargo deny check bans (denies HTTP client crates)
make audit-unsafe       # every unsafe block carries a // SAFETY: comment
make audit-thin-boundary # AUDIT-002: src/ has no parse/render/validate logic
make bench              # hyperfine p95 ≤ 50 ms (NFR-001 gate)
make refresh-fixtures   # re-sync tests/fixtures/iso from ../quire-rs
make ci                 # full local CI
```

## Spec & plan

Requirements live in `spec/`; the implementation plan + task index live
in `plan/`. The traceability matrix (`spec/tests.md`) maps every AC to
at least one IT / BENCH / AUDIT.

Upstream engine: [`quire-rs`](https://github.com/agent-ix/quire-rs).

## License

MIT
