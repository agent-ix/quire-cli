#!/bin/sh
# trace: FR-047-AC-4, FR-047-AC-10
set -eu

record="$CASE_TMP/selected-run.json"
TMPDIR="$CASE_TMP" "$QUIRE" selftest --run --case parse-empty-input --out "$record" > "$CASE_TMP/stdout"
[ ! -s "$CASE_TMP/stdout" ]
grep -q '"tool":"quire"' "$record"
grep -q '"id":"parse-empty-input"' "$record"
grep -q '"status":"pass"' "$record"
