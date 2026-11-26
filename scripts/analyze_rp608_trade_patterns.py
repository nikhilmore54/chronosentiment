#!/usr/bin/env python3
"""
scripts/analyze_rp608_trade_patterns.py

Comprehensive Trade-Level Pattern Discovery & Attribution Engine for RP-608.
Analyzes all 1,801 trades from Phase 3 Replay Ledger against 15m OHLCV history.

Features Reconstructed:
  1. Entry-Time Predictors (known strictly before or at entry):
     - traj15 momentum value
     - Signal candle structure: body ratio, upper wick ratio, lower wick ratio, range bps
     - Pre-entry runup: 3-bar and 5-bar cumulative return (bps)
     - Volume surge ratio: Signal bar volume vs trailing 5-bar average volume
     - Trailing volatility (bps)
     - Time-of-day: entry_bar_idx
     - Cross-sectional universe breadth at signal candle (% advancing)
  2. Early Post-Entry Telemetry (diagnostics visible at Bar 1, 2, 3 after entry):
     - Bar 1 MFE and MAE (bps)
     - Bar 1 Close Return (bps)
     - Bar 2 MFE and MAE (bps)
     - Bar 3 MFE and MAE (bps)
     - Time to first adverse dip below entry
  3. Ranked Cohorts:
     - Top 100 Winners vs Bottom 100 Losers
     - Full population (1,801 trades)
     - In-Sample (60%) vs Out-of-Sample (40%) splits
"""

from __future__ import annotations

import argparse
from datetime import datetime, timedelta, timezone
import json
import math
from pathlib import Path
import sys
from typing import Any, Dict, List, Optional, Tuple

WORKSPACE_DIR = Path(__file__).resolve().parent.parent
sys.path.insert(0, str(WORKSPACE_DIR))

from scripts.run_rp608_nse_shadow_daemon import NSE_2026_HOLIDAYS

IST_OFFSET = timedelta(hours=5, minutes=30)


def open_time_min(dt_ist: datetime) -> datetime:
    return dt_ist.replace(hour=9, minute=15, second=0, microsecond=0)


def load_ohlcv_universe(data_dir: Path) -> Tuple[Dict[str, Dict[str, Dict[int, Dict[str, float]]]], List[str]]:
    symbol_bars: Dict[str, Dict[str, Dict[int, Dict[str, float]]]] = {}
    all_sessions = set()

    for p in data_dir.glob("*.json"):
        sym = p.stem
        try:
            with open(p, "r", encoding="utf-8") as f:
                raw = json.load(f)
        except Exception:
            continue

        parsed = []
        for c in raw:
            ts = c.get("timestamp")
            if ts is None:
                continue
            ts_ms = int(ts * 1000) if ts < 1e11 else int(ts)
            o = float(c.get("open", 0.0))
            h = float(c.get("high", 0.0))
            l = float(c.get("low", 0.0))
            cl = float(c.get("close", 0.0))
            v = float(c.get("volume", 0.0))
            if o <= 0 or h <= 0 or l <= 0 or cl <= 0:
                continue
            parsed.append({"ts_ms": ts_ms, "open": o, "high": h, "low": l, "close": cl, "volume": v})

        parsed.sort(key=lambda x: x["ts_ms"])

        seen_ts = set()
        symbol_bars[sym] = {}

        for bar in parsed:
            if bar["ts_ms"] in seen_ts:
                continue
            seen_ts.add(bar["ts_ms"])

            dt_utc = datetime.fromtimestamp(bar["ts_ms"] / 1000.0, tz=timezone.utc)
            dt_ist = dt_utc + IST_OFFSET
            date_str = str(dt_ist.date())

            if dt_ist.weekday() >= 5 or dt_ist.date() in NSE_2026_HOLIDAYS:
                continue

            open_ist = dt_ist.replace(hour=9, minute=15, second=0, microsecond=0)
            close_ist = dt_ist.replace(hour=15, minute=30, second=0, microsecond=0)
            if dt_ist < open_ist or dt_ist >= close_ist:
                continue

            el_min = (dt_ist - open_time_min(dt_ist)).total_seconds() / 60.0
            bar_idx = int(el_min // 15) + 1
            if not (1 <= bar_idx <= 25):
                continue

            all_sessions.add(date_str)
            if date_str not in symbol_bars[sym]:
                symbol_bars[sym][date_str] = {}
            symbol_bars[sym][date_str][bar_idx] = bar

    return symbol_bars, sorted(all_sessions)


def compute_features(
    trades: List[Dict[str, Any]],
    symbol_bars: Dict[str, Dict[str, Dict[int, Dict[str, float]]]]
) -> List[Dict[str, Any]]:
    enriched = []

    for t in trades:
        sym = t["symbol"]
        sess = t["session_date"]
        e_bar = t["entry_bar_idx"]
        sig_bar_idx = e_bar - 1
        entry_p = t["entry_price"]

        bars_dict = symbol_bars.get(sym, {}).get(sess, {})
        sig_bar = bars_dict.get(sig_bar_idx)

        # Reconstruct trailing history up to signal bar
        history = [bars_dict[b]["close"] for b in sorted(bars_dict.keys()) if b <= sig_bar_idx]
        volumes = [bars_dict[b]["volume"] for b in sorted(bars_dict.keys()) if b <= sig_bar_idx]

        # Trajectory computation
        if len(history) < 2:
            traj15 = 0.0
            hist_vol = 0.0
        else:
            lookback = min(len(history), 15)
            ret = (history[-1] - history[-lookback]) / (history[-lookback] + 1e-9)
            diffs = [(history[i] - history[i-1]) / (history[i-1] + 1e-9) for i in range(-lookback + 1, 0)]
            vol = (sum(d*d for d in diffs) / len(diffs))**0.5 if diffs else 1e-3
            denom = (vol * (lookback**0.5) * 2.5) + 1e-9
            traj15 = math.tanh(ret / denom)
            hist_vol = vol * 10000.0

        # Signal candle structure
        if sig_bar:
            s_open = sig_bar["open"]
            s_high = sig_bar["high"]
            s_low = sig_bar["low"]
            s_close = sig_bar["close"]
            s_vol = sig_bar["volume"]

            c_range = max(s_high - s_low, 1e-6)
            c_range_bps = (c_range / s_open) * 10000.0
            body_size = abs(s_close - s_open)
            body_ratio = body_size / c_range
            upper_wick = (s_high - max(s_open, s_close)) / c_range
            lower_wick = (min(s_open, s_close) - s_low) / c_range
            body_ret_bps = ((s_close - s_open) / s_open) * 10000.0

            # Volume ratio vs trailing 5 bars
            prior_vols = volumes[-6:-1]
            avg_prior_vol = (sum(prior_vols) / len(prior_vols)) if prior_vols else s_vol
            vol_ratio = (s_vol / avg_prior_vol) if avg_prior_vol > 0 else 1.0

            # Pre-entry runup
            ret_3b = ((history[-1] - history[-4]) / history[-4] * 10000.0) if len(history) >= 4 else body_ret_bps
            ret_5b = ((history[-1] - history[-6]) / history[-6] * 10000.0) if len(history) >= 6 else ret_3b
        else:
            c_range_bps = 0.0
            body_ratio = 0.0
            upper_wick = 0.0
            lower_wick = 0.0
            body_ret_bps = 0.0
            vol_ratio = 1.0
            ret_3b = 0.0
            ret_5b = 0.0

        # Early trade progression: Bar 1, 2, 3
        # Bar 1 is e_bar
        bar1 = bars_dict.get(e_bar)
        bar2 = bars_dict.get(e_bar + 1)
        bar3 = bars_dict.get(e_bar + 2)

        if bar1:
            b1_mfe_bps = max(0.0, (bar1["high"] - entry_p) / entry_p * 10000.0)
            b1_mae_bps = max(0.0, (entry_p - bar1["low"]) / entry_p * 10000.0)
            b1_close_ret_bps = (bar1["close"] - entry_p) / entry_p * 10000.0
        else:
            b1_mfe_bps = 0.0
            b1_mae_bps = 0.0
            b1_close_ret_bps = 0.0

        if bar2:
            b2_mfe_bps = max(b1_mfe_bps, (bar2["high"] - entry_p) / entry_p * 10000.0)
            b2_mae_bps = max(b1_mae_bps, (entry_p - bar2["low"]) / entry_p * 10000.0)
            b2_close_ret_bps = (bar2["close"] - entry_p) / entry_p * 10000.0
        else:
            b2_mfe_bps = b1_mfe_bps
            b2_mae_bps = b1_mae_bps
            b2_close_ret_bps = b1_close_ret_bps

        if bar3:
            b3_mfe_bps = max(b2_mfe_bps, (bar3["high"] - entry_p) / entry_p * 10000.0)
            b3_mae_bps = max(b2_mae_bps, (entry_p - bar3["low"]) / entry_p * 10000.0)
            b3_close_ret_bps = (bar3["close"] - entry_p) / entry_p * 10000.0
        else:
            b3_mfe_bps = b2_mfe_bps
            b3_mae_bps = b2_mae_bps
            b3_close_ret_bps = b2_close_ret_bps

        rec = dict(t)
        rec.update({
            "traj15": round(traj15, 4),
            "trailing_vol_bps": round(hist_vol, 2),
            "signal_range_bps": round(c_range_bps, 2),
            "signal_body_ratio": round(body_ratio, 4),
            "signal_upper_wick_ratio": round(upper_wick, 4),
            "signal_lower_wick_ratio": round(lower_wick, 4),
            "signal_body_return_bps": round(body_ret_bps, 2),
            "signal_vol_surge_ratio": round(vol_ratio, 2),
            "pre_entry_runup_3b_bps": round(ret_3b, 2),
            "pre_entry_runup_5b_bps": round(ret_5b, 2),
            "bar1_mfe_bps": round(b1_mfe_bps, 2),
            "bar1_mae_bps": round(b1_mae_bps, 2),
            "bar1_close_ret_bps": round(b1_close_ret_bps, 2),
            "bar2_mfe_bps": round(b2_mfe_bps, 2),
            "bar2_mae_bps": round(b2_mae_bps, 2),
            "bar2_close_ret_bps": round(b2_close_ret_bps, 2),
            "bar3_mfe_bps": round(b3_mfe_bps, 2),
            "bar3_mae_bps": round(b3_mae_bps, 2),
            "bar3_close_ret_bps": round(b3_close_ret_bps, 2),
        })
        enriched.append(rec)

    return enriched


def calc_stats(values: List[float]) -> Dict[str, float]:
    if not values:
        return {"mean": 0.0, "median": 0.0, "std": 0.0, "min": 0.0, "max": 0.0}
    vals = sorted(values)
    n = len(vals)
    mean = sum(vals) / n
    median = vals[n // 2] if n % 2 != 0 else (vals[n // 2 - 1] + vals[n // 2]) / 2.0
    var = sum((x - mean) ** 2 for x in vals) / n
    std = math.sqrt(var)
    return {
        "mean": round(mean, 2),
        "median": round(median, 2),
        "std": round(std, 2),
        "min": round(vals[0], 2),
        "max": round(vals[-1], 2)
    }


def cohens_d(group1: List[float], group2: List[float]) -> float:
    if not group1 or not group2:
        return 0.0
    n1, n2 = len(group1), len(group2)
    m1, m2 = sum(group1) / n1, sum(group2) / n2
    v1 = sum((x - m1) ** 2 for x in group1) / (n1 - 1) if n1 > 1 else 0.0
    v2 = sum((x - m2) ** 2 for x in group2) / (n2 - 1) if n2 > 1 else 0.0
    s_pooled = math.sqrt(((n1 - 1) * v1 + (n2 - 1) * v2) / (n1 + n2 - 2)) if (n1 + n2 - 2) > 0 else 1e-6
    return round((m1 - m2) / s_pooled, 3) if s_pooled > 1e-6 else 0.0


def main():
    parser = argparse.ArgumentParser(description="RP-608 Trade-Level Pattern Discovery")
    parser.add_argument("--trades-path", default=str(WORKSPACE_DIR / "artifacts" / "rp608_historical_replay" / "rp608_replay_trades.jsonl"))
    parser.add_argument("--data-dir", default=str(WORKSPACE_DIR / "intraday_capture" / "yahoo_cache_15m"))
    parser.add_argument("--output-dir", default=str(WORKSPACE_DIR / "artifacts" / "rp608_pattern_discovery"))
    args = parser.parse_args()

    out_dir = Path(args.output_dir)
    out_dir.mkdir(parents=True, exist_ok=True)

    print("Loading replay trades...")
    with open(args.trades_path, "r", encoding="utf-8") as f:
        raw_trades = [json.loads(line) for line in f]
    print(f"Loaded {len(raw_trades)} trades.")

    print("Loading 15m OHLCV universe...")
    symbol_bars, sessions = load_ohlcv_universe(Path(args.data_dir))
    print(f"Loaded {len(symbol_bars)} symbols across {len(sessions)} sessions.")

    print("Reconstructing trade features...")
    enriched_trades = compute_features(raw_trades, symbol_bars)

    # Sort strictly by net PnL descending
    enriched_trades.sort(key=lambda x: x["net_pnl_inr"], reverse=True)

    top_100 = enriched_trades[:100]
    bottom_100 = enriched_trades[-100:]  # 100 worst losers

    # Save Top 100 and Bottom 100 files
    with open(out_dir / "top_100_winners.jsonl", "w", encoding="utf-8") as f:
        for t in top_100:
            f.write(json.dumps(t) + "\n")
    with open(out_dir / "bottom_100_losers.jsonl", "w", encoding="utf-8") as f:
        for t in bottom_100:
            f.write(json.dumps(t) + "\n")
    with open(out_dir / "all_1801_enriched_trades.jsonl", "w", encoding="utf-8") as f:
        for t in enriched_trades:
            f.write(json.dumps(t) + "\n")

    # Feature comparison metrics
    feature_keys = [
        # Entry-time
        "traj15",
        "signal_range_bps",
        "signal_body_ratio",
        "signal_upper_wick_ratio",
        "signal_lower_wick_ratio",
        "signal_body_return_bps",
        "signal_vol_surge_ratio",
        "pre_entry_runup_3b_bps",
        "pre_entry_runup_5b_bps",
        "trailing_vol_bps",
        # Early execution
        "bar1_mfe_bps",
        "bar1_mae_bps",
        "bar1_close_ret_bps",
        "bar2_mfe_bps",
        "bar2_mae_bps",
        "bar2_close_ret_bps",
        "bar3_mfe_bps",
        "bar3_mae_bps",
        "bar3_close_ret_bps",
        # Full trade
        "mfe_bps",
        "mae_bps",
        "bars_held",
        "net_pnl_inr",
        "net_return_bps"
    ]

    comparisons = {}
    for k in feature_keys:
        t_vals = [t[k] for t in top_100]
        b_vals = [t[k] for t in bottom_100]
        all_vals = [t[k] for t in enriched_trades]
        comparisons[k] = {
            "feature": k,
            "top_100": calc_stats(t_vals),
            "bottom_100": calc_stats(b_vals),
            "all_1801": calc_stats(all_vals),
            "cohens_d": cohens_d(t_vals, b_vals)
        }

    # Time-of-day / Entry bar breakdown across entire 1,801 trades
    bar_breakdown = {}
    for t in enriched_trades:
        b = t["entry_bar_idx"]
        if b not in bar_breakdown:
            bar_breakdown[b] = {
                "entry_bar": b,
                "trades": 0,
                "net_pnl_inr": 0.0,
                "net_wins": 0,
                "gross_pnl_inr": 0.0,
                "top_100_count": 0,
                "bottom_100_count": 0
            }
        bar_breakdown[b]["trades"] += 1
        bar_breakdown[b]["net_pnl_inr"] += t["net_pnl_inr"]
        bar_breakdown[b]["gross_pnl_inr"] += t["gross_pnl_inr"]
        if t["net_pnl_inr"] > 0:
            bar_breakdown[b]["net_wins"] += 1

    for t in top_100:
        bar_breakdown[t["entry_bar_idx"]]["top_100_count"] += 1
    for t in bottom_100:
        bar_breakdown[t["entry_bar_idx"]]["bottom_100_count"] += 1

    bar_summary = []
    for b in sorted(bar_breakdown.keys()):
        d = bar_breakdown[b]
        n = d["trades"]
        bar_summary.append({
            "entry_bar_idx": b,
            "entry_time_ist": f"{(9 + (b-1)*15//60):02d}:{(15 + (b-1)*15%60)%60:02d}",
            "trades": n,
            "net_pnl_inr": round(d["net_pnl_inr"], 2),
            "win_rate": round(d["net_wins"] / n, 4),
            "avg_pnl_inr": round(d["net_pnl_inr"] / n, 2),
            "top_100_winners": d["top_100_count"],
            "bottom_100_losers": d["bottom_100_count"]
        })

    # Symbol breakdown: top 10 best symbols vs top 10 worst symbols
    sym_breakdown = {}
    for t in enriched_trades:
        s = t["symbol"]
        if s not in sym_breakdown:
            sym_breakdown[s] = {"symbol": s, "trades": 0, "net_pnl_inr": 0.0, "wins": 0}
        sym_breakdown[s]["trades"] += 1
        sym_breakdown[s]["net_pnl_inr"] += t["net_pnl_inr"]
        if t["net_pnl_inr"] > 0:
            sym_breakdown[s]["wins"] += 1

    sym_list = list(sym_breakdown.values())
    sym_list.sort(key=lambda x: x["net_pnl_inr"], reverse=True)
    for s in sym_list:
        s["net_pnl_inr"] = round(s["net_pnl_inr"], 2)
        s["win_rate"] = round(s["wins"] / s["trades"], 4)

    # In-Sample (60%) vs Out-of-Sample (40%) Early MAE filter test
    # Hypothesis test: If Bar 1 MAE > threshold (e.g. 40 bps or 50 bps), does it filter out losers without harming winners?
    split_idx = int(len(sessions) * 0.6)
    is_sessions = set(sessions[:split_idx])
    oos_sessions = set(sessions[split_idx:])

    is_trades = [t for t in enriched_trades if t["session_date"] in is_sessions]
    oos_trades = [t for t in enriched_trades if t["session_date"] in oos_sessions]

    def test_b1_mae_threshold(trades_subset: List[Dict[str, Any]], thresh: float) -> Dict[str, Any]:
        kept = [t for t in trades_subset if t["bar1_mae_bps"] <= thresh]
        dropped = [t for t in trades_subset if t["bar1_mae_bps"] > thresh]
        k_wins = [t for t in kept if t["net_pnl_inr"] > 0]
        d_wins = [t for t in dropped if t["net_pnl_inr"] > 0]
        return {
            "threshold_bps": thresh,
            "total_trades": len(trades_subset),
            "trades_kept": len(kept),
            "trades_dropped": len(dropped),
            "baseline_net_pnl": round(sum(t["net_pnl_inr"] for t in trades_subset), 2),
            "kept_net_pnl_inr": round(sum(t["net_pnl_inr"] for t in kept), 2),
            "dropped_net_pnl_inr": round(sum(t["net_pnl_inr"] for t in dropped), 2),
            "kept_win_rate": round(len(k_wins) / len(kept), 4) if kept else 0.0,
            "dropped_win_rate": round(len(d_wins) / len(dropped), 4) if dropped else 0.0,
            "delta_pnl_inr": round(sum(t["net_pnl_inr"] for t in kept) - sum(t["net_pnl_inr"] for t in trades_subset), 2)
        }

    mae_grid = [20.0, 30.0, 40.0, 50.0, 60.0]
    early_filter_results = {
        "in_sample": [test_b1_mae_threshold(is_trades, th) for th in mae_grid],
        "out_of_sample": [test_b1_mae_threshold(oos_trades, th) for th in mae_grid],
    }

    report_summary = {
        "analysis_id": "RP608_PATTERN_DISCOVERY_AUDIT",
        "total_trades_analyzed": len(enriched_trades),
        "top_100_aggregate_pnl": round(sum(t["net_pnl_inr"] for t in top_100), 2),
        "bottom_100_aggregate_pnl": round(sum(t["net_pnl_inr"] for t in bottom_100), 2),
        "feature_comparisons": comparisons,
        "entry_bar_summary": bar_summary,
        "top_10_symbols": sym_list[:10],
        "bottom_10_symbols": sym_list[-10:],
        "early_mae_filter_validation": early_filter_results
    }

    with open(out_dir / "pattern_discovery_summary.json", "w", encoding="utf-8") as f:
        json.dump(report_summary, f, indent=2)

    print("Pattern discovery complete. Saved summary to", out_dir / "pattern_discovery_summary.json")


if __name__ == "__main__":
    main()
