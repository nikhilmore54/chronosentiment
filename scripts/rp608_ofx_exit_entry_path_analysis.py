#!/usr/bin/env python3
"""OFX volume-paper path diagnostic: exit timing vs entry quality (research-only).

Streams crypto aggTrades once and stamps each closed OFX paper trade with:
  - MFE/MAE (signed bps vs entry)
  - early MAE (first 60s)
  - giveback vs peak MFE at exit
  - post-exit 15m favorable/adverse continuation from exit mark

Labels (hypotheses only; does not change live rules / authority 0%):
  premature — had meaningful MFE, gave back a lot, and price continued favorably after exit
  late      — loss after having MFE>=15bps that fully gave back, with little post-exit favor
  bad_entry — first-minute MAE <= -8bps and MFE never reached +10bps

Usage:
  python3 scripts/research/rp608_ofx_exit_entry_path_analysis.py
"""

from __future__ import annotations

import json
import sys
from collections import Counter, defaultdict
from pathlib import Path
from typing import Any, Dict, List, Optional

WORKSPACE = Path(__file__).resolve().parents[2]
OUT = WORKSPACE / "live_capture/shadow_validation/crypto/ofx_aggtrade_capture_v1/ofx_volume_paper_outcomes.jsonl"
AGG = WORKSPACE / "live_capture/shadow_validation/crypto/ofx_aggtrade_capture_v1/aggtrades.jsonl"
REPORT = WORKSPACE / "live_capture/shadow_validation/crypto/ofx_aggtrade_capture_v1/ofx_exit_entry_path_analysis.json"

POST_MS = 15 * 60 * 1000
EARLY_MS = 60 * 1000


def _f(x: Any, d: float = 0.0) -> float:
    try:
        return float(x)
    except (TypeError, ValueError):
        return d


def signed_bps(side: str, entry: float, px: float) -> float:
    if entry <= 0:
        return 0.0
    if side == "LONG":
        return (px - entry) / entry * 1e4
    return (entry - px) / entry * 1e4


def avg(xs) -> Optional[float]:
    xs = list(xs)
    if not xs:
        return None
    return round(sum(xs) / len(xs), 2)


def main() -> int:
    if not OUT.exists() or not AGG.exists():
        print(f"missing inputs OUT={OUT.exists()} AGG={AGG.exists()}", file=sys.stderr)
        return 2

    rows = [json.loads(l) for l in OUT.read_text().splitlines() if l.strip()]
    trades: List[Dict[str, Any]] = []
    for r in rows:
        e = int(r.get("entry_time_ms") or 0)
        x = int(r.get("exited_ms") or 0)
        px = _f(r.get("entry_price"))
        xp = _f(r.get("exit_price"))
        if e <= 0 or x <= e or px <= 0:
            continue
        tempo = None
        if isinstance(r.get("entry_side_tempo"), dict):
            tempo = r["entry_side_tempo"].get("tempo")
        trades.append(
            {
                "id": r.get("ofx_trade_id"),
                "entry_ms": e,
                "exit_ms": x,
                "entry_px": px,
                "exit_px": xp,
                "side": str(r.get("direction") or "").upper(),
                "net": _f(r.get("net_pnl_usdt")),
                "bps": _f(r.get("net_return_bps")),
                "win": _f(r.get("net_pnl_usdt")) > 0,
                "exit_reason": r.get("exit_reason"),
                "tempo": tempo,
                "overshoot": bool(r.get("exit_match_overshoot")),
                "hold_min": (x - e) / 60000.0,
                "mfe_bps": 0.0,
                "mae_bps": 0.0,
                "early_mae_bps": 0.0,
                "post_fav_bps": 0.0,
                "post_adv_bps": 0.0,
                "giveback_bps": 0.0,
                "captured_frac": None,
                "exit_signed_bps": None,
                "ticks": 0,
                "post_ticks": 0,
            }
        )
    trades.sort(key=lambda t: t["entry_ms"])

    i_start = 0
    active: List[int] = []
    post_active: List[tuple] = []
    n_lines = 0

    with AGG.open() as f:
        for line in f:
            n_lines += 1
            if n_lines % 2_000_000 == 0:
                print(
                    f"progress lines={n_lines} active={len(active)} post={len(post_active)}",
                    flush=True,
                )
            try:
                o = json.loads(line)
            except Exception:
                continue
            ts = o.get("trade_time_ms") or o.get("T") or o.get("timestamp_ms") or o.get("event_time_ms")
            try:
                ts = int(ts)
                px = float(o.get("price"))
            except (TypeError, ValueError):
                continue

            while i_start < len(trades) and trades[i_start]["entry_ms"] <= ts:
                active.append(i_start)
                i_start += 1

            still: List[int] = []
            for idx in active:
                t = trades[idx]
                if ts < t["entry_ms"]:
                    still.append(idx)
                    continue
                if ts > t["exit_ms"]:
                    sb_exit = signed_bps(t["side"], t["entry_px"], t["exit_px"])
                    t["exit_signed_bps"] = sb_exit
                    if t["mfe_bps"] > 1e-9:
                        t["captured_frac"] = max(0.0, min(1.5, sb_exit / t["mfe_bps"]))
                        t["giveback_bps"] = max(0.0, t["mfe_bps"] - sb_exit)
                    else:
                        t["captured_frac"] = None
                        t["giveback_bps"] = 0.0
                    post_active.append((idx, t["exit_ms"] + POST_MS))
                    continue
                sb = signed_bps(t["side"], t["entry_px"], px)
                t["ticks"] += 1
                if sb > t["mfe_bps"]:
                    t["mfe_bps"] = sb
                if sb < t["mae_bps"]:
                    t["mae_bps"] = sb
                if (ts - t["entry_ms"]) <= EARLY_MS and sb < t["early_mae_bps"]:
                    t["early_mae_bps"] = sb
                still.append(idx)
            active = still

            stillp = []
            for idx, end in post_active:
                if ts > end:
                    continue
                t = trades[idx]
                if ts < t["exit_ms"]:
                    stillp.append((idx, end))
                    continue
                sb_from_exit = signed_bps(t["side"], t["exit_px"], px)
                t["post_ticks"] += 1
                if sb_from_exit > t["post_fav_bps"]:
                    t["post_fav_bps"] = sb_from_exit
                if sb_from_exit < t["post_adv_bps"]:
                    t["post_adv_bps"] = sb_from_exit
                stillp.append((idx, end))
            post_active = stillp

    # finalize any still-open actives (agg ended before exit)
    for idx in active:
        t = trades[idx]
        sb_exit = signed_bps(t["side"], t["entry_px"], t["exit_px"])
        t["exit_signed_bps"] = sb_exit
        if t["mfe_bps"] > 1e-9:
            t["captured_frac"] = max(0.0, min(1.5, sb_exit / t["mfe_bps"]))
            t["giveback_bps"] = max(0.0, t["mfe_bps"] - sb_exit)

    n_prem = n_late = n_bad = n_bad_sig = 0
    prem_net = late_net = bad_net = 0.0
    post_opp: List[float] = []

    for t in trades:
        exit_sb = t.get("exit_signed_bps")
        if exit_sb is None:
            exit_sb = signed_bps(t["side"], t["entry_px"], t["exit_px"])
            t["exit_signed_bps"] = exit_sb
            if t["mfe_bps"] > 1e-9:
                t["captured_frac"] = max(0.0, min(1.5, exit_sb / t["mfe_bps"]))
                t["giveback_bps"] = max(0.0, t["mfe_bps"] - exit_sb)

        premature = t["mfe_bps"] >= 15 and t["giveback_bps"] >= 10 and t["post_fav_bps"] >= 10
        left_on_table = (
            t["win"]
            and t["mfe_bps"] >= 20
            and (t.get("captured_frac") or 0) < 0.4
            and t["post_fav_bps"] >= 10
        )
        late = (not t["win"]) and t["mfe_bps"] >= 15 and exit_sb <= 0 and t["post_fav_bps"] < 5
        late2 = (not t["win"]) and t["mae_bps"] <= -20 and abs(exit_sb - t["mae_bps"]) <= 5
        bad_entry = t["early_mae_bps"] <= -8 and t["mfe_bps"] < 10
        bad_entry_signal = t["early_mae_bps"] <= -15

        t["premature"] = bool(premature or left_on_table)
        t["late"] = bool(late or late2)
        t["bad_entry"] = bool(bad_entry)
        t["bad_entry_signal"] = bool(bad_entry_signal)

        if t["premature"]:
            n_prem += 1
            prem_net += t["net"]
        if t["late"]:
            n_late += 1
            late_net += t["net"]
        if t["bad_entry"]:
            n_bad += 1
            bad_net += t["net"]
        if t["bad_entry_signal"]:
            n_bad_sig += 1
        if (not t["win"]) and t["post_fav_bps"] >= 10:
            post_opp.append(t["post_fav_bps"])

    loss_edge = [t for t in trades if (not t["win"]) and t["mfe_bps"] >= 15]
    wins = [t for t in trades if t["win"]]
    losses = [t for t in trades if not t["win"]]

    def cohort(label: str, S: List[Dict[str, Any]]) -> Dict[str, Any]:
        if not S:
            return {"n": 0}
        return {
            "n": len(S),
            "avg_mfe_bps": avg(t["mfe_bps"] for t in S),
            "avg_mae_bps": avg(t["mae_bps"] for t in S),
            "avg_exit_bps": avg(t["exit_signed_bps"] for t in S),
            "avg_giveback_bps": avg(t["giveback_bps"] for t in S),
            "avg_captured_frac": avg(
                t["captured_frac"] for t in S if t.get("captured_frac") is not None
            ),
            "avg_post_fav_bps": avg(t["post_fav_bps"] for t in S),
            "avg_early_mae_bps": avg(t["early_mae_bps"] for t in S),
            "pct_premature": round(100 * sum(1 for t in S if t["premature"]) / len(S), 1),
            "pct_late": round(100 * sum(1 for t in S if t["late"]) / len(S), 1),
            "pct_bad_entry": round(100 * sum(1 for t in S if t["bad_entry"]) / len(S), 1),
            "net_sum": round(sum(t["net"] for t in S), 4),
        }

    # structural hold / tempo (no path)
    def group(key: str) -> Dict[str, Any]:
        g: Dict[Any, List] = defaultdict(list)
        for t in trades:
            g[t.get(key)].append(t)
        out = {}
        for k, vs in sorted(g.items(), key=lambda kv: -len(kv[1])):
            out[str(k)] = {
                "n": len(vs),
                "wr_pct": round(100 * sum(1 for t in vs if t["win"]) / len(vs), 1),
                "net": round(sum(t["net"] for t in vs), 3),
                "avg_hold_min": round(sum(t["hold_min"] for t in vs) / len(vs), 2),
            }
        return out

    keep_bad = [t for t in trades if not t["bad_entry"]]
    keep_mixed = [t for t in trades if t["tempo"] != "MIXED"]

    report = {
        "protocol_id": "RP-608-OFX-EXIT-ENTRY-PATH-ANALYSIS-001",
        "baseline_of_inert": True,
        "live_order_authority": 0.0,
        "n_trades": len(trades),
        "aggtrade_lines_streamed": n_lines,
        "post_exit_horizon_ms": POST_MS,
        "early_window_ms": EARLY_MS,
        "labels": {
            "premature_exit": {"n": n_prem, "net_usdt": round(prem_net, 4)},
            "late_exit": {"n": n_late, "net_usdt": round(late_net, 4)},
            "bad_entry": {"n": n_bad, "net_usdt": round(bad_net, 4)},
            "bad_entry_signal_early15bps": {"n": n_bad_sig},
        },
        "wins": cohort("wins", wins),
        "losses": cohort("losses", losses),
        "losses_with_mfe_ge_15bps": {
            "n": len(loss_edge),
            "avg_giveback_bps": avg(t["giveback_bps"] for t in loss_edge),
            "avg_post_fav_bps": avg(t["post_fav_bps"] for t in loss_edge),
            "net_sum": round(sum(t["net"] for t in loss_edge), 4),
        },
        "lost_opportunity_post15m_on_losses_fav_ge_10bps": {
            "n": len(post_opp),
            "median_post_fav_bps": (
                round(sorted(post_opp)[len(post_opp) // 2], 2) if post_opp else None
            ),
            "sum_post_fav_bps": round(sum(post_opp), 1) if post_opp else 0.0,
        },
        "by_tempo": group("tempo"),
        "counterfactual_skip_bad_entry": {
            "n": len(keep_bad),
            "wr_pct": round(100 * sum(1 for t in keep_bad if t["win"]) / len(keep_bad), 1)
            if keep_bad
            else None,
            "net_usdt": round(sum(t["net"] for t in keep_bad), 4) if keep_bad else None,
        },
        "counterfactual_skip_MIXED_tempo": {
            "n": len(keep_mixed),
            "wr_pct": round(100 * sum(1 for t in keep_mixed if t["win"]) / len(keep_mixed), 1)
            if keep_mixed
            else None,
            "net_usdt": round(sum(t["net"] for t in keep_mixed), 4) if keep_mixed else None,
        },
        "notes": [
            "Hypotheses only — does not change OFX entry/exit rules or authority.",
            "NSE OFX book excluded (0 closed trades post reset).",
            "Do not pool into baseline four books.",
        ],
    }

    # slim per-trade extract for top opportunity cases
    top_prem = sorted(
        [t for t in trades if t["premature"]],
        key=lambda t: t["post_fav_bps"] + t["giveback_bps"],
        reverse=True,
    )[:15]
    top_bad = sorted(
        [t for t in trades if t["bad_entry"]],
        key=lambda t: t["early_mae_bps"],
    )[:15]
    report["examples_premature"] = [
        {
            "id": t["id"],
            "side": t["side"],
            "tempo": t["tempo"],
            "net": t["net"],
            "mfe_bps": round(t["mfe_bps"], 2),
            "exit_bps": round(t["exit_signed_bps"] or 0, 2),
            "giveback_bps": round(t["giveback_bps"], 2),
            "post_fav_bps": round(t["post_fav_bps"], 2),
            "hold_min": round(t["hold_min"], 2),
        }
        for t in top_prem
    ]
    report["examples_bad_entry"] = [
        {
            "id": t["id"],
            "side": t["side"],
            "tempo": t["tempo"],
            "net": t["net"],
            "early_mae_bps": round(t["early_mae_bps"], 2),
            "mfe_bps": round(t["mfe_bps"], 2),
            "exit_bps": round(t["exit_signed_bps"] or 0, 2),
        }
        for t in top_bad
    ]

    REPORT.write_text(json.dumps(report, indent=2))
    print(json.dumps(report, indent=2))
    print(f"wrote {REPORT}", flush=True)
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
