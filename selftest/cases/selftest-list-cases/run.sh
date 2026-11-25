#!/bin/sh
# trace: FR-047-AC-1
set -eu

cases=$("$QUIRE" selftest --list)
case "$cases" in
    *"parse-empty-input"*) ;;
    *) printf '%s\n' 'missing quire-cli case' >&2; exit 1 ;;
esac
case "$cases" in
    *"fr-032-placeholder-detection"*) ;;
    *) printf '%s\n' 'missing linked quire-rs case' >&2; exit 1 ;;
esac
