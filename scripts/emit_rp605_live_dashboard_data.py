#!/usr/bin/env python3
"""
emit_rp605_live_dashboard_data.py

Compiles live RP-605 Dual-Ledger Validation results for Crypto & NSE Indian Equities
and emits JSON files for Gate5PaperDashboard.tsx frontend UI.
"""

import json
from pathlib import Path
from datetime import datetime, timezone, timedelta

WORKSPACE_DIR = Path(__file__).parent.parent
LOG_DIR = WORKSPACE_DIR / "live_capture" / "shadow_validation"
DATA_DIR = WORKSPACE_DIR / "services" / "ui" / "src" / "data"
DATA_DIR.mkdir(parents=True, exist_ok=True)

import re

def read_jsonl(path):
    if not path.exists():
        return []
    with open(path) as f:
        return [json.loads(line) for line in f if line.strip()]

def get_active_tick_count(report_file, fallback_ticks):
    if report_file and report_file.exists():
        try:
            txt = report_file.read_text()
            m = re.search(r"\((\d+)/\d+\s*Ticks\)", txt)
            if m:
                return int(m.group(1))
        except Exception:
            pass
    return fallback_ticks

def read_json_dict(path):
    if not path.exists():
        return {}
    try:
        with open(path) as f:
            return json.load(f)
    except Exception:
        return {}

def compile_session(baseline_file, freshness_file, rejected_file, cards_file, report_file, session_name, base_active_file=None, fresh_active_file=None, rej_active_file=None, is_nse=False):
    completed_base = read_jsonl(baseline_file)
    completed_fresh = read_jsonl(freshness_file)
    completed_rej = read_jsonl(rejected_file)
    cards = read_jsonl(cards_file)
    
    # Map latest spot price per symbol from decision cards
    latest_prices = {}
    for c in cards:
        sym = c.get("instrument")
        p = c.get("current_price")
        if sym and p:
            latest_prices[sym] = float(p)

    active_base_dict = read_json_dict(base_active_file) if base_active_file else {}
    active_fresh_dict = read_json_dict(fresh_active_file) if fresh_active_file else {}
    active_rej_dict = read_json_dict(rej_active_file) if rej_active_file else {}

    now_utc = datetime.now(timezone.utc)
    
    prec = 2 if is_nse else 6

    active_base_trades = []
    for sym, pos in active_base_dict.items():
        entry_dt = datetime.fromisoformat(pos["entry_timestamp"].replace("Z", "+00:00"))
        held_m = int((now_utc - entry_dt).total_seconds() / 60)
        entry_p = float(pos["entry_price"])
        cur_p = latest_prices.get(sym, entry_p)
        qty = float(pos["quantity"])
        side = pos["side"]
        unrealized_pnl = (cur_p - entry_p) * qty if side == "LONG" else (entry_p - cur_p) * qty
        
        active_base_trades.append({
            **pos,
            "exit_timestamp": "ACTIVE",
            "exit_price": cur_p,
            "holding_mins": held_m,
            "exit_reason": "OPEN_POSITION",
            "pnl": round(unrealized_pnl, prec)
        })
        
    active_fresh_trades = []
    for sym, pos in active_fresh_dict.items():
        entry_dt = datetime.fromisoformat(pos["entry_timestamp"].replace("Z", "+00:00"))
        held_m = int((now_utc - entry_dt).total_seconds() / 60)
        entry_p = float(pos["entry_price"])
        cur_p = latest_prices.get(sym, entry_p)
        qty = float(pos["quantity"])
        side = pos["side"]
        unrealized_pnl = (cur_p - entry_p) * qty if side == "LONG" else (entry_p - cur_p) * qty
        
        active_fresh_trades.append({
            **pos,
            "exit_timestamp": "ACTIVE",
            "exit_price": cur_p,
            "holding_mins": held_m,
            "exit_reason": "OPEN_POSITION",
            "pnl": round(unrealized_pnl, prec)
        })

    active_rej_trades = []
    for sym, pos in active_rej_dict.items():
        entry_dt = datetime.fromisoformat(pos["entry_timestamp"].replace("Z", "+00:00"))
        held_m = int((now_utc - entry_dt).total_seconds() / 60)
        entry_p = float(pos["entry_price"])
        cur_p = latest_prices.get(sym, entry_p)
        qty = float(pos["quantity"])
        side = pos["side"]
        unrealized_pnl = (cur_p - entry_p) * qty if side == "LONG" else (entry_p - cur_p) * qty
        
        active_rej_trades.append({
            **pos,
            "exit_timestamp": "ACTIVE",
            "exit_price": cur_p,
            "holding_mins": held_m,
            "exit_reason": "OPEN_POSITION (BLOCKED)",
            "counterfactual_pnl": round(unrealized_pnl, prec)
        })

    # Deduplicate trades by trade_id to enforce 1 candidate = 1 record invariant
    def dedupe_trades(trades):
        seen = set()
        unique = []
        for t in trades:
            tid = t.get("trade_id")
            if tid:
                if tid not in seen:
                    seen.add(tid)
                    unique.append(t)
            else:
                unique.append(t)
        return unique

    all_base_trades = dedupe_trades(active_base_trades + completed_base)
    all_fresh_trades = dedupe_trades(active_fresh_trades + completed_fresh)
    all_rej_trades = dedupe_trades(active_rej_trades + completed_rej)

    # Calculate stats with strict separation between realized (closed) and unrealized (open MTM)
    def calc_stats(active_trades, completed_trades):
        wins = [t for t in completed_trades if float(t.get("pnl", 0)) > 0]
        losses = [t for t in completed_trades if float(t.get("pnl", 0)) < 0]
        gp = sum(float(t.get("pnl", 0)) for t in wins)
        gl = abs(sum(float(t.get("pnl", 0)) for t in losses))
        
        realized_net = sum(float(t.get("pnl", 0)) for t in completed_trades)
        unrealized_net = sum(float(t.get("pnl", 0)) for t in active_trades)
        total_net = realized_net + unrealized_net
        
        n_closed = len(completed_trades)
        n_open = len(active_trades)
        wr = (len(wins) / n_closed * 100) if n_closed > 0 else 0.0
        pf = (gp / gl) if gl > 0 else (999.0 if gp > 0 else 0.0)
        
        return {
            "realized_pnl": round(realized_net, prec),
            "unrealized_pnl": round(unrealized_net, prec),
            "net_pnl": round(total_net, prec), # Total MTM P&L
            "closed_trades": n_closed,
            "open_trades": n_open,
            "total_trades": n_closed + n_open,
            "completed_trades": n_closed,
            "active_trades": n_open,
            "wins": len(wins),
            "losses": len(losses),
            "win_rate": round(wr, 2),
            "gross_profit": round(gp, prec),
            "gross_loss": round(gl, prec),
            "profit_factor": round(pf, 2)
        }

    b_stats = calc_stats(active_base_trades, completed_base)
    f_stats = calc_stats(active_fresh_trades, completed_fresh)
    
    # Realized counterfactuals (Closed rejected trades ONLY)
    r_realized_losers = [t for t in completed_rej if float(t.get("counterfactual_pnl", 0)) < 0]
    r_realized_winners = [t for t in completed_rej if float(t.get("counterfactual_pnl", 0)) > 0]
    realized_avoided_loss = abs(sum(float(t.get("counterfactual_pnl", 0)) for t in r_realized_losers))
    realized_sacrificed_profit = sum(float(t.get("counterfactual_pnl", 0)) for t in r_realized_winners)
    net_realized_counterfactual = realized_avoided_loss - realized_sacrificed_profit

    # MTM Mark-to-market counterfactuals (Active open blocked positions - PENDING)
    r_mtm_losers = [t for t in active_rej_trades if float(t.get("counterfactual_pnl", 0)) < 0]
    r_mtm_winners = [t for t in active_rej_trades if float(t.get("counterfactual_pnl", 0)) > 0]
    mtm_avoided_loss = abs(sum(float(t.get("counterfactual_pnl", 0)) for t in r_mtm_losers))
    mtm_sacrificed_profit = sum(float(t.get("counterfactual_pnl", 0)) for t in r_mtm_winners)
    mtm_pending_net = mtm_avoided_loss - mtm_sacrificed_profit

    total_avoided_loss = realized_avoided_loss + mtm_avoided_loss
    total_sacrificed_profit = realized_sacrificed_profit + mtm_sacrificed_profit
    
    # Counterfactual Opportunity Improvement = Total Avoided Loss - Total Sacrificed Profit
    counterfactual_opportunity_improvement = total_avoided_loss - total_sacrificed_profit
    net_improvement = f_stats["net_pnl"] - b_stats["net_pnl"]
    net_realized_improvement = f_stats["realized_pnl"] - b_stats["realized_pnl"]
    
    fallback_ticks = len(set(c.get("timestamp_utc") for c in cards)) if cards else 0
    unique_ticks = get_active_tick_count(report_file, fallback_ticks)
    total_expected_ticks = 375 if is_nse else 720
    
    now_ist = datetime.now(timezone.utc) + timedelta(hours=5, minutes=30)
    ist_str = now_ist.strftime("%I:%M:%S %p IST")
    
    if is_nse:
        if now_ist.hour < 9 or (now_ist.hour == 9 and now_ist.minute < 15):
            status_text = f"⏳ WAITING FOR NSE OPEN (09:15 AM IST) | Current: {ist_str}"
        elif now_ist.hour > 15 or (now_ist.hour == 15 and now_ist.minute >= 30):
            status_text = f"🏁 NSE SESSION COMPLETE (03:30 PM IST)"
        else:
            status_text = f"🟢 LIVE NSE SESSION RUNNING ({unique_ticks}/{total_expected_ticks} Ticks) | {ist_str}"
    else:
        status_text = f"🟢 LIVE 24/7 CRYPTO SHADOW RUNNING ({unique_ticks}/{total_expected_ticks} Ticks)"

    return {
        "session_name": session_name,
        "is_nse": is_nse,
        "status_text": status_text,
        "unique_ticks": unique_ticks,
        "total_ticks": total_expected_ticks,
        "baseline_ledger": b_stats,
        "freshness_ledger": f_stats,
        "avoided_loss": round(total_avoided_loss, prec),
        "sacrificed_profit": round(total_sacrificed_profit, prec),
        "realized_avoided_loss": round(realized_avoided_loss, prec),
        "realized_sacrificed_profit": round(realized_sacrificed_profit, prec),
        "net_realized_counterfactual": round(net_realized_counterfactual, prec),
        "mtm_avoided_loss": round(mtm_avoided_loss, prec),
        "mtm_sacrificed_profit": round(mtm_sacrificed_profit, prec),
        "mtm_pending_net": round(mtm_pending_net, prec),
        "pending_open_count": len(active_rej_trades),
        "closed_rejected_count": len(completed_rej),
        "counterfactual_opportunity_improvement": round(counterfactual_opportunity_improvement, prec),
        "net_improvement": round(net_improvement, prec),
        "net_realized_improvement": round(net_realized_improvement, prec),
        "baseline_trades": all_base_trades[-20:], # latest 20 unique trades
        "freshness_trades": all_fresh_trades[-20:],
        "rejected_opportunities": all_rej_trades[-20:],
        "updated_at_utc": datetime.now(timezone.utc).strftime("%Y-%m-%dT%H:%M:%SZ")
    }

def main():
    crypto_data = compile_session(
        LOG_DIR / "rp605_baseline_journal.jsonl",
        LOG_DIR / "rp605_freshness_journal.jsonl",
        LOG_DIR / "rp605_rejected_opportunities.jsonl",
        LOG_DIR / "rp605_decision_cards_log.jsonl",
        LOG_DIR / "rp605_prospective_freshness_validation.md",
        "🌐 24/7 Live Crypto Dual-Ledger Validation",
        base_active_file=LOG_DIR / "rp605_baseline_active.json",
        fresh_active_file=LOG_DIR / "rp605_freshness_active.json",
        rej_active_file=LOG_DIR / "rp605_rejected_active.json",
        is_nse=False
    )
    
    nse_data = compile_session(
        LOG_DIR / "rp605_nse_baseline_journal.jsonl",
        LOG_DIR / "rp605_nse_freshness_journal.jsonl",
        LOG_DIR / "rp605_nse_rejected_opportunities.jsonl",
        LOG_DIR / "rp605_nse_decision_cards_log.jsonl",
        LOG_DIR / "rp605_nse_prospective_freshness_validation.md",
        "🇮🇳 NSE Indian Equity Dual-Ledger Validation",
        base_active_file=LOG_DIR / "rp605_nse_baseline_active.json",
        fresh_active_file=LOG_DIR / "rp605_nse_freshness_active.json",
        rej_active_file=LOG_DIR / "rp605_nse_rejected_active.json",
        is_nse=True
    )

    us_data = compile_session(
        LOG_DIR / "rp605_us_baseline_journal.jsonl",
        LOG_DIR / "rp605_us_freshness_journal.jsonl",
        LOG_DIR / "rp605_us_rejected_opportunities.jsonl",
        LOG_DIR / "rp605_us_decision_cards_log.jsonl",
        LOG_DIR / "rp605_us_prospective_freshness_validation.md",
        "🇺🇸 US Equities Dual-Ledger Validation",
        base_active_file=LOG_DIR / "rp605_us_baseline_active.json",
        fresh_active_file=LOG_DIR / "rp605_us_freshness_active.json",
        rej_active_file=LOG_DIR / "rp605_us_rejected_active.json",
        is_nse=False
    )
    
    with open(DATA_DIR / "rp605_crypto_live_shadow_results.json", "w") as f:
        json.dump(crypto_data, f, indent=2)
        
    with open(DATA_DIR / "rp605_nse_live_shadow_results.json", "w") as f:
        json.dump(nse_data, f, indent=2)

    with open(DATA_DIR / "rp605_us_live_shadow_results.json", "w") as f:
        json.dump(us_data, f, indent=2)
        
    print("Emitted RP-605 Crypto, NSE & US Equities UI dataset files to services/ui/src/data/ successfully.")

if __name__ == "__main__":
    main()
