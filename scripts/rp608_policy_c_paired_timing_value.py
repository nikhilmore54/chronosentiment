#!/usr/bin/env python3
"""
RP-608-POLICY-C-PAIRED-ECON-001

Paired incremental economic-value report for Policy C timing advances.

For each symbol/session where C signals earlier than A:
  - Early C entry vs later A entry
  - Identical exit: A's exit bar/price (entry-timing only)
  - Identical friction assumptions (stress grid)

Does not change live G3. Authority: 0.0%.
"""

from __future__ import annotations

import argparse
import json
from datetime import datetime, timezone
from pathlib import Path
from typing import Any, Dict, List, Optional, Sequence


# Base round-trip friction applies equally and cancels in paired deltas.
# Early-exposure add-on is charged only to C (worse open liquidity / longer risk).
BASE_FRICTION_BPS = 12.0
EARLY_EXTRA_FRICTION_GRID_BPS = (0.0, 5.0, 10.0, 20.0, 35.0)


def _net(entry: float, exit_px: float, friction_bps: float) -> float:
    if entry <= 0:
        return 0.0
    raw = ((exit_px / entry) - 1.0) * 1e4
    return raw - float(friction_bps)


def _mean(xs: Sequence[float]) -> Optional[float]:
    return round(sum(xs) / len(xs), 2) if xs else None


def build_pairs(econ: Dict[str, Any]) -> List[Dict[str, Any]]:
    pairs: List[Dict[str, Any]] = []
    for r in econ.get("results") or []:
        trades = {t["policy"]: t for t in (r.get("trades") or []) if isinstance(t, dict)}
        a = trades.get("same_session")
        c = trades.get("cross_gap_neutral")
        if not a or not c:
            continue
        if c.get("classification") != "timing_advance":
            continue
        entry_c = float(c["entry_price"])
        entry_a = float(a["entry_price"])
        exit_px = float(a["exit_price"])  # identical exit = baseline A exit
        exit_bar = int(a["exit_bar"])
        # Price path endpoint from early entry to baseline entry (not full MAE path).
        move_to_a_bps = ((entry_a / entry_c) - 1.0) * 1e4 if entry_c > 0 else 0.0
        entry_edge_bps = (1.0 - (entry_c / entry_a)) * 1e4 if entry_a > 0 else 0.0

        early_extra_rows = {}
        for extra in EARLY_EXTRA_FRICTION_GRID_BPS:
            net_a = _net(entry_a, exit_px, BASE_FRICTION_BPS)
            net_c = _net(entry_c, exit_px, BASE_FRICTION_BPS + float(extra))
            delta = net_c - net_a
            early_extra_rows[str(extra)] = {
                "base_friction_bps": BASE_FRICTION_BPS,
                "early_extra_friction_bps": float(extra),
                "net_A_bps": round(net_a, 2),
                "net_C_early_bps": round(net_c, 2),
                "incremental_delta_bps": round(delta, 2),
                "early_better": delta > 0,
            }

        primary = early_extra_rows["0.0"]
        pairs.append(
            {
                "symbol": r.get("symbol"),
                "session_date": r.get("session_date"),
                "C_entry_bar": int(c["entry_bar"]),
                "A_entry_bar": int(a["entry_bar"]),
                "bars_advanced": int(a["entry_bar"]) - int(c["entry_bar"]),
                "C_entry_price": entry_c,
                "A_entry_price": entry_a,
                "identical_exit_bar": exit_bar,
                "identical_exit_price": exit_px,
                "entry_edge_bps": round(entry_edge_bps, 2),
                "price_change_C_entry_to_A_entry_bps": round(move_to_a_bps, 2),
                "A_own_exit_mae_bps": a.get("mae_bps"),
                "C_own_exit_mae_bps": c.get("mae_bps"),
                "early_extra_friction_grid": early_extra_rows,
                "primary_friction_bps": BASE_FRICTION_BPS,
                "incremental_delta_bps_at_12": primary["incremental_delta_bps"],
                "early_better_at_12": primary["early_better"],
            }
        )
    return pairs


def summarize(pairs: List[Dict[str, Any]]) -> Dict[str, Any]:
    n = len(pairs)
    pos = [p for p in pairs if p["early_better_at_12"]]
    neg = [p for p in pairs if not p["early_better_at_12"]]
    deltas = [float(p["incremental_delta_bps_at_12"]) for p in pairs]
    edges = [float(p["entry_edge_bps"]) for p in pairs]
    moves = [float(p["price_change_C_entry_to_A_entry_bps"]) for p in pairs]

    by_early_extra = {}
    for extra in EARLY_EXTRA_FRICTION_GRID_BPS:
        key = str(extra)
        ds = [float(p["early_extra_friction_grid"][key]["incremental_delta_bps"]) for p in pairs]
        better = sum(1 for p in pairs if p["early_extra_friction_grid"][key]["early_better"])
        by_early_extra[key] = {
            "early_extra_friction_bps": float(extra),
            "mean_incremental_delta_bps": _mean(ds),
            "pct_early_better": round(better / n, 4) if n else None,
            "n_early_better": better,
            "n_early_worse": n - better,
        }

    return {
        "n_paired_timing_advances": n,
        "identical_exit_rule": "baseline_A_exit_bar_and_price",
        "primary_friction_bps": BASE_FRICTION_BPS,
        "note_on_friction": (
            "Equal round-trip friction cancels in paired deltas; stress is an add-on "
            "charged only to early C entries (open-liquidity / longer-risk proxy)."
        ),
        "at_12bps_equal_friction": {
            "mean_incremental_delta_bps": _mean(deltas),
            "median_incremental_delta_bps": (
                round(sorted(deltas)[n // 2], 2) if n else None
            ),
            "n_positive_net_benefit": len(pos),
            "n_negative_net_benefit": len(neg),
            "pct_early_better": round(len(pos) / n, 4) if n else None,
            "mean_delta_when_positive": _mean([float(p["incremental_delta_bps_at_12"]) for p in pos]),
            "mean_delta_when_negative": _mean([float(p["incremental_delta_bps_at_12"]) for p in neg]),
            "mean_entry_edge_bps": _mean(edges),
            "mean_price_change_C_to_A_entry_bps": _mean(moves),
            "interpretation": (
                "incremental_delta = net(C early → A exit) − net(A → A exit). "
                "Positive => earlier C entry improved outcome under identical exit."
            ),
        },
        "early_extra_friction_stress": by_early_extra,
        "reconcile_vs_portfolio_summary": (
            "Portfolio A-vs-C can still favor A because C uses its own exits, includes "
            "non-paired trades, and may exit earlier/later than A. Paired identical-exit "
            "isolates entry-timing value only."
        ),
        "hypothesis_status": {
            "availability": "SUPPORTED (timing advances exist)",
            "signal_quality": "INCONCLUSIVE (not tested in this paired exit lock)",
            "economic_paired_identical_exit": (
                "SUPPORTED_ON_THIS_SAMPLE"
                if (n and _mean(deltas) is not None and _mean(deltas) > 0)
                else "NOT_SUPPORTED_ON_THIS_SAMPLE"
            ),
            "economic_portfolio_level": "NOT_SUPPORTED_IN_PRIOR_TWO_SESSION_SUMMARY",
        },
        "direction_bias_note": "LONG-only cohort; do not generalize to SHORT.",
        "governance": {
            "live_g3": "FROZEN",
            "deployment": "NOT_AUTHORIZED",
            "authority": "0.0%",
        },
    }


def main() -> int:
    ap = argparse.ArgumentParser(description="Paired C vs A timing-advance economic value (0% authority)")
    ap.add_argument(
        "--econ",
        required=True,
        help="Path to policy_c_econ_*.json from rp608_policy_c_multisession_economics.py",
    )
    ap.add_argument("--out", required=True)
    args = ap.parse_args()

    econ = json.loads(Path(args.econ).read_text(encoding="utf-8"))
    pairs = build_pairs(econ)
    payload = {
        "protocol_id": "RP-608-POLICY-C-PAIRED-ECON-001",
        "authority": "0.0%",
        "disposition": "DIAGNOSTIC_ONLY_NO_LIVE_G3_CHANGE",
        "generated_at_utc": datetime.now(timezone.utc).isoformat(),
        "source_econ_protocol": econ.get("protocol_id"),
        "source_sessions": econ.get("sessions"),
        "summary": summarize(pairs),
        "pairs": pairs,
    }
    out = Path(args.out)
    out.parent.mkdir(parents=True, exist_ok=True)
    out.write_text(json.dumps(payload, indent=2), encoding="utf-8")
    print(json.dumps(payload["summary"], indent=2))
    print(f"[wrote] {out}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
