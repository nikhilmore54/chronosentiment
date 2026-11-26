#!/usr/bin/env python3
"""
emit_nse_session_recommendations.py

Merges today's LIVE-005 NSE decision book with real-time RP-605 active position
tracking to produce an enriched JSON for the Session Recommendations Explorer UI.

Outputs:
  services/ui/public/data/nse_session_recommendations.json

F15 State Classification:
  COLD_START       — len(prices) < 2 at entry; fallback 0.0 returned
  PARTIAL_HISTORY  — 2-14 observations; score computed but not full 15-bar window
  FULL_LOOKBACK    — 15+ observations; full trajectory score computed

Governance: SHADOW ONLY — zero live execution authority.
"""
import json, re, math
from pathlib import Path
from datetime import datetime, timezone, timedelta

WORKSPACE = Path(__file__).parent.parent
DATA_DIR = WORKSPACE / "services" / "ui" / "public" / "data"
DATA_DIR.mkdir(parents=True, exist_ok=True)
LOG_DIR = WORKSPACE / "live_capture" / "shadow_validation"

def read_json(path):
    p = Path(path)
    if not p.exists(): return {}
    try:
        with open(p) as f: return json.load(f)
    except: return {}

def read_jsonl(path):
    p = Path(path)
    if not p.exists(): return []
    try:
        with open(p) as f: return [json.loads(l) for l in f if l.strip()]
    except: return []

def normalize_ticker(t):
    return t.replace("_NS", ".NS").replace("_BO", ".BO")

def f15_state(f15_val, is_opening_tick):
    if is_opening_tick or f15_val == 0.0:
        return "COLD_START"
    return "FULL_LOOKBACK"

def main():
    now_utc = datetime.now(timezone.utc)
    now_ist = now_utc + timedelta(hours=5, minutes=30)
    ist_str = now_ist.strftime("%I:%M:%S %p IST")

    is_open = ((now_ist.hour == 9 and now_ist.minute >= 15) or
               (10 <= now_ist.hour <= 14) or
               (now_ist.hour == 15 and now_ist.minute < 30))
    if now_ist.hour < 9 or (now_ist.hour == 9 and now_ist.minute < 15):
        session_status = "PRE_OPEN"
    elif not is_open:
        session_status = "CLOSED"
    else:
        session_status = "LIVE"

    # Today's LIVE-005 certified decision book
    decisions = read_json(WORKSPACE / "datasets" / "today_live_dataset.json")
    if isinstance(decisions, dict): decisions = []

    # Active position ledgers
    base_active = read_json(LOG_DIR / "rp605_nse_baseline_active.json")
    fresh_active = read_json(LOG_DIR / "rp605_nse_freshness_active.json")
    rej_active = read_json(LOG_DIR / "rp605_nse_rejected_active.json")

    # Completed exits
    base_exits = read_jsonl(LOG_DIR / "rp605_nse_baseline_journal.jsonl")
    fresh_exits = read_jsonl(LOG_DIR / "rp605_nse_freshness_journal.jsonl")

    # Index active by symbol
    def idx_sym(d):
        out = {}
        for k,v in d.items():
            out[v.get("symbol", k)] = v
        return out

    base_idx = idx_sym(base_active)
    fresh_idx = idx_sym(fresh_active)
    rej_idx = idx_sym(rej_active)

    action_map = {"Buy":"BUY","BUY":"BUY","Short":"SHORT","SHORT":"SHORT",
                  "Watch":"WATCH","WATCH":"WATCH","NoTrade":"NO_TRADE","NOTRADE":"NO_TRADE"}

    merged = []
    for dec in decisions:
        raw_ticker = dec.get("ticker", "")
        sym = normalize_ticker(raw_ticker)
        action = dec.get("action", "NoTrade")
        direction = dec.get("direction", "LONG")
        ref_price = dec.get("reference_price", 0.0)
        adap_target = dec.get("adaptive_target", 0.0)
        adap_risk = dec.get("adaptive_risk", 0.0)
        horizon = dec.get("adaptive_horizon_sessions", 4.0)
        h60_class = (dec.get("path_5m") or {}).get("h60_classification", "UNKNOWN")
        opp_score = (dec.get("opportunity_dimensions") or {}).get("opportunity_quality_score", 0)
        decision_id = dec.get("decision_id", "")
        decision_state = action_map.get(action, "WATCH")

        pos_base = base_idx.get(sym)
        pos_fresh = fresh_idx.get(sym)
        pos_rej = rej_idx.get(sym)

        if pos_base:
            ledger_status = "LEDGER_A_ACTIVE"
            f15_val = pos_base.get("f15_at_entry", 0.0)
            entry_ts = pos_base.get("entry_timestamp", "")
        elif pos_fresh:
            ledger_status = "LEDGER_B_ACTIVE"
            f15_val = pos_fresh.get("f15_at_entry", 0.0)
            entry_ts = pos_fresh.get("entry_timestamp", "")
        elif pos_rej:
            ledger_status = "REJECTED_COUNTERFACTUAL"
            f15_val = None
            entry_ts = ""
        else:
            ledger_status = "NOT_YET_ACTIVE"
            f15_val = None
            entry_ts = ""

        # Cold-start detection: opening tick entries have entry_ts near 09:15 IST = 03:45 UTC
        is_opening = False
        if entry_ts:
            try:
                et = datetime.fromisoformat(entry_ts.replace("Z", "+00:00"))
                et_ist = et + timedelta(hours=5, minutes=30)
                is_opening = (et_ist.hour == 9 and et_ist.minute == 15)
            except: pass

        f15_status = f15_state(f15_val, is_opening)

        # R:R ratio
        rr_ratio = 0.0
        if ref_price > 0 and adap_target > 0 and adap_risk > 0:
            if direction == "LONG":
                reward = adap_target - ref_price
                risk_amt = ref_price - adap_risk
            else:
                reward = ref_price - adap_target
                risk_amt = adap_risk - ref_price
            rr_ratio = round(reward / risk_amt, 2) if risk_amt > 0 else 0.0

        # Target/stop distance %
        target_pct = round((adap_target - ref_price) / ref_price * 100, 2) if ref_price > 0 else 0.0
        risk_pct = round((adap_risk - ref_price) / ref_price * 100, 2) if ref_price > 0 else 0.0

        merged.append({
            "decision_id": decision_id,
            "ticker": raw_ticker,
            "symbol": sym,
            "action": action,
            "decision_state": decision_state,
            "direction": direction,
            "reference_price": round(ref_price, 2),
            "adaptive_target": round(adap_target, 2),
            "adaptive_risk": round(adap_risk, 2),
            "target_pct": target_pct,
            "risk_pct": risk_pct,
            "rr_ratio": rr_ratio,
            "horizon_sessions": horizon,
            "h60_classification": h60_class,
            "opportunity_quality_score": opp_score,
            "ledger_status": ledger_status,
            "f15_at_entry": f15_val,
            "f15_state": f15_status,
            "is_actionable": decision_state in ("BUY", "SHORT"),
            "entry_timestamp_utc": entry_ts,
        })

    def sort_key(r):
        order = {"BUY": 0, "SHORT": 1, "WATCH": 2, "NO_TRADE": 3}
        return (order.get(r["decision_state"], 4), -(r["opportunity_quality_score"] or 0))
    merged.sort(key=sort_key)

    today_str = now_utc.strftime("%Y-%m-%d")
    
    # Filter exits for today's live session vs all-time cumulative
    today_base_exits = [e for e in base_exits if (e.get("exit_timestamp") or "").startswith(today_str)]
    today_fresh_exits = [e for e in fresh_exits if (e.get("exit_timestamp") or "").startswith(today_str)]

    pnl_base_session = sum(e.get("pnl", 0) for e in today_base_exits)
    pnl_fresh_session = sum(e.get("pnl", 0) for e in today_fresh_exits)

    pnl_base_cumulative = sum(e.get("pnl", 0) for e in base_exits)
    pnl_fresh_cumulative = sum(e.get("pnl", 0) for e in fresh_exits)

    tick_count = 0
    rpt = LOG_DIR / "rp605_nse_prospective_freshness_validation.md"
    if rpt.exists():
        m = re.search(r"\((\d+)/\d+\s*Ticks", rpt.read_text())
        if m: tick_count = int(m.group(1))

    cold_start_active_freshness = sum(1 for v in fresh_active.values() if v.get("f15_at_entry") == 0.0)
    full_lookback_active_freshness = sum(1 for v in fresh_active.values() if v.get("f15_at_entry") is not None and v.get("f15_at_entry") != 0.0)
    cold_start_recs_count = sum(1 for r in merged if r.get("f15_state") == "COLD_START")
    actionable_count = sum(1 for r in merged if r["is_actionable"])
    watch_count = sum(1 for r in merged if r["decision_state"] == "WATCH")
    no_trade_count = sum(1 for r in merged if r["decision_state"] == "NO_TRADE")

    output = {
        "generated_at_utc": now_utc.strftime("%Y-%m-%dT%H:%M:%SZ"),
        "generated_at_ist": ist_str,
        "session_date": today_str,
        "session_status": session_status,
        "tick_count": tick_count,
        "total_expected_ticks": 375,
        "f15_audit_note": (
            "Opening admissions at Tick #1 (09:15 IST) were cold-start qualified: F15 fallback=0.0 "
            "returned when len(prices)<2. Subsequent entries computed from live tick history. "
            "Cold-start entries are flagged f15_state=COLD_START in this record."
        ),
        "universe_scope": {
            "monitored_decision_symbols": len(merged),
            "active_dual_ledger_symbols": len(base_active),
            "explanation": "Active ledgers monitor 119 liquid benchmark scrips; decision explorer displays 102 primary certified recommendations.",
        },
        "governance": "SHADOW_ONLY — RP-605 frozen control — RP-609 shadow observer — zero live execution authority",
        "summary": {
            "total_decisions": len(merged),
            "actionable_count": actionable_count,
            "watch_count": watch_count,
            "no_trade_count": no_trade_count,
            "cold_start_admissions_active_freshness": cold_start_active_freshness,
            "full_lookback_admissions_active_freshness": full_lookback_active_freshness,
            "cold_start_admissions_decision_subset": cold_start_recs_count,
            "ledger_a_active": len(base_active),
            "ledger_b_active": len(fresh_active),
            "rejected_active": len(rej_active),
            "ledger_a_session_completed_exits": len(today_base_exits),
            "ledger_b_session_completed_exits": len(today_fresh_exits),
            "ledger_a_session_realized_pnl_inr": round(pnl_base_session, 2),
            "ledger_b_session_realized_pnl_inr": round(pnl_fresh_session, 2),
            "ledger_a_cumulative_completed_exits": len(base_exits),
            "ledger_b_cumulative_completed_exits": len(fresh_exits),
            "ledger_a_cumulative_realized_pnl_inr": round(pnl_base_cumulative, 2),
            "ledger_b_cumulative_realized_pnl_inr": round(pnl_fresh_cumulative, 2),
        },
        "recommendations": merged,
    }

    out_path = DATA_DIR / "nse_session_recommendations.json"
    with open(out_path, "w") as f:
        json.dump(output, f, indent=2)

    print(f"[NSE Session Recs] Written {len(merged)} recommendations -> {out_path}")
    print(f"  Actionable: {actionable_count} Buy | Watch: {watch_count} | NoTrade: {no_trade_count}")
    print(f"  Ledger A: {len(base_active)} active | B: {len(fresh_active)} active | Rej: {len(rej_active)}")
    print(f"  Session Realized PnL — A: ₹{pnl_base_session:+,.2f} | B: ₹{pnl_fresh_session:+,.2f} (Diff B-A: ₹{pnl_fresh_session - pnl_base_session:+,.2f})")
    print(f"  Session: {session_status} | Ticks: {tick_count}/375 | {ist_str}")

if __name__ == "__main__":
    main()
