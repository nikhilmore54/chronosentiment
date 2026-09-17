#!/usr/bin/env python3
"""
portfolio_exposure_diagnostic.py

Evaluates the portfolio-level exposure, concurrency, and drawdown 
for the 35 actionable trades from the Sep 10-11 cohort.
"""

import sys
import json
import math
from pathlib import Path
from datetime import datetime, timezone, timedelta
from collections import defaultdict, Counter

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
    return 300

def get_trade_trajectory(decision: dict):
    ticker = decision["ticker"]
    as_of = decision["as_of"]
    as_of_dt = datetime.strptime(as_of, "%Y-%m-%dT%H:%M:%SZ").replace(tzinfo=timezone.utc)
    as_of_ts = as_of_dt.timestamp()
    
    direction = decision["direction"]
    entry = float(decision["entry_price"]) if "entry_price" in decision else float(decision.get("reference_price", 0.0))
    base_target = float(decision.get("adaptive_target", 0.0))
    base_risk = float(decision.get("adaptive_risk", 0.0))
    
    horizon_bars = get_horizon(decision)
    bars = load_bars(ticker)
    
    post_bars = [b for b in bars if b["timestamp"] > as_of_ts]
    window = post_bars[:horizon_bars]
    
    if not window:
        return None
        
    trajectory = {}
    hit_exit = False
    
    tgt_ret = (base_target - entry) / entry if direction == "LONG" else (entry - base_target) / entry
    rsk_ret = (base_risk - entry) / entry if direction == "LONG" else (entry - base_risk) / entry
    
    for i, b in enumerate(window):
        ts = b["timestamp"]
        o, h, l, c = b["open"], b["high"], b["low"], b["close"]
        
        if direction == "LONG":
            o_ret = (o - entry)/entry; h_ret = (h - entry)/entry; l_ret = (l - entry)/entry; c_ret = (c - entry)/entry
        else:
            o_ret = (entry - o)/entry; h_ret = (entry - l)/entry; l_ret = (entry - h)/entry; c_ret = (entry - c)/entry
            
        # Gap checks
        if i == 0:
            if o_ret >= tgt_ret: 
                trajectory[ts] = o_ret
                break
            if o_ret <= rsk_ret:
                trajectory[ts] = o_ret
                break
        else:
            if o_ret >= tgt_ret:
                trajectory[ts] = o_ret
                break
            if o_ret <= rsk_ret:
                trajectory[ts] = o_ret
                break
                
        # Intra-bar
        if h_ret >= tgt_ret:
            trajectory[ts] = tgt_ret
            break
        if l_ret <= rsk_ret:
            trajectory[ts] = rsk_ret
            break
            
        trajectory[ts] = c_ret

    return {
        "ticker": ticker,
        "direction": direction,
        "entry_ts": as_of_ts,
        "exit_ts": window[-1]["timestamp"] if window else as_of_ts,
        "trajectory": trajectory
    }

def main():
    out_dir = Path("/Users/nikhil/.gemini/antigravity-ide/brain/ac8b3565-b88c-4945-8459-7e723a29389a")
    
    decisions = []
    if TIME_MACHINE_DIR.exists():
        for d_dir in (TIME_MACHINE_DIR / "ledger").glob("*"):
            if not d_dir.is_dir(): continue
            for f in (d_dir / "entries").glob("*.json"):
                try: decisions.append(json.loads(f.read_text()))
                except: pass
                
    actionable = []
    for v in decisions:
        act = v.get("action", "")
        if act in ["Buy", "Sell"]:
            sig = v.get("signal", v.get("direction", act))
            v["direction"] = "LONG" if sig in ["Buy", "LONG"] else "SHORT"
            actionable.append(v)
            
    trades = []
    for d in actionable:
        t = get_trade_trajectory(d)
        if t: trades.append(t)
        
    # Build minute-by-minute portfolio state
    timeline = defaultdict(lambda: {"longs": 0, "shorts": 0, "unrealized": 0.0, "adverse": 0, "tickers": []})
    
    for t in trades:
        for ts, ret in t["trajectory"].items():
            state = timeline[ts]
            if t["direction"] == "LONG":
                state["longs"] += 1
            else:
                state["shorts"] += 1
                
            state["unrealized"] += ret
            if ret < 0:
                state["adverse"] += 1
            state["tickers"].append(t["ticker"])
            
    sorted_ts = sorted(timeline.keys())
    
    # Calculate portfolio metrics
    max_concurrent = 0
    max_longs = 0
    max_shorts = 0
    max_adverse = 0
    worst_unrealized = 0.0
    
    for ts in sorted_ts:
        s = timeline[ts]
        concurrent = s["longs"] + s["shorts"]
        if concurrent > max_concurrent: max_concurrent = concurrent
        if s["longs"] > max_longs: max_longs = s["longs"]
        if s["shorts"] > max_shorts: max_shorts = s["shorts"]
        if s["adverse"] > max_adverse: max_adverse = s["adverse"]
        if s["unrealized"] < worst_unrealized: worst_unrealized = s["unrealized"]
        
    # Repeated names
    tickers = [t["ticker"] for t in trades]
    ticker_counts = Counter(tickers)
    repeats = {k: v for k, v in ticker_counts.items() if v > 1}
    
    # Generate Report
    report_path = out_dir / "portfolio_exposure_diagnostic.md"
    with open(report_path, "w") as f:
        f.write("# PORTFOLIO EXPOSURE DIAGNOSTIC (Sep 10-11 Cohort)\n\n")
        f.write("## 1. Concurrent Exposure & Drawdown\n")
        f.write(f"- Total Actionable Trades: {len(trades)}\n")
        f.write(f"- Maximum Concurrent Positions: {max_concurrent}\n")
        f.write(f"- Maximum LONG Positions: {max_longs}\n")
        f.write(f"- Maximum SHORT Positions: {max_shorts}\n")
        f.write(f"- Maximum Simultaneous Adverse Positions: {max_adverse}\n")
        f.write(f"- Worst Simultaneous Unrealized P&L (Sum of %): {worst_unrealized*100:.2f}%\n")
        
        f.write("\n## 2. Repeated Tickers (Sep 10 & 11)\n")
        if repeats:
            for k, v in repeats.items():
                f.write(f"- {k}: {v} trades\n")
        else:
            f.write("- None\n")
            
        f.write("\n## 3. Time-Synchronized Exposure Table (Sampled Every 30 Mins)\n")
        f.write("| Timestamp (UTC) | Concurrent | LONGs | SHORTs | Unrealized P&L | Adverse Positions |\n")
        f.write("|---|---|---|---|---|---|\n")
        
        # Sample every 30 minutes for readability
        last_sampled = None
        for ts in sorted_ts:
            dt = datetime.fromtimestamp(ts, timezone.utc)
            if last_sampled is None or (ts - last_sampled) >= 1800:
                s = timeline[ts]
                f.write(f"| {dt.strftime('%Y-%m-%d %H:%M')} | {s['longs']+s['shorts']} | {s['longs']} | {s['shorts']} | {s['unrealized']*100:.2f}% | {s['adverse']} |\n")
                last_sampled = ts
                
    print(f"Diagnostic written to {report_path}")

if __name__ == "__main__":
    main()
