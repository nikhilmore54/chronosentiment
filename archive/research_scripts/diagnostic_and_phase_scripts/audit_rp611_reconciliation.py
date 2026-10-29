#!/usr/bin/env python3
import json
import csv
from pathlib import Path
from collections import defaultdict
from datetime import datetime, timezone
import numpy as np

WORKSPACE = Path(".")
JOURNAL_PATH = WORKSPACE / "live_capture/shadow_validation/rp605_nse_baseline_journal.jsonl"
C28_PATH = WORKSPACE / "live_capture/shadow_validation/rp611_nse_20260928_trades.csv"
C29_PATH = WORKSPACE / "live_capture/shadow_validation/rp611_nse_20260929_trades.csv"

# 1. Load RP-605
rp605_trades = defaultdict(list)
with JOURNAL_PATH.open(encoding="utf-8") as f:
    for line in f:
        if line.strip():
            d = json.loads(line)
            day = d["entry_timestamp"][:10]
            if day in ["2026-09-28", "2026-09-29"]:
                rp605_trades[day].append(d)

print("=" * 80)
print("1. RP-605 BASELINE CONTROL CANONICAL AUDIT")
print("=" * 80)
for day in ["2026-09-28", "2026-09-29"]:
    trades = rp605_trades[day]
    tot = len(trades)
    pnls = [float(t["pnl"]) for t in trades]
    wins = [p for p in pnls if p > 0]
    losses = [p for p in pnls if p < 0]
    flats = [p for p in pnls if p == 0]
    sum_pnl = sum(pnls)
    sum_wins = sum(wins)
    sum_losses = sum(abs(p) for p in losses)
    pf = sum_wins / sum_losses if sum_losses > 0 else 0.0
    print(f"[{day}] Count: {tot} | Wins: {len(wins)} ({len(wins)/tot*100:.2f}%) | Losses: {len(losses)} | Flats: {len(flats)}")
    print(f"[{day}] Gross Wins: ₹{sum_wins:,.2f} | Gross Losses: ₹{sum_losses:,.2f} | Net PnL: ₹{sum_pnl:,.2f} | PF: {pf:.4f}")

pooled_605 = rp605_trades["2026-09-28"] + rp605_trades["2026-09-29"]
tot_605 = len(pooled_605)
pnls_605 = [float(t["pnl"]) for t in pooled_605]
wins_605 = [p for p in pnls_605 if p > 0]
losses_605 = [p for p in pnls_605 if p < 0]
flats_605 = [p for p in pnls_605 if p == 0]
print(f"[POOLED RP-605] Count: {tot_605} | Wins: {len(wins_605)} ({len(wins_605)/tot_605*100:.2f}%) | Losses: {len(losses_605)} | Flats: {len(flats_605)}")
print(f"[POOLED RP-605] Gross Wins: ₹{sum(wins_605):,.2f} | Gross Losses: ₹{sum(abs(p) for p in losses_605):,.2f} | Net PnL: ₹{sum(pnls_605):,.2f} | PF: {sum(wins_605)/sum(abs(p) for p in losses_605):.4f}")

# 2. Load RP-611
rp611_trades = defaultdict(list)
for day, p in [("2026-09-28", C28_PATH), ("2026-09-29", C29_PATH)]:
    with p.open(encoding="utf-8") as f:
        reader = csv.DictReader(f)
        for r in reader:
            rp611_trades[day].append(r)

print("\n" + "=" * 80)
print("2. RP-611 PULLBACK STRATEGY CANONICAL AUDIT")
print("=" * 80)
for day in ["2026-09-28", "2026-09-29"]:
    trades = rp611_trades[day]
    tot = len(trades)
    net_pnls = [float(t["net_pnl"]) for t in trades]
    gross_pnls = [float(t["gross_pnl"]) for t in trades]
    frictions = [float(t["friction_inr"]) for t in trades]
    mfes = [float(t["mfe_pnl"]) for t in trades]
    maes = [float(t["mae_pnl"]) for t in trades]

    net_wins = [p for p in net_pnls if p > 0]
    net_losses = [p for p in net_pnls if p < 0]
    net_flats = [p for p in net_pnls if p == 0]

    gross_wins = [p for p in gross_pnls if p > 0]
    gross_losses = [abs(p) for p in gross_pnls if p < 0]

    zero_mfe = sum(1 for m in mfes if m <= 0.0)

    sum_net_pnl = sum(net_pnls)
    sum_gross_profit = sum(gross_wins)
    sum_gross_loss = sum(gross_losses)
    sum_friction = sum(frictions)

    # Net PF definitions
    pf_trade_level = sum(net_wins) / sum(abs(p) for p in net_losses) if net_losses else 0.0
    pf_gross_derived = sum_gross_profit / (sum_gross_loss + sum_friction) if (sum_gross_loss + sum_friction) > 0 else 0.0

    print(f"[{day}] Count: {tot} | Net Wins: {len(net_wins)} ({len(net_wins)/tot*100:.2f}%) | Net Losses: {len(net_losses)} | Zero-MFE: {zero_mfe} ({zero_mfe/tot*100:.2f}%)")
    print(f"[{day}] Gross Profit: ₹{sum_gross_profit:,.4f} | Gross Loss: ₹{sum_gross_loss:,.4f} | Friction: ₹{sum_friction:,.4f}")
    print(f"[{day}] Net PnL: ₹{sum_net_pnl:,.4f}")
    print(f"[{day}] PF (Trade-Level Net Wins/Losses): {pf_trade_level:.4f} | PF (Gross-Derived): {pf_gross_derived:.4f}")

pooled_611 = rp611_trades["2026-09-28"] + rp611_trades["2026-09-29"]
tot_611 = len(pooled_611)
net_pnls_611 = [float(t["net_pnl"]) for t in pooled_611]
gross_pnls_611 = [float(t["gross_pnl"]) for t in pooled_611]
frictions_611 = [float(t["friction_inr"]) for t in pooled_611]
mfes_611 = [float(t["mfe_pnl"]) for t in pooled_611]
net_wins_611 = [p for p in net_pnls_611 if p > 0]
net_losses_611 = [p for p in net_pnls_611 if p < 0]
gross_wins_611 = [p for p in gross_pnls_611 if p > 0]
gross_losses_611 = [abs(p) for p in gross_pnls_611 if p < 0]
zero_mfe_611 = sum(1 for m in mfes_611 if m <= 0.0)

sum_net_pnl_611 = sum(net_pnls_611)
sum_gross_profit_611 = sum(gross_wins_611)
sum_gross_loss_611 = sum(gross_losses_611)
sum_friction_611 = sum(frictions_611)

pf_trade_level_611 = sum(net_wins_611) / sum(abs(p) for p in net_losses_611)
pf_gross_derived_611 = sum_gross_profit_611 / (sum_gross_loss_611 + sum_friction_611)

print(f"\n[POOLED RP-611] Count: {tot_611} | Net Wins: {len(net_wins_611)} ({len(net_wins_611)/tot_611*100:.2f}%) | Net Losses: {len(net_losses_611)} | Zero-MFE: {zero_mfe_611} ({zero_mfe_611/tot_611*100:.2f}%)")
print(f"[POOLED RP-611] Exact Gross Profit: ₹{sum_gross_profit_611:.4f} (Round 2dp: ₹{sum_gross_profit_611:,.2f})")
print(f"[POOLED RP-611] Exact Gross Loss: ₹{sum_gross_loss_611:.4f} (Round 2dp: ₹{sum_gross_loss_611:,.2f})")
print(f"[POOLED RP-611] Exact Total Friction: ₹{sum_friction_611:.4f} (Round 2dp: ₹{sum_friction_611:,.2f})")
print(f"[POOLED RP-611] Exact Net PnL: ₹{sum_net_pnl_611:.4f} (Round 2dp: ₹{sum_net_pnl_611:,.2f})")
print(f"[POOLED RP-611] PF (Trade-Level Net: sum(wins)/sum(losses)): {pf_trade_level_611:.4f}")
print(f"[POOLED RP-611] PF (Gross-Derived: gross_profit/(gross_loss+friction)): {pf_gross_derived_611:.4f}")

# 3. POPULATION RECONCILIATION & OPPORTUNITY COST AUDIT
print("\n" + "=" * 80)
print("3. EXPLICIT MUTUALLY EXCLUSIVE POPULATION RECONCILIATION")
print("=" * 80)

for day in ["2026-09-28", "2026-09-29"]:
    day_605 = rp605_trades[day]
    day_611 = rp611_trades[day]

    matched_605_ids = set()
    matched_611_ids = set()
    matched_pairs = []

    for t611 in day_611:
        t611_dt = datetime.fromisoformat(t611["entry_timestamp"].replace("Z", "+00:00"))
        for t605 in day_605:
            if t605["trade_id"] in matched_605_ids:
                continue
            if t605["symbol"] == t611["symbol"] and t605["side"] == t611["side"]:
                t605_dt = datetime.fromisoformat(t605["entry_timestamp"].replace("Z", "+00:00"))
                if abs((t611_dt - t605_dt).total_seconds()) <= 300:
                    matched_605_ids.add(t605["trade_id"])
                    matched_611_ids.add(t611["trade_id"])
                    matched_pairs.append((t605, t611))
                    break

    unmatched_605 = [t for t in day_605 if t["trade_id"] not in matched_605_ids]
    unmatched_605_wins = [t for t in unmatched_605 if float(t["pnl"]) > 0]
    unmatched_605_losses = [t for t in unmatched_605 if float(t["pnl"]) < 0]
    unmatched_605_flats = [t for t in unmatched_605 if float(t["pnl"]) == 0]

    matched_605 = [t for t in day_605 if t["trade_id"] in matched_605_ids]
    matched_605_wins = [t for t in matched_605 if float(t["pnl"]) > 0]
    matched_605_losses = [t for t in matched_605 if float(t["pnl"]) < 0]
    matched_605_flats = [t for t in matched_605 if float(t["pnl"]) == 0]

    unmatched_611 = [t for t in day_611 if t["trade_id"] not in matched_611_ids]

    missed_gross_profit = sum(float(t["pnl"]) for t in unmatched_605_wins)
    avoided_gross_loss = sum(abs(float(t["pnl"])) for t in unmatched_605_losses)

    print(f"\n--- {day} Reconciliation ---")
    print(f"Total RP-605: {len(day_605)}")
    print(f"  ├── Matched RP-605: {len(matched_605)} (Wins: {len(matched_605_wins)}, Losses: {len(matched_605_losses)}, Flats: {len(matched_605_flats)}) | Net PnL: ₹{sum(float(t['pnl']) for t in matched_605):,.2f}")
    print(f"  └── Unmatched RP-605 (Filtered by RP-611): {len(unmatched_605)}")
    print(f"        ├── Unmatched Winners (Missed Opportunities): {len(unmatched_605_wins)} | Gross Profit Sacrificed: ₹{missed_gross_profit:,.2f}")
    print(f"        ├── Unmatched Losers (Avoided Losses): {len(unmatched_605_losses)} | Gross Loss Avoided: ₹{avoided_gross_loss:,.2f}")
    print(f"        └── Unmatched Flats: {len(unmatched_605_flats)}")
    print(f"  Check: {len(matched_605)} + {len(unmatched_605_wins)} + {len(unmatched_605_losses)} + {len(unmatched_605_flats)} == {len(day_605)} -> {len(matched_605) + len(unmatched_605_wins) + len(unmatched_605_losses) + len(unmatched_605_flats) == len(day_605)}")

    print(f"Total RP-611: {len(day_611)}")
    print(f"  ├── Matched with RP-605: {len(matched_611_ids)}")
    print(f"  └── Unmatched (Secondary/Delayed Pullback triggers): {len(unmatched_611)}")
    print(f"  Check: {len(matched_611_ids)} + {len(unmatched_611)} == {len(day_611)} -> {len(matched_611_ids) + len(unmatched_611) == len(day_611)}")

# Pooled Reconciliation
print("\n--- POOLED (28 & 29 Sep) Reconciliation ---")
p_605_matched_ids = set()
p_611_matched_ids = set()
for day in ["2026-09-28", "2026-09-29"]:
    day_605 = rp605_trades[day]
    day_611 = rp611_trades[day]
    for t611 in day_611:
        t611_dt = datetime.fromisoformat(t611["entry_timestamp"].replace("Z", "+00:00"))
        for t605 in day_605:
            if t605["trade_id"] in p_605_matched_ids:
                continue
            if t605["symbol"] == t611["symbol"] and t605["side"] == t611["side"]:
                t605_dt = datetime.fromisoformat(t605["entry_timestamp"].replace("Z", "+00:00"))
                if abs((t611_dt - t605_dt).total_seconds()) <= 300:
                    p_605_matched_ids.add(t605["trade_id"])
                    p_611_matched_ids.add(t611["trade_id"])
                    break

p_unmatched_605 = [t for t in pooled_605 if t["trade_id"] not in p_605_matched_ids]
p_unmatched_605_wins = [t for t in p_unmatched_605 if float(t["pnl"]) > 0]
p_unmatched_605_losses = [t for t in p_unmatched_605 if float(t["pnl"]) < 0]
p_unmatched_605_flats = [t for t in p_unmatched_605 if float(t["pnl"]) == 0]
p_matched_605 = [t for t in pooled_605 if t["trade_id"] in p_605_matched_ids]
p_matched_605_wins = [t for t in p_matched_605 if float(t["pnl"]) > 0]
p_matched_605_losses = [t for t in p_matched_605 if float(t["pnl"]) < 0]
p_matched_605_flats = [t for t in p_matched_605 if float(t["pnl"]) == 0]

p_missed_gross_profit = sum(float(t["pnl"]) for t in p_unmatched_605_wins)
p_avoided_gross_loss = sum(abs(float(t["pnl"])) for t in p_unmatched_605_losses)

print(f"Total RP-605 Control Trades: {len(pooled_605)}")
print(f"  ├── Coincident/Matched Trades: {len(p_matched_605)} (Wins: {len(p_matched_605_wins)}, Losses: {len(p_matched_605_losses)}, Flats: {len(p_matched_605_flats)})")
print(f"  └── Filtered RP-605 Trades: {len(p_unmatched_605)}")
print(f"        ├── Filtered Profitable Trades (Missed Opportunities): {len(p_unmatched_605_wins)} | Gross Profit: ₹{p_missed_gross_profit:,.2f}")
print(f"        ├── Filtered Losing Trades (Avoided Bleed): {len(p_unmatched_605_losses)} | Gross Loss: ₹{p_avoided_gross_loss:,.2f}")
print(f"        └── Filtered Flat Trades: {len(p_unmatched_605_flats)}")
print(f"  Verification: {len(p_matched_605)} + {len(p_unmatched_605_wins)} + {len(p_unmatched_605_losses)} + {len(p_unmatched_605_flats)} = {len(p_matched_605) + len(p_unmatched_605_wins) + len(p_unmatched_605_losses) + len(p_unmatched_605_flats)} (Expected: 3590)")

print(f"\nTotal RP-611 Strategy Trades: {len(pooled_611)}")
print(f"  ├── Matched with RP-605 Breakouts: {len(p_611_matched_ids)}")
print(f"  └── Secondary Pullback Triggers (No coincident RP-605 trade): {len(pooled_611) - len(p_611_matched_ids)}")
print(f"  Verification: {len(p_611_matched_ids)} + {len(pooled_611) - len(p_611_matched_ids)} = {len(pooled_611)} (Expected: 1007)")
