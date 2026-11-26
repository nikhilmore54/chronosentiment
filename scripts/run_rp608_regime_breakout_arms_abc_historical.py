#!/usr/bin/env python3
"""
RP-608-REGIME-BREAKOUT-001 Gate 4: Historical Arms A/B/C research (standalone).

- NSE: Sessions 1–40 from yahoo_cache_15m (holdout 41+ sealed / never read for metrics)
- Crypto: first 40 UTC day-blocks from binance 15m archive
- Output labelled development_is / diagnostic_replay research mode
- Separate output directory from prospective shadow ledgers
- Live order authority: 0.0%
- Promotion remains INCONCLUSIVE until conjoint statistical gates pass with floors met

Does NOT touch running shadow daemons or production checkpoints.
"""

from __future__ import annotations

import argparse
import json
import math
import sys
from datetime import datetime, timezone, timedelta, date
from pathlib import Path
from typing import Any, Dict, List, Optional, Sequence, Tuple

WORKSPACE = Path(__file__).resolve().parent.parent
sys.path.insert(0, str(WORKSPACE))

from scripts.breakout_lifecycle import LifecycleState, evaluate_quote, pack_bars_for_breakout_eval
from scripts.market_regime_classifier import classify_market_regime
from scripts.nse_market_context import apply_market_chop_to_regime, compute_market_context
from scripts.rp608_arm_a_baseline import (
    NseBar,
    nse_arm_a_decide_entry,
    nse_arm_a_simulate_trade,
)
from scripts.run_rp608_regime_breakout_arms_abc import (
    holm_bonferroni,
    paired_bootstrap_p,
    profit_factor,
)

IST_OFFSET = timedelta(hours=5, minutes=30)
NSE_2026_HOLIDAYS = {
    date(2026, 1, 26), date(2026, 3, 3), date(2026, 3, 20), date(2026, 3, 26),
    date(2026, 3, 27), date(2026, 4, 2), date(2026, 4, 3), date(2026, 4, 14),
    date(2026, 5, 1), date(2026, 5, 28), date(2026, 6, 17), date(2026, 7, 2),
    date(2026, 8, 15), date(2026, 10, 2), date(2026, 10, 20),
    date(2026, 11, 8), date(2026, 11, 9), date(2026, 11, 24), date(2026, 12, 25),
}

NSE_FLOOR_SESSIONS = 40
NSE_FLOOR_OPPS = 100
CRYPTO_FLOOR_BLOCKS = 20
CRYPTO_FLOOR_OPPS = 50


def _ts_ms(ts: float) -> int:
    return int(ts * 1000) if ts < 1e11 else int(ts)


def _ist_date(ts: float) -> Optional[date]:
    dt = datetime.fromtimestamp(_ts_ms(ts) / 1000.0, tz=timezone.utc) + IST_OFFSET
    d = dt.date()
    if dt.weekday() >= 5 or d in NSE_2026_HOLIDAYS:
        return None
    return d


def _bar_idx(ts: float) -> Optional[int]:
    dt = datetime.fromtimestamp(_ts_ms(ts) / 1000.0, tz=timezone.utc) + IST_OFFSET
    open_t = dt.replace(hour=9, minute=15, second=0, microsecond=0)
    mins = (dt - open_t).total_seconds() / 60.0
    if mins < 0 or mins >= 375:
        return None
    return int(mins // 15)


def load_nse_symbol_bars(cache_dir: Path) -> Tuple[List[str], Dict[str, Dict[str, List[NseBar]]]]:
    all_dates: set = set()
    symbol_bars: Dict[str, Dict[str, List[NseBar]]] = {}
    for p in sorted(cache_dir.glob("*.json")):
        try:
            candles = json.loads(p.read_text(encoding="utf-8"))
        except Exception:
            continue
        by_sess: Dict[str, List[NseBar]] = {}
        for c in candles:
            ts = c.get("timestamp")
            if ts is None:
                continue
            d = _ist_date(float(ts))
            bi = _bar_idx(float(ts))
            if d is None or bi is None or bi < 1:
                continue
            ds = str(d)
            all_dates.add(ds)
            by_sess.setdefault(ds, []).append(
                NseBar(
                    bar_idx=bi,
                    open=float(c["open"]),
                    high=float(c["high"]),
                    low=float(c["low"]),
                    close=float(c["close"]),
                )
            )
        for ds, bars in by_sess.items():
            bars.sort(key=lambda b: b.bar_idx)
            # dedupe bar_idx keeping last
            uniq: Dict[int, NseBar] = {}
            for b in bars:
                uniq[b.bar_idx] = b
            by_sess[ds] = [uniq[k] for k in sorted(uniq)]
        if by_sess:
            symbol_bars[p.stem] = by_sess
    sessions = sorted(all_dates)
    return sessions, symbol_bars


def run_nse_abc(
    cache_dir: Path,
    max_opportunities: Optional[int] = None,
) -> Dict[str, Any]:
    sessions, symbol_bars = load_nse_symbol_bars(cache_dir)
    is_sessions = sessions[:40]
    holdout = sessions[40:]
    # Rolling completed-bar history per symbol across IS sessions only (never holdout).
    # Required because ER/VolRatio/ADX need >62 bars; a single NSE session has ~25.
    rolling: Dict[str, List[Dict[str, float]]] = {sym: [] for sym in symbol_bars}
    opportunities: List[Dict[str, Any]] = []

    for sess_i, sess in enumerate(is_sessions):
        rets_for_chop: List[float] = []
        for sym, by_sess in symbol_bars.items():
            bars = by_sess.get(sess)
            if bars and len(bars) >= 2:
                rets_for_chop.append(((bars[-1].close / bars[0].close) - 1.0) * 10_000.0)
        mctx = compute_market_context(rets_for_chop)
        print(f"[NSE] session {sess_i+1}/40 {sess} opps_so_far={len(opportunities)}", flush=True)

        for sym, by_sess in symbol_bars.items():
            bars = by_sess.get(sess)
            if not bars:
                continue
            # Intraday closes for Arm A entry rule (session-local), plus rolling hist for regime
            session_closes: List[float] = []
            for b in bars:
                bar_d = {"open": b.open, "high": b.high, "low": b.low, "close": b.close}
                rolling[sym].append(bar_d)
                if len(rolling[sym]) > 120:
                    rolling[sym] = rolling[sym][-120:]
                session_closes.append(b.close)

                if not (1 <= b.bar_idx <= 20):
                    continue
                decision = nse_arm_a_decide_entry(session_closes, bar_idx=b.bar_idx)
                if not decision.get("signal_entry"):
                    continue

                forward = [x for x in bars if x.bar_idx > b.bar_idx]
                trade = nse_arm_a_simulate_trade(b.bar_idx, b.close, forward)
                arm_a_net = float(trade["net_return_bps"])

                # Regime uses all completed bars including signal bar t.
                hist = rolling[sym]
                regime = classify_market_regime(hist)
                regime = apply_market_chop_to_regime(regime, mctx)

                mid = b.close
                half = mid * 0.00015
                ask = mid + half
                bid = mid - half
                # Breakout geometry: Donchian t-20..t-1; signal is forming/eval candle.
                lc = evaluate_quote(
                    state=LifecycleState(),
                    regime=str(regime.get("regime") or "UNAVAILABLE"),
                    bars_completed_plus_forming=pack_bars_for_breakout_eval(hist),
                    ask=ask,
                    bid=bid,
                    quote_age_ms=50.0,
                    session_date=sess,
                    symbol=sym,
                    half_spread_cap_bps=1.80,
                    target_distance=150.0,
                    stop_distance=100.0,
                    friction=12.0,
                )

                take_b = regime.get("regime") == "DIRECTIONAL"
                take_c = take_b and lc.state == "TRADE_QUALIFIED"

                def pack(taken: bool, fail_disp: str) -> Dict[str, Any]:
                    if taken:
                        return {
                            "taken": True,
                            "opp_net_bps": arm_a_net,
                            "taken_net_bps": arm_a_net,
                            "outcome_source": "taken",
                            "disposition": "taken",
                        }
                    return {
                        "taken": False,
                        "opp_net_bps": arm_a_net,
                        "taken_net_bps": None,
                        "outcome_source": "arm_a_counterfactual",
                        "disposition": fail_disp,
                    }

                opportunities.append({
                    "opportunity_id": f"NSE|{sess}|{sym}|{b.bar_idx}|LONG",
                    "market": "NSE",
                    "session_date": sess,
                    "symbol": sym,
                    "bar_idx": b.bar_idx,
                    "arm_a_net_bps": arm_a_net,
                    "arm_a_exit_reason": trade["exit_reason"],
                    "arm_a_holding_bars": trade["holding_bars"],
                    "regime": regime.get("regime"),
                    "market_chop": bool(mctx.get("market_chop")),
                    "breakout_state": lc.state,
                    "arm_b": pack(take_b, "regime_reject"),
                    "arm_c": pack(take_c, "breakout_reject" if take_b else "regime_reject"),
                })
                if max_opportunities and len(opportunities) >= max_opportunities:
                    return _summarize_market(
                        market="NSE",
                        opportunities=opportunities,
                        is_blocks=is_sessions,
                        holdout_blocks=holdout,
                        floor_blocks=NSE_FLOOR_SESSIONS,
                        floor_opps=NSE_FLOOR_OPPS,
                        cache_path=str(cache_dir),
                    )

    return _summarize_market(
        market="NSE",
        opportunities=opportunities,
        is_blocks=is_sessions,
        holdout_blocks=holdout,
        floor_blocks=NSE_FLOOR_SESSIONS,
        floor_opps=NSE_FLOOR_OPPS,
        cache_path=str(cache_dir),
    )


def run_crypto_abc(
    path: Path,
    max_opportunities: Optional[int] = None,
) -> Dict[str, Any]:
    if not path.exists():
        return {
            "market": "CRYPTO_BTCUSDT",
            "error": "DATA_MISSING",
            "promotion": "INCONCLUSIVE",
            "sample_feasibility": "INCONCLUSIVE",
        }
    candles = json.loads(path.read_text(encoding="utf-8"))
    by_day: Dict[str, List[dict]] = {}
    for c in candles:
        ts = c.get("open_time_ms") or c.get("close_time_ms")
        if ts is None:
            continue
        day = datetime.fromtimestamp(int(ts) / 1000.0, tz=timezone.utc).strftime("%Y-%m-%d")
        by_day.setdefault(day, []).append(c)
    days = sorted(by_day.keys())
    is_days = days[:40]
    holdout = days[40:68]
    is_set = set(is_days)

    from scripts.rp608_arm_a_baseline import crypto_arm_a_simulate_trade

    resolved: List[Dict[str, Any]] = []
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
    print(f"[CRYPTO] scanning {len(flat)} bars; IS days={len(is_days)}", flush=True)
    for i, (d, cl, bar) in enumerate(flat):
        closes_roll.append(cl)
        hist_roll.append(bar)
        if len(hist_roll) > 120:
            hist_roll = hist_roll[-120:]
            closes_roll = closes_roll[-120:]
        if d not in is_set or len(closes_roll) < 20:
            continue
        ret = (closes_roll[-1] - closes_roll[-2]) / (closes_roll[-2] + 1e-9)
        window = closes_roll[-15:]
        rets = [(window[j] - window[j - 1]) / (window[j - 1] + 1e-9) for j in range(1, len(window))]
        atr_proxy = sum(abs(r) for r in rets) / max(len(rets), 1)
        if abs(ret) < max(0.0005, 0.5 * atr_proxy):
            continue
        side = "LONG" if ret > 0 else "SHORT"
        fwd = [flat[j][1] for j in range(i + 1, min(i + 1 + 8, len(flat)))]
        if not fwd:
            continue
        trade = crypto_arm_a_simulate_trade(side, cl, fwd)
        arm_a_net = float(trade["net_return_bps"])
        regime = classify_market_regime(hist_roll)
        mid = cl
        half = mid * 0.0002
        lc = evaluate_quote(
            state=LifecycleState(),
            regime=str(regime.get("regime") or "UNAVAILABLE"),
            bars_completed_plus_forming=pack_bars_for_breakout_eval(hist_roll),
            ask=mid + half,
            bid=mid - half,
            quote_age_ms=50.0,
            session_date=d,
            symbol="BTCUSDT",
            half_spread_cap_bps=2.50,
            target_distance=150.0,
            stop_distance=100.0,
            friction=12.0,
        )
        take_b = regime.get("regime") == "DIRECTIONAL"
        take_c = take_b and lc.state == "TRADE_QUALIFIED"

        def pack(taken: bool, fail_disp: str) -> Dict[str, Any]:
            if taken:
                return {
                    "taken": True,
                    "opp_net_bps": arm_a_net,
                    "taken_net_bps": arm_a_net,
                    "outcome_source": "taken",
                    "disposition": "taken",
                }
            return {
                "taken": False,
                "opp_net_bps": arm_a_net,
                "taken_net_bps": None,
                "outcome_source": "arm_a_counterfactual",
                "disposition": fail_disp,
            }

        resolved.append({
            "opportunity_id": f"CRYPTO|{d}|BTCUSDT|{i}|{side}",
            "market": "CRYPTO_BTCUSDT",
            "session_date": d,
            "symbol": "BTCUSDT",
            "side": side,
            "arm_a_net_bps": arm_a_net,
            "arm_a_exit_reason": trade["exit_reason"],
            "arm_a_holding_bars": trade["holding_bars"],
            "regime": regime.get("regime"),
            "breakout_state": lc.state,
            "arm_b": pack(take_b, "regime_reject"),
            "arm_c": pack(take_c, "breakout_reject" if take_b else "regime_reject"),
        })
        if max_opportunities and len(resolved) >= max_opportunities:
            break

    return _summarize_market(
        market="CRYPTO_BTCUSDT",
        opportunities=resolved,
        is_blocks=is_days,
        holdout_blocks=holdout,
        floor_blocks=CRYPTO_FLOOR_BLOCKS,
        floor_opps=CRYPTO_FLOOR_OPPS,
        cache_path=str(path),
    )


def _summarize_market(
    *,
    market: str,
    opportunities: List[Dict[str, Any]],
    is_blocks: List[str],
    holdout_blocks: List[str],
    floor_blocks: int,
    floor_opps: int,
    cache_path: str,
) -> Dict[str, Any]:
    n = len(opportunities)
    a_nets = [o["arm_a_net_bps"] for o in opportunities]
    # Audit trail: rejected opportunities keep Arm A counterfactual in opp_net_bps.
    b_opp_audit = [o["arm_b"]["opp_net_bps"] for o in opportunities]
    c_opp_audit = [o["arm_c"]["opp_net_bps"] for o in opportunities]
    # Economic book for paired deltas: taken => realized path; reject/stand-aside => 0.
    # (Using counterfactual for both arms would force delta≡0 and hide filter value.)
    b_econ = [
        (o["arm_b"]["taken_net_bps"] if o["arm_b"]["taken"] else 0.0)
        for o in opportunities
    ]
    c_econ = [
        (o["arm_c"]["taken_net_bps"] if o["arm_c"]["taken"] else 0.0)
        for o in opportunities
    ]
    delta_b = [b - a for a, b in zip(a_nets, b_econ)]
    delta_c = [c - a for a, c in zip(a_nets, c_econ)]
    delta_cb = [c - b for b, c in zip(b_econ, c_econ)]

    taken_b = [o["arm_b"]["taken_net_bps"] for o in opportunities if o["arm_b"]["taken"]]
    taken_c = [o["arm_c"]["taken_net_bps"] for o in opportunities if o["arm_c"]["taken"]]

    p_b = paired_bootstrap_p(delta_b)
    p_c_inc = paired_bootstrap_p(delta_cb)
    p_c_a = paired_bootstrap_p(delta_c)
    holm = holm_bonferroni([p_b, p_c_inc, p_c_a])

    floors_ok = len(is_blocks) >= floor_blocks and n >= floor_opps
    cov_b = (sum(1 for o in opportunities if o["arm_b"]["taken"]) / n) if n else 0.0
    cov_c = (sum(1 for o in opportunities if o["arm_c"]["taken"]) / n) if n else 0.0

    # Conjoint promotion — remain INCONCLUSIVE unless all gates pass
    promotion = "INCONCLUSIVE"
    reasons = []
    if not floors_ok:
        reasons.append("eligibility_floors_unmet_or_borderline_check")
    if n == 0:
        reasons.append("zero_opportunities")
        promotion = "INCONCLUSIVE"
    else:
        # Explicit: historical IS development is not prospective validation
        reasons.append("development_is_partition_not_prospective_validation")
        # Check numeric gates but still label development_is
        exp_b_econ = sum(b_econ) / n
        exp_c_econ = sum(c_econ) / n
        pf_b = profit_factor(taken_b) if taken_b else None
        pf_c = profit_factor(taken_c) if taken_c else None
        gate_notes = {
            "arm_b_econ_expectancy_positive": exp_b_econ > 0,
            "arm_c_econ_expectancy_positive": exp_c_econ > 0,
            "arm_b_taken_pf_ge_1_30": (pf_b == float("inf") or (pf_b is not None and pf_b >= 1.30)),
            "arm_c_taken_pf_ge_1_30": (pf_c == float("inf") or (pf_c is not None and pf_c >= 1.30)),
            "coverage_b_ge_20": cov_b >= 0.20,
            "coverage_c_ge_20": cov_c >= 0.20,
            "holm_any_reject": any(h["reject"] for h in holm),
            "note": "Paired deltas use economic book (reject=0). opp_net_bps retains Arm A counterfactual audit trail.",
        }
        # Never promote from this runner automatically — always INCONCLUSIVE until explicit unlock
        promotion = "INCONCLUSIVE"
        reasons.append("auto_promotion_disabled_pending_manual_review")
        reasons.append(json.dumps(gate_notes))

    return {
        "market": market,
        "label": "development_is",
        "mode": "diagnostic_replay_research",
        "live_order_authority": "0.0%",
        "holdout_status": "SEALED_UNTOUCHED",
        "cache_path": cache_path,
        "is_blocks_count": len(is_blocks),
        "is_block_start": is_blocks[0] if is_blocks else None,
        "is_block_end": is_blocks[-1] if is_blocks else None,
        "holdout_blocks_count": len(holdout_blocks),
        "holdout_block_start": holdout_blocks[0] if holdout_blocks else None,
        "holdout_block_end": holdout_blocks[-1] if holdout_blocks else None,
        "n_universe": n,
        "eligibility_floors": {
            "min_blocks": floor_blocks,
            "min_opportunities": floor_opps,
            "blocks_ok": len(is_blocks) >= floor_blocks,
            "opportunities_ok": n >= floor_opps,
            "floors_met": floors_ok,
        },
        "arm_a": {
            "opp_expectancy_bps": round(sum(a_nets) / n, 4) if n else None,
            "profit_factor": profit_factor(a_nets),
        },
        "arm_b": {
            "n_taken": len(taken_b),
            "coverage": round(cov_b, 4),
            "opp_expectancy_bps_audit_counterfactual": round(sum(b_opp_audit) / n, 4) if n else None,
            "econ_expectancy_bps": round(sum(b_econ) / n, 4) if n else None,
            "taken_expectancy_bps": round(sum(taken_b) / len(taken_b), 4) if taken_b else None,
            "mean_delta_vs_a_econ": round(sum(delta_b) / n, 4) if n else None,
            "profit_factor_taken": profit_factor(taken_b) if taken_b else None,
            "bootstrap_p_H_B": p_b,
        },
        "arm_c": {
            "n_taken": len(taken_c),
            "coverage": round(cov_c, 4),
            "opp_expectancy_bps_audit_counterfactual": round(sum(c_opp_audit) / n, 4) if n else None,
            "econ_expectancy_bps": round(sum(c_econ) / n, 4) if n else None,
            "taken_expectancy_bps": round(sum(taken_c) / len(taken_c), 4) if taken_c else None,
            "mean_delta_vs_a_econ": round(sum(delta_c) / n, 4) if n else None,
            "mean_delta_vs_b_econ": round(sum(delta_cb) / n, 4) if n else None,
            "profit_factor_taken": profit_factor(taken_c) if taken_c else None,
            "bootstrap_p_H_C_inc": p_c_inc,
            "bootstrap_p_H_C_vs_A": p_c_a,
        },
        "holm_bonferroni": {"family": ["H_B", "H_C_inc", "H_C_vs_A"], "results": holm},
        "promotion": promotion,
        "promotion_reasons": reasons,
        # Keep opportunity rows for audit but cap in summary file via caller
        "opportunities_sample": opportunities[:50],
        "opportunities_full_count": n,
    }


def main() -> None:
    parser = argparse.ArgumentParser(description="RP-608 historical Arms A/B/C research")
    parser.add_argument("--market", choices=["nse", "crypto", "both"], default="both")
    parser.add_argument("--max-opportunities", type=int, default=None)
    parser.add_argument(
        "--output-dir",
        type=Path,
        default=WORKSPACE / "artifacts" / "rp608_regime_breakout" / "historical_abc_research",
    )
    parser.add_argument(
        "--nse-cache",
        type=Path,
        default=WORKSPACE / "intraday_capture" / "yahoo_cache_15m",
    )
    parser.add_argument(
        "--crypto-path",
        type=Path,
        default=WORKSPACE / "datasets" / "crypto_15m" / "binance_btcusdt_2024_15m.json",
    )
    args = parser.parse_args()

    out_dir = args.output_dir
    out_dir.mkdir(parents=True, exist_ok=True)

    report: Dict[str, Any] = {
        "protocol": "RP-608-REGIME-BREAKOUT-001",
        "gate": "GATE4_HISTORICAL_ABC",
        "timestamp_utc": datetime.now(timezone.utc).isoformat(),
        "live_order_authority": "0.0%",
        "holdout_sessions_41_68": "SEALED_UNTOUCHED",
        "label": "development_is",
        "mode": "diagnostic_replay_research",
        "note": "Standalone research job; not prospective shadow telemetry; not live trading.",
        "markets": {},
    }

    if args.market in ("nse", "both"):
        print(f"[NSE] Running historical A/B/C on {args.nse_cache} ...")
        nse = run_nse_abc(args.nse_cache, max_opportunities=args.max_opportunities)
        report["markets"]["nse"] = nse
        # Write full opportunity ledger separately
        # Re-run is expensive; store sample only in summary. Full ledger optional.
        print(
            f"[NSE] n={nse.get('n_universe')} promotion={nse.get('promotion')} "
            f"floors={nse.get('eligibility_floors')}"
        )

    if args.market in ("crypto", "both"):
        print(f"[CRYPTO] Running historical A/B/C on {args.crypto_path} ...")
        crypto = run_crypto_abc(args.crypto_path, max_opportunities=args.max_opportunities)
        report["markets"]["crypto"] = crypto
        print(
            f"[CRYPTO] n={crypto.get('n_universe')} promotion={crypto.get('promotion')} "
            f"floors={crypto.get('eligibility_floors')}"
        )

    # Overall remains INCONCLUSIVE until both markets manually cleared
    report["overall_promotion"] = "INCONCLUSIVE"
    out_path = out_dir / "historical_abc_summary.json"
    out_path.write_text(json.dumps(report, indent=2, default=str) + "\n", encoding="utf-8")
    print(f"Wrote {out_path}")


if __name__ == "__main__":
    main()
