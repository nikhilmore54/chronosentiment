#!/usr/bin/env python3
"""
forensic_losing_trades.py

Examines every losing actionable trade in the Time Machine cohort (Sep 10-11).
Generates a detailed forensic report analyzing MFE, MAE, trajectory, and clustering.
"""

import sys
import json
import csv
import math
from pathlib import Path
from datetime import datetime, timezone
from collections import Counter

WORKSPACE_DIR = Path(__file__).parent.parent
TIME_MACHINE_DIR = WORKSPACE_DIR / "time_machine"
BARS_DIR = WORKSPACE_DIR / "intraday_capture" / "yahoo_cache_1m"

_bar_cache = {}
def load_bars(ticker_ns: str) -> list[dict]:
    if ticker_ns in _bar_cache:
        return _bar_cache[ticker_ns]
    p = BARS_DIR / (ticker_ns.replace("_NS", ".NS") + ".json")
    if not p.exists():
        _bar_cache[ticker_ns] = []
        return []
    with open(p) as f:
        bars = [b for b in json.load(f) if b.get("volume", 0) > 0]
    bars.sort(key=lambda b: b["timestamp"])
    _bar_cache[ticker_ns] = bars
    return bars

def get_horizon(decision: dict) -> int:
    h = decision.get("adaptive_horizon_sessions")
    if h is None:
        return 20
    val = math.ceil(float(h))
    return max(1, val)

def evaluate_trade_trajectory(decision: dict):
    ticker = decision["ticker"]
    as_of = decision["as_of"]
    as_of_dt = datetime.strptime(as_of, "%Y-%m-%dT%H:%M:%SZ").replace(tzinfo=timezone.utc)
    as_of_ts = as_of_dt.timestamp()
    
    direction = decision["direction"]
    entry_price = float(decision["entry_price"]) if "entry_price" in decision else float(decision.get("reference_price", 0.0))
    base_target = float(decision.get("adaptive_target", 0.0))
    base_risk = float(decision.get("adaptive_risk", 0.0))
    
    horizon_bars = get_horizon(decision)
    bars = load_bars(ticker)
    
    post_bars = [b for b in bars if b["timestamp"] > as_of_ts]
    window = post_bars[:horizon_bars]
    
    if not window:
        return None
        
    sim_exit = "HORIZON"
    sim_ret = 0.0
    hit_exit = False
    
    mfe = -999.0
    mae = 999.0
    mfe_idx = 0
    mae_idx = 0
    
    for i, b in enumerate(window):
        o, h, l, c = b["open"], b["high"], b["low"], b["close"]
        
        if direction == "LONG":
            cur_mfe = (h - entry_price) / entry_price
            cur_mae = (l - entry_price) / entry_price
        else:
            cur_mfe = (entry_price - l) / entry_price
            cur_mae = (entry_price - h) / entry_price
            
        if cur_mfe > mfe:
            mfe = cur_mfe
            mfe_idx = i
        if cur_mae < mae:
            mae = cur_mae
            mae_idx = i
            
        if not hit_exit:
            if direction == "LONG":
                if i == 0 and o >= base_target:
                    sim_ret = (o - entry_price) / entry_price
                    sim_exit = "TARGET_GAP_THROUGH"
                    hit_exit = True
                elif i == 0 and o <= base_risk:
                    sim_ret = (o - entry_price) / entry_price
                    sim_exit = "RISK_GAP_THROUGH"
                    hit_exit = True
                elif h >= base_target:
                    sim_ret = (base_target - entry_price) / entry_price
                    sim_exit = "TARGET"
                    hit_exit = True
                elif l <= base_risk:
                    sim_ret = (base_risk - entry_price) / entry_price
                    sim_exit = "RISK"
                    hit_exit = True
            else:
                if i == 0 and o <= base_target:
                    sim_ret = (entry_price - o) / entry_price
                    sim_exit = "TARGET_GAP_THROUGH"
                    hit_exit = True
                elif i == 0 and o >= base_risk:
                    sim_ret = (entry_price - o) / entry_price
                    sim_exit = "RISK_GAP_THROUGH"
                    hit_exit = True
                elif l <= base_target:
                    sim_ret = (entry_price - base_target) / entry_price
                    sim_exit = "TARGET"
                    hit_exit = True
                elif h >= base_risk:
                    sim_ret = (entry_price - base_risk) / entry_price
                    sim_exit = "RISK"
                    hit_exit = True

    if not hit_exit:
        last_close = window[-1]["close"]
        sim_ret = (last_close - entry_price) / entry_price if direction == "LONG" else (entry_price - last_close) / entry_price
        
    # Classify Trajectory
    # Immediate adverse: MAE occurs very early, MFE is negligible
    # Deterioration: MFE was decent and occurred before MAE, but trade drifted into loss
    if mfe < 0.005 and mae_idx < len(window)*0.2:
        traj = "IMMEDIATE_ADVERSE"
    elif mfe > 0.01 and mfe_idx < mae_idx:
        traj = "DETERIORATION_FROM_WIN"
    elif mae_idx > len(window)*0.5 and mfe < 0.01:
        traj = "SLOW_BLEED"
    else:
        traj = "CHOPPY_NO_DIRECTION"
        
    return {
        "return": sim_ret,
        "exit_reason": sim_exit,
        "mfe": mfe,
        "mae": mae,
        "mfe_idx": mfe_idx,
        "mae_idx": mae_idx,
        "trajectory": traj,
        "bars_held": len(window),
    }

def main():
    out_dir = Path("/Users/nikhil/.gemini/antigravity-ide/brain/ac8b3565-b88c-4945-8459-7e723a29389a")
    
    decisions = []
    if TIME_MACHINE_DIR.exists():
        for d_dir in (TIME_MACHINE_DIR / "ledger").glob("*"):
            if not d_dir.is_dir(): continue
            for f in (d_dir / "entries").glob("*.json"):
                try:
                    decisions.append(json.loads(f.read_text()))
                except: pass
                
    # Filter actionable (Buy/Sell)
    actionable = []
    for v in decisions:
        act = v.get("action", "")
        if act in ["Buy", "Sell"]:
            sig = v.get("signal", v.get("direction", act))
            v["direction"] = "LONG" if sig in ["Buy", "LONG"] else "SHORT"
            actionable.append(v)
            
    losers = []
    for d in actionable:
        res = evaluate_trade_trajectory(d)
        if res and res["return"] < 0:
            d["sim_res"] = res
            losers.append(d)
            
    losers.sort(key=lambda x: x["sim_res"]["return"])
    
    # Clustering stats
    tickers = Counter([d["ticker"] for d in losers])
    ev_classes = Counter([d.get("evidence_class", "Unknown") for d in losers])
    trajectories = Counter([d["sim_res"]["trajectory"] for d in losers])
    trends = Counter([d.get("trend", "Unknown") for d in losers])
    
    summary_path = out_dir / "losing_trades_forensic_report.md"
    with open(summary_path, "w") as f:
        f.write("# FORENSIC ANALYSIS: ACTIONABLE LOSING TRADES (Sep 10-11 Cohort)\n\n")
        f.write(f"**Total Actionable Trades:** {len(actionable)}\n")
        f.write(f"**Total Losing Trades:** {len(losers)}\n\n")
        
        f.write("## 1. Clustering Analysis\n")
        f.write("### By Trajectory Type\n")
        for k, v in trajectories.most_common():
            f.write(f"- {k}: {v}\n")
            
        f.write("\n### By Evidence Class\n")
        for k, v in ev_classes.most_common():
            f.write(f"- {k}: {v}\n")
            
        f.write("\n### By Trend Context\n")
        for k, v in trends.most_common():
            f.write(f"- {k}: {v}\n")
            
        f.write("\n### By Ticker (Top 10)\n")
        for k, v in tickers.most_common(10):
            f.write(f"- {k}: {v}\n")
            
        f.write("\n## 2. Trade-by-Trade Breakdown\n")
        f.write("| Ticker | As Of | Dir | Ev Class | Rank | Entry | MFE | MAE | Return | Exit Reason | Trajectory |\n")
        f.write("|---|---|---|---|---|---|---|---|---|---|---|\n")
        
        for d in losers:
            r = d["sim_res"]
            tick = d["ticker"]
            as_of = d["as_of"]
            dire = d["direction"]
            ev = d.get("evidence_class", "-")
            rank = d.get("rank_score", 0.0)
            entry = d.get("entry_price", d.get("reference_price", 0.0))
            
            f.write(f"| {tick} | {as_of} | {dire} | {ev} | {rank:.3f} | {entry:.2f} | {r['mfe']*100:.2f}% | {r['mae']*100:.2f}% | {r['return']*100:.2f}% | {r['exit_reason']} | {r['trajectory']} |\n")

    print(f"Report generated at {summary_path}")

if __name__ == "__main__":
    main()
