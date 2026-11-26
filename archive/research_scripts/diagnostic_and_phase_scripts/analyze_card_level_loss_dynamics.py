#!/usr/bin/env python3
"""
analyze_card_level_loss_dynamics.py

Detailed analysis of decision_cards_log.jsonl for all 48 loss-making shadow trades:
1. DecisionCard state & trajectory score at exact entry tick vs exact exit tick.
2. Velocity of 15m/30m score deterioration per minute.
3. Transition of cross_resolution_relationship & risk_state.
4. Exact DecisionCard JSON snapshots at entry, midpoint, and exit for top loss trades.
"""

import json
from pathlib import Path
from datetime import datetime, timezone, timedelta
from collections import defaultdict

WORKSPACE_DIR = Path(__file__).parent.parent
JOURNAL_PATH = WORKSPACE_DIR / "live_capture" / "shadow_validation" / "shadow_trade_journal.jsonl"
CARDS_PATH = WORKSPACE_DIR / "live_capture" / "shadow_validation" / "decision_cards_log.jsonl"

def parse_iso(ts):
    return datetime.strptime(ts, "%Y-%m-%dT%H:%M:%SZ").replace(tzinfo=timezone.utc)

def main():
    with open(JOURNAL_PATH) as f:
        trades = [json.loads(line) for line in f if line.strip()]
        
    with open(CARDS_PATH) as f:
        cards = [json.loads(line) for line in f if line.strip()]
        
    # Group cards by (instrument, dt)
    card_list_by_sym = defaultdict(list)
    for c in cards:
        sym = c.get("instrument")
        ts = c.get("timestamp_utc")
        if sym and ts:
            dt = parse_iso(ts)
            card_list_by_sym[sym].append((dt, c))
            
    # Sort cards by dt
    for sym in card_list_by_sym:
        card_list_by_sym[sym].sort(key=lambda x: x[0])
        
    def get_card(sym, target_ts, max_diff_sec=45):
        if not target_ts:
            return None
        target_dt = parse_iso(target_ts)
        best = None
        best_diff = max_diff_sec
        for dt, c in card_list_by_sym[sym]:
            diff = abs((dt - target_dt).total_seconds())
            if diff < best_diff:
                best_diff = diff
                best = c
        return best

    loss_trades = [t for t in trades if float(t.get("realized_pnl", 0)) < 0]
    engine_losses = [t for t in loss_trades if t.get("exit_reason") == "ENGINE_EXIT"]
    target_losses = [t for t in loss_trades if t.get("exit_reason") == "TARGET_EXIT"]
    
    print(f"Total Shadow Trades: {len(trades)}")
    print(f"Total Loss Trades: {len(loss_trades)} (ENGINE Losses: {len(engine_losses)}, TARGET Losses: {len(target_losses)})")
    
    # 1. State at Exit Tick for ENGINE Losses
    exit_card_states = defaultdict(int)
    exit_alignments = defaultdict(int)
    exit_risk_states = defaultdict(int)
    
    score_diff_15m = []
    vel_15m_list = []
    
    for t in engine_losses:
        sym = t["symbol"]
        side = t["side"]
        entry_ts = t["entry_timestamp"]
        exit_ts = t["exit_timestamp"]
        held = t["holding_duration_mins"]
        
        entry_card = get_card(sym, entry_ts)
        exit_card = get_card(sym, exit_ts)
        
        if entry_card and exit_card:
            st_exit = exit_card.get("decision_state")
            align_exit = exit_card.get("cross_resolution_relationship")
            risk_exit = exit_card.get("risk_state")
            
            exit_card_states[st_exit] += 1
            exit_alignments[align_exit] += 1
            exit_risk_states[risk_exit] += 1
            
            e_15m = float(entry_card.get("15m_score_trajectory", 0))
            x_15m = float(exit_card.get("15m_score_trajectory", 0))
            
            # Directional score change
            d_15m = (x_15m - e_15m) if side == "LONG" else (e_15m - x_15m)
            score_diff_15m.append(d_15m)
            
            if held > 0:
                vel_15m = d_15m / held
                vel_15m_list.append(vel_15m)
                
    print("\n" + "="*80)
    print("1. DECISION CARD STATE AT EXACT EXIT TICK (ENGINE_EXIT LOSSES)")
    print("="*80)
    print(f"Decision States at Exit Tick: {dict(exit_card_states)}")
    print(f"Cross-Resolution Alignment at Exit Tick: {dict(exit_alignments)}")
    print(f"Risk States at Exit Tick: {dict(exit_risk_states)}")
    if score_diff_15m:
        print(f"Avg Directional 15m Score Delta (Exit - Entry): {sum(score_diff_15m)/len(score_diff_15m):+.4f}")
    if vel_15m_list:
        print(f"Avg 15m Trajectory Velocity: {sum(vel_15m_list)/len(vel_15m_list):+.4f} / min")

    # 2. Card snapshot comparison at T+0, T+5, Exit for Top Loss Trades
    print("\n" + "="*80)
    print("2. EXACT DECISION CARD SNAPSHOTS AT ENTRY, T+5m & EXIT (TOP LOSS TRADES)")
    print("="*80)
    
    top_losses = sorted(loss_trades, key=lambda x: float(x["realized_pnl"]))[:12]
    
    for t in top_losses:
        tid = t["trade_id"]
        sym = t["symbol"]
        side = t["side"]
        pnl = float(t["realized_pnl"])
        held = t["holding_duration_mins"]
        reason = t["exit_reason"]
        entry_ts = t["entry_timestamp"]
        exit_ts = t["exit_timestamp"]
        entry_dt = parse_iso(entry_ts)
        t5_ts = (entry_dt + timedelta(minutes=5)).strftime("%Y-%m-%dT%H:%M:%SZ")
        
        c_entry = get_card(sym, entry_ts) or {}
        c_t5 = get_card(sym, t5_ts) or {}
        c_exit = get_card(sym, exit_ts) or {}
        
        print(f"\n[{tid}] {sym} ({side}) | PnL: ${pnl:.2f} | Held: {held}m | Exit Reason: {reason}")
        print(f"  Entry ({entry_ts[-9:-1]}): State={c_entry.get('decision_state')}, Align={c_entry.get('cross_resolution_relationship')}, Scores=[{c_entry.get('15m_score_trajectory')}, {c_entry.get('30m_score_trajectory')}, {c_entry.get('1d_score_trajectory')}]")
        if c_t5:
            print(f"  T+5m  ({t5_ts[-9:-1]}): State={c_t5.get('decision_state')}, Align={c_t5.get('cross_resolution_relationship')}, Scores=[{c_t5.get('15m_score_trajectory')}, {c_t5.get('30m_score_trajectory')}, {c_t5.get('1d_score_trajectory')}]")
        print(f"  Exit  ({exit_ts[-9:-1]}): State={c_exit.get('decision_state')}, Align={c_exit.get('cross_resolution_relationship')}, Scores=[{c_exit.get('15m_score_trajectory')}, {c_exit.get('30m_score_trajectory')}, {c_exit.get('1d_score_trajectory')}]")

if __name__ == "__main__":
    main()
