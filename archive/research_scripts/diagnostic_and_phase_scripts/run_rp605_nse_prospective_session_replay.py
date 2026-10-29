#!/usr/bin/env python3
"""
scripts/run_rp605_nse_prospective_session_replay.py

RP-605 / RP-610 Prospective Session Replay for NSE Indian Equities.
Replays the exact frozen RP-605 Decision Engine across prospective sessions:
  - 2026-09-30 (Session 1: Wednesday, 375 intraday minutes)
  - 2026-10-01 (Session 2: Thursday, 375 intraday minutes)
  (2026-10-02 is Gandhi Jayanti, an official NSE exchange holiday).

Outputs:
  - Logs decision cards to live_capture/shadow_validation/rp605_nse_decision_cards_log.jsonl
  - Logs baseline journal trades to live_capture/shadow_validation/rp605_nse_baseline_journal.jsonl
  - Generates session replication CSVs:
      live_capture/shadow_validation/rp609_nse_20260930_replication_dataset.csv
      live_capture/shadow_validation/rp609_nse_20261001_replication_dataset.csv
"""

from __future__ import annotations

import csv
import hashlib
import json
import math
import sys
from collections import defaultdict
from datetime import datetime, timedelta, timezone
from pathlib import Path
import numpy as np

WORKSPACE = Path(__file__).resolve().parent.parent
sys.path.insert(0, str(WORKSPACE))

CACHE_DIR = WORKSPACE / "intraday_capture" / "yahoo_cache_1m"
LOG_DIR = WORKSPACE / "live_capture" / "shadow_validation"
CARDS_LOG = LOG_DIR / "rp605_nse_decision_cards_log.jsonl"
BASELINE_JOURNAL = LOG_DIR / "rp605_nse_baseline_journal.jsonl"

from scripts.run_rp605_nse_prospective_freshness_validation import SYMBOLS, compute_coralys_scores

IST = timezone(timedelta(hours=5, minutes=30))
FRESHNESS_THRESHOLD = -0.01
POSITION_CAPITAL_INR = 100000.0


def replay_session(target_date: str) -> tuple[list[dict], list[dict]]:
    """
    Replay a single trading session across all symbols in SYMBOLS.
    Returns (completed_trades, decision_cards).
    """
    print(f"\n========================================================")
    print(f"Replaying RP-605 Session: {target_date}")
    print(f"========================================================")

    # 1. Load bars for target date
    bars_by_sym_minute = defaultdict(dict)
    minutes_set = set()

    for sym in SYMBOLS:
        p = CACHE_DIR / f"{sym}.json"
        if not p.exists():
            continue
        data = json.loads(p.read_text(encoding="utf-8"))
        for b in data:
            dt = datetime.fromtimestamp(b["timestamp"], IST)
            if dt.strftime("%Y-%m-%d") == target_date:
                # Market session only: 09:15 to 15:30 IST
                if (dt.hour == 9 and dt.minute >= 15) or (9 < dt.hour < 15) or (dt.hour == 15 and dt.minute <= 30):
                    tod = dt.hour * 60 + dt.minute
                    minutes_set.add((tod, dt))
                    bars_by_sym_minute[sym][dt] = b

    sorted_minutes = [x[1] for x in sorted(list(minutes_set), key=lambda x: x[0])]
    print(f"[{target_date}] Intraday trading minutes: {len(sorted_minutes)}")
    if not sorted_minutes:
        print(f"⚠️ No bars found for {target_date}!")
        return [], []

    price_history = defaultdict(list)
    score_history = defaultdict(list)
    active_positions = {}
    completed_trades = []
    decision_cards = []
    trade_counter = 0

    for minute_idx, current_dt in enumerate(sorted_minutes, 1):
        # Entry timestamp simulated with 15-second latency after bar close
        ts_utc_dt = current_dt.astimezone(timezone.utc) + timedelta(seconds=15)
        ts_utc_str = ts_utc_dt.strftime("%Y-%m-%dT%H:%M:%SZ")

        for sym in SYMBOLS:
            bar = bars_by_sym_minute[sym].get(current_dt)
            if not bar:
                continue
            price = float(bar["close"])

            price_history[sym].append((ts_utc_str, price))
            if len(price_history[sym]) > 60:
                price_history[sym].pop(0)

            prices_list = [p[1] for p in price_history[sym]]
            s15, s30, s1d = compute_coralys_scores(prices_list)

            score_history[sym].append((ts_utc_str, s15, s30, s1d))
            if len(score_history[sym]) > 30:
                score_history[sym].pop(0)

            if s15 > 0.15 and s30 > 0.10:
                state, side, align = "BUY", "LONG", "UNANIMOUS CONTINUATION"
            elif s15 < -0.15 and s30 < -0.10:
                state, side, align = "SHORT", "SHORT", "UNANIMOUS CONTINUATION"
            elif abs(s15) < 0.05:
                state, side, align = "PULLBACK", "NONE", "INTRADAY ALIGNMENT"
            else:
                state, side, align = "WATCH", "NONE", "DIVERGENT CONFLICT"

            card_rec = {
                "timestamp_utc": ts_utc_str,
                "instrument": sym,
                "current_price": price,
                "15m_score_trajectory": s15,
                "30m_score_trajectory": s30,
                "1d_score_trajectory": s1d,
                "cross_resolution_relationship": align,
                "decision_state": state
            }
            decision_cards.append(card_rec)

            # Check entry
            if state in ["BUY", "SHORT"] and sym not in active_positions:
                trade_counter += 1
                trade_id = f"RP605-NSE-{trade_counter:04d}"

                history_15 = [x[1] if side == "LONG" else -x[1] for x in score_history[sym][-10:]]
                a15_t0 = history_15[-1] if history_15 else (s15 if side == "LONG" else -s15)
                peak_15 = max(history_15) if history_15 else a15_t0
                f15 = a15_t0 - peak_15
                is_fresh = f15 >= FRESHNESS_THRESHOLD

                quantity = max(1, math.floor(POSITION_CAPITAL_INR / price))

                trade_record = {
                    "trade_id": trade_id,
                    "symbol": sym,
                    "side": side,
                    "entry_timestamp": ts_utc_str,
                    "entry_price": price,
                    "quantity": quantity,
                    "f15_at_entry": f15,
                    "is_fresh": is_fresh,
                    "_max_pnl": 0.0,
                    "_entry_dt": ts_utc_dt,
                }
                active_positions[sym] = trade_record

            # Check active position update & exit
            if sym in active_positions:
                pos = active_positions[sym]
                entry_p = pos["entry_price"]
                side_pos = pos["side"]
                qty = pos["quantity"]

                cur_pnl = (price - entry_p) * qty if side_pos == "LONG" else (entry_p - price) * qty
                if cur_pnl > pos["_max_pnl"]:
                    pos["_max_pnl"] = cur_pnl

                held_mins = int((ts_utc_dt - pos["_entry_dt"]).total_seconds() / 60)

                should_exit = False
                exit_reason = None

                if state in ["EXIT", "PULLBACK", "SHORT" if side_pos == "LONG" else "BUY"]:
                    should_exit = True
                    exit_reason = "ENGINE_EXIT"
                elif held_mins >= 60:
                    should_exit = True
                    exit_reason = "TARGET_EXIT"

                if should_exit:
                    pos["exit_timestamp"] = ts_utc_str
                    pos["exit_price"] = price
                    pos["holding_mins"] = held_mins
                    pos["exit_reason"] = exit_reason
                    pos["pnl"] = cur_pnl
                    pos["mfe_pnl"] = pos["_max_pnl"]
                    completed_trades.append(pos)
                    del active_positions[sym]

    # Session close: exit any positions still open at final minute
    last_dt = sorted_minutes[-1]
    last_ts_str = (last_dt.astimezone(timezone.utc) + timedelta(seconds=15)).strftime("%Y-%m-%dT%H:%M:%SZ")
    for sym, pos in list(active_positions.items()):
        bar = bars_by_sym_minute[sym].get(last_dt)
        px = float(bar["close"]) if bar else pos["entry_price"]
        qty = pos["quantity"]
        pnl = (px - pos["entry_price"]) * qty if pos["side"] == "LONG" else (pos["entry_price"] - px) * qty
        if pnl > pos["_max_pnl"]:
            pos["_max_pnl"] = pnl
        held_mins = int(((last_dt.astimezone(timezone.utc) + timedelta(seconds=15)) - pos["_entry_dt"]).total_seconds() / 60)
        pos["exit_timestamp"] = last_ts_str
        pos["exit_price"] = px
        pos["holding_mins"] = held_mins
        pos["exit_reason"] = "ENGINE_EXIT"
        pos["pnl"] = pnl
        pos["mfe_pnl"] = pos["_max_pnl"]
        completed_trades.append(pos)
        del active_positions[sym]

    # Clean internal temporary keys
    for t in completed_trades:
        t.pop("_max_pnl", None)
        t.pop("_entry_dt", None)

    # Sort chronologically by entry timestamp
    completed_trades.sort(key=lambda x: x["entry_timestamp"])

    n_tot = len(completed_trades)
    n_wins = sum(1 for t in completed_trades if t["pnl"] > 0)
    net_pnl = sum(t["pnl"] for t in completed_trades)
    gross_gains = sum(t["pnl"] for t in completed_trades if t["pnl"] > 0)
    gross_losses = sum(abs(t["pnl"]) for t in completed_trades if t["pnl"] < 0)
    pf = gross_gains / gross_losses if gross_losses > 0 else 0.0

    print(f"[{target_date}] Trades completed: {n_tot}")
    print(f"[{target_date}] Win Rate: {n_wins}/{n_tot} ({n_wins/n_tot*100:.2f}%)")
    print(f"[{target_date}] Net Realized P&L: ₹{net_pnl:+,.2f} (PF: {pf:.4f})")
    print(f"[{target_date}] Decision Cards Generated: {len(decision_cards)}")

    return completed_trades, decision_cards


def build_replication_csv(date_str: str, trades: list[dict], out_path: Path):
    """Write standardized replication CSV for the session."""
    fieldnames = [
        "trade_id", "symbol", "side", "t_entry_utc", "t_exit_utc",
        "feature_entry_price", "feature_quantity", "feature_f15_at_entry",
        "feature_is_fresh", "label_realized_pnl", "label_mfe_pnl", "label_zero_mfe"
    ]
    with out_path.open("w", newline="", encoding="utf-8") as f:
        writer = csv.DictWriter(f, fieldnames=fieldnames)
        writer.writeheader()
        for t in trades:
            mfe = t.get("mfe_pnl", max(0.0, t["pnl"]))
            writer.writerow({
                "trade_id": t["trade_id"],
                "symbol": t["symbol"],
                "side": t["side"],
                "t_entry_utc": t["entry_timestamp"],
                "t_exit_utc": t["exit_timestamp"],
                "feature_entry_price": t["entry_price"],
                "feature_quantity": t["quantity"],
                "feature_f15_at_entry": t["f15_at_entry"],
                "feature_is_fresh": t["is_fresh"],
                "label_realized_pnl": t["pnl"],
                "label_mfe_pnl": mfe,
                "label_zero_mfe": 1 if mfe <= 0.0 else 0,
            })
    print(f"  ✓ Written replication CSV: {out_path} ({len(trades)} rows)")


def main():
    prospective_sessions = ["2026-09-30", "2026-10-01"]
    all_prospective_trades = []
    all_prospective_cards = []

    for session_date in prospective_sessions:
        trades, cards = replay_session(session_date)
        all_prospective_trades.extend(trades)
        all_prospective_cards.extend(cards)

        # Write session replication CSV
        csv_path = LOG_DIR / f"rp609_nse_{session_date.replace('-', '')}_replication_dataset.csv"
        build_replication_csv(session_date, trades, csv_path)

    # Append to canonical decision cards log
    print(f"\nAppending {len(all_prospective_cards)} prospective decision cards to {CARDS_LOG.name}...")
    with CARDS_LOG.open("a", encoding="utf-8") as f:
        for c in all_prospective_cards:
            f.write(json.dumps(c) + "\n")

    # Append to canonical baseline journal
    print(f"Appending {len(all_prospective_trades)} prospective baseline trades to {BASELINE_JOURNAL.name}...")
    with BASELINE_JOURNAL.open("a", encoding="utf-8") as f:
        for t in all_prospective_trades:
            # Drop mfe_pnl from journal to match canonical journal schema
            rec = {k: v for k, v in t.items() if k != "mfe_pnl"}
            f.write(json.dumps(rec) + "\n")

    print("\n✓ Prospective session replay completed successfully.")


if __name__ == "__main__":
    main()
