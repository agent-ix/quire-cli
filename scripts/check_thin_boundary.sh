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
  'quire_rs::render_by_name'
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

if [ "$fail" -ne 0 ]; then
  echo "thin-boundary audit failed; see violations above" >&2
  exit 1
fi
echo "thin-boundary audit ok"
