#!/usr/bin/env python3
"""
analyze_shadow_loss_trajectories.py

Forensic trajectory reconstruction for 12-hour prospective live shadow run:
Reconstructs canonical DecisionState & trajectory scores (15m, 30m, 1d)
at T+0, T+5, T+10, T+15, T+30, T+45, T+60 for:
1. Forensic sample of 11 key losing trades (8 ENGINE, 3 TARGET)
2. Aggregate cohort comparison: Early ENGINE Losses (<30m) vs TARGET Losses vs Winners
"""

import json
from pathlib import Path
from datetime import datetime, timezone, timedelta
from collections import defaultdict

WORKSPACE_DIR = Path(__file__).parent.parent
JOURNAL_PATH = WORKSPACE_DIR / "live_capture" / "shadow_validation" / "shadow_trade_journal.jsonl"
CARDS_PATH = WORKSPACE_DIR / "live_capture" / "shadow_validation" / "decision_cards_log.jsonl"

def parse_iso(ts_str):
    if not ts_str:
        return None
    return datetime.strptime(ts_str, "%Y-%m-%dT%H:%M:%SZ").replace(tzinfo=timezone.utc)

def format_iso(dt):
    return dt.strftime("%Y-%m-%dT%H:%M:%SZ")

def load_data():
    with open(JOURNAL_PATH) as f:
        trades = [json.loads(line) for line in f if line.strip()]
    
    with open(CARDS_PATH) as f:
        cards = [json.loads(line) for line in f if line.strip()]
        
    # Map (sym, iso_timestamp) -> card
    cards_by_time = defaultdict(dict)
    all_timestamps = set()
    for c in cards:
        sym = c.get("instrument")
        ts = c.get("timestamp_utc")
        if sym and ts:
            cards_by_time[ts][sym] = c
            all_timestamps.add(ts)
            
    sorted_ts = sorted(list(all_timestamps))
    ts_dt_map = {ts: parse_iso(ts) for ts in sorted_ts}
    
    return trades, cards_by_time, sorted_ts, ts_dt_map

def find_nearest_card(sym, target_dt, cards_by_time, sorted_ts, ts_dt_map):
    # Find card within 2 minutes of target_dt
    best_ts = None
    min_diff = 120 # max 2 min difference
    for ts in sorted_ts:
        dt = ts_dt_map[ts]
        diff = abs((dt - target_dt).total_seconds())
        if diff < min_diff:
            if sym in cards_by_time[ts]:
                min_diff = diff
                best_ts = ts
    if best_ts:
        return cards_by_time[best_ts][sym], best_ts
    return None, None

def main():
    trades, cards_by_time, sorted_ts, ts_dt_map = load_data()
    print(f"Loaded {len(trades)} trades and {len(sorted_ts)} unique minute ticks.")
    
    # Forensic sample trade IDs from user request
    forensic_ids = [
        # ENGINE losses
        "SHADOW-0063", # DOGE -$122.40 (28m)
        "SHADOW-0027", # DOGE -$112.20 (50m)
        "SHADOW-0067", # DOGE -$61.80 (11m)
        "SHADOW-0071", # ADA -$50.96 (3m)
        "SHADOW-0066", # ADA -$46.92 (32m)
        "SHADOW-0043", # XRP -$40.92 (29m)
        "SHADOW-0036", # ETH -$36.04 (47m)
        "SHADOW-0025", # ADA -$35.82 (16m)
        # TARGET losses
        "SHADOW-0040", # XRP -$84.50 (60m)
        "SHADOW-0016", # ADA -$71.64 (60m)
        "SHADOW-0037"  # ADA -$54.18 (60m)
    ]
    
    results = []
    
    for t in trades:
        tid = t.get("trade_id")
        sym = t.get("symbol")
        side = t.get("side")
        entry_price = float(t.get("entry_price", 0))
        exit_price = float(t.get("exit_price", 0))
        pnl = float(t.get("realized_pnl", 0))
        exit_reason = t.get("exit_reason")
        exit_state = t.get("state_at_exit", "")
        entry_dt = parse_iso(t.get("entry_timestamp"))
        exit_dt = parse_iso(t.get("exit_timestamp"))
        held_mins = t.get("holding_duration_mins", 0)
        
        time_offsets = [0, 5, 10, 15, 30, 45, 60]
        timeline = {}
        
        for offset in time_offsets:
            target_dt = entry_dt + timedelta(minutes=offset)
            if target_dt <= exit_dt + timedelta(minutes=1):
                c, ts_found = find_nearest_card(sym, target_dt, cards_by_time, sorted_ts, ts_dt_map)
                if c:
                    cur_p = float(c.get("current_price", entry_price))
                    t15 = float(c.get("15m_score_trajectory", 0.0))
                    t30 = float(c.get("30m_score_trajectory", 0.0))
                    t1d = float(c.get("1d_score_trajectory", 0.0))
                    state = c.get("decision_state", "")
                    align = c.get("cross_resolution_relationship", "")
                    
                    ret_pct = (cur_p - entry_price)/entry_price * 100 if side == "LONG" else (entry_price - cur_p)/entry_price * 100
                    
                    timeline[f"T+{offset}m"] = {
                        "ts": ts_found,
                        "price": cur_p,
                        "state": state,
                        "alignment": align,
                        "traj_15m": t15,
                        "traj_30m": t30,
                        "traj_1d": t1d,
                        "ret_pct": ret_pct
                    }
                    
        results.append({
            "trade": t,
            "timeline": timeline
        })

    # Print summary of forensic 11 trades
    print("\n" + "="*95)
    print("FORENSIC SAMPLE ANALYSIS (11 KEY LOSING TRADES)")
    print("="*95)
    
    for r in results:
        t = r["trade"]
        tid = t.get("trade_id")
        if tid not in forensic_ids:
            continue
        
        sym = t.get("symbol")
        side = t.get("side")
        pnl = float(t.get("realized_pnl", 0))
        held = t.get("holding_duration_mins")
        reason = t.get("exit_reason")
        exit_st = t.get("state_at_exit")
        
        print(f"\n[{tid}] {sym} ({side}) | PnL: ${pnl:.2f} | Held: {held}m | Exit: {reason} -> {exit_st}")
        print(f"  Timeline Trajectory:")
        for off, data in r["timeline"].items():
            st = data["state"]
            align = data["alignment"]
            t15 = data["traj_15m"]
            t30 = data["traj_30m"]
            t1d = data["traj_1d"]
            ret = data["ret_pct"]
            print(f"    {off:>6}: P: ${data['price']:<9.2f} | State: {st:<9} | Align: {align:<22} | Scores[15m/30m/1d]: [{t15:+.3f}, {t30:+.3f}, {t1d:+.3f}] | Ret: {ret:+.2f}%")

    # Group Cohort Stats
    early_engine_losses = [r for r in results if r["trade"]["exit_reason"] == "ENGINE_EXIT" and float(r["trade"]["realized_pnl"]) < 0 and r["trade"]["holding_duration_mins"] <= 30]
    target_losses = [r for r in results if r["trade"]["exit_reason"] == "TARGET_EXIT" and float(r["trade"]["realized_pnl"]) < 0]
    target_winners = [r for r in results if r["trade"]["exit_reason"] == "TARGET_EXIT" and float(r["trade"]["realized_pnl"]) > 0]
    
    print("\n" + "="*95)
    print("COHORT TRAJECTORY EVOLUTION (AVERAGES & EARLY WARNING SIGNS)")
    print("="*95)
    
    def analyze_cohort_evolution(name, cohort):
        print(f"\n--- {name} (N={len(cohort)}) ---")
        for offset in [0, 5, 10, 15, 30]:
            off_key = f"T+{offset}m"
            items = [r["timeline"][off_key] for r in cohort if off_key in r["timeline"]]
            if not items:
                continue
            avg_15m = sum(i["traj_15m"] for i in items) / len(items)
            avg_30m = sum(i["traj_30m"] for i in items) / len(items)
            avg_ret = sum(i["ret_pct"] for i in items) / len(items)
            state_counts = defaultdict(int)
            align_counts = defaultdict(int)
            for i in items:
                state_counts[i["state"]] += 1
                align_counts[i["alignment"]] += 1
            print(f"  {off_key:>6}: N={len(items):<2} | Avg 15m Traj: {avg_15m:+.3f} | Avg 30m Traj: {avg_30m:+.3f} | Avg Ret: {avg_ret:+.2f}%")
            print(f"          States: {dict(state_counts)}")
            print(f"          Alignments: {dict(align_counts)}")

    analyze_cohort_evolution("Problem A: Early ENGINE_EXIT Losses (<=30m)", early_engine_losses)
    analyze_cohort_evolution("Problem B: TARGET_EXIT Losses (60m)", target_losses)
    analyze_cohort_evolution("Benchmark: TARGET_EXIT Winners (60m)", target_winners)

if __name__ == "__main__":
    main()
