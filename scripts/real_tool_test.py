#!/usr/bin/env python3
# SPDX-License-Identifier: AGPL-3.0-or-later
# Copyright (C) 2026 Agent-IX

"""Run ignored real-converter export tests and retain their outcomes."""

from __future__ import annotations

import datetime as dt
import json
import os
import re
import shutil
import subprocess
import sys
import tempfile
from pathlib import Path


ROOT = Path(__file__).resolve().parents[1]
SOURCE = ROOT / "tests/cli_export.rs"
SUITE = "SUITE-004"
TEST_COMMAND = ["cargo", "test", "--locked", "--test", "cli_export", "--", "--ignored"]
TRACE_ID = re.compile(r'"([A-Z]+-[0-9]+-AC-[0-9]+)"')
TEST_RESULT = re.compile(r"^test\s+(\w+)\s+\.\.\.\s+(ok|FAILED)$")
VERSION_ARGS = {
    "pandoc": ["--version"],
    "typst": ["--version"],
    "mmdr": ["--version"],
    "pdfimages": ["-v"],
    "pdfinfo": ["-v"],
    "pdftotext": ["-v"],
    "unzip": ["-v"],
    "python3": ["--version"],
    "engineering-assurance": ["--version"],
    "git": ["--version"],
    "cargo": ["--version"],
    "quoin": ["--version"],
}
REQUIRED_TOOLS = tuple(VERSION_ARGS)


def version(tool: str, executable: str) -> tuple[str | None, str | None]:
    try:
        result = subprocess.run(
            [executable, *VERSION_ARGS[tool]], capture_output=True, text=True, check=False, timeout=10
        )
    except (OSError, subprocess.TimeoutExpired) as error:
        return None, str(error)
    output = (result.stdout or result.stderr).strip()
    if result.returncode != 0 or not output:
        return None, f"{tool} version command exited {result.returncode}: {output}"
    return output.splitlines()[0], None


def ignored_tests() -> list[dict[str, object]]:
    """Read ignored test names and their trace IDs from the test source."""
    if not SOURCE.is_file():
        raise ValueError(f"missing test source: {SOURCE}")
    lines = SOURCE.read_text(encoding="utf-8").splitlines()
    tests: list[dict[str, object]] = []
    trace_ids: list[str] | None = None
    ignored = False
    is_test = False
    index = 0
    while index < len(lines):
        line = lines[index].strip()
        if line.startswith("#[trace("):
            attribute = line
            while not attribute.endswith(")]") and index + 1 < len(lines):
                index += 1
                attribute += "\n" + lines[index].strip()
            trace_ids = sorted(set(TRACE_ID.findall(attribute)))
        elif line.startswith("#[ignore"):
            ignored = True
        elif line == "#[test]":
            is_test = True
        elif line.startswith("fn ") and "(" in line:
            name = line[3 : line.index("(")].strip()
            if ignored and is_test:
                if not trace_ids:
                    raise ValueError(f"ignored test {name} has no trace IDs")
                tests.append({"name": name, "symbol": f"cli_export::{name}", "trace_ids": trace_ids})
            trace_ids = None
            ignored = False
            is_test = False
        elif line and not line.startswith(("#[", "//", "/*", "*")):
            trace_ids = None
            ignored = False
            is_test = False
        index += 1
    if not tests:
        raise ValueError(f"no ignored tests found in {SOURCE}")
    return tests


def resolve_locked_build() -> str | None:
    configured = os.environ.get("LOCKED_BUILD", "locked-build.sh")
    candidate = Path(configured).expanduser()
    if candidate.is_file() and os.access(candidate, os.X_OK):
        return str(candidate.resolve())
    found = shutil.which(configured)
    if found:
        return found
    default = os.environ.get("LOCKED_BUILD_DEFAULT") == "1" or "LOCKED_BUILD" not in os.environ
    fallback = Path.home() / "dev/worktrees/locked-build.sh"
    if default and fallback.is_file() and os.access(fallback, os.X_OK):
        return str(fallback)
    return None


def timestamp() -> str:
    return dt.datetime.now(dt.timezone.utc).replace(microsecond=0).isoformat().replace("+00:00", "Z")


def source_commit() -> str:
    try:
        result = subprocess.run(
            ["git", "rev-parse", "HEAD"], cwd=ROOT, capture_output=True, text=True, check=False, timeout=10
        )
    except (OSError, subprocess.TimeoutExpired):
        return "unavailable"
    return result.stdout.strip() if result.returncode == 0 else "unavailable"


def entry(test: dict[str, object], outcome: str, config: dict[str, object]) -> dict[str, object]:
    return {
        "symbol": test["symbol"],
        "traceIds": test["trace_ids"],
        "outcome": outcome,
        "config": config,
    }


def main() -> int:
    try:
        tests = ignored_tests()
    except (OSError, ValueError) as error:
        print(f"real-tool-test configuration error: {error}", file=sys.stderr)
        return 2

    tool_versions: dict[str, str] = {}
    missing: list[str] = []
    for tool in REQUIRED_TOOLS:
        configured_executable = (
            os.environ.get("CARGO", "cargo")
            if tool == "cargo"
            else os.environ.get("QUOIN", "quoin")
            if tool == "quoin"
            else tool
        )
        executable = shutil.which(configured_executable)
        if executable is None:
            tool_versions[tool] = "unavailable"
            missing.append(f"{configured_executable} is unavailable on PATH")
            continue
        resolved, error = version(tool, executable)
        if error:
            tool_versions[tool] = "unavailable"
            missing.append(error)
        else:
            tool_versions[tool] = resolved or "unknown"

    locked_build = resolve_locked_build()
    if locked_build is None:
        missing.append("LOCKED_BUILD does not resolve to the locked-build.sh wrapper")

    base_config: dict[str, str] = {
        "tool_versions": json.dumps(tool_versions, sort_keys=True),
        "source_commit": source_commit(),
        "test_source": "tests/cli_export.rs",
        "fixtures": "tests/fixtures/export",
        "command": " ".join(TEST_COMMAND),
    }
    entries: list[dict[str, object]] = []
    failed = False
    if missing:
        reason = "; ".join(missing)
        for test in tests:
            config = {**base_config, "execution_state": "not-executed", "error": reason}
            entries.append(entry(test, "error", config))
        failed = True
    else:
        command = [locked_build, str(ROOT / "target"), os.environ.get("CARGO", "cargo"), *TEST_COMMAND[1:]]
        print("Focused command: " + " ".join(command), flush=True)
        try:
            result = subprocess.run(command, cwd=ROOT, capture_output=True, text=True, check=False)
        except OSError as error:
            result = None
            output = str(error)
        else:
            output = result.stdout + result.stderr
        print(output, end="" if output.endswith("\n") else "\n")
        observed: dict[str, str] = {}
        for line in output.splitlines():
            match = TEST_RESULT.match(line.strip())
            if match:
                observed[match.group(1)] = match.group(2)
        if result is None:
            failed = True
        for test in tests:
            state = observed.get(str(test["name"]))
            config = {**base_config, "execution_state": "executed" if state else "not-executed"}
            if result is None:
                outcome = "error"
                config["error"] = output
            elif state == "ok":
                outcome = "pass"
            elif state == "FAILED":
                outcome = "error"
                config["error"] = "ignored real-tool test failed; see focused command output"
                failed = True
            else:
                outcome = "error"
                config["error"] = f"test produced no result line; cargo exit={result.returncode}"
                failed = True
            entries.append(entry(test, outcome, config))
        if result is not None and result.returncode != 0:
            failed = True

    results = tempfile.NamedTemporaryFile(prefix="quire-cli-real-tool-", suffix=".json", delete=False, mode="w", encoding="utf-8")
    with results:
        json.dump({"entries": entries}, results, sort_keys=True)
        results.write("\n")

    quoin = shutil.which(os.environ.get("QUOIN", "quoin"))
    if quoin is None:
        print("cannot record SUITE-004: quoin is unavailable on PATH", file=sys.stderr)
        return 2
    recorded = subprocess.run(
        [
            quoin,
            "evidence",
            "record",
            "--suite",
            SUITE,
            "--tool",
            f"quire-cli test-export-real ({tool_versions['cargo']}; quoin {tool_versions['quoin']})",
            "--adapter",
            "entries",
            "--kind",
            "Integration",
            "--repo",
            str(ROOT),
            "--results",
            results.name,
            "--timestamp",
            timestamp(),
        ],
        cwd=ROOT,
        check=False,
    )
    if recorded.returncode != 0:
        return recorded.returncode
    if failed:
        print("real-tool-test recorded an error or not-executed entry; no passing live-tool result is claimed", file=sys.stderr)
        return 1
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
