#!/usr/bin/env bash
# Single-source the release version across the Rust crate and the npm packages.
#
#   scripts/set_version.sh <X.Y.Z>
#
# Updates:
#   - Cargo.toml            [package] version
#   - npm/quire-cli/package.json   version + the @agent-ix/quire-cli-* optionalDependencies
#
# The per-platform packages are generated at release time by npm/build-packages.mjs
# with the same version, so they do not need editing here. After running this,
# review CHANGELOG.md (rename the [Unreleased] section), commit, and tag vX.Y.Z.
set -euo pipefail

VERSION="${1:?usage: set_version.sh <X.Y.Z>}"
if [[ ! "$VERSION" =~ ^[0-9]+\.[0-9]+\.[0-9]+([-.+][0-9A-Za-z.-]+)?$ ]]; then
  echo "error: '$VERSION' is not a semver version" >&2
  exit 1
fi

ROOT="$(cd "$(dirname "$0")/.." && pwd)"

# Cargo.toml: only the line-anchored [package] `version = "..."` (dependency
# versions live inside `{ ... }` tables and are not at column 0).
perl -i -pe 'if (!$done && /^version = "/) { s/^version = ".*"/version = "'"$VERSION"'"/; $done = 1 }' \
  "$ROOT/Cargo.toml"

# npm launcher: bump version and every @agent-ix/quire-cli-* optional dep in lockstep.
node -e '
  const fs = require("fs");
  const [file, v] = process.argv.slice(1);
  const j = JSON.parse(fs.readFileSync(file, "utf8"));
  j.version = v;
  for (const k of Object.keys(j.optionalDependencies || {})) {
    if (k.startsWith("@agent-ix/quire-cli-")) j.optionalDependencies[k] = v;
  }
  fs.writeFileSync(file, JSON.stringify(j, null, 2) + "\n");
' "$ROOT/npm/quire-cli/package.json" "$VERSION"

echo "set version to $VERSION in Cargo.toml and npm/quire-cli/package.json"
echo "next: update CHANGELOG.md, then  git commit && git tag v$VERSION && git push --tags"
