#!/usr/bin/env bash
set -euo pipefail

ROOT=$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)
python3 - "$ROOT" <<'PY'
from __future__ import annotations

import pathlib
import re
import sys

root = pathlib.Path(sys.argv[1])
errors: list[str] = []
toolchain = (root / "rust-toolchain.toml").read_text()
if not re.search(r'^channel\s*=\s*"\d+\.\d+\.\d+"', toolchain, re.MULTILINE):
    errors.append("rust-toolchain.toml must pin an exact x.y.z toolchain")

for workflow in sorted((root / ".github/workflows").glob("*.yml")):
    text = workflow.read_text()
    lines = text.splitlines()
    for number, line in enumerate(lines, 1):
        if line.lstrip().startswith("#") or re.match(r"\s*-\s+name:", line):
            continue
        use = re.search(r"\buses:\s*[^\s@]+@([^\s#]+)", line)
        if use and not re.fullmatch(r"[0-9a-f]{40}", use.group(1)):
            errors.append(f"{workflow.relative_to(root)}:{number}: action is not a full SHA")
        if re.search(r"runs-on:\s*[^#\n]*latest", line) or re.search(r"\bos:\s*[^#\n]*latest", line):
            errors.append(f"{workflow.relative_to(root)}:{number}: mutable runner label")
        if "uses: dtolnay/rust-toolchain@" in line:
            window = "\n".join(lines[number:number + 5])
            if "toolchain: 1.94.1" not in window:
                errors.append(f"{workflow.relative_to(root)}:{number}: compiler is not exact")
        if re.search(r"tool:\s*(cargo-deny|hyperfine)\s*$", line):
            errors.append(f"{workflow.relative_to(root)}:{number}: installed utility is not exact")
        if "npm install -g npm@latest" in line:
            errors.append(f"{workflow.relative_to(root)}:{number}: npm is mutable")
        if re.search(r"cargo (?:bench|build|check|clippy|test)\b", line) and "--locked" not in line:
            errors.append(f"{workflow.relative_to(root)}:{number}: Cargo resolution is not --locked")
        if re.search(r"cargo deny\b", line) and "--locked" not in line:
            errors.append(f"{workflow.relative_to(root)}:{number}: cargo-deny resolution is not --locked")

makefile = (root / "Makefile").read_text()
for number, line in enumerate(makefile.splitlines(), 1):
    if re.search(r"\$\(CARGO\)\s+(?:bench|build|check|clippy|test)\b", line) and "--locked" not in line:
        errors.append(f"Makefile:{number}: canonical Cargo command is not --locked")
    if re.search(r"\$\(CARGO\)\s+deny\b", line) and "--locked" not in line:
        errors.append(f"Makefile:{number}: cargo-deny resolution is not --locked")

if errors:
    print("tool-drift audit failed:", file=sys.stderr)
    for error in errors:
        print(f"- {error}", file=sys.stderr)
    raise SystemExit(1)
print("tool-drift audit: CLI compiler, actions, runners, utilities, npm, and Cargo are exact")
PY
