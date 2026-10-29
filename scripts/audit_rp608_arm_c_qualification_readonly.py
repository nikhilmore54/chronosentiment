#!/usr/bin/env python3
"""
Read-only Arm C qualification audit for RP-608-REGIME-BREAKOUT-001 Gate 4.

Traces why TRADE_QUALIFIED never appears in the historical A/B/C harness.
Does NOT change thresholds, decision logic, Arm A, or promotion criteria.
Does NOT touch holdout Sessions 41–68 or live order authority (0.0%).

Labels output: diagnostic_replay / development_is.
"""

from __future__ import annotations

import argparse
import json
import sys
from collections import Counter
from datetime import datetime, timezone
from pathlib import Path
from typing import Any, Dict, List, Optional, Tuple

WORKSPACE = Path(__file__).resolve().parent.parent
sys.path.insert(0, str(WORKSPACE))

from scripts.breakout_lifecycle import (
    LifecycleState,
    delta_buffer,
    donchian_20,
    evaluate_quote,
    half_spread_bps,
    pack_bars_for_breakout_eval,
)
from scripts.market_regime_classifier import classify_market_regime
from scripts.nse_market_context import apply_market_chop_to_regime, compute_market_context
from scripts.run_rp608_regime_breakout_arms_abc_historical import (
    load_nse_symbol_bars,
    _ts_ms,
)


def audit_quote_path(
    *,
    hist: List[Dict[str, float]],
    mid: float,
    half_frac: float,
    regime: str,
    session_date: str,
    symbol: str,
    half_spread_cap_bps: float,
    friction: float = 12.0,
    packing: str = "corrected",
) -> Dict[str, Any]:
    """
    Step through quote-vs-Donchian geometry; record blockers.

    packing:
      - legacy_gate4: signal already in hist + fake forming (Gate 4 defect)
      - corrected: prior completed + signal as forming (protocol t-20..t-1)
    """
    ask = mid * (1.0 + half_frac)
    bid = mid * (1.0 - half_frac)
    hs = half_spread_bps(ask, bid)
    if packing == "legacy_gate4":
        bars_for_lc = hist + [
            {"open": mid, "high": mid * 1.002, "low": mid * 0.999, "close": mid * 1.001}
        ]
    else:
        # hist includes signal as last completed; canonical pack excludes it from Donchian
        bars_for_lc = pack_bars_for_breakout_eval(hist)

    bounds = donchian_20(bars_for_lc, exclude_forming=True)
    blockers: List[str] = []
    details: Dict[str, Any] = {
        "mid": mid,
        "ask": ask,
        "bid": bid,
        "half_spread_bps": hs,
        "half_spread_cap_bps": half_spread_cap_bps,
        "regime": regime,
        "packing": packing,
    }

    if hs is None:
        blockers.append("invalid_book")
    elif hs > half_spread_cap_bps:
        blockers.append("spread_cap")

    if bounds is None:
        blockers.append("donchian_unavailable_need_20_completed")
        details["donchian"] = None
    else:
        upper, lower = bounds
        buf = delta_buffer(mid, ask, bid)
        details["donchian_upper"] = upper
        details["donchian_lower"] = lower
        details["delta_buffer"] = buf
        details["long_cross_level"] = upper + buf
        details["short_cross_level"] = lower - buf
        details["ask_vs_long_cross"] = ask - (upper + buf)
        details["bid_vs_short_cross"] = bid - (lower - buf)
        long_cross = ask > upper + buf
        short_cross = bid < lower - buf
        details["long_cross"] = long_cross
        details["short_cross"] = short_cross
        if not long_cross and not short_cross:
            approach_long = abs(ask - upper) / upper <= 0.0025 if upper > 0 else False
            approach_short = abs(bid - lower) / lower <= 0.0025 if lower > 0 else False
            details["approach_long"] = approach_long
            details["approach_short"] = approach_short
            if not (approach_long or approach_short):
                blockers.append("no_cross_and_no_approach")
            elif regime not in ("TRANSITIONING", "DIRECTIONAL"):
                blockers.append("approach_but_regime_blocks_watch")
            else:
                blockers.append("watch_only_no_executable_cross")

    lc = evaluate_quote(
        state=LifecycleState(),
        regime=regime,
        bars_completed_plus_forming=bars_for_lc,
        ask=ask,
        bid=bid,
        quote_age_ms=50.0,
        session_date=session_date,
        symbol=symbol,
        half_spread_cap_bps=half_spread_cap_bps,
        target_distance=150.0,
        stop_distance=100.0,
        friction=friction,
    )
    details["lifecycle_state"] = lc.state
    details["lifecycle_reason"] = lc.reason
    details["net_rr"] = lc.net_rr
    if lc.state == "INVALIDATED" and lc.reason:
        blockers.append(f"lifecycle_{lc.reason}")
    if lc.state == "TRADE_QUALIFIED":
        blockers = ["NONE_QUALIFIED"]

    return {"blockers": blockers, "details": details, "final_state": lc.state}


def audit_nse(cache_dir: Path, max_directional: int = 500) -> Dict[str, Any]:
    sessions, symbol_bars = load_nse_symbol_bars(cache_dir)
    is_sessions = sessions[:40]
    rolling: Dict[str, List[Dict[str, float]]] = {sym: [] for sym in symbol_bars}
    blocker_counts: Counter = Counter()
    regime_when_b_eligible: Counter = Counter()
    final_states: Counter = Counter()
    n_arm_a = 0
    n_directional = 0
    n_audited = 0
    examples: List[Dict[str, Any]] = []

    for sess in is_sessions:
        rets = []
        for sym, by_sess in symbol_bars.items():
            bars = by_sess.get(sess)
            if bars and len(bars) >= 2:
                rets.append(((bars[-1].close / bars[0].close) - 1.0) * 10_000.0)
        mctx = compute_market_context(rets)

        for sym, by_sess in symbol_bars.items():
            bars = by_sess.get(sess)
            if not bars:
                continue
            session_closes: List[float] = []
            for b in bars:
                bar_d = {"open": b.open, "high": b.high, "low": b.low, "close": b.close}
                rolling[sym].append(bar_d)
                if len(rolling[sym]) > 120:
                    rolling[sym] = rolling[sym][-120:]
                session_closes.append(b.close)
                if not (1 <= b.bar_idx <= 20):
                    continue
                # Same Arm A entry proxy as harness
                from scripts.rp608_arm_a_baseline import nse_arm_a_decide_entry
                if not nse_arm_a_decide_entry(session_closes, bar_idx=b.bar_idx).get("signal_entry"):
                    continue
                n_arm_a += 1
                hist = rolling[sym]
                regime = apply_market_chop_to_regime(classify_market_regime(hist), mctx)
                reg = str(regime.get("regime") or "UNAVAILABLE")
                if reg != "DIRECTIONAL":
                    continue
                n_directional += 1
                if n_audited >= max_directional:
                    continue
                n_audited += 1
                regime_when_b_eligible[reg] += 1
                mid = b.close
                # Exact harness proxy: half = mid * 0.00015
                res = audit_quote_path(
                    hist=hist,
                    mid=mid,
                    half_frac=0.00015,
                    regime=reg,
                    session_date=sess,
                    symbol=sym,
                    half_spread_cap_bps=1.80,
                    packing="corrected",
                )
                for b_ in res["blockers"]:
                    blocker_counts[b_] += 1
                final_states[res["final_state"]] += 1
                if len(examples) < 8:
                    examples.append({
                        "id": f"NSE|{sess}|{sym}|{b.bar_idx}",
                        "blockers": res["blockers"],
                        "final_state": res["final_state"],
                        "ask_vs_long_cross": res["details"].get("ask_vs_long_cross"),
                        "half_spread_bps": res["details"].get("half_spread_bps"),
                        "donchian_upper": res["details"].get("donchian_upper"),
                        "ask": res["details"].get("ask"),
                        "mid": mid,
                    })

    return {
        "market": "NSE",
        "arm_a_opportunities_scanned": n_arm_a,
        "directional_eligible_for_b": n_directional,
        "directional_audited": n_audited,
        "final_state_counts": dict(final_states),
        "blocker_counts": dict(blocker_counts),
        "examples": examples,
        "harness_quote_model": (
            "ask=mid*(1+0.00015), bid=mid*(1-0.00015); mid=bar.close; "
            "packing=corrected (prior completed + signal as forming)"
        ),
        "packing": "corrected",
        "sample_note": (
            f"At most {max_directional} B-eligible DIRECTIONAL opportunities audited; "
            "do not generalize state counts to the full opportunity universe."
        ),
        "primary_finding": _primary_finding(blocker_counts, final_states),
    }


def audit_crypto(path: Path, max_directional: int = 500) -> Dict[str, Any]:
    candles = json.loads(path.read_text(encoding="utf-8"))
    by_day: Dict[str, List[dict]] = {}
    for c in candles:
        ts = c.get("open_time_ms") or c.get("close_time_ms")
        if ts is None:
            continue
        day = datetime.fromtimestamp(int(ts) / 1000.0, tz=timezone.utc).strftime("%Y-%m-%d")
        by_day.setdefault(day, []).append(c)
    days = sorted(by_day.keys())
    is_days = set(days[:40])

    flat: List[Tuple[str, float, Dict[str, float]]] = []
    for d in days:
        for c in sorted(by_day[d], key=lambda x: x.get("open_time_ms") or 0):
            flat.append((
                d,
                float(c["close"]),
                {"open": float(c["open"]), "high": float(c["high"]), "low": float(c["low"]), "close": float(c["close"])},
            ))

    closes_roll: List[float] = []
    hist_roll: List[Dict[str, float]] = []
    blocker_counts: Counter = Counter()
    final_states: Counter = Counter()
    n_arm_a = 0
    n_directional = 0
    n_audited = 0
    examples: List[Dict[str, Any]] = []

    for i, (d, cl, bar) in enumerate(flat):
        closes_roll.append(cl)
        hist_roll.append(bar)
        if len(hist_roll) > 120:
            hist_roll = hist_roll[-120:]
            closes_roll = closes_roll[-120:]
        if d not in is_days or len(closes_roll) < 20:
            continue
        ret = (closes_roll[-1] - closes_roll[-2]) / (closes_roll[-2] + 1e-9)
        window = closes_roll[-15:]
        rets = [(window[j] - window[j - 1]) / (window[j - 1] + 1e-9) for j in range(1, len(window))]
        atr_proxy = sum(abs(r) for r in rets) / max(len(rets), 1)
        if abs(ret) < max(0.0005, 0.5 * atr_proxy):
            continue
        n_arm_a += 1
        regime = classify_market_regime(hist_roll)
        reg = str(regime.get("regime") or "UNAVAILABLE")
        if reg != "DIRECTIONAL":
            continue
        n_directional += 1
        if n_audited >= max_directional:
            continue
        n_audited += 1
        res = audit_quote_path(
            hist=hist_roll,
            mid=cl,
            half_frac=0.0002,
            regime=reg,
            session_date=d,
            symbol="BTCUSDT",
            half_spread_cap_bps=2.50,
            packing="corrected",
        )
        for b_ in res["blockers"]:
            blocker_counts[b_] += 1
        final_states[res["final_state"]] += 1
        if len(examples) < 8:
            examples.append({
                "id": f"CRYPTO|{d}|{i}",
                "side_proxy": "LONG" if ret > 0 else "SHORT",
                "blockers": res["blockers"],
                "final_state": res["final_state"],
                "ask_vs_long_cross": res["details"].get("ask_vs_long_cross"),
                "bid_vs_short_cross": res["details"].get("bid_vs_short_cross"),
                "half_spread_bps": res["details"].get("half_spread_bps"),
            })

    return {
        "market": "CRYPTO_BTCUSDT",
        "arm_a_opportunities_scanned": n_arm_a,
        "directional_eligible_for_b": n_directional,
        "directional_audited": n_audited,
        "final_state_counts": dict(final_states),
        "blocker_counts": dict(blocker_counts),
        "examples": examples,
        "harness_quote_model": (
            "ask=mid*(1+0.0002), bid=mid*(1-0.0002); mid=bar.close; "
            "packing=corrected (prior completed + signal as forming)"
        ),
        "packing": "corrected",
        "sample_note": (
            f"At most {max_directional} B-eligible DIRECTIONAL opportunities audited; "
            "do not generalize state counts to the full opportunity universe."
        ),
        "primary_finding": _primary_finding(blocker_counts, final_states),
    }


def _primary_finding(blocker_counts: Counter, final_states: Counter) -> str:
    if final_states.get("TRADE_QUALIFIED", 0) > 0:
        return "Some TRADE_QUALIFIED events observed in audit sample."
    top = blocker_counts.most_common(3)
    if not top:
        return "No directional B-eligible opportunities audited."
    lead = top[0][0]
    if lead in ("no_cross_and_no_approach", "watch_only_no_executable_cross"):
        return (
            "Executable ask/bid under the harness synthetic L1 model does not cross "
            "Donchian-20+buffer; lifecycle stays IDLE/WATCH. Specification may be intact; "
            "harness quote geometry never reaches BREAKOUT_DETECTED."
        )
    if lead.startswith("lifecycle_insufficient_rr"):
        return "Crosses occur but Net R:R < 1.0 under fixed 150/100 bps distances + friction."
    if lead == "spread_cap":
        return "Half-spread cap blocks before crossing evaluation."
    return f"Dominant blocker: {lead} ({top[0][1]} occurrences)."


def main() -> None:
    ap = argparse.ArgumentParser()
    ap.add_argument("--max-directional", type=int, default=500)
    ap.add_argument(
        "--output",
        type=Path,
        default=WORKSPACE / "artifacts/rp608_regime_breakout/arm_c_qualification_readonly_audit.json",
    )
    args = ap.parse_args()

    report = {
        "protocol": "RP-608-REGIME-BREAKOUT-001",
        "audit": "ARM_C_QUALIFICATION_READONLY",
        "timestamp_utc": datetime.now(timezone.utc).isoformat(),
        "live_order_authority": "0.0%",
        "holdout": "SEALED_UNTOUCHED",
        "label": "diagnostic_replay",
        "intent": "Explain why Arm C never TRADE_QUALIFIED — not to increase qualifications.",
        "thresholds_unchanged": True,
        "nse": audit_nse(WORKSPACE / "intraday_capture/yahoo_cache_15m", args.max_directional),
        "crypto": audit_crypto(
            WORKSPACE / "datasets/crypto_15m/binance_btcusdt_2024_15m.json",
            args.max_directional,
        ),
    }
    args.output.parent.mkdir(parents=True, exist_ok=True)
    args.output.write_text(json.dumps(report, indent=2) + "\n", encoding="utf-8")
    print(json.dumps({
        "wrote": str(args.output),
        "nse_finding": report["nse"]["primary_finding"],
        "nse_blockers": report["nse"]["blocker_counts"],
        "nse_states": report["nse"]["final_state_counts"],
        "crypto_finding": report["crypto"]["primary_finding"],
        "crypto_blockers": report["crypto"]["blocker_counts"],
        "crypto_states": report["crypto"]["final_state_counts"],
    }, indent=2))


if __name__ == "__main__":
    main()
