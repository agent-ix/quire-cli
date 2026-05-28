#!/usr/bin/env python3
"""Assert hyperfine reports p95 wall-time ≤ threshold_ms.

Usage: check_bench_p95.py target/bench.json <threshold_ms>

hyperfine doesn't emit p95 directly; it emits per-run `times` (seconds).
We compute the 95th percentile ourselves to keep this dep-free.
"""

from __future__ import annotations

import json
import math
import sys
from pathlib import Path


def p95(samples: list[float]) -> float:
    if not samples:
        raise ValueError("no samples in bench JSON")
    s = sorted(samples)
    # Nearest-rank: smallest sample at or above 95%.
    k = max(0, math.ceil(0.95 * len(s)) - 1)
    return s[k]


def main(argv: list[str]) -> int:
    if len(argv) != 3:
        print(f"usage: {argv[0]} <bench.json> <threshold_ms>", file=sys.stderr)
        return 2
    bench_path = Path(argv[1])
    threshold_ms = float(argv[2])

    data = json.loads(bench_path.read_text())
    results = data.get("results") or []
    if not results:
        print("no results array in bench JSON", file=sys.stderr)
        return 2
    times_s = results[0].get("times") or []
    val_ms = p95(times_s) * 1000.0
    print(f"BENCH-001: p95={val_ms:.2f} ms (threshold {threshold_ms:.0f} ms)")
    if val_ms > threshold_ms:
        print(
            f"BENCH-001 failed: p95 {val_ms:.2f} ms > {threshold_ms:.0f} ms threshold",
            file=sys.stderr,
        )
        return 1
    return 0


if __name__ == "__main__":
    sys.exit(main(sys.argv))
