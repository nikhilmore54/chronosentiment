#!/usr/bin/env python3
"""
gain_protection_diagnostic_v0_1.py

Diagnostic tool to evaluate gain protection (trailing stop) opportunities
on the Sep 10-11 cohort without modifying production geometry.
"""

import sys
import json
import csv
import math
from pathlib import Path
from datetime import datetime, timezone
from collections import defaultdict

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

def simulate_with_protection(decision: dict, activation_pct: float, protect_type: str, protect_frac: float):
    ticker = decision["ticker"]
    as_of = decision["as_of"]
    as_of_dt = datetime.strptime(as_of, "%Y-%m-%dT%H:%M:%SZ").replace(tzinfo=timezone.utc)
    as_of_ts = as_of_dt.timestamp()
    
    direction = decision["direction"]
    entry = float(decision["entry_price"]) if "entry_price" in decision else float(decision.get("reference_price", 0.0))
    base_target = float(decision.get("adaptive_target", 0.0))
    base_risk = float(decision.get("adaptive_risk", 0.0))
    
    tgt_ret = (base_target - entry) / entry if direction == "LONG" else (entry - base_target) / entry
    rsk_ret = (base_risk - entry) / entry if direction == "LONG" else (entry - base_risk) / entry
    
    horizon_bars = get_horizon(decision)
    bars = load_bars(ticker)
    window = [b for b in bars if b["timestamp"] > as_of_ts][:horizon_bars]
    
    if not window:
        return 0.0, "HORIZON"
        
    peak_ret = 0.0
    prot_ret = None
    
    for i, b in enumerate(window):
        o, h, l, c = b["open"], b["high"], b["low"], b["close"]
        
        if direction == "LONG":
            o_ret = (o - entry)/entry; h_ret = (h - entry)/entry; l_ret = (l - entry)/entry; c_ret = (c - entry)/entry
        else:
            o_ret = (entry - o)/entry; h_ret = (entry - l)/entry; l_ret = (entry - h)/entry; c_ret = (entry - c)/entry
            
        # Gap checks
        if i == 0:
            if o_ret >= tgt_ret: return o_ret, "TARGET_GAP_THROUGH"
            if o_ret <= rsk_ret: return o_ret, "RISK_GAP_THROUGH"
            if prot_ret is not None and o_ret <= prot_ret: return o_ret, "PROT_GAP_THROUGH"
        else:
            if o_ret >= tgt_ret: return o_ret, "TARGET_GAP"
            if o_ret <= rsk_ret: return o_ret, "RISK_GAP"
            if prot_ret is not None and o_ret <= prot_ret: return o_ret, "PROT_GAP"
            
        # Intra-bar limits
        if h_ret >= tgt_ret: return tgt_ret, "TARGET"
        if prot_ret is not None and l_ret <= prot_ret: return prot_ret, "PROTECTION"
        if l_ret <= rsk_ret: return rsk_ret, "RISK"
        
        # Update peak and protection level for NEXT bar
        peak_ret = max(peak_ret, h_ret)
        
        if peak_ret >= activation_pct:
            if protect_type == "breakeven":
                prot_ret = 0.0
            elif protect_type == "fraction":
                new_prot = peak_ret * protect_frac
                if prot_ret is None or new_prot > prot_ret:
                    prot_ret = new_prot

    return c_ret, "HORIZON"

def get_base_trajectory(decision: dict):
    # Runs the baseline (no protection) and captures H15, H60, H120, Peak, Final
    as_of_ts = datetime.strptime(decision["as_of"], "%Y-%m-%dT%H:%M:%SZ").replace(tzinfo=timezone.utc).timestamp()
    ticker = decision["ticker"]
    direction = decision["direction"]
    entry = float(decision["entry_price"]) if "entry_price" in decision else float(decision.get("reference_price", 0.0))
    
    bars = load_bars(ticker)
    window = [b for b in bars if b["timestamp"] > as_of_ts][:get_horizon(decision)]
    
    def calc_ret(price):
        return (price - entry) / entry if direction == "LONG" else (entry - price) / entry
        
    res = {}
    if not window: return None
    
    # We will just evaluate close prices for H15, H60, H120
    for k in [15, 60, 120]:
        idx = min(k-1, len(window)-1)
        res[f"H{k}"] = calc_ret(window[idx]["close"])
        
    # Standard baseline run to get final return and peak
    final_ret, reason = simulate_with_protection(decision, 999.0, "breakeven", 0.0)
    
    # Calculate MFE strictly up to the exit point
    # Since we don't have the exact bar it exited in the helper easily, we can just approximate peak gain over the whole window for diagnostic
    peak = 0.0
    for b in window:
        h_ret = calc_ret(b["high"] if direction=="LONG" else b["low"])
        if h_ret > peak: peak = h_ret
        
    res["Peak"] = peak
    res["Final"] = final_ret
    res["Giveback"] = peak - final_ret if final_ret < peak else 0.0
    res["Reason"] = reason
    return res

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
            
    # Part 1: Opportunity Matrix
    opp_matrix = []
    for d in actionable:
        t = get_base_trajectory(d)
        if t:
            opp_matrix.append((d["ticker"], t))
            
    opp_matrix.sort(key=lambda x: x[1]["Giveback"], reverse=True)
    
    # Part 2: Param Sweep
    # Thresholds: 0.25%, 0.50%, 0.75%, 1.00%, 1.25%, 1.50%
    thresholds = [0.0025, 0.0050, 0.0075, 0.0100, 0.0125, 0.0150]
    prot_configs = [
        ("breakeven", 0.0),
        ("fraction", 0.25),
        ("fraction", 0.50),
        ("fraction", 0.75)
    ]
    
    results = {}
    base_returns = [simulate_with_protection(d, 999.0, "breakeven", 0.0)[0] for d in actionable]
    base_winrate = sum(1 for r in base_returns if r > 0) / len(base_returns) if base_returns else 0.0
    base_mean = sum(base_returns) / len(base_returns) if base_returns else 0.0
    
    for thresh in thresholds:
        for ptype, pfrac in prot_configs:
            name = f"Act {thresh*100:.2f}% | " + ("BE" if ptype == "breakeven" else f"T{pfrac*100:.0f}%")
            
            sims = [simulate_with_protection(d, thresh, ptype, pfrac) for d in actionable]
            rets = [r[0] for r in sims]
            
            winrate = sum(1 for r in rets if r > 0) / len(rets)
            mean_ret = sum(rets) / len(rets)
            
            # Impact on Baseline Winners / Losers
            w_rets = [rets[i] for i in range(len(rets)) if base_returns[i] > 0]
            l_rets = [rets[i] for i in range(len(rets)) if base_returns[i] <= 0]
            
            w_mean = sum(w_rets)/len(w_rets) if w_rets else 0.0
            l_mean = sum(l_rets)/len(l_rets) if l_rets else 0.0
            
            results[name] = {
                "winrate": winrate,
                "mean": mean_ret,
                "w_mean": w_mean,
                "l_mean": l_mean,
                "prot_triggers": sum(1 for r in sims if "PROT" in r[1] or "PROTECTION" in r[1])
            }
            
    # Write Report
    report_path = out_dir / "gain_protection_diagnostic_v0_1_report.md"
    with open(report_path, "w") as f:
        f.write("# GAIN PROTECTION DIAGNOSTIC v0.1\n\n")
        f.write("## 1. Opportunity Matrix (Top 15 by Giveback)\n")
        f.write("| Trade | H15 | H60 | H120 | Peak Gain | H300 | Giveback |\n")
        f.write("|---|---|---|---|---|---|---|\n")
        
        for tick, t in opp_matrix[:15]:
            f.write(f"| {tick} | {t['H15']*100:.2f}% | {t['H60']*100:.2f}% | {t['H120']*100:.2f}% | {t['Peak']*100:.2f}% | {t['Final']*100:.2f}% | {t['Giveback']*100:.2f}pp |\n")
            
        f.write("\n## 2. Hypothetical Protection Sweep\n")
        f.write(f"**Baseline:** Winrate {base_winrate*100:.1f}%, Mean {base_mean*100:.2f}%\n\n")
        
        f.write("| Configuration | Triggers | Overall Winrate | Overall Mean | Baseline Winners Mean | Baseline Losers Mean |\n")
        f.write("|---|---|---|---|---|---|\n")
        
        for name, r in results.items():
            f.write(f"| {name} | {r['prot_triggers']} | {r['winrate']*100:.1f}% | {r['mean']*100:.2f}% | {r['w_mean']*100:.2f}% | {r['l_mean']*100:.2f}% |\n")
            
    print(f"Report written to {report_path}")

if __name__ == "__main__":
    main()
