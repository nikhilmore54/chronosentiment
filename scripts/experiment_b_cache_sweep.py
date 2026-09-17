#!/usr/bin/env python3
"""
experiment_b_cache_sweep.py

Performs a broader cache-universe diagnostic of the 1.25T / 1.00S candidate.
Classifies all available market data into:
- TIME-004 actionable (Buy/Sell)
- TIME-004 counterfactual (Watch/NoTrade)
- TIME-004 unavailable (Cache exists, no decision)

Outputs a comprehensive summary markdown and detailed csv.
"""

import sys
import json
import csv
import math
from pathlib import Path
from datetime import datetime, timezone

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

def simulate_trade(decision: dict, target_mul: float, stop_mul: float) -> dict:
    ticker = decision["ticker"]
    as_of = decision["as_of"]
    as_of_dt = datetime.strptime(as_of, "%Y-%m-%dT%H:%M:%SZ").replace(tzinfo=timezone.utc)
    as_of_ts = as_of_dt.timestamp()
    
    sig = decision.get("signal", decision.get("direction", decision.get("action", "")))
    direction = "LONG" if sig in ["Buy", "LONG"] else "SHORT"
        
    entry_price = float(decision["entry_price"]) if "entry_price" in decision else float(decision.get("reference_price", 0.0))
    base_target = float(decision.get("adaptive_target", 0.0))
    base_risk = float(decision.get("adaptive_risk", 0.0))
    
    target_price = entry_price + (base_target - entry_price) * target_mul if target_mul else base_target
    stop_price = entry_price + (base_risk - entry_price) * stop_mul if stop_mul else base_risk
    
    horizon_bars = get_horizon(decision)
    bars = load_bars(ticker)
    
    post_bars = [b for b in bars if b["timestamp"] > as_of_ts]
    window = post_bars[:horizon_bars]
    
    sim_exit = "HORIZON"
    sim_ret = 0.0
    hit_exit = False
    exit_price = 0.0
    
    for i, b in enumerate(window):
        o, h, l, c = b["open"], b["high"], b["low"], b["close"]
        
        if direction == "LONG":
            if i == 0 and o >= target_price:
                sim_ret = (o - entry_price) / entry_price
                sim_exit = "TARGET_GAP_THROUGH"
                hit_exit = True
                exit_price = o
                break
            if i == 0 and o <= stop_price:
                sim_ret = (o - entry_price) / entry_price
                sim_exit = "RISK_GAP_THROUGH"
                hit_exit = True
                exit_price = o
                break
            if h >= target_price:
                sim_ret = (target_price - entry_price) / entry_price
                sim_exit = "TARGET"
                hit_exit = True
                exit_price = target_price
                break
            if l <= stop_price:
                sim_ret = (stop_price - entry_price) / entry_price
                sim_exit = "RISK"
                hit_exit = True
                exit_price = stop_price
                break
                
        elif direction == "SHORT":
            if i == 0 and o <= target_price:
                sim_ret = (entry_price - o) / entry_price
                sim_exit = "TARGET_GAP_THROUGH"
                hit_exit = True
                exit_price = o
                break
            if i == 0 and o >= stop_price:
                sim_ret = (entry_price - o) / entry_price
                sim_exit = "RISK_GAP_THROUGH"
                hit_exit = True
                exit_price = o
                break
            if l <= target_price:
                sim_ret = (entry_price - target_price) / entry_price
                sim_exit = "TARGET"
                hit_exit = True
                exit_price = target_price
                break
            if h >= stop_price:
                sim_ret = (entry_price - stop_price) / entry_price
                sim_exit = "RISK"
                hit_exit = True
                exit_price = stop_price
                break

    if not hit_exit:
        if window:
            last_close = window[-1]["close"]
            sim_ret = (last_close - entry_price) / entry_price if direction == "LONG" else (entry_price - last_close) / entry_price
            exit_price = last_close
        else:
            sim_ret = 0.0
            
    return {
        "sim_return": sim_ret,
        "sim_exit_reason": sim_exit,
        "exit_price": exit_price,
        "decision": decision,
        "target_price": target_price,
        "stop_price": stop_price,
    }

def main():
    out_dir = Path("/Users/nikhil/.gemini/antigravity-ide/brain/ac8b3565-b88c-4945-8459-7e723a29389a")
    
    # 1. Load all available symbols from cache
    cache_symbols = set()
    if BARS_DIR.exists():
        for f in BARS_DIR.glob("*.json"):
            cache_symbols.add(f.stem.replace(".NS", "_NS"))
            
    # 2. Load all TIME-004 decisions
    decisions = []
    if TIME_MACHINE_DIR.exists():
        for d_dir in (TIME_MACHINE_DIR / "ledger").glob("*"):
            if not d_dir.is_dir(): continue
            for f in (d_dir / "entries").glob("*.json"):
                try:
                    decisions.append(json.loads(f.read_text()))
                except: pass
                
    # 3. Classify symbols
    actionable_cohort = []
    counterfactual_cohort = []
    decision_symbols = set()
    
    for v in decisions:
        tick = v["ticker"].replace(".NS", "_NS")
        decision_symbols.add(tick)
        act = v.get("action", "")
        if act in ["Buy", "Sell"]:
            actionable_cohort.append(v)
        elif act in ["Watch", "NoTrade"]:
            counterfactual_cohort.append(v)
            
    unavailable_symbols = cache_symbols - decision_symbols
    
    # Process Category C (Cache-only)
    c_sufficient = 0
    c_insufficient = 0
    as_of_ts = datetime.strptime("2026-09-11T10:00:00Z", "%Y-%m-%dT%H:%M:%SZ").replace(tzinfo=timezone.utc).timestamp()
    
    for tick in unavailable_symbols:
        bars = load_bars(tick)
        post_bars = [b for b in bars if b["timestamp"] > as_of_ts]
        # arbitrary check for "sufficient" = at least 1 bar
        if len(post_bars) > 0:
            c_sufficient += 1
        else:
            c_insufficient += 1
            
    # Simulate A & B
    def sim_cohort(cohort):
        results = []
        for d in cohort:
            b = simulate_trade(d, 1.0, 1.0)
            c = simulate_trade(d, 1.25, 1.0)
            results.append((b, c))
        return results
        
    actionable_results = sim_cohort(actionable_cohort)
    counterfactual_results = sim_cohort(counterfactual_cohort)
    
    def summarize_results(results):
        n = len(results)
        improved = worsened = unchanged = 0
        deltas = []
        for b, c in results:
            d = c["sim_return"] - b["sim_return"]
            deltas.append(d)
            if d > 1e-6: improved += 1
            elif d < -1e-6: worsened += 1
            else: unchanged += 1
        deltas.sort()
        sum_d = sum(deltas)
        mean_d = sum_d / n if n else 0
        med_d = deltas[n//2] if n else 0
        return {
            "n": n, "improved": improved, "worsened": worsened, "unchanged": unchanged,
            "sum_d": sum_d, "mean_d": mean_d, "med_d": med_d
        }
        
    a_summ = summarize_results(actionable_results)
    b_summ = summarize_results(counterfactual_results)
    
    # Sensitivity Distribution
    sensitivities = []
    for b, c in actionable_results + counterfactual_results:
        ref = b["decision"].get("reference_price", b["decision"].get("entry_price", 0.0))
        btgt = b["target_price"]
        ctgt = c["target_price"]
        bdist = abs(btgt - ref)
        cdist = abs(ctgt - ref)
        if bdist > 0:
            sensitivities.append((cdist - bdist) / bdist * 100)
            
    sens_buckets = {"0-10%": 0, "10-25%": 0, "25-50%": 0, "50%+": 0}
    for s in sensitivities:
        if s <= 10.01: sens_buckets["0-10%"] += 1
        elif s <= 25.01: sens_buckets["10-25%"] += 1
        elif s <= 50.01: sens_buckets["25-50%"] += 1
        else: sens_buckets["50%+"] += 1

    summary_path = out_dir / "experiment_b_summary.md"
    with open(summary_path, "w") as f:
        f.write("# EXPERIMENT B: CACHE-UNIVERSE GEOMETRY SWEEP\n\n")
        f.write("## 1. Required Classification\n")
        f.write(f"- TIME-004 actionable (Buy/Sell): {len(actionable_cohort)}\n")
        f.write(f"- TIME-004 counterfactual (Watch/NoTrade): {len(counterfactual_cohort)}\n")
        f.write(f"- TIME-004 unavailable (Cache exists, no decision): {len(unavailable_symbols)}\n")
        
        f.write("\n## 2. A. Buy/Sell (Actionable)\n")
        f.write(f"- n: {a_summ['n']}\n")
        f.write(f"- unchanged: {a_summ['unchanged']}\n")
        f.write(f"- changed: {a_summ['improved'] + a_summ['worsened']}\n")
        f.write(f"- improved: {a_summ['improved']}\n")
        f.write(f"- worsened: {a_summ['worsened']}\n")
        f.write(f"- mean delta: {a_summ['mean_d']*100:.4f}%\n")
        f.write(f"- median delta: {a_summ['med_d']*100:.4f}%\n")
        f.write(f"- sum delta: {a_summ['sum_d']*100:.4f}%\n")
        
        f.write("\n## 3. B. Watch/NoTrade (Counterfactual)\n")
        f.write(f"- n: {b_summ['n']}\n")
        f.write(f"- unchanged: {b_summ['unchanged']}\n")
        f.write(f"- changed: {b_summ['improved'] + b_summ['worsened']}\n")
        f.write(f"- improved: {b_summ['improved']}\n")
        f.write(f"- worsened: {b_summ['worsened']}\n")
        f.write(f"- mean delta: {b_summ['mean_d']*100:.4f}%\n")
        f.write(f"- median delta: {b_summ['med_d']*100:.4f}%\n")
        f.write(f"- sum delta: {b_summ['sum_d']*100:.4f}%\n")
        
        f.write("\n## 4. C. Cache-only symbols (Informational)\n")
        f.write(f"- n symbols: {len(unavailable_symbols)}\n")
        f.write(f"- n with sufficient forward bars: {c_sufficient}\n")
        f.write(f"- n without sufficient forward bars: {c_insufficient}\n")
        
        f.write("\n## 5. Geometry Sensitivity Distribution\n")
        f.write("Ratio of (Candidate Target Distance - Baseline Target Distance) / Baseline Target Distance\n")
        for k, v in sens_buckets.items():
            f.write(f"- {k}: {v}\n")

    print(f"Experiment B successfully generated at {out_dir}")

if __name__ == "__main__":
    main()
