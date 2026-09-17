#!/usr/bin/env python3
"""
h300_time_profile.py

Evaluates the time profile of actionable trades from the Sep 10-11 cohort
at specific bar intervals (H15, H30, H60, H120, H180, H240, H300).
"""

import sys
import json
import csv
import math
from pathlib import Path
from datetime import datetime, timezone
import statistics

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

def evaluate_time_profile(decision: dict):
    ticker = decision["ticker"]
    as_of = decision["as_of"]
    as_of_dt = datetime.strptime(as_of, "%Y-%m-%dT%H:%M:%SZ").replace(tzinfo=timezone.utc)
    as_of_ts = as_of_dt.timestamp()
    
    direction = decision["direction"]
    entry_price = float(decision["entry_price"]) if "entry_price" in decision else float(decision.get("reference_price", 0.0))
    
    bars = load_bars(ticker)
    post_bars = [b for b in bars if b["timestamp"] > as_of_ts]
    
    # We want returns at exactly 15, 30, 60, 120, 180, 240, 300 bars
    intervals = [15, 30, 60, 120, 180, 240, 300]
    returns = {k: None for k in intervals}
    
    if not post_bars:
        return None
        
    for k in intervals:
        idx = k - 1
        if idx < len(post_bars):
            b = post_bars[idx]
            c = b["close"]
            ret = (c - entry_price) / entry_price if direction == "LONG" else (entry_price - c) / entry_price
            returns[k] = ret
        else:
            # If we don't have enough bars, just take the last available close
            b = post_bars[-1]
            c = b["close"]
            ret = (c - entry_price) / entry_price if direction == "LONG" else (entry_price - c) / entry_price
            returns[k] = ret
            
    # Also calculate H300 max MFE and min MAE
    window = post_bars[:300]
    mfe = -999.0
    mae = 999.0
    
    for b in window:
        h, l = b["high"], b["low"]
        if direction == "LONG":
            cur_mfe = (h - entry_price) / entry_price
            cur_mae = (l - entry_price) / entry_price
        else:
            cur_mfe = (entry_price - l) / entry_price
            cur_mae = (entry_price - h) / entry_price
        if cur_mfe > mfe: mfe = cur_mfe
        if cur_mae < mae: mae = cur_mae
            
    final_ret = returns[300]
    
    # Classification logic
    if final_ret > 0:
        win_loser = "WINNER"
    else:
        win_loser = "LOSER"
        
    # Basic trajectory classification
    if returns[15] > 0 and returns[60] > 0 and final_ret > 0:
        prof = "Early winner"
    elif returns[15] < 0 and returns[60] < 0 and final_ret < 0:
        prof = "Persistent adverse"
    elif returns[120] < 0 and final_ret > 0:
        prof = "Late winner"
    elif returns[60] > 0 and final_ret < 0:
        prof = "Late deterioration"
    elif returns[15] < -0.005 and final_ret < 0:
        prof = "Early loser"
    else:
        prof = "Choppy"
        
    return {
        "ticker": ticker,
        "direction": direction,
        "win_loser": win_loser,
        "profile": prof,
        "returns": returns,
        "mfe": mfe,
        "mae": mae,
        "final_ret": final_ret
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
                
    actionable = []
    for v in decisions:
        act = v.get("action", "")
        if act in ["Buy", "Sell"]:
            sig = v.get("signal", v.get("direction", act))
            v["direction"] = "LONG" if sig in ["Buy", "LONG"] else "SHORT"
            actionable.append(v)
            
    profiles = []
    for d in actionable:
        p = evaluate_time_profile(d)
        if p:
            profiles.append(p)
            
    winners = [p for p in profiles if p["win_loser"] == "WINNER"]
    losers = [p for p in profiles if p["win_loser"] == "LOSER"]
    
    def avg_ret_at(cohort, k):
        if not cohort: return 0.0
        return sum(p["returns"][k] for p in cohort) / len(cohort)
        
    intervals = [15, 30, 60, 120, 180, 240, 300]
    
    summary_path = out_dir / "h300_time_profile_report.md"
    with open(summary_path, "w") as f:
        f.write("# H300 TIME-PROFILE DIAGNOSTIC (Actionable Cohort)\n\n")
        f.write(f"Total Actionable Trades: {len(profiles)}\n")
        f.write(f"Winners (at H300): {len(winners)}\n")
        f.write(f"Losers (at H300): {len(losers)}\n\n")
        
        f.write("## 1. Mean Return by Milestone (Winners vs Losers)\n")
        f.write("| Milestone | Winners Mean | Losers Mean | Overall Mean |\n")
        f.write("|---|---|---|---|\n")
        
        for k in intervals:
            w_avg = avg_ret_at(winners, k)
            l_avg = avg_ret_at(losers, k)
            o_avg = avg_ret_at(profiles, k)
            f.write(f"| H{k} | {w_avg*100:.2f}% | {l_avg*100:.2f}% | {o_avg*100:.2f}% |\n")
            
        f.write("\n## 2. Profile Classification\n")
        f.write("| Profile | Count | % of Cohort |\n")
        f.write("|---|---|---|\n")
        
        counts = {}
        for p in profiles:
            c = p["profile"]
            counts[c] = counts.get(c, 0) + 1
            
        for c, count in sorted(counts.items(), key=lambda x: x[1], reverse=True):
            f.write(f"| {c} | {count} | {count/len(profiles)*100:.1f}% |\n")
            
        f.write("\n## 3. Individual Trade breakdown\n")
        f.write("| Ticker | Dir | Outcome | Profile | H15 | H60 | H120 | H300 | MFE | MAE |\n")
        f.write("|---|---|---|---|---|---|---|---|---|---|\n")
        
        for p in sorted(profiles, key=lambda x: x["final_ret"], reverse=True):
            r = p["returns"]
            f.write(f"| {p['ticker']} | {p['direction']} | {p['win_loser']} | {p['profile']} | {r[15]*100:.2f}% | {r[60]*100:.2f}% | {r[120]*100:.2f}% | {r[300]*100:.2f}% | {p['mfe']*100:.2f}% | {p['mae']*100:.2f}% |\n")

    print(f"Report generated at {summary_path}")

if __name__ == "__main__":
    main()
