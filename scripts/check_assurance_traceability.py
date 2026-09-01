#!/usr/bin/env python3
"""Fail closed if issue #74's declared assurance evidence loses its bindings."""

import json
import sys
from pathlib import Path


def fail(message: str) -> None:
    print(f"assurance traceability gate: {message}", file=sys.stderr)
    raise SystemExit(1)


if len(sys.argv) != 2:
    fail("expected one coverage JSON path")

try:
    report = json.loads(Path(sys.argv[1]).read_text(encoding="utf-8"))
except (OSError, json.JSONDecodeError) as error:
    fail(f"cannot read coverage JSON: {error}")
if not isinstance(report, dict):
    fail("coverage JSON root is not an object")


def list_field(name: str) -> list:
    value = report.get(name)
    if not isinstance(value, list):
        fail(f"coverage JSON field {name!r} is absent or not an array")
    return value


required = {
    *(f"FR-020-AC-{index}" for index in range(1, 10)),
    *(f"IT-{index}" for index in range(136, 146)),
    "TC-814",
    "StR-004-VC-2",
    "StR-004-VC-3",
}
minted = {entry.get("id"): entry for entry in list_field("minted_targets")}

missing = sorted(required - minted.keys())
if missing:
    fail(f"required targets were not minted: {missing}")

unbacked = sorted(target for target in required if not minted[target].get("backed"))
if unbacked:
    fail(f"required targets are not backed: {unbacked}")

invalid_user_story_targets = sorted(
    target for target in minted if isinstance(target, str) and target.startswith("US-006-AC-")
)
if invalid_user_story_targets:
    fail(
        "user-story examples were incorrectly promoted to binding acceptance: "
        f"{invalid_user_story_targets}"
    )

unmatched_issue_tags = [
    entry
    for entry in list_field("unmatched_tags")
    if any(prefix in json.dumps(entry) for prefix in ("FR-020-AC-", "US-006-AC-"))
]
if unmatched_issue_tags:
    fail(f"issue #74 has unmatched trace tags: {unmatched_issue_tags}")

for key in ("status_lies", "untracked_symbols"):
    values = list_field(key)
    if values:
        fail(f"coverage reports non-empty {key}: {values}")

print(f"assurance traceability ok: {len(required)}/{len(required)} required targets backed")
