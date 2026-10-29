#!/usr/bin/env python3
"""Build empirical EV table from [TRADE_PATH] logs for paper intent expiry.

Deterministic, integer bucket keys (no float formatting drift). Aligns with
`core/src/paper.rs` (`paper_bucket_int`, `paper_intent_lookup_keys`) and
`PAPER_EV_TABLE_PATH` / `PAPER_EV_EXPIRE_THRESHOLD`.

Training label supports two deterministic modes:
  - survival: ev = p_survive * avg_pnl
  - payoff:   ev = p_win * avg_win - p_loss * avg_loss
"""

from __future__ import annotations

import argparse
import json
import re
from collections import defaultdict
from pathlib import Path

# Centi-confidence (conf * 100, rounded int). Must match paper.rs conf_cent bins.
CONF_BUCKETS = [50, 60, 70, 80, 100]
AGE_BUCKETS = [0, 2, 5, 10, 20]

TRADE_RE = re.compile(
    r"\[TRADE_PATH\]"
    r".*?pnl=(?P<pnl>[-\d.]+)"
    r".*?rec_conf=(?P<rec_conf>[-\d.]+)\s+"
    r"rec_voters=(?P<rec_voters>\d+)"
    r"(?:\s+intent_age=(?P<intent_age>\d+))?"
    r".*?dur=(?P<dur>\d+)"
)


def bucket_int(val: int, bins: list[int]) -> str:
    """Integer bin label; must match `paper_bucket_int` in core/src/paper.rs."""
    if len(bins) < 2:
        return "0_plus"
    v = val
    if v < bins[0]:
        v = bins[0]
    for i in range(len(bins) - 1):
        if bins[i] <= v < bins[i + 1]:
            return f"{bins[i]}_{bins[i + 1]}"
    return f"{bins[-1]}_plus"


def conf_cent(conf: float) -> int:
    """Match Rust: (conf * 100.0).round() as i32, clamped to [0, 100]."""
    x = int(round(conf * 100.0))
    return max(0, min(100, x))


def linear_percentile(sorted_vals: list[float], p_pct: float) -> float:
    """Return value at p_pct in [0, 100] using linear interpolation (sorted_vals ascending)."""
    if not sorted_vals:
        return float("nan")
    xs = sorted_vals
    if len(xs) == 1:
        return xs[0]
    rank = (p_pct / 100.0) * (len(xs) - 1)
    lo = int(rank)
    hi = min(lo + 1, len(xs) - 1)
    w = rank - lo
    return xs[lo] * (1.0 - w) + xs[hi] * w


def main() -> None:
    ap = argparse.ArgumentParser(description=__doc__)
    ap.add_argument(
        "input_logs",
        nargs="*",
        type=Path,
        default=[Path("analysis/adaptive_intent_decay.log")],
        help="One or more log files (concatenated in order)",
    )
    ap.add_argument(
        "-o",
        "--output",
        type=Path,
        default=Path("analysis/ev_table.json"),
        help="Output JSON path",
    )
    ap.add_argument(
        "--min-count",
        type=int,
        default=5,
        help="Drop buckets with fewer trades (noise floor)",
    )
    ap.add_argument(
        "--min-survive-dur",
        type=int,
        default=5,
        help="Survival: dur >= this and pnl > 0",
    )
    ap.add_argument(
        "--ev-mode",
        choices=["survival", "payoff"],
        default="payoff",
        help="EV formula: survival (legacy) or payoff (payoff-aware)",
    )
    args = ap.parse_args()

    stats: dict[tuple[str, str], dict[str, float | int]] = defaultdict(
        lambda: {
            "count": 0,
            "sum_pnl": 0.0,
            "survive_count": 0,
            "win_count": 0,
            "loss_count": 0,
            "sum_win_pnl": 0.0,
            "sum_loss_abs_pnl": 0.0,
        }
    )

    text = "\n".join(p.read_text(errors="replace") for p in args.input_logs)
    for line in text.splitlines():
        m = TRADE_RE.search(line)
        if not m:
            continue
        pnl = float(m.group("pnl"))
        conf = float(m.group("rec_conf"))
        _voters = int(m.group("rec_voters"))
        dur = int(m.group("dur"))
        age_s = m.group("intent_age")
        age = int(age_s) if age_s is not None else 0

        key = (
            bucket_int(age, AGE_BUCKETS),
            bucket_int(conf_cent(conf), CONF_BUCKETS),
        )
        cell = stats[key]
        cell["count"] = int(cell["count"]) + 1  # type: ignore[assignment]
        cell["sum_pnl"] = float(cell["sum_pnl"]) + pnl  # type: ignore[assignment]
        if dur >= args.min_survive_dur and pnl > 0:
            cell["survive_count"] = int(cell["survive_count"]) + 1  # type: ignore[assignment]
        if pnl > 0:
            cell["win_count"] = int(cell["win_count"]) + 1  # type: ignore[assignment]
            cell["sum_win_pnl"] = float(cell["sum_win_pnl"]) + pnl  # type: ignore[assignment]
        elif pnl < 0:
            cell["loss_count"] = int(cell["loss_count"]) + 1  # type: ignore[assignment]
            cell["sum_loss_abs_pnl"] = float(cell["sum_loss_abs_pnl"]) + abs(pnl)  # type: ignore[assignment]

    ev_table: dict[str, dict[str, dict[str, float | int]]] = {}
    for key, s in stats.items():
        count = int(s["count"])
        if count < args.min_count:
            continue
        sum_pnl = float(s["sum_pnl"])
        survive = int(s["survive_count"])
        win_count = int(s["win_count"])
        loss_count = int(s["loss_count"])
        sum_win_pnl = float(s["sum_win_pnl"])
        sum_loss_abs_pnl = float(s["sum_loss_abs_pnl"])
        avg_pnl = sum_pnl / count
        p_survive = survive / count
        p_win = win_count / count
        p_loss = loss_count / count
        avg_win = (sum_win_pnl / win_count) if win_count > 0 else 0.0
        avg_loss = (sum_loss_abs_pnl / loss_count) if loss_count > 0 else 0.0
        if args.ev_mode == "survival":
            ev = p_survive * avg_pnl
        else:
            ev = p_win * avg_win - p_loss * avg_loss
        age_b, conf_b = key
        ev_table.setdefault(age_b, {})[conf_b] = {
            "count": count,
            "p_survive": round(p_survive, 4),
            "p_win": round(p_win, 4),
            "p_loss": round(p_loss, 4),
            "avg_win": round(avg_win, 6),
            "avg_loss": round(avg_loss, 6),
            "avg_pnl": round(avg_pnl, 6),
            "ev": round(ev, 6),
        }

    args.output.parent.mkdir(parents=True, exist_ok=True)
    args.output.write_text(json.dumps(ev_table, indent=2) + "\n")

    total_rows = sum(int(s["count"]) for s in stats.values())
    leaf_cells = sum(len(cm) for cm in ev_table.values())
    age_buckets = len(ev_table)

    print("\n=== EV TABLE SUMMARY ===")
    print(f"Input files: {[str(p) for p in args.input_logs]}")
    print(f"Total parsed [TRADE_PATH] rows: {total_rows}")
    print(f"Age-bucket keys (after min_count): {age_buckets}")
    print(f"  age key names: {sorted(ev_table.keys())}")
    print(f"Leaf cells (after min_count): {leaf_cells}")
    print(f"Written: {args.output}")

    flat: list[tuple[float, str, str, int]] = []
    for a, cmap in ev_table.items():
        for c, val in cmap.items():
            flat.append((float(val["ev"]), a, c, int(val["count"])))

    flat.sort(reverse=True, key=lambda t: t[0])
    print("\nTop EV buckets:")
    for ev, a, c, cnt in flat[:10]:
        print(f"  {a} | conf={c} -> EV={ev:.6f} (n={cnt})")

    ev_values = [
        float(v["ev"])
        for ag in ev_table.values()
        for v in ag.values()
    ]
    ev_values.sort()
    print("\n--- EV calibration (leaf cells) ---")
    print(f"  cell count: {len(ev_values)}")
    if ev_values:
        print(f"  sorted EV: {[round(x, 6) for x in ev_values]}")
        for p in (25, 30, 35, 40, 50):
            q = linear_percentile(ev_values, float(p))
            print(f"  {p:>2}th percentile EV -> {q:.6f}")
        print(
            "  hint: PAPER_EV_EXPIRE_THRESHOLD ≈ 30th–40th pct often separates low-EV cells; merge logs for coverage."
        )


if __name__ == "__main__":
    main()
