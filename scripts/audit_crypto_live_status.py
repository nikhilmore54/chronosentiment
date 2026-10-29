#!/usr/bin/env python3
"""
scripts/audit_crypto_live_status.py
Reads live crypto observer and event engine status directly on LCUtest.
"""
import json
from pathlib import Path
from collections import Counter
from datetime import datetime, timezone

BASE = Path("live_capture/shadow_validation/crypto")

# 1. Canonical Shadow Observer Checkpoint
ckpt_path = BASE / "rp608_checkpoint.json"
if ckpt_path.exists():
    with open(ckpt_path) as f:
        ckpt = json.load(f)
    print("=== 1. CANONICAL SHADOW OBSERVER (Binance BTC/USDT) ===")
    print(f"Status: {ckpt.get('status')} | Session: {ckpt.get('session_idx')}")
    print(f"Current Bar: Bar {ckpt.get('last_durable_bar_idx')} ({ckpt.get('last_heartbeat_iso')})")
    print(f"Cumulative Accounted Bars: {ckpt.get('last_accounted_bar_idx')}")
    print(f"Completed Trades: {ckpt.get('completed_trades_count')}")
    print(f"Directional Estimand B Cards: {ckpt.get('directional_cards_count')}")
    print(f"Consecutive Outages: {ckpt.get('consecutive_outages')}")
    print(f"Checksum: {ckpt.get('checksum')[:16]}...")

# 2. Decision Cards Analysis
cards_path = BASE / "rp608_decision_cards.jsonl"
if cards_path.exists():
    cards = []
    with open(cards_path) as f:
        for line in f:
            if line.strip():
                try: cards.append(json.loads(line))
                except: pass
    print(f"\nTotal Evaluated Decision Cards: {len(cards)}")
    if cards:
        last_card = cards[-1]
        print(f"Latest Bar Decision: Bar {last_card.get('bar_idx')} ({last_card.get('timestamp_iso')})")
        print(f"  Estimand B: Direction={last_card.get('estimand_b', {}).get('predicted_direction')} | Traj15={last_card.get('estimand_b', {}).get('score_traj15', 0.0):+.4f} | Prob={last_card.get('estimand_b', {}).get('forecast_probability')}")
        print(f"  Estimand C: Recommendation={last_card.get('estimand_c', {}).get('recommendation')} | Traj15={last_card.get('estimand_c', {}).get('score_traj15', 0.0):+.4f}")

# 3. Trade Plan Overlay V1 Outcomes
overlay_state_path = BASE / "trade_plan_overlay_v1/overlay_state.json"
if overlay_state_path.exists():
    with open(overlay_state_path) as f:
        ov = json.load(f)
    print("\n=== 2. CANONICAL TRADE PLAN OVERLAY V1 ===")
    print(f"Total Plans Emitted: {ov.get('total_plans_emitted', 0)}")
    print(f"Total Plans Closed: {ov.get('total_plans_closed', 0)}")
    active_plans = ov.get("active_plans", {})
    print(f"Currently Active Plans: {len(active_plans)}")
    for pid, p in active_plans.items():
        print(f"  - Plan #{pid}: Dir={p.get('direction')} | Entry=${p.get('entry_price', 0):.2f} @ Bar {p.get('entry_bar_idx')} ({p.get('entry_timestamp_iso')})")

plans_path = BASE / "trade_plan_overlay_v1/trade_plan_outcomes.jsonl"
if plans_path.exists():
    plans = []
    with open(plans_path) as f:
        for line in f:
            if line.strip():
                try: plans.append(json.loads(line))
                except: pass
    print(f"\nReconciled Trade Plan Outcomes: {len(plans)}")
    if plans:
        # Tally metrics
        wins = sum(1 for p in plans if p.get('net_pnl_bps', 0) > 0)
        net_pnls = [p.get('net_pnl_bps', 0) for p in plans]
        win_rate = (wins / len(plans)) * 100.0 if plans else 0.0
        cum_net = sum(net_pnls)
        mean_net = cum_net / len(plans) if plans else 0.0
        print(f"Win Rate: {win_rate:.1f}% ({wins}/{len(plans)})")
        print(f"Cumulative Net PnL (12.0 bps cost model): {cum_net:+.1f} bps | Mean Net: {mean_net:+.1f} bps")
        reasons = Counter(p.get('exit_reason') for p in plans)
        print(f"Exit Reasons: {dict(reasons)}")
        print("\nLast 3 Closed Outcomes:")
        for p in plans[-3:]:
            print(f"  - Plan #{p.get('plan_id')} | Dir: {p.get('direction')} | Entry: ${p.get('entry_price', 0):.2f} -> Exit: ${p.get('exit_price', 0):.2f} | Reason: {p.get('exit_reason')} | Net: {p.get('net_pnl_bps', 0.0):+.1f} bps")

# 4. Gate C Multi-Timescale Tick Event Engine
event_dir = BASE / "event_driven"
if event_dir.exists():
    print("\n=== 3. GATE C MULTI-TIMESCALE TICK ENGINE ===")
    cands = ["candidate_16b", "candidate_32b", "candidate_64b", "candidate_120b"]
    for c in cands:
        recs_file = event_dir / f"event_recs_{c}.jsonl"
        if recs_file.exists():
            n_recs = 0
            dirs = Counter()
            last_rec = None
            with open(recs_file) as f:
                for line in f:
                    if line.strip():
                        n_recs += 1
                        try:
                            r = json.loads(line)
                            dirs[r.get("signal_type", "UNKNOWN")] += 1
                            last_rec = r
                        except: pass
            last_sig = last_rec.get("signal_type") if last_rec else "N/A"
            last_trend = last_rec.get("trend_bps") if last_rec else 0.0
            print(f"  - {c}: Total Recommendations = {n_recs:,} | Breakdown: {dict(dirs)}")
            if last_rec:
                print(f"    Latest Signal: {last_sig} (Trend: {last_trend:+.1f} bps @ {last_rec.get('timestamp_iso')})")
