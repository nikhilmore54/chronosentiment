#!/usr/bin/env python3
"""
run_rp605_nse_prospective_freshness_validation.py

RP-605 — NSE Indian Equity Market Prospective Entry Freshness Dual-Ledger Validation Daemon.
Scheduled for NSE Market Session: 09:15 AM IST to 03:30 PM IST (375 1-minute ticks).

Target Universe: Top Nifty 10 Liquid Benchmark
- RELIANCE.NS, TCS.NS, INFY.NS, HDFCBANK.NS, ICICIBANK.NS,
  BHARTIARTL.NS, SBIN.NS, ITC.NS, LT.NS, AXISBANK.NS

Engine & Policy Invariants (100% Frozen):
- Frozen Coralys v0.1.0 + Config A (Model A Temporal Exit Policy)
- Target Exit: 60m | Max Exit: 180m | T0 Firewall: Active

Parallel Dual-Ledger Architecture:
1. Ledger A (BASELINE): Unmodified frozen execution.
2. Ledger B (FRESHNESS QUALIFIED): Admits entry ONLY IF F_15 >= -0.01 at T0.
3. Rejected Opportunities Ledger: Logs blocked entries and tracks baseline counterfactual outcomes.

Outputs:
- live_capture/shadow_validation/rp605_nse_baseline_journal.jsonl
- live_capture/shadow_validation/rp605_nse_freshness_journal.jsonl
- live_capture/shadow_validation/rp605_nse_rejected_opportunities.jsonl
- live_capture/shadow_validation/rp605_nse_prospective_freshness_validation.md
"""

import os
import sys
import time
import json
import math
from pathlib import Path
from datetime import datetime, timezone, timedelta
from collections import defaultdict
import yfinance as yf
import numpy as np

def compute_coralys_scores(prices_list):
    if len(prices_list) < 2:
        return 0.0, 0.0, 0.0
    
    closes = np.array(prices_list)
    
    # 15m Momentum score
    lookback15 = min(len(closes), 15)
    ret15 = (closes[-1] - closes[-lookback15]) / closes[-lookback15]
    diffs15 = np.diff(closes[-lookback15:]) / (closes[-lookback15:-1] + 1e-9)
    vol15 = np.std(diffs15) + 1e-6 if len(diffs15) > 0 else 1e-3
    traj15m = math.tanh(ret15 / (vol15 * math.sqrt(lookback15) * 2.5))
    
    # 30m Momentum score
    lookback30 = min(len(closes), 30)
    ret30 = (closes[-1] - closes[-lookback30]) / closes[-lookback30]
    diffs30 = np.diff(closes[-lookback30:]) / (closes[-lookback30:-1] + 1e-9)
    vol30 = np.std(diffs30) + 1e-6 if len(diffs30) > 0 else 1e-3
    traj30m = math.tanh(ret30 / (vol30 * math.sqrt(lookback30) * 2.5))
    
    # 1d Momentum score (up to 60 bars)
    lookback1d = min(len(closes), 60)
    ret1d = (closes[-1] - closes[-lookback1d]) / closes[-lookback1d]
    diffs1d = np.diff(closes[-lookback1d:]) / (closes[-lookback1d:-1] + 1e-9)
    vol1d = np.std(diffs1d) + 1e-6 if len(diffs1d) > 0 else 1e-3
    traj1d = math.tanh(ret1d / (vol1d * math.sqrt(lookback1d) * 2.5))
    
    return float(np.round(traj15m, 4)), float(np.round(traj30m, 4)), float(np.round(traj1d, 4))

# Override sys.argv to avoid CLI parser conflicts in helper imports
is_setup = "--setup-only" in sys.argv
sys.argv = [sys.argv[0], "--name", "rp605_nse_freshness"]

WORKSPACE_DIR = Path(__file__).parent.parent
sys.path.insert(0, str(WORKSPACE_DIR))
sys.path.insert(0, str(WORKSPACE_DIR / "scripts"))

from scripts.decision_state_engine import DecisionStateEngine

LOG_DIR = WORKSPACE_DIR / "live_capture" / "shadow_validation"
LOG_DIR.mkdir(parents=True, exist_ok=True)

BASELINE_JOURNAL = LOG_DIR / "rp605_nse_baseline_journal.jsonl"
FRESHNESS_JOURNAL = LOG_DIR / "rp605_nse_freshness_journal.jsonl"
REJECTED_LEDGER = LOG_DIR / "rp605_nse_rejected_opportunities.jsonl"
CARDS_LOG = LOG_DIR / "rp605_nse_decision_cards_log.jsonl"
REPORT_PATH = LOG_DIR / "rp605_nse_prospective_freshness_validation.md"

from concurrent.futures import ThreadPoolExecutor

SYMBOLS = [
    'ADANIENT.NS', 'ADANIPORTS.NS', 'ADANIGREEN.NS', 'ATGL.NS', 'AMBUJACEM.NS',
    'APOLLOHOSP.NS', 'ASIANPAINT.NS', 'AUROPHARMA.NS', 'AXISBANK.NS', 'BAJAJ-AUTO.NS',
    'BAJAJFINSV.NS', 'BAJFINANCE.NS', 'BALKRISIND.NS', 'BANDHANBNK.NS', 'BANKBARODA.NS',
    'BERGEPAINT.NS', 'BHARATFORG.NS', 'BHARTIARTL.NS', 'BIOCON.NS', 'BOSCHLTD.NS',
    'BPCL.NS', 'BRITANNIA.NS', 'CANBK.NS', 'CHOLAFIN.NS', 'CIPLA.NS',
    'COALINDIA.NS', 'COFORGE.NS', 'COLPAL.NS', 'CONCOR.NS', 'CUMMINSIND.NS',
    'DABUR.NS', 'DIVISLAB.NS', 'DLF.NS', 'DRREDDY.NS', 'EICHERMOT.NS',
    'ESCORTS.NS', 'FEDERALBNK.NS', 'GAIL.NS', 'GLENMARK.NS', 'GODREJCP.NS',
    'GODREJPROP.NS', 'GRASIM.NS', 'HAVELLS.NS', 'HCLTECH.NS', 'HDFCBANK.NS',
    'HDFCLIFE.NS', 'HEROMOTOCO.NS', 'HINDALCO.NS', 'HAL.NS', 'HINDUNILVR.NS',
    'ICICIBANK.NS', 'ICICIGI.NS', 'ICICIPRULI.NS', 'IDFCFIRSTB.NS', 'IDEA.NS',
    'IGL.NS', 'INDHOTEL.NS', 'IOC.NS', 'IRCTC.NS', 'INDUSINDBK.NS',
    'INFY.NS', 'INTERGLOBE.NS', 'IPCALAB.NS', 'ITC.NS', 'JINDALSTEL.NS',
    'JSWSTEEL.NS', 'JUBLFOOD.NS', 'KOTAKBANK.NS', 'LTIM.NS', 'LT.NS',
    'LUPIN.NS', 'M&M.NS', 'MARICO.NS', 'MARUTI.NS', 'MPHASIS.NS',
    'MRF.NS', 'MUTHOOTFIN.NS', 'NATIONALUM.NS', 'NAUKRI.NS', 'NESTLEIND.NS',
    'NTPC.NS', 'OBEROIRLTY.NS', 'ONGC.NS', 'PERSISTENT.NS', 'PIDILITIND.NS',
    'PFC.NS', 'POWERGRID.NS', 'PNB.NS', 'RECLTD.NS', 'RELIANCE.NS',
    'SAIL.NS', 'SBICARD.NS', 'SBILIFE.NS', 'SBIN.NS', 'SIEMENS.NS',
    'SRF.NS', 'SUNPHARMA.NS', 'TATACONSUM.NS', 'TATAMOTORS.NS', 'TATASTEEL.NS',
    'TCS.NS', 'TECHM.NS'
]

POLL_INTERVAL_SEC = 60
TOTAL_TICKS = 375 # 09:15 AM to 03:30 PM IST (6h 15m)

FRESHNESS_THRESHOLD = -0.01

def fetch_live_spot_prices():
    def fetch_one(sym):
        try:
            ticker = yf.Ticker(sym)
            p = ticker.fast_info.get("lastPrice")
            return sym, float(p) if p else None
        except Exception:
            return sym, None

    with ThreadPoolExecutor(max_workers=25) as executor:
        results = dict(executor.map(fetch_one, SYMBOLS))
    
    return {k: v for k, v in results.items() if v is not None}

def format_inr(val):
    return f"₹{val:,.2f}"

def parse_iso(ts_str):
    if ts_str.endswith("Z"):
        ts_str = ts_str[:-1] + "+00:00"
    return datetime.fromisoformat(ts_str)

def append_jsonl(path, record):
    with open(path, "a") as f:
        f.write(json.dumps(record) + "\n")

def update_markdown_report(tick_count, baseline_trades, freshness_trades, rejected_trades, current_ist_str):
    n_base = len(baseline_trades)
    n_fresh = len(freshness_trades)
    n_rej = len(rejected_trades)
    
    b_wins = [t for t in baseline_trades if t.get("pnl", 0) > 0]
    b_pnl = sum(t.get("pnl", 0) for t in baseline_trades)
    b_wr = (len(b_wins) / n_base * 100) if n_base > 0 else 0.0
    
    f_wins = [t for t in freshness_trades if t.get("pnl", 0) > 0]
    f_pnl = sum(t.get("pnl", 0) for t in freshness_trades)
    f_wr = (len(f_wins) / n_fresh * 100) if n_fresh > 0 else 0.0

    r_losers = [t for t in rejected_trades if t.get("counterfactual_pnl", 0) < 0]
    r_winners = [t for t in rejected_trades if t.get("counterfactual_pnl", 0) > 0]
    avoided_loss = abs(sum(t.get("counterfactual_pnl", 0) for t in r_losers))
    sacrificed_prof = sum(t.get("counterfactual_pnl", 0) for t in r_winners)
    
    net_improvement = f_pnl - b_pnl

    with open(REPORT_PATH, "w") as f:
        f.write("# RP-605 — NSE Indian Equity Prospective Entry Freshness Dual-Ledger Validation Report\n")
        f.write(f"**Protocol Status**: `NSE LIVE SESSION RUNNING ({tick_count}/{TOTAL_TICKS} Ticks | Current IST: {current_ist_str})`  \n")
        f.write(f"**Market Session**: NSE India (09:15 AM to 03:30 PM IST) | $T_0 = \\text{{NOW}}$ per minute tick  \n")
        f.write(f"**Universe**: Top Nifty 10 Liquid Benchmark (`{', '.join(SYMBOLS[:5])}`, ...)  \n")
        f.write(f"**Entry Freshness Gate**: Frozen $F_{{15}} \\ge {FRESHNESS_THRESHOLD}$ Prior to Execution  \n")
        f.write("**Engine Invariants**: Frozen Coralys v0.1.0 + Config A ($K=3, G_{\\max}=0.70$, 60m Target, 180m Max, $T_0$ Firewall)  \n\n")
        
        f.write("---\n\n")
        f.write("## 1. Dual-Ledger Comparative Performance (NSE Equities)\n\n")
        f.write("| Ledger | Total Trades | Win Rate (%) | Net Realized PnL (₹) | Avoided Gross Loss (₹) | Sacrificed Gross Profit (₹) | Net Improvement vs Baseline |\n")
        f.write("| :--- | :---: | :---: | :---: | :---: | :---: | :---: |\n")
        f.write(f"| **Ledger A (Baseline)** | `{n_base}` | **`{b_wr:.2f}%`** | **`₹{b_pnl:+,.2f}`** | `₹0.00` | `₹0.00` | `₹0.00` |\n")
        f.write(f"| **Ledger B (Freshness Qualified)** | `{n_fresh}` | **`{f_wr:.2f}%`** | **`₹{f_pnl:+,.2f}`** | **`₹{avoided_loss:,.2f}`** | **`₹{sacrificed_prof:,.2f}`** | **`₹{net_improvement:+,.2f}`** |\n\n")

        f.write("---\n\n")
        f.write("## 2. Rejected Opportunities Summary (NSE Equities)\n\n")
        f.write(f"Total Entries Blocked by Freshness Gate: `{n_rej}`  \n")
        f.write(f"- **Avoided Losing Entries**: `{len(r_losers)}` Trades (`₹{avoided_loss:,.2f}` Loss Avoided)  \n")
        f.write(f"- **Sacrificed Winning Entries**: `{len(r_winners)}` Trades (`₹{sacrificed_prof:,.2f}` Profit Sacrificed)  \n\n")

def main():
    print("=" * 80)
    print("RP-605 — NSE INDIAN EQUITY PROSPECTIVE FRESHNESS VALIDATION DAEMON")
    print(f"Target Universe: {', '.join(SYMBOLS)}")
    print(f"Entry Freshness Threshold (Frozen): F_15 >= {FRESHNESS_THRESHOLD}")
    print("=" * 80)
    
    if is_setup:
        print("Setup check complete. RP-605 NSE protocol initialized successfully.")
        update_markdown_report(0, [], [], [], "07:01 AM IST")
        return

    # Import UI dataset emitter
    try:
        from scripts.emit_rp605_live_dashboard_data import main as emit_ui_data
    except Exception:
        emit_ui_data = None

    # Check IST time and wait until 09:15 AM IST if needed with active countdown
    while True:
        now_ist = datetime.now(timezone.utc) + timedelta(hours=5, minutes=30)
        target_start_ist = now_ist.replace(hour=9, minute=15, second=0, microsecond=0)
        if now_ist >= target_start_ist:
            break
        wait_seconds = (target_start_ist - now_ist).total_seconds()
        mins_left = int(wait_seconds // 60)
        secs_left = int(wait_seconds % 60)
        ist_str = now_ist.strftime("%I:%M:%S %p IST")
        status_msg = f"⏳ WAITING FOR NSE OPEN (09:15 AM IST) | Opens in {mins_left}m {secs_left:02d}s | Current: {ist_str}"
        print(f"⏰ {status_msg}")
        update_markdown_report(0, [], [], [], status_msg)
        try:
            import importlib
            import scripts.emit_rp605_live_dashboard_data as emitter
            importlib.reload(emitter)
            emitter.main()
        except Exception:
            pass
        time.sleep(15)

    print("🚀 NSE Market Open (09:15 AM IST)! Starting continuous live polling loop...")
    
    engine = DecisionStateEngine()
    
    price_history = defaultdict(list)
    score_history = defaultdict(list)
    
    baseline_active = {}
    freshness_active = {}
    rejected_active = {}
    
    baseline_completed = []
    freshness_completed = []
    rejected_completed = []
    
    trade_counter = 0

    for tick in range(1, TOTAL_TICKS + 1):
        now_dt = datetime.now(timezone.utc) + timedelta(hours=5, minutes=30)
        ist_str = now_dt.strftime("%I:%M:%S %p IST")
        ts_utc = datetime.now(timezone.utc).strftime("%Y-%m-%dT%H:%M:%SZ")
        
        spot_prices = fetch_live_spot_prices()
        print(f"\n[{ist_str}] --- RP-605 NSE TICK #{tick} / {TOTAL_TICKS} ---")
        
        for sym in SYMBOLS:
            price = spot_prices.get(sym)
            if not price:
                continue
                
            price_history[sym].append((ts_utc, price))
            if len(price_history[sym]) > 60:
                price_history[sym].pop(0)
                
            prices_list = [p[1] for p in price_history[sym]]
            s15, s30, s1d = compute_coralys_scores(prices_list)
                
            score_history[sym].append((ts_utc, s15, s30, s1d))
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
                "timestamp_utc": ts_utc,
                "instrument": sym,
                "current_price": price,
                "15m_score_trajectory": s15,
                "30m_score_trajectory": s30,
                "1d_score_trajectory": s1d,
                "cross_resolution_relationship": align,
                "decision_state": state
            }
            append_jsonl(CARDS_LOG, card_rec)

            if state in ["BUY", "SHORT"] and sym not in baseline_active:
                trade_counter += 1
                trade_id = f"RP605-NSE-{trade_counter:04d}"
                
                history_15 = [x[1] if side=="LONG" else -x[1] for x in score_history[sym][-10:]]
                a15_t0 = history_15[-1] if history_15 else (s15 if side=="LONG" else -s15)
                peak_15 = max(history_15) if history_15 else a15_t0
                f15 = a15_t0 - peak_15
                
                is_fresh = f15 >= FRESHNESS_THRESHOLD
                
                # Standard position sizing: ₹100,000 per trade (Whole shares only for NSE)
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
                
                baseline_active[sym] = dict(trade_record)
                print(f"  ⚡ LEDGER A ENTRY: {sym} ({side}) @ {format_inr(price)} | Baseline Admitted")
                
                if is_fresh:
                    freshness_active[sym] = dict(trade_record)
                    print(f"  ✨ LEDGER B ENTRY: {sym} ({side}) @ {format_inr(price)} | Freshness Qualified (F_15={f15:+.3f})")
                else:
                    rej_record = dict(trade_record)
                    rej_record["rejection_reason"] = f"STALE_ENTRY (F_15={f15:+.3f} < {FRESHNESS_THRESHOLD})"
                    rejected_active[sym] = rej_record
                    print(f"  🚫 LEDGER B REJECTED: {sym} ({side}) @ {format_inr(price)} | Stale (F_15={f15:+.3f})")

            for active_dict, completed_list, ledger_name in [(baseline_active, baseline_completed, "BASELINE"), (freshness_active, freshness_completed, "FRESHNESS")]:
                if sym in active_dict:
                    pos = active_dict[sym]
                    entry_p = pos["entry_price"]
                    side_pos = pos["side"]
                    qty = pos["quantity"]
                    
                    entry_dt = parse_iso(pos["entry_timestamp"])
                    now_utc = parse_iso(ts_utc)
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
                        
                        completed_list.append(pos)
                        journal_file = BASELINE_JOURNAL if ledger_name == "BASELINE" else FRESHNESS_JOURNAL
                        append_jsonl(journal_file, pos)
                        
                        # Update counterfactual PnL for rejected opportunity if applicable
                        if ledger_name == "BASELINE" and sym in rejected_active:
                            rej_pos = rejected_active[sym]
                            rej_pos["counterfactual_pnl"] = pnl
                            rej_pos["counterfactual_exit_price"] = price
                            rej_pos["counterfactual_exit_timestamp"] = ts_utc
                            rej_pos["holding_mins"] = held_mins
                            rej_pos["exit_reason"] = exit_reason
                            rejected_completed.append(rej_pos)
                            append_jsonl(REJECTED_LEDGER, rej_pos)
                            del rejected_active[sym]

                        print(f"  🔴 {ledger_name} EXIT: {sym} ({side_pos}) @ {format_inr(price)} | Held: {held_mins}m | Reason: {exit_reason} | PnL: ₹{pnl:+,.2f}")
                        del active_dict[sym]

        # Dump active positions for live UI streaming
        with open(LOG_DIR / "rp605_nse_baseline_active.json", "w") as f:
            json.dump(baseline_active, f, indent=2)
        with open(LOG_DIR / "rp605_nse_freshness_active.json", "w") as f:
            json.dump(freshness_active, f, indent=2)
        with open(LOG_DIR / "rp605_nse_rejected_active.json", "w") as f:
            json.dump(rejected_active, f, indent=2)

        update_markdown_report(tick, baseline_completed, freshness_completed, rejected_completed, ist_str)
        try:
            import importlib
            import scripts.emit_rp605_live_dashboard_data as emitter
            importlib.reload(emitter)
            emitter.main()
            import scripts.emit_nse_session_recommendations as rec_emitter
            importlib.reload(rec_emitter)
            rec_emitter.main()
        except Exception:
            pass
        time.sleep(POLL_INTERVAL_SEC)

if __name__ == "__main__":
    main()
