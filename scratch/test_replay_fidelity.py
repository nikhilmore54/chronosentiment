#!/usr/bin/env python3
"""
Simulate RP-605 execution over 1m bars for 2026-09-29 and compare with journal.
"""
import json
import math
import sys
from pathlib import Path
from datetime import datetime, timezone, timedelta
from collections import defaultdict
import numpy as np

WORKSPACE = Path("/Users/nikhil/ChronoSentiment_MEGA_FINAL")
sys.path.insert(0, str(WORKSPACE))
CACHE_DIR = WORKSPACE / "intraday_capture" / "yahoo_cache_1m"
JOURNAL_PATH = WORKSPACE / "live_capture" / "shadow_validation" / "rp605_nse_baseline_journal.jsonl"
from scripts.run_rp605_nse_prospective_freshness_validation import SYMBOLS, compute_coralys_scores

IST = timezone(timedelta(hours=5, minutes=30))
TARGET_DATE = "2026-09-29"

# 1. Load bars for target date
print(f"Loading bars for {TARGET_DATE}...")
bars_by_minute = defaultdict(dict)
minutes_set = set()

for sym in SYMBOLS:
    p = CACHE_DIR / f"{sym}.json"
    if not p.exists():
        continue
    data = json.loads(p.read_text())
    for b in data:
        dt = datetime.fromtimestamp(b["timestamp"], IST)
        if dt.strftime("%Y-%m-%d") == TARGET_DATE:
            # We index by minute string e.g. 09:15
            m_str = dt.strftime("%H:%M")
            minutes_set.add((dt.hour * 60 + dt.minute, dt))
            bars_by_minute[sym][dt] = b

sorted_minutes = [x[1] for x in sorted(list(minutes_set), key=lambda x: x[0])]
print(f"Total trading minutes found: {len(sorted_minutes)}")

# Replay RP-605 state
price_history = defaultdict(list)
score_history = defaultdict(list)
baseline_active = {}
baseline_completed = []
trade_counter = 0

FRESHNESS_THRESHOLD = -0.01

for current_dt in sorted_minutes:
    ts_utc = current_dt.astimezone(timezone.utc).strftime("%Y-%m-%dT%H:%M:%SZ")
    
    for sym in SYMBOLS:
        bar = bars_by_minute[sym].get(current_dt)
        if not bar:
            continue
        price = bar["close"]
        
        price_history[sym].append((ts_utc, price))
        if len(price_history[sym]) > 60:
            price_history[sym].pop(0)
            
        prices_list = [p[1] for p in price_history[sym]]
        s15, s30, s1d = compute_coralys_scores(prices_list)
        
        score_history[sym].append((ts_utc, s15, s30, s1d))
        if len(score_history[sym]) > 30:
            score_history[sym].pop(0)
            
        if s15 > 0.15 and s30 > 0.10:
            state, side = "BUY", "LONG"
        elif s15 < -0.15 and s30 < -0.10:
            state, side = "SHORT", "SHORT"
        elif abs(s15) < 0.05:
            state, side = "PULLBACK", "NONE"
        else:
            state, side = "WATCH", "NONE"
            
        # Entry check
        if state in ["BUY", "SHORT"] and sym not in baseline_active:
            trade_counter += 1
            trade_id = f"RP605-NSE-{trade_counter:04d}"
            
            history_15 = [x[1] if side=="LONG" else -x[1] for x in score_history[sym][-10:]]
            a15_t0 = history_15[-1] if history_15 else (s15 if side=="LONG" else -s15)
            peak_15 = max(history_15) if history_15 else a15_t0
            f15 = a15_t0 - peak_15
            is_fresh = f15 >= FRESHNESS_THRESHOLD
            
            position_capital_inr = 100000.0
            quantity = max(1, math.floor(position_capital_inr / price))
            
            trade_record = {
                "trade_id": trade_id,
                "symbol": sym,
                "side": side,
                "entry_timestamp": ts_utc,
                "entry_price": price,
                "quantity": quantity,
                "f15_at_entry": f15,
                "is_fresh": is_fresh
            }
            baseline_active[sym] = trade_record
            
        # Exit check
        if sym in baseline_active:
            pos = baseline_active[sym]
            entry_p = pos["entry_price"]
            side_pos = pos["side"]
            qty = pos["quantity"]
            entry_dt = datetime.fromisoformat(pos["entry_timestamp"].replace("Z", "+00:00"))
            now_utc = datetime.fromisoformat(ts_utc.replace("Z", "+00:00"))
            held_mins = int((now_utc - entry_dt).total_seconds() / 60)
            
            should_exit = False
            exit_reason = None
            if state in ["EXIT", "PULLBACK", "SHORT" if side_pos=="LONG" else "BUY"]:
                should_exit = True
                exit_reason = "ENGINE_EXIT"
            elif held_mins >= 60:
                should_exit = True
                exit_reason = "TARGET_EXIT"
                
            if should_exit:
                pnl = (price - entry_p) * qty if side_pos == "LONG" else (entry_p - price) * qty
                pos["exit_timestamp"] = ts_utc
                pos["exit_price"] = price
                pos["holding_mins"] = held_mins
                pos["exit_reason"] = exit_reason
                pos["pnl"] = pnl
                baseline_completed.append(pos)
                del baseline_active[sym]

# Close remaining at end
for sym, pos in list(baseline_active.items()):
    last_bar = bars_by_minute[sym].get(sorted_minutes[-1])
    if last_bar:
        px = last_bar["close"]
        pnl = (px - pos["entry_price"]) * pos["quantity"] if pos["side"] == "LONG" else (pos["entry_price"] - px) * pos["quantity"]
        pos["exit_timestamp"] = sorted_minutes[-1].astimezone(timezone.utc).strftime("%Y-%m-%dT%H:%M:%SZ")
        pos["exit_price"] = px
        pos["pnl"] = pnl
        pos["exit_reason"] = "ENGINE_EXIT"
        baseline_completed.append(pos)

print(f"Replay {TARGET_DATE} trades completed: {len(baseline_completed)}")
pnl_tot = sum(t["pnl"] for t in baseline_completed)
wins = [t for t in baseline_completed if t["pnl"] > 0]
print(f"Replay {TARGET_DATE} Net P&L: ₹{pnl_tot:,.2f} | Win Rate: {len(wins)/len(baseline_completed)*100:.2f}%")

# Load actual journal 29 Sep
recorded_29 = []
with JOURNAL_PATH.open() as f:
    for line in f:
        if line.strip():
            d = json.loads(line)
            if d.get("entry_timestamp", "").startswith("2026-09-29"):
                recorded_29.append(d)
rec_pnl = sum(t["pnl"] for t in recorded_29)
rec_wins = [t for t in recorded_29 if t["pnl"] > 0]
print(f"Recorded 29 Sep: {len(recorded_29)} trades | Net P&L: ₹{rec_pnl:,.2f} | Win Rate: {len(rec_wins)/len(recorded_29)*100:.2f}%")
