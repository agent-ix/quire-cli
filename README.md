# quire-cli

`quire-cli` is a static command-line wrapper around
[`quire-rs`](https://github.com/agent-ix/quire-rs). It gives agents and
humans one fast binary for rendering, parsing, extracting, looking up, and
validating Markdown artifacts.

The crate is intentionally a thin process boundary: Markdown parsing,
template rendering, extraction, and schema validation live in `quire-rs`.

## Commands

```bash
quire render <ARCHETYPE> --module <PATH> --data <FILE|->
quire parse <DOC|->
quire lookup <DOC|-> (--heading <TEXT> [--level <1..6>] | --id <ID> | --block-id <BLOCK_ID>) [--content]
quire extract <DOC|-> --module <PATH> [--archetype <NAME>]
quire validate <ARCHETYPE> --module <PATH> --data <FILE|->
```

Global flags:

| Flag | Default | Purpose |
|------|---------|---------|
| `--diagnostics-format <human\|json>` | `human` | stderr diagnostic encoding |
| `--pretty` | off | indented JSON output for `parse`, `lookup`, and `extract` |

Exit codes:

| Code | Meaning |
|------|---------|
| 0 | success |
| 1 | user error: parse failure, schema violation, unknown archetype, I/O error, lookup miss |
| 2 | argv error: missing required flag, unknown flag, invalid flag combination |
| 134 | panic, never expected |

## Install

From Git:

```bash
cargo install --git https://github.com/agent-ix/quire-cli
```

From a checkout:

```bash
cargo build --release
target/release/quire --help
```

During development, `target/debug/quire` is fine for local testing.

## Usage Instructions

### Render Markdown

Render a registered archetype from context JSON:

```bash
quire render FR --module ./tests/fixtures/iso --data ./tests/fixtures/contexts/FR.json
```

Read context JSON from stdin:

```bash
cat FR.json | quire render FR --module ./iso --data -
```

Write to a file:

```bash
quire render FR --module ./iso --data FR.json --out spec/functional/FR-123.md
```

Use `validate` on context JSON before rendering when you want a schema-only
check without producing Markdown.

### Parse And Outline Markdown

Parse a document into a `QuireDocument` JSON envelope:

```bash
quire --pretty parse spec/functional/FR-001.md
```

Print a compact outline:

```bash
quire parse spec/functional/FR-001.md \
  | jq -r '.. | objects | select(has("heading") and has("level")) | "\(.level) \(.heading) id=\(.id) block_id=\(.block_id // "-") lines=\(.start_line)-\(.end_line)"'
```

Inspect frontmatter:

```bash
quire parse spec/functional/FR-001.md | jq '.frontmatter'
```

### Lookup One Section

Fetch one parsed section as JSON:

```bash
quire lookup spec/functional/FR-001.md --heading Behavior
quire lookup spec/functional/FR-001.md --heading "Document Title" --level 1
quire lookup spec/functional/FR-001.md --id behavior-L14
quire lookup spec/functional/FR-001.md --block-id blk-behavior
```

Fetch only section body bytes:

```bash
quire lookup spec/functional/FR-001.md --heading Acceptance --content
quire lookup spec/functional/FR-001.md --block-id blk-behavior --content
```

`--id` is generated as `<slug>-L<line>` and can change when lines move.
For stable machine addressing, author headings with Pandoc block IDs:

```markdown
## Behavior {#blk-behavior}
```

Then use:

```bash
quire lookup doc.md --block-id blk-behavior
```

### Extract Records And Links

Run an archetype's `body_extraction` DSL and collect edges:

```bash
quire extract spec/objects/EX-001.md --module ./extract-mod
```

Override archetype inference when needed:

```bash
quire extract EX-001.md --module ./extract-mod --archetype ExtractSample
```

Show only harvested edges:

```bash
quire extract EX-001.md --module ./extract-mod | jq '.edges'
```

`extract` does not auto-validate. Run `validate` separately for context
JSON schema checks.

### Validate Context JSON

Validate input data against an archetype schema:

```bash
quire validate FR --module ./iso --data ./FR-001.json
```

Read JSON from stdin:

```bash
cat FR-001.json | quire validate FR --module ./iso --data -
```

`validate` writes nothing to stdout. A nonzero exit means stderr carries the
diagnostic.

## Agent Skills

This repository ships Codex-style agent skills under `skills/`. They are
intended to be packaged with or installed from `quire-cli` so agents can use
the CLI consistently without relearning command patterns.

Available skills:

| Skill | Slash-style name | Purpose |
|-------|------------------|---------|
| `explore-markdown` | `/explore-markdown` | Outline Markdown and fetch targeted sections with `parse` and `lookup`. |
| `write-markdown` | `/write-markdown` | Generate Markdown artifacts from structured data with `render`. |
| `validate-markdown` | `/validate-markdown` | Check context JSON and rendered Markdown structure with `validate`, `parse`, and `lookup`. |
| `link-markdown` | `/link-markdown` | Inspect relationships, `ix://` links, and blast radius with `extract`. |

Each skill has:

```text
skills/<name>/SKILL.md
skills/<name>/agents/openai.yaml
```

When adding a new shipped skill, keep it general, command-oriented, and
domain-neutral. Domain review guidance belongs in domain-specific skills,
not here.

Validate skills with:

```bash
python3 path/to/quick_validate.py skills/explore-markdown
python3 path/to/quick_validate.py skills/write-markdown
python3 path/to/quick_validate.py skills/validate-markdown
python3 path/to/quick_validate.py skills/link-markdown
```

## Safety

Path safety is part of the CLI contract:

- `--module`, `--data`, `--out`, and positional document paths reject `..`
  traversal where the command applies path safety.
- Symlink escapes are rejected.
- `--data -` and document `-` read from stdin by design.
- The CLI makes no network calls in normal operation; tests audit this with
  `strace`.

## Development

```bash
make build               # release build
make test                # cargo test, including integration tests and audits
make lint                # clippy -D warnings
make fmt-check           # rustfmt --check
make deny                # cargo deny check licenses
make deny-bans           # cargo deny check bans
make audit-unsafe        # every unsafe block carries a // SAFETY: comment
make audit-thin-boundary # src/ stays a thin wrapper over quire-rs
make bench               # hyperfine p95 <= 50 ms gate
make refresh-fixtures    # re-sync tests/fixtures/iso from ../quire-rs
make ci                  # local CI gauntlet
```

The release binary is audited to link only baseline system libraries on
Linux. See `tests/audit_ldd.rs`.

## Spec And Plan

Requirements live in `spec/`; the implementation plan and task index live in
`plan/`. The traceability matrix in `spec/tests.md` maps acceptance criteria
to integration tests, benchmarks, or static audits.

## License

MIT
