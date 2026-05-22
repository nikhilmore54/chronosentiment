#!/usr/bin/env python3
"""One-shot MFE integrity report for paper logs.

ChronoSentiment validation helper (deterministic, read-only):
- Check 1: PnL should not exceed MFE.
- Check 2: MFE vs |MAE| distribution summary.
- Check 3: Entry-bar MFE contribution proxy from first EXIT_TRACE line.
"""

from __future__ import annotations

import argparse
import re
from pathlib import Path
from statistics import mean, median


TRADE_RE = re.compile(
    r"\[TRADE_PATH\] rec_id=(\d+) sym=([^ ]+) mfe=([-\d.]+) mae=([-\d.]+) pnl=([-\d.]+) .*dur=(\d+) .*exit_type=([^\s]+)"
)
EXIT_RE = re.compile(r"\[EXIT_TRACE\] rec_id=(\d+) sym=([^ ]+) .*mfe=([-\d.]+) .*")


def pct(sorted_vals: list[float], p: float) -> float:
    if not sorted_vals:
        return 0.0
    idx = int(p * (len(sorted_vals) - 1))
    return sorted_vals[idx]


def main() -> int:
    parser = argparse.ArgumentParser(description="MFE integrity checks from a paper log")
    parser.add_argument("log_path", help="Path to log containing [TRADE_PATH] and optional [EXIT_TRACE]")
    args = parser.parse_args()

    log_path = Path(args.log_path)
    if not log_path.exists():
        print(f"ERROR: log file not found: {log_path}")
        return 2

    lines = log_path.read_text(errors="replace").splitlines()
    trades: dict[int, dict[str, float | int | str]] = {}
    entry_mfe: dict[int, float] = {}

    for line in lines:
        tm = TRADE_RE.search(line)
        if tm:
            rid, sym, mfe, mae, pnl, dur, ex = tm.groups()
            trades[int(rid)] = {
                "sym": sym,
                "mfe": float(mfe),
                "mae": float(mae),
                "pnl": float(pnl),
                "dur": int(dur),
                "exit": ex,
            }
            continue
        em = EXIT_RE.search(line)
        if em:
            rid, _sym, mfe = em.groups()
            rid_i = int(rid)
            if rid_i not in entry_mfe:
                entry_mfe[rid_i] = float(mfe)

    n = len(trades)
    print(f"trades={n}")
    if n == 0:
        print("No [TRADE_PATH] rows found.")
        return 1

    # Check 1
    violations = [
        (rid, t)
        for rid, t in trades.items()
        if float(t["pnl"]) > float(t["mfe"]) + 1e-9
    ]
    print(f"check1_pnl_le_mfe_violations={len(violations)}")

    # Check 2
    mfes = [float(t["mfe"]) for t in trades.values()]
    abs_maes = [abs(float(t["mae"])) for t in trades.values()]
    ratios = [m / a for m, a in zip(mfes, abs_maes) if a > 1e-12]
    ratios_sorted = sorted(ratios)
    print(
        "check2 "
        f"avg_mfe={mean(mfes):.6f} avg_abs_mae={mean(abs_maes):.6f} "
        f"median_mfe={median(mfes):.6f} median_abs_mae={median(abs_maes):.6f} "
        f"mfe_over_absmae_median={median(ratios):.3f} "
        f"mfe_over_absmae_p75={pct(ratios_sorted, 0.75):.3f}"
    )

    # Check 3
    contrib = []
    for rid, t in trades.items():
        total_mfe = float(t["mfe"])
        em = entry_mfe.get(rid)
        if em is None or total_mfe <= 1e-12:
            continue
        contrib.append(em / total_mfe)
    if contrib:
        c_sorted = sorted(contrib)
        gt70 = sum(1 for c in contrib if c > 0.7) / len(contrib)
        gt90 = sum(1 for c in contrib if c > 0.9) / len(contrib)
        print(
            "check3 "
            f"samples={len(contrib)} mean={mean(contrib):.3f} median={median(contrib):.3f} "
            f"p75={pct(c_sorted, 0.75):.3f} gt70={gt70:.3f} gt90={gt90:.3f}"
        )
    else:
        print("check3 samples=0 (no [EXIT_TRACE] rows or no positive MFE trades)")

    dur1_share = sum(1 for t in trades.values() if int(t["dur"]) == 1) / n
    mfe_pos_share = sum(1 for t in trades.values() if float(t["mfe"]) > 1e-12) / n
    print(f"extra dur1_share={dur1_share:.3f} mfe_pos_share={mfe_pos_share:.3f}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
