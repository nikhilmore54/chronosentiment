#!/usr/bin/env python3
"""
candidate_validation_time_machine_v0_1.py

Validates the 1.25T / 1.00S geometry candidate using the time_machine pipeline outputs.
Generates comprehensive reports including all-symbols audit and transition matrices.

Usage:
    python3 candidate_validation_time_machine_v0_1.py --mode validation --out-dir <dir>
"""

import sys
import json
import csv
import math
import argparse
import statistics
from pathlib import Path
from datetime import datetime, timezone

# ---------------------------------------------------------------------------
# Configuration
# ---------------------------------------------------------------------------
WORKSPACE_DIR = Path(__file__).parent.parent
TIME_MACHINE_DIR = WORKSPACE_DIR / "time_machine"
BARS_DIR = WORKSPACE_DIR / "intraday_capture" / "yahoo_cache_1m"

NOTIONAL = 1_000_000  # ₹
MAX_CONCURRENT = 10   # Maximum simultaneous positions
COST_SCENARIOS = {
    "Zero-cost baseline": 0,
    "Low": 5,
    "Moderate": 10,
    "Conservative": 20,
    "Stress": 30,
}

# ---------------------------------------------------------------------------
# Loaders
# ---------------------------------------------------------------------------
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

# ---------------------------------------------------------------------------
# Simulation Core
# ---------------------------------------------------------------------------
def simulate_trade(decision: dict, target_mul: float, stop_mul: float) -> dict:
    ticker = decision["ticker"]
    as_of = decision["as_of"]
    as_of_dt = datetime.strptime(as_of, "%Y-%m-%dT%H:%M:%SZ").replace(tzinfo=timezone.utc)
    as_of_ts = as_of_dt.timestamp()
    
    # Force LONG/SHORT mapping even for Watch/NoTrade
    sig = decision.get("signal", decision.get("direction", decision.get("action", "")))
    direction = "LONG" if sig in ["Buy", "LONG"] else "SHORT"
    if "direction" not in decision: decision["direction"] = direction
        
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
    exit_ts = 0
    
    for i, b in enumerate(window):
        o, h, l, c = b["open"], b["high"], b["low"], b["close"]
        ts = b["timestamp"]
        
        if direction == "LONG":
            if i == 0 and o >= target_price:
                sim_ret = (o - entry_price) / entry_price
                sim_exit = "TARGET_GAP_THROUGH"
                hit_exit = True
                exit_price = o
                exit_ts = ts
                break
            if i == 0 and o <= stop_price:
                sim_ret = (o - entry_price) / entry_price
                sim_exit = "RISK_GAP_THROUGH"
                hit_exit = True
                exit_price = o
                exit_ts = ts
                break
            if h >= target_price:
                sim_ret = (target_price - entry_price) / entry_price
                sim_exit = "TARGET"
                hit_exit = True
                exit_price = target_price
                exit_ts = ts
                break
            if l <= stop_price:
                sim_ret = (stop_price - entry_price) / entry_price
                sim_exit = "RISK"
                hit_exit = True
                exit_price = stop_price
                exit_ts = ts
                break
                
        elif direction == "SHORT":
            if i == 0 and o <= target_price:
                sim_ret = (entry_price - o) / entry_price
                sim_exit = "TARGET_GAP_THROUGH"
                hit_exit = True
                exit_price = o
                exit_ts = ts
                break
            if i == 0 and o >= stop_price:
                sim_ret = (entry_price - o) / entry_price
                sim_exit = "RISK_GAP_THROUGH"
                hit_exit = True
                exit_price = o
                exit_ts = ts
                break
            if l <= target_price:
                sim_ret = (entry_price - target_price) / entry_price
                sim_exit = "TARGET"
                hit_exit = True
                exit_price = target_price
                exit_ts = ts
                break
            if h >= stop_price:
                sim_ret = (entry_price - stop_price) / entry_price
                sim_exit = "RISK"
                hit_exit = True
                exit_price = stop_price
                exit_ts = ts
                break

    if not hit_exit:
        if window:
            last_close = window[-1]["close"]
            sim_ret = (last_close - entry_price) / entry_price if direction == "LONG" else (entry_price - last_close) / entry_price
            exit_price = last_close
            exit_ts = window[-1]["timestamp"]
        else:
            sim_ret = 0.0
            
    return {
        "sim_return": sim_ret,
        "sim_exit_reason": sim_exit,
        "exit_price": exit_price,
        "exit_ts": exit_ts,
        "decision": decision,
        "target_price": target_price,
        "stop_price": stop_price,
    }

# ---------------------------------------------------------------------------
# Main Routine
# ---------------------------------------------------------------------------
def main():
    parser = argparse.ArgumentParser()
    parser.add_argument("--mode", default="validation")
    parser.add_argument("--out-dir", default=".")
    args = parser.parse_args()
    out_dir = Path(args.out_dir)
    
    print("Loading TIME-004 to TIME-006 Data...")
    
    decisions = []
    observations = {}
    
    if not TIME_MACHINE_DIR.exists():
        print("NO UNSEEN WINDOW AVAILABLE")
        sys.exit(0)
        
    for d_dir in (TIME_MACHINE_DIR / "ledger").glob("*"):
        if not d_dir.is_dir(): continue
        for f in (d_dir / "entries").glob("*.json"):
            try:
                decisions.append(json.loads(f.read_text()))
            except: pass
            
    for d_dir in (TIME_MACHINE_DIR / "observations").glob("*"):
        if not d_dir.is_dir(): continue
        for f in d_dir.glob("*.json"):
            try:
                data = json.loads(f.read_text())
                if "decision_id" in data:
                    observations[data["decision_id"]] = data
            except: pass
            
    d_count = len(decisions)
    o_count = len(observations)
    j_count = len(set(d["decision_id"] for d in decisions).intersection(set(observations.keys())))
    
    print(f"TIME-004 decisions       {d_count}")
    print(f"TIME-005 observations   {o_count}")
    print(f"TIME-006 joins          {j_count}")
    
    if d_count == 0:
        print("NO UNSEEN WINDOW AVAILABLE")
        sys.exit(0)
        
    # Baseline simulation
    base_sims = [simulate_trade(d, 1.0, 1.0) for d in decisions]
    cand_sims = [simulate_trade(d, 1.25, 1.0) for d in decisions]
    
    # Export CSVs
    all_sym_path = out_dir / "candidate_validation_time_machine_v0_1_all_symbols.csv"
    with open(all_sym_path, "w", newline="") as f:
        writer = csv.writer(f)
        writer.writerow([
            "ticker", "date", "direction", "action", "reference_price",
            "baseline_target", "candidate_target", "baseline_risk", "candidate_risk",
            "baseline_exit_reason", "candidate_exit_reason",
            "baseline_exit_price", "candidate_exit_price",
            "baseline_return", "candidate_return",
            "delta_return", "delta_return_bps",
            "baseline_exit_timestamp", "candidate_exit_timestamp"
        ])
        
        for b, c in zip(base_sims, cand_sims):
            dec = b["decision"]
            as_of_dt = datetime.strptime(dec["as_of"], "%Y-%m-%dT%H:%M:%SZ").replace(tzinfo=timezone.utc)
            date_str = as_of_dt.strftime("%Y%m%d")
            
            bret = b["sim_return"]
            cret = c["sim_return"]
            delta = cret - bret
            delta_bps = delta * 10000
            
            bts = datetime.fromtimestamp(b["exit_ts"], timezone.utc).isoformat() if b["exit_ts"] else ""
            cts = datetime.fromtimestamp(c["exit_ts"], timezone.utc).isoformat() if c["exit_ts"] else ""
            
            writer.writerow([
                dec["ticker"], date_str, dec["direction"], dec.get("action", ""), dec.get("reference_price", dec.get("entry_price")),
                b["target_price"], c["target_price"], b["stop_price"], c["stop_price"],
                b["sim_exit_reason"], c["sim_exit_reason"],
                b["exit_price"], c["exit_price"],
                bret, cret,
                delta, delta_bps,
                bts, cts
            ])
            
    # Transition matrix
    trans_path = out_dir / "candidate_validation_time_machine_v0_1_transition_matrix.csv"
    exits = ["TARGET_GAP_THROUGH", "TARGET", "RISK_GAP_THROUGH", "RISK", "HORIZON"]
    trans_mat = {be: {ce: 0 for ce in exits} for be in exits}
    
    for b, c in zip(base_sims, cand_sims):
        trans_mat[b["sim_exit_reason"]][c["sim_exit_reason"]] += 1
        
    with open(trans_path, "w", newline="") as f:
        writer = csv.writer(f)
        writer.writerow(["BASE \\ CAND"] + exits)
        for be in exits:
            row = [be] + [trans_mat[be][ce] for ce in exits]
            writer.writerow(row)
            
    # Summary Generation
    summary_path = out_dir / "candidate_validation_time_machine_v0_1_summary.md"
    
    active_b = [b for b in base_sims if b["decision"].get("action") in ["Buy", "Sell", "LONG", "SHORT"]]
    active_c = [c for c in cand_sims if c["decision"].get("action") in ["Buy", "Sell", "LONG", "SHORT"]]
    
    improved = 0
    worsened = 0
    unchanged = 0
    deltas = []
    delta_by_action = {"Buy": [], "Sell": [], "Watch": [], "NoTrade": [], "LONG": [], "SHORT": []}
    
    for b, c in zip(base_sims, cand_sims):
        d = c["sim_return"] - b["sim_return"]
        act = b["decision"].get("action", "")
        if act in delta_by_action:
            delta_by_action[act].append(d)
            
        if act in ["Buy", "Sell", "LONG", "SHORT"]:
            deltas.append(d)
            if d > 1e-6: improved += 1
            elif d < -1e-6: worsened += 1
            else: unchanged += 1
            
    deltas.sort()
    sum_d = sum(deltas)
    mean_d = sum_d / max(1, len(deltas))
    med_d = deltas[len(deltas)//2] if deltas else 0
    p25 = deltas[int(len(deltas)*0.25)] if deltas else 0
    p75 = deltas[int(len(deltas)*0.75)] if deltas else 0
    best = deltas[-1] if deltas else 0
    worst = deltas[0] if deltas else 0
    
    # Sort transitions for concentration
    sorted_trades = sorted(zip(active_b, active_c), key=lambda x: abs(x[1]["sim_return"] - x[0]["sim_return"]), reverse=True)
    top_1_delta = sum(c["sim_return"]-b["sim_return"] for b,c in sorted_trades[:1])
    top_5_delta = sum(c["sim_return"]-b["sim_return"] for b,c in sorted_trades[:5])
    top_10_delta = sum(c["sim_return"]-b["sim_return"] for b,c in sorted_trades[:10])
    top_20_delta = sum(c["sim_return"]-b["sim_return"] for b,c in sorted_trades[:20])

    with open(summary_path, "w") as f:
        f.write("# TIME MACHINE GEOMETRY VALIDATION SUMMARY\n\n")
        f.write("## 1. Universe Coverage\n")
        f.write(f"- Total TIME-004 decisions: {d_count}\n")
        f.write(f"- Usable 1m data joins: {j_count}\n")
        f.write("\n## 2. Geometry Distribution (Actionable Cohort Only)\n")
        f.write(f"- Improved trades: {improved}\n")
        f.write(f"- Worsened trades: {worsened}\n")
        f.write(f"- Unchanged trades: {unchanged}\n")
        f.write(f"- Sum delta: {sum_d*100:.4f}%\n")
        f.write(f"- Mean delta: {mean_d*100:.4f}%\n")
        f.write(f"- Median delta: {med_d*100:.4f}%\n")
        f.write(f"- 25th percentile: {p25*100:.4f}%\n")
        f.write(f"- 75th percentile: {p75*100:.4f}%\n")
        f.write(f"- Best delta: {best*100:.4f}%\n")
        f.write(f"- Worst delta: {worst*100:.4f}%\n")
        
        f.write("\n## 3. Concentration of Geometry Effect\n")
        f.write(f"- Top 1 transition accounts for: {top_1_delta*100:.4f}%\n")
        f.write(f"- Top 5 transitions account for: {top_5_delta*100:.4f}%\n")
        f.write(f"- Top 10 transitions account for: {top_10_delta*100:.4f}%\n")
        f.write(f"- Top 20 transitions account for: {top_20_delta*100:.4f}%\n")
        
        f.write("\n## 4. Delta by Action\n")
        for act, vals in delta_by_action.items():
            if vals:
                f.write(f"- {act}: {sum(vals)*100:.4f}% (n={len(vals)})\n")

    print(f"Artifacts successfully written to {out_dir}")

if __name__ == "__main__":
    main()
