#!/usr/bin/env bash
# AUDIT-002: src/ is a thin process boundary over quire-rs.
#
# Fail if any file under src/ references a parse/render/validate primitive
# directly except at the documented dispatch sites in src/commands/*.rs and
# src/main.rs. Every such call MUST go through quire_rs::*; the CLI does
# not implement markdown parsing, template rendering, or JSON-schema
# validation.

set -euo pipefail

REPO_ROOT="$(cd "$(dirname "$0")/.." && pwd)"
SRC="$REPO_ROOT/src"

# Permitted dispatch sites — these files own the call into quire-rs and
# are listed here explicitly so adding a new dispatch site is a deliberate
# act (advances AUDIT-002 traceability).
ALLOWED_REGEX='^src/(main\.rs|commands/.*\.rs)$'

# Patterns we ban anywhere outside the allowed dispatch sites.
PATTERNS=(
  'quire_rs::parse_document'
  'quire_rs::render\b'
  'quire_rs::render_block'
  'quire_rs::render_with_env'
  'quire_rs::validate\b'
  'quire_rs::validate_all'
  'quire_rs::validate_block'
  'quire_rs::extract\b'
  'quire_rs::harvest_edges'
  'jsonschema::'
  'minijinja::'
)

cd "$REPO_ROOT"
fail=0
for pat in "${PATTERNS[@]}"; do
  # Grep for the pattern in every .rs file under src/ (relative paths).
  while IFS= read -r hit; do
    file="${hit%%:*}"
    rel="${file#./}"
    if [[ "$rel" =~ $ALLOWED_REGEX ]]; then
      continue
    fi
    echo "AUDIT-002 violation: $hit" >&2
    fail=1
  done < <(grep -RHn -E "$pat" src/ --include='*.rs' || true)
done

# FR-020 / TC-814: the assurance command may compose the authoritative engine
# surfaces, but it may not grow a CLI-owned graph, schema, parser, or runner.
ASSURANCE="src/commands/assurance.rs"
for required in \
  'Spec::from_path' \
  'extract_tree_scoped' \
  'symbols::trace::bind' \
  'build_assurance_export' \
  'read_assurance_export' \
  'to_json_bytes'; do
  if ! grep -Eq "$required" "$ASSURANCE"; then
    echo "TC-814 violation: assurance command does not delegate through $required" >&2
    fail=1
  fi
done

for forbidden in \
  'struct[[:space:]]+AssuranceExport' \
  'enum[[:space:]]+AssuranceRelation' \
  'ASSURANCE_V1_SCHEMA' \
  'jsonschema::' \
  'parse_document' \
  'std::process::Command' \
  'Command::new'; do
  if grep -Eq "$forbidden" "$ASSURANCE"; then
    echo "TC-814 violation: assurance command contains forbidden boundary logic: $forbidden" >&2
    fail=1
  fi
done

# StR-004-VC-3: make the upstream-ownership review question an executable
# repository invariant rather than an unverified process claim.
REVIEW_CHECKLIST='Does any new logic belong upstream in `quire-rs`?'
if ! grep -Fqx -- "- $REVIEW_CHECKLIST" CONTRIBUTING.md; then
  echo "TC-814 violation: CONTRIBUTING.md lacks the thin-boundary review question" >&2
  fail=1
fi

if [ "$fail" -ne 0 ]; then
  echo "thin-boundary audit failed; see violations above" >&2
  exit 1
fi
echo "thin-boundary audit ok"
