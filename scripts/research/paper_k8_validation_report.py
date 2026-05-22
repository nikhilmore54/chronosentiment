#!/usr/bin/env python3
"""
Parse paper engine logs for exit forensics (avg pnl, retention-at-exit, exit mix).

Reads stdin or file path(s). Expects lines:
  [EXIT] type=... pnl=...
  [TRADE_PATH] ... ret_at_exit=... exit_type=...

Older logs without ret_at_exit: falls back to pnl/mfe from the same TRADE_PATH line.

Usage:
  python3 scripts/paper_k8_validation_report.py run_K8_4.log
  cargo run ... 2>&1 | python3 scripts/paper_k8_validation_report.py
"""

from __future__ import annotations

import re
import sys
from collections import Counter

EXIT_RE = re.compile(
    r"\[EXIT\]\s+type=(?P<type>[^\s]+)\s+sym=\S+\s+strategy=\S+\s+pnl=(?P<pnl>[-0-9.eE+]+)"
)
# ret_at_exit optional for backward compatibility
TPATH_RE = re.compile(
    r"\[TRADE_PATH\]\s+.*?\bmfe=(?P<mfe>[-0-9.eE+]+)\s+mae=(?P<mae>[-0-9.eE+]+)\s+pnl=(?P<pnl>[-0-9.eE+]+)"
    r"(?:\s+ret_at_exit=(?P<ret>[-0-9.eE+]+))?"
    r".*?\bexit_type=(?P<exit_type>\S+)"
)


def retention_from_match(m: re.Match[str]) -> float | None:
    if m.group("ret") is not None:
        return float(m.group("ret"))
    pnl = float(m.group("pnl"))
    mfe = float(m.group("mfe"))
    if abs(mfe) <= 1e-12:
        return None
    return pnl / mfe


def summarize(text: str, label: str) -> None:
    exits = list(EXIT_RE.finditer(text))
    tpaths = list(TPATH_RE.finditer(text))

    by_type = Counter(m.group("type") for m in exits)
    trail_prefixes = ("TRAIL_BREAK_", "TRAIL_DD")
    time_n = by_type.get("TIME", 0) + by_type.get("FINALIZE_TIME", 0)
    trail_n = sum(c for k, c in by_type.items() if k.startswith(trail_prefixes))

    rets: list[float] = []
    for m in tpaths:
        r = retention_from_match(m)
        if r is not None:
            rets.append(r)

    over_half = sum(1 for r in rets if r > 0.5)
    ret_pct = 100.0 * over_half / len(rets) if rets else 0.0

    dd_weak = sum(
        1 for m in tpaths if m.group("exit_type") == "TRAIL_BREAK_STRONG_DD_WEAK_PATH"
    )

    pnls = [float(m.group("pnl")) for m in exits]
    avg_pnl = sum(pnls) / len(pnls) if pnls else 0.0

    print(f"=== {label} ===")
    print(f"closed_trades: {len(exits)}")
    print(f"avg_pnl: {avg_pnl:.6f}")
    print(f"pct_exits_retention_gt_0_5: {ret_pct:.2f}%  ({over_half}/{len(rets)} with parsed retention)")
    print(f"TRAIL_BREAK_STRONG_DD_WEAK_PATH: {dd_weak}")
    print(f"exit_mix_TIME_or_FINALIZE: {time_n}")
    print(f"exit_mix_TRAIL_star: {trail_n}")
    print("exit_mix_by_type:")
    for k in sorted(by_type.keys()):
        print(f"  {k}: {by_type[k]}")
    print()


def main() -> None:
    paths = [a for a in sys.argv[1:] if not a.startswith("-")]
    if paths:
        for p in paths:
            with open(p, encoding="utf-8", errors="replace") as f:
                summarize(f.read(), p)
    else:
        summarize(sys.stdin.read(), "stdin")


if __name__ == "__main__":
    main()
