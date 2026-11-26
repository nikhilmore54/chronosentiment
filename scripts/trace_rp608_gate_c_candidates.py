#!/usr/bin/env python3
"""
Trace Gate C (RP-608-EVENT-001) Candidates A–D and rank by episode-level economics.

Unit of analysis: first-of-episode (60s quiescence OR direction change).
Forward-only outcomes; 30 bps adverse stop; 12 bps RT friction; 15/30/60m horizons.
0.0% order authority — observational only.
"""

from __future__ import annotations

import argparse
import json
import math
import sys
import time
from collections import Counter, defaultdict
from datetime import datetime, timezone
from pathlib import Path
from typing import Any, Dict, List, Optional, Tuple

WORKSPACE = Path(__file__).resolve().parent.parent
sys.path.insert(0, str(WORKSPACE))

EVENT_DIR = WORKSPACE / "live_capture" / "shadow_validation" / "crypto" / "event_driven"
CANDIDATES = {
    16: {"letter": "A", "label": "Candidate A (16-Bar · 4h)"},
    32: {"letter": "B", "label": "Candidate B (32-Bar · 8h)"},
    64: {"letter": "C", "label": "Candidate C (64-Bar · 16h)"},
    120: {"letter": "D", "label": "Candidate D (120-Bar · 30h)"},
}
FRICTION_BPS = 12.0
STOP_BPS = 30.0
EPISODE_GAP_MS = 60_000
HORIZONS = [(900, "15m"), (1800, "30m"), (3600, "60m")]


def load_jsonl(path: Path) -> List[Dict[str, Any]]:
    out: List[Dict[str, Any]] = []
    if not path.exists():
        return out
    with path.open("r", encoding="utf-8") as fh:
        for line in fh:
            line = line.strip()
            if not line:
                continue
            try:
                out.append(json.loads(line))
            except json.JSONDecodeError:
                continue
    return out


def cluster_episodes(recs: List[Dict[str, Any]]) -> List[List[Dict[str, Any]]]:
    episodes: List[List[Dict[str, Any]]] = []
    cur: List[Dict[str, Any]] = []
    for r in sorted(recs, key=lambda x: x["timestamp_epoch_ms"]):
        if not cur:
            cur = [r]
            continue
        gap = r["timestamp_epoch_ms"] - cur[-1]["timestamp_epoch_ms"]
        prev_dir = cur[-1]["recommendation"]["direction"]
        cur_dir = r["recommendation"]["direction"]
        if gap > EPISODE_GAP_MS or cur_dir != prev_dir:
            episodes.append(cur)
            cur = [r]
        else:
            cur.append(r)
    if cur:
        episodes.append(cur)
    return episodes


def resolve_trade(
    *,
    direction: str,
    entry: float,
    inval: float,
    t_rec: int,
    timeline: List[Tuple[int, float, float]],
    horizon_ms: int,
) -> Tuple[str, Optional[float], Optional[int], float, float]:
    """Returns (reason, exit_price, exit_ms, mfe_bps, mae_bps)."""
    mfe = 0.0
    mae = 0.0
    # binary search start
    lo, hi = 0, len(timeline)
    while lo < hi:
        mid = (lo + hi) // 2
        if timeline[mid][0] <= t_rec:
            lo = mid + 1
        else:
            hi = mid
    for i in range(lo, len(timeline)):
        q_t, bid, ask = timeline[i]
        if direction == "LONG":
            fav = ((bid - entry) / entry) * 10000.0
            adv = ((entry - bid) / entry) * 10000.0
            if fav > mfe:
                mfe = fav
            if adv > mae:
                mae = adv
            if bid <= inval:
                return "STOP_INVALIDATED", bid, q_t, mfe, mae
            if q_t >= t_rec + horizon_ms:
                return "HORIZON_MATURED", bid, q_t, mfe, mae
        else:
            fav = ((entry - ask) / entry) * 10000.0
            adv = ((ask - entry) / entry) * 10000.0
            if fav > mfe:
                mfe = fav
            if adv > mae:
                mae = adv
            if ask >= inval:
                return "STOP_INVALIDATED", ask, q_t, mfe, mae
            if q_t >= t_rec + horizon_ms:
                return "HORIZON_MATURED", ask, q_t, mfe, mae
    return "UNRESOLVED", None, None, mfe, mae


def gross_bps(direction: str, entry: float, exit_px: float) -> float:
    if direction == "LONG":
        return ((exit_px - entry) / entry) * 10000.0
    return ((entry - exit_px) / entry) * 10000.0


def summarize_returns(nets: List[float]) -> Dict[str, float]:
    if not nets:
        return {
            "n": 0,
            "win_rate_pct": 0.0,
            "net_exp_bps": 0.0,
            "median_net_bps": 0.0,
            "total_net_bps": 0.0,
            "max_dd_bps": 0.0,
            "profit_factor": 0.0,
        }
    wins = [x for x in nets if x > 0]
    losses = [x for x in nets if x <= 0]
    sorted_n = sorted(nets)
    mid = len(sorted_n) // 2
    median = (
        sorted_n[mid]
        if len(sorted_n) % 2
        else 0.5 * (sorted_n[mid - 1] + sorted_n[mid])
    )
    cum = peak = max_dd = 0.0
    for x in nets:
        cum += x
        peak = max(peak, cum)
        max_dd = max(max_dd, peak - cum)
    gross_win = sum(wins)
    gross_loss = abs(sum(losses))
    pf = (gross_win / gross_loss) if gross_loss > 0 else (math.inf if gross_win > 0 else 0.0)
    return {
        "n": len(nets),
        "win_rate_pct": round(100.0 * len(wins) / len(nets), 2),
        "net_exp_bps": round(sum(nets) / len(nets), 2),
        "median_net_bps": round(median, 2),
        "total_net_bps": round(sum(nets), 2),
        "max_dd_bps": round(max_dd, 2),
        "profit_factor": round(pf, 3) if math.isfinite(pf) else None,
    }


def evaluate_candidate(
    bars: int,
    recs: List[Dict[str, Any]],
    timeline: List[Tuple[int, float, float]],
) -> Dict[str, Any]:
    meta = CANDIDATES[bars]
    episodes = cluster_episodes(recs)
    dir_counts = Counter(r["recommendation"]["direction"] for r in recs)
    ep_dirs = Counter(ep[0]["recommendation"]["direction"] for ep in episodes)

    first_ts = recs[0]["timestamp_utc"] if recs else None
    last_ts = recs[-1]["timestamp_utc"] if recs else None
    last_dec = recs[-1]["recommendation"] if recs else None

    horizons_out: Dict[str, Any] = {}
    for h_sec, h_label in HORIZONS:
        matured = stopped = unresolved = 0
        nets: List[float] = []
        grosses: List[float] = []
        mfes: List[float] = []
        maes: List[float] = []
        by_dir: Dict[str, List[float]] = {"LONG": [], "SHORT": []}

        for ep in episodes:
            r = ep[0]
            direction = r["recommendation"]["direction"]
            entry = float(r["recommendation"]["suggested_entry_price"])
            inval = float(r["recommendation"]["invalidation_price"])
            t_rec = int(r["timestamp_epoch_ms"])
            reason, exit_px, _, mfe, mae = resolve_trade(
                direction=direction,
                entry=entry,
                inval=inval,
                t_rec=t_rec,
                timeline=timeline,
                horizon_ms=h_sec * 1000,
            )
            if reason == "UNRESOLVED" or exit_px is None:
                unresolved += 1
                continue
            if reason == "STOP_INVALIDATED":
                stopped += 1
            else:
                matured += 1
            g = gross_bps(direction, entry, exit_px)
            n = g - FRICTION_BPS
            nets.append(n)
            grosses.append(g)
            mfes.append(mfe)
            maes.append(mae)
            by_dir[direction].append(n)

        closed = matured + stopped
        horizons_out[h_label] = {
            "episodes": len(episodes),
            "closed": closed,
            "matured": matured,
            "stopped": stopped,
            "unresolved": unresolved,
            "stop_rate_pct": round(100.0 * stopped / closed, 2) if closed else 0.0,
            "avg_mfe_bps": round(sum(mfes) / len(mfes), 2) if mfes else 0.0,
            "avg_mae_bps": round(sum(maes) / len(maes), 2) if maes else 0.0,
            "summary": summarize_returns(nets),
            "by_direction": {
                d: summarize_returns(vals) for d, vals in by_dir.items()
            },
            "gross_exp_bps": round(sum(grosses) / len(grosses), 2) if grosses else 0.0,
        }

    return {
        "candidate": meta["letter"],
        "bars": bars,
        "label": meta["label"],
        "raw_emissions": len(recs),
        "episodes": len(episodes),
        "avg_signals_per_episode": round(len(recs) / len(episodes), 2) if episodes else 0.0,
        "direction_mix_raw": dict(dir_counts),
        "direction_mix_episodes": dict(ep_dirs),
        "first_utc": first_ts,
        "last_utc": last_ts,
        "last_recommendation": last_dec,
        "horizons": horizons_out,
    }


def composite_score(row: Dict[str, Any]) -> float:
    """
    Robustness-oriented score across 15/30/60m episode net expectancy.
    Penalize high stop rate and drawdown; require closed episodes.
    """
    hs = row["horizons"]
    exps = []
    stop_rates = []
    dds = []
    closed_n = 0
    for lab in ("15m", "30m", "60m"):
        h = hs[lab]
        s = h["summary"]
        if s["n"] <= 0:
            continue
        exps.append(s["net_exp_bps"])
        stop_rates.append(h["stop_rate_pct"])
        dds.append(s["max_dd_bps"])
        closed_n += s["n"]
    if not exps or closed_n < 5:
        return float("-inf")
    mean_exp = sum(exps) / len(exps)
    consistency = min(exps)  # worst horizon
    avg_stop = sum(stop_rates) / len(stop_rates)
    avg_dd = sum(dds) / len(dds)
    # Prefer positive mean + positive worst-horizon; penalize stop/dd
    return mean_exp + 0.5 * consistency - 0.15 * avg_stop - 0.02 * (avg_dd / 100.0)


def main() -> int:
    ap = argparse.ArgumentParser()
    ap.add_argument("--dir", type=str, default=str(EVENT_DIR))
    ap.add_argument("--out", type=str, default="")
    args = ap.parse_args()
    base = Path(args.dir)

    print("=" * 92)
    print("  GATE C TRACE: RP-608-EVENT-001 Cross-Candidate Episode Economics")
    print(f"  dir={base}")
    print(f"  stop={STOP_BPS} bps · friction={FRICTION_BPS} bps · episode_gap={EPISODE_GAP_MS}ms")
    print(f"  authority=0.0% · unit=first-of-episode")
    print("=" * 92)

    by_bars: Dict[int, List[Dict[str, Any]]] = {}
    timeline_map: Dict[int, Tuple[float, float]] = {}
    for bars in CANDIDATES:
        recs = load_jsonl(base / f"event_recs_candidate_{bars}b.jsonl")
        by_bars[bars] = recs
        for r in recs:
            q = r.get("quote") or {}
            bid = float(q.get("bid_price") or 0)
            ask = float(q.get("ask_price") or 0)
            t = int(r["timestamp_epoch_ms"])
            if bid > 0 and ask > bid:
                timeline_map[t] = (bid, ask)

    timeline = sorted((t, b, a) for t, (b, a) in timeline_map.items())
    print(f"\n[*] Forward quote timeline points: {len(timeline)}")

    results = []
    for bars in CANDIDATES:
        row = evaluate_candidate(bars, by_bars[bars], timeline)
        row["composite_score"] = composite_score(row)
        results.append(row)
        print(f"\n--- {row['label']} ---")
        print(
            f"    raw={row['raw_emissions']}  episodes={row['episodes']}  "
            f"avg/ep={row['avg_signals_per_episode']}  "
            f"dirs_ep={row['direction_mix_episodes']}"
        )
        print(f"    window: {row['first_utc']} → {row['last_utc']}")
        if row["last_recommendation"]:
            lr = row["last_recommendation"]
            print(f"    last: {lr.get('decision')} {lr.get('direction')}")
        for lab in ("15m", "30m", "60m"):
            h = row["horizons"][lab]
            s = h["summary"]
            print(
                f"    {lab}: closed={h['closed']} matured={h['matured']} stopped={h['stopped']} "
                f"unres={h['unresolved']} stop%={h['stop_rate_pct']} "
                f"netExp={s['net_exp_bps']} wr={s['win_rate_pct']}% "
                f"totNet={s['total_net_bps']} dd={s['max_dd_bps']} "
                f"L={h['by_direction']['LONG']['net_exp_bps']}/"
                f"S={h['by_direction']['SHORT']['net_exp_bps']}"
            )
        print(f"    composite_score={row['composite_score']}")

    # Agreement snapshot on latest overlapping update ids (if present)
    ranked = sorted(
        results,
        key=lambda r: (
            r["composite_score"] if math.isfinite(r["composite_score"]) else -1e18,
            r["horizons"]["30m"]["summary"]["net_exp_bps"],
            r["horizons"]["30m"]["summary"]["total_net_bps"],
        ),
        reverse=True,
    )
    print("\n" + "=" * 92)
    print("  RANKING (episode-level; multi-horizon composite)")
    for i, r in enumerate(ranked, 1):
        s30 = r["horizons"]["30m"]["summary"]
        flag = " << BEST" if i == 1 and math.isfinite(r["composite_score"]) else ""
        print(
            f"  #{i} {r['candidate']} {r['bars']}b  score={r['composite_score']:.2f}  "
            f"30m netExp={s30['net_exp_bps']} bps  wr={s30['win_rate_pct']}%  "
            f"episodes_closed={s30['n']}{flag}"
        )
    print("=" * 92)

    out = {
        "audit_name": "RP-608-EVENT-001 Gate C Candidate Trace",
        "timestamp_utc": datetime.now(timezone.utc).isoformat(),
        "order_authority": 0.0,
        "method": {
            "unit": "first_of_episode",
            "episode_gap_ms": EPISODE_GAP_MS,
            "stop_bps": STOP_BPS,
            "friction_bps": FRICTION_BPS,
            "horizons_sec": [h for h, _ in HORIZONS],
            "timeline_points": len(timeline),
        },
        "candidates": results,
        "ranking": [
            {
                "rank": i,
                "candidate": r["candidate"],
                "bars": r["bars"],
                "composite_score": r["composite_score"],
                "net_exp_15m": r["horizons"]["15m"]["summary"]["net_exp_bps"],
                "net_exp_30m": r["horizons"]["30m"]["summary"]["net_exp_bps"],
                "net_exp_60m": r["horizons"]["60m"]["summary"]["net_exp_bps"],
                "episodes": r["episodes"],
            }
            for i, r in enumerate(ranked, 1)
        ],
        "verdict": {
            "best_candidate": ranked[0]["candidate"] if ranked else None,
            "best_bars": ranked[0]["bars"] if ranked else None,
            "best_label": ranked[0]["label"] if ranked else None,
            "caveat": (
                "Episode-level shadow economics only; 0.0% authority; "
                "do not promote solely on raw emission counts."
            ),
        },
    }
    out_path = Path(args.out) if args.out else base / "gate_c_candidate_trace.json"
    out_path.write_text(json.dumps(out, indent=2) + "\n", encoding="utf-8")
    print(f"\n[+] Wrote {out_path}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
