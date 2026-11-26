#!/usr/bin/env python3
"""
scripts/run_rp608_exp003_h3_regime_audit.py

RP-608-EXP-003: Pre-Registered Regime-Conditioned Entry Audit (Gate 3A / Hypothesis H3).
Evaluated STRICTLY on In-Sample Development Data (Sessions 1–40, 2026-06-22 to 2026-08-17).
The Out-of-Sample Holdout (Sessions 41–68) remains SEALED and UNTOUCHED.

Objectives:
  1. Test whether broad-market regime filters (Market Breadth, Market Direction)
     improve qualifying hit rate (Net PnL >= +1.0 R after all statutory costs)
     from the baseline 3.06% towards the 90.0% target.
  2. Enforce the Minimum 20% Opportunity Coverage constraint.
  3. Replay with complete 10-position portfolio concurrency and 8-component statutory friction.
"""

from __future__ import annotations

from collections import defaultdict
from dataclasses import dataclass
from datetime import datetime, timedelta, timezone
import json
import math
from pathlib import Path
import sys
from typing import Any, Dict, List, Optional, Set, Tuple

WORKSPACE_DIR = Path(__file__).resolve().parent.parent
sys.path.insert(0, str(WORKSPACE_DIR))

from scripts.run_rp608_nse_shadow_daemon import (
    calc_indian_equity_friction,
    NSE_119_SYMBOLS,
    NSE_2026_HOLIDAYS,
)

IST_OFFSET = timedelta(hours=5, minutes=30)
DEFAULT_NOTIONAL_PER_TRADE_INR = 100000.0
TARGET_PROFIT_BPS = 150.0   # +1.50%
STOP_LOSS_BPS = 100.0       # -1.00%
DEFAULT_HALF_SPREAD_BPS = 2.5
DEFAULT_SLIPPAGE_BPS = 2.0


def open_time_min(dt_ist: datetime) -> datetime:
    return dt_ist.replace(hour=9, minute=15, second=0, microsecond=0)


@dataclass
class ReplayBar:
    timestamp_utc_ms: int
    open: float
    high: float
    low: float
    close: float
    volume: float
    session_date: str
    bar_idx: int
    interval_start_iso: str
    interval_close_iso: str


class H3RegimeHarness:
    def __init__(
        self,
        data_dir: Path,
        output_dir: Path,
        max_concurrent_trades: int = 10
    ):
        self.data_dir = Path(data_dir)
        self.output_dir = Path(output_dir)
        self.output_dir.mkdir(parents=True, exist_ok=True)
        self.max_concurrent_trades = max_concurrent_trades

        self.symbol_bars: Dict[str, Dict[str, Dict[int, ReplayBar]]] = {}
        self.all_sessions: List[str] = []
        self.is_sessions: List[str] = []

    def load_dataset(self) -> None:
        all_dates: Set[str] = set()
        for p in self.data_dir.glob("*.json"):
            sym = p.stem
            try:
                with open(p, "r", encoding="utf-8") as f:
                    raw_candles = json.load(f)
            except Exception:
                continue

            parsed = []
            for c in raw_candles:
                ts = c.get("timestamp")
                if ts is None: continue
                ts_ms = int(ts * 1000) if ts < 1e11 else int(ts)
                o = float(c.get("open", 0.0))
                h = float(c.get("high", 0.0))
                l = float(c.get("low", 0.0))
                cl = float(c.get("close", 0.0))
                v = float(c.get("volume", 0.0))
                if o <= 0 or h <= 0 or l <= 0 or cl <= 0: continue
                parsed.append({"ts_ms": ts_ms, "open": o, "high": h, "low": l, "close": cl, "volume": v})

            parsed.sort(key=lambda x: x["ts_ms"])

            seen_ts = set()
            self.symbol_bars[sym] = {}

            for b in parsed:
                if b["ts_ms"] in seen_ts: continue
                seen_ts.add(b["ts_ms"])

                dt_utc = datetime.fromtimestamp(b["ts_ms"] / 1000.0, tz=timezone.utc)
                dt_ist = dt_utc + IST_OFFSET
                date_str = str(dt_ist.date())

                if dt_ist.weekday() >= 5 or dt_ist.date() in NSE_2026_HOLIDAYS:
                    continue

                open_ist = dt_ist.replace(hour=9, minute=15, second=0, microsecond=0)
                close_ist = dt_ist.replace(hour=15, minute=30, second=0, microsecond=0)
                if dt_ist < open_ist or dt_ist >= close_ist:
                    continue

                el_min = (dt_ist - open_time_min(dt_ist)).total_seconds() / 60.0
                bar_idx = int(el_min // 15) + 1
                if not (1 <= bar_idx <= 25): continue

                c_start = open_ist + timedelta(minutes=(bar_idx - 1) * 15)
                c_close = open_ist + timedelta(minutes=bar_idx * 15)

                rb = ReplayBar(
                    timestamp_utc_ms=b["ts_ms"],
                    open=b["open"],
                    high=b["high"],
                    low=b["low"],
                    close=b["close"],
                    volume=b["volume"],
                    session_date=date_str,
                    bar_idx=bar_idx,
                    interval_start_iso=c_start.isoformat(),
                    interval_close_iso=c_close.isoformat()
                )

                all_dates.add(date_str)
                if date_str not in self.symbol_bars[sym]:
                    self.symbol_bars[sym][date_str] = {}
                self.symbol_bars[sym][date_str][bar_idx] = rb

        self.all_sessions = sorted(all_dates)
        split_idx = int(len(self.all_sessions) * 0.6)
        self.is_sessions = self.all_sessions[:split_idx]

    def compute_causal_market_regimes(self) -> Dict[Tuple[str, int], Dict[str, float]]:
        """
        Computes market breadth and market direction at each completed Bar k Close.
        Strictly causal: uses only data from 09:15 up to Bar k Close of that session.
        """
        regimes: Dict[Tuple[str, int], Dict[str, float]] = {}
        all_syms = list(self.symbol_bars.keys())

        for s in self.is_sessions:
            for b in range(1, 26):
                advancing_vs_open = 0
                advancing_vs_prev = 0
                total_active = 0
                cum_returns = []

                for sym in all_syms:
                    b1 = self.symbol_bars[sym].get(s, {}).get(1)
                    bk = self.symbol_bars[sym].get(s, {}).get(b)
                    bk_prev = self.symbol_bars[sym].get(s, {}).get(b - 1) if b > 1 else b1

                    if b1 and bk and b1.open > 0:
                        total_active += 1
                        ret_from_open = (bk.close - b1.open) / b1.open
                        cum_returns.append(ret_from_open * 10000.0)

                        if bk.close > b1.open:
                            advancing_vs_open += 1
                        if bk_prev and bk.close > bk_prev.close:
                            advancing_vs_prev += 1

                if total_active > 10:
                    breadth_sess = advancing_vs_open / total_active
                    breadth_bar = advancing_vs_prev / total_active
                    mean_ret_bps = sum(cum_returns) / total_active
                    # Dispersion (cross-sectional vol)
                    var = sum((r - mean_ret_bps) ** 2 for r in cum_returns) / total_active
                    disp_bps = math.sqrt(var)
                else:
                    breadth_sess = 0.50
                    breadth_bar = 0.50
                    mean_ret_bps = 0.0
                    disp_bps = 0.0

                regimes[(s, b)] = {
                    "session_breadth": round(breadth_sess, 4),
                    "bar_breadth": round(breadth_bar, 4),
                    "market_return_bps": round(mean_ret_bps, 2),
                    "market_dispersion_bps": round(disp_bps, 2)
                }

        return regimes

    def run_regime_replay(
        self,
        config_name: str,
        regimes: Dict[Tuple[str, int], Dict[str, float]],
        filter_fn: Any
    ) -> Dict[str, Any]:
        """
        Executes causal dynamic portfolio simulation on In-Sample Development data.
        """
        active_trades: Dict[str, Dict[str, Any]] = {}
        pending_entries: List[Dict[str, Any]] = []
        closed_trades: List[Dict[str, Any]] = []

        session_history: Dict[str, List[float]] = {sym: [] for sym in self.symbol_bars}
        current_active_session_date: Optional[str] = None
        symbols = sorted(self.symbol_bars.keys())

        eligible_opportunities = 0
        accepted_by_filter = 0

        for session_date in self.is_sessions:
            if session_date != current_active_session_date:
                session_history = {sym: [] for sym in symbols}
                current_active_session_date = session_date
                pending_entries.clear()
                active_trades.clear()

            for bar_idx in range(1, 26):
                # Phase A: Execute pending entries at Bar k Open
                if pending_entries:
                    for entry in list(pending_entries):
                        sym = entry["symbol"]
                        curr_bar = self.symbol_bars.get(sym, {}).get(session_date, {}).get(bar_idx)
                        if not curr_bar: continue

                        raw_open_p = curr_bar.open
                        entry_slip = raw_open_p * ((DEFAULT_HALF_SPREAD_BPS + DEFAULT_SLIPPAGE_BPS) / 10000.0)
                        effective_entry_p = round(raw_open_p + entry_slip, 2)
                        qty = max(1.0, math.floor(DEFAULT_NOTIONAL_PER_TRADE_INR / effective_entry_p))

                        target_p = round(effective_entry_p * (1.0 + TARGET_PROFIT_BPS / 10000.0), 2)
                        stop_p = round(effective_entry_p * (1.0 - STOP_LOSS_BPS / 10000.0), 2)

                        active_trades[sym] = {
                            "trade_id": entry["signal_id"],
                            "symbol": sym,
                            "session_date": session_date,
                            "entry_bar_idx": bar_idx,
                            "entry_price": effective_entry_p,
                            "quantity": qty,
                            "target_price": target_p,
                            "stop_price": stop_p
                        }

                    pending_entries.clear()

                # Phase B: Process exits for active trades
                for sym in list(active_trades.keys()):
                    trade = active_trades[sym]
                    curr_bar = self.symbol_bars.get(sym, {}).get(session_date, {}).get(bar_idx)
                    if not curr_bar: continue

                    bars_held = bar_idx - trade["entry_bar_idx"]
                    entry_p = trade["entry_price"]
                    tgt_p = trade["target_price"]
                    stp_p = trade["stop_price"]

                    exit_reason = None
                    raw_exit_p = None

                    # Baseline exit policy
                    if bar_idx >= 24:
                        exit_reason = "MANDATORY_INTRADAY_SQUARE_OFF"
                        raw_exit_p = curr_bar.close
                    elif curr_bar.low <= stp_p and curr_bar.high >= tgt_p:
                        exit_reason = "CONSERVATIVE_STOP_LOSS_SIMULTANEOUS"
                        raw_exit_p = stp_p
                    elif curr_bar.low <= stp_p:
                        exit_reason = "STOP_LOSS_HIT"
                        raw_exit_p = stp_p
                    elif curr_bar.high >= tgt_p:
                        exit_reason = "TARGET_PROFIT_HIT"
                        raw_exit_p = tgt_p
                    elif bars_held >= 12:
                        exit_reason = "MAX_HOLD_HORIZON_180M"
                        raw_exit_p = curr_bar.close
                    elif bars_held >= 4 and curr_bar.close > entry_p:
                        exit_reason = "PROFIT_TIMED_EXIT_60M"
                        raw_exit_p = curr_bar.close

                    if exit_reason and raw_exit_p:
                        exit_slip = raw_exit_p * ((DEFAULT_HALF_SPREAD_BPS + DEFAULT_SLIPPAGE_BPS) / 10000.0)
                        effective_exit_p = round(raw_exit_p - exit_slip, 2)
                        qty = trade["quantity"]
                        friction = calc_indian_equity_friction(
                            buy_price=entry_p,
                            sell_price=effective_exit_p,
                            quantity=qty,
                            half_spread_bps=DEFAULT_HALF_SPREAD_BPS
                        )
                        gross_pnl = (effective_exit_p - entry_p) * qty
                        net_pnl = gross_pnl - friction["total_friction"]
                        initial_risk_inr = entry_p * 0.0100 * qty
                        r_mult = net_pnl / initial_risk_inr if initial_risk_inr > 0 else 0.0

                        record = {
                            "trade_id": trade["trade_id"],
                            "symbol": sym,
                            "session_date": session_date,
                            "entry_bar_idx": trade["entry_bar_idx"],
                            "exit_bar_idx": bar_idx,
                            "bars_held": bars_held,
                            "entry_price": entry_p,
                            "exit_price": effective_exit_p,
                            "quantity": qty,
                            "gross_pnl_inr": round(gross_pnl, 2),
                            "total_friction_inr": round(friction["total_friction"], 2),
                            "net_pnl_inr": round(net_pnl, 2),
                            "initial_risk_inr": round(initial_risk_inr, 2),
                            "realized_r_multiple": round(r_mult, 4),
                            "is_winner_r1_0": bool(r_mult >= 1.0),
                            "is_winner_breakeven": bool(net_pnl > 0.0)
                        }
                        del active_trades[sym]
                        closed_trades.append(record)

                # Phase C: Evaluate signals at Bar k Close
                reg_info = regimes.get((session_date, bar_idx), {
                    "session_breadth": 0.5, "bar_breadth": 0.5, "market_return_bps": 0.0, "market_dispersion_bps": 0.0
                })

                for sym in symbols:
                    curr_bar = self.symbol_bars.get(sym, {}).get(session_date, {}).get(bar_idx)
                    if not curr_bar: continue

                    history = session_history[sym]
                    history.append(curr_bar.close)

                    # Gates G1, G2, G3 base checks
                    if not (1 <= bar_idx <= 20): continue
                    if sym in active_trades or any(p["symbol"] == sym for p in pending_entries): continue
                    if len(active_trades) + len(pending_entries) >= self.max_concurrent_trades: continue
                    if len(history) < 5: continue

                    # Trajectory calculation
                    lookback = min(len(history), 15)
                    ret = (history[-1] - history[-lookback]) / (history[-lookback] + 1e-9)
                    diffs = [(history[i] - history[i-1]) / (history[i-1] + 1e-9) for i in range(-lookback + 1, 0)]
                    vol = (sum(d*d for d in diffs) / len(diffs))**0.5 if diffs else 1e-3
                    denom = (vol * (lookback**0.5) * 2.5) + 1e-9
                    traj15 = math.tanh(ret / denom)

                    if traj15 >= 0.0500:
                        eligible_opportunities += 1
                        # Apply candidate regime filter function
                        if filter_fn(reg_info):
                            accepted_by_filter += 1
                            sig_id = f"h3_{config_name}_{session_date}_{bar_idx}_{sym}"
                            pending_entries.append({
                                "signal_id": sig_id,
                                "symbol": sym,
                                "signal_bar_idx": bar_idx
                            })

        # Calculate metrics
        n = len(closed_trades)
        wins_r1 = [t for t in closed_trades if t["is_winner_r1_0"]]
        wins_be = [t for t in closed_trades if t["is_winner_breakeven"]]
        coverage = (accepted_by_filter / eligible_opportunities) if eligible_opportunities > 0 else 0.0

        gross_pnl = sum(t["gross_pnl_inr"] for t in closed_trades)
        total_friction = sum(t["total_friction_inr"] for t in closed_trades)
        net_pnl = sum(t["net_pnl_inr"] for t in closed_trades)

        g_wins = [t for t in closed_trades if t["gross_pnl_inr"] > 0]
        g_losses = [t for t in closed_trades if t["gross_pnl_inr"] < 0]
        gp = sum(t["gross_pnl_inr"] for t in g_wins)
        gl = abs(sum(t["gross_pnl_inr"] for t in g_losses))
        gross_pf = round(gp / gl, 4) if gl > 0 else 0.0

        n_wins = [t for t in closed_trades if t["net_pnl_inr"] > 0]
        n_losses = [t for t in closed_trades if t["net_pnl_inr"] <= 0]
        np_ = sum(t["net_pnl_inr"] for t in n_wins)
        nl = abs(sum(t["net_pnl_inr"] for t in n_losses))
        net_pf = round(np_ / nl, 4) if nl > 0 else 0.0

        cum = 0.0
        peak = 0.0
        mdd = 0.0
        for t in closed_trades:
            cum += t["net_pnl_inr"]
            if cum > peak: peak = cum
            dd = peak - cum
            if dd > mdd: mdd = dd

        avg_win_inr = round(np_ / len(n_wins), 2) if n_wins else 0.0
        avg_loss_inr = round(nl / len(n_losses), 2) if n_losses else 0.0
        avg_net_pnl_inr = round(net_pnl / n, 2) if n > 0 else 0.0

        avg_ret_bps = round((net_pnl / (n * DEFAULT_NOTIONAL_PER_TRADE_INR)) * 10000.0, 2) if n > 0 else 0.0
        avg_win_bps = round((np_ / (len(n_wins) * DEFAULT_NOTIONAL_PER_TRADE_INR)) * 10000.0, 2) if n_wins else 0.0
        avg_loss_bps = round((nl / (len(n_losses) * DEFAULT_NOTIONAL_PER_TRADE_INR)) * 10000.0, 2) if n_losses else 0.0

        avg_r_mult = round(sum(t["realized_r_multiple"] for t in closed_trades) / n, 4) if n > 0 else 0.0
        avg_win_r = round(sum(t["realized_r_multiple"] for t in n_wins) / len(n_wins), 4) if n_wins else 0.0
        avg_loss_r = round(sum(t["realized_r_multiple"] for t in n_losses) / len(n_losses), 4) if n_losses else 0.0

        payoff_ratio = round(avg_win_inr / avg_loss_inr, 4) if avg_loss_inr > 0 else 0.0
        p_win = len(n_wins) / n if n > 0 else 0.0
        be_payoff = round((1.0 - p_win) / p_win, 4) if p_win > 0 else 0.0

        return {
            "config_name": config_name,
            "eligible_opportunities": eligible_opportunities,
            "accepted_by_filter": accepted_by_filter,
            "coverage_pct": round(coverage * 100.0, 2),
            "executed_trades": n,
            "qualifying_r1_0_trades": len(wins_r1),
            "qualifying_hit_rate_pct": round((len(wins_r1) / n * 100.0), 2) if n > 0 else 0.0,
            "nominal_win_rate_pct": round((len(wins_be) / n * 100.0), 2) if n > 0 else 0.0,
            "per_trade_rates": {
                "expectancy_net_pnl_inr_per_trade": avg_net_pnl_inr,
                "expectancy_net_return_bps_per_trade": avg_ret_bps,
                "expectancy_r_multiple_per_trade": avg_r_mult,
                "avg_net_winner_inr_per_win": avg_win_inr,
                "avg_net_winner_bps_per_win": avg_win_bps,
                "avg_net_winner_r_per_win": avg_win_r,
                "avg_net_loser_inr_per_loss": -avg_loss_inr,
                "avg_net_loser_bps_per_loss": -avg_loss_bps,
                "avg_net_loser_r_per_loss": avg_loss_r,
                "realized_payoff_ratio": payoff_ratio,
                "breakeven_payoff_ratio": be_payoff,
                "avg_friction_per_trade_inr": round(total_friction / n, 2) if n > 0 else 0.0
            },
            "gross_pnl_inr": round(gross_pnl, 2),
            "gross_profit_factor": gross_pf,
            "total_friction_inr": round(total_friction, 2),
            "net_pnl_inr": round(net_pnl, 2),
            "net_profit_factor": net_pf,
            "max_drawdown_inr": round(mdd, 2)
        }


def main():
    data_dir = WORKSPACE_DIR / "intraday_capture" / "yahoo_cache_15m"
    out_dir = WORKSPACE_DIR / "artifacts" / "rp608_successor_charter"
    out_dir.mkdir(parents=True, exist_ok=True)

    harness = H3RegimeHarness(data_dir=data_dir, output_dir=out_dir)
    print("Loading 15m dataset...")
    harness.load_dataset()
    print(f"Loaded {len(harness.symbol_bars)} symbols across {len(harness.all_sessions)} sessions.")
    print(f"In-Sample Development period: {len(harness.is_sessions)} sessions ({harness.is_sessions[0]} to {harness.is_sessions[-1]}).")
    print(f"Out-of-Sample Holdout period remains SEALED ({len(harness.all_sessions) - len(harness.is_sessions)} sessions).")

    print("\nComputing causal cross-sectional market regimes at each 15m interval...")
    regimes = harness.compute_causal_market_regimes()
    print(f"Reconstructed regimes for {len(regimes)} session-bar intervals.")

    # Pre-registered H3 Hypothesis Grid:
    # Testing Market Breadth, Market Direction, and Combined filters
    grid = [
        ("BASELINE_UNFILTERED", lambda r: True),
        ("H3_G1_BREADTH_GE_50", lambda r: r["session_breadth"] >= 0.50),
        ("H3_G1_BREADTH_GE_60", lambda r: r["session_breadth"] >= 0.60),
        ("H3_G1_BREADTH_BALANCED_40_60", lambda r: 0.40 <= r["session_breadth"] <= 0.60),
        ("H3_G2_MKT_RETURN_POSITIVE", lambda r: r["market_return_bps"] >= 0.0),
        ("H3_G2_MKT_RETURN_GE_20BPS", lambda r: r["market_return_bps"] >= 20.0),
        ("H3_G3_TREND_ALIGN_BREADTH50_RET0", lambda r: r["session_breadth"] >= 0.50 and r["market_return_bps"] >= 0.0),
        ("H3_G3_STRONG_ALIGN_BREADTH60_RET20", lambda r: r["session_breadth"] >= 0.60 and r["market_return_bps"] >= 20.0),
        ("H3_G4_LOW_DISPERSION_CALM", lambda r: r["market_dispersion_bps"] <= 80.0),
    ]

    results = []
    print("\n--- Running Pre-Registered Gate 3A Regime Audits on Development Data ---")
    for name, f_fn in grid:
        res = harness.run_regime_replay(name, regimes, f_fn)
        results.append(res)
        print(f"[{name}]")
        print(f"  Coverage: {res['coverage_pct']}% ({res['accepted_by_filter']}/{res['eligible_opportunities']} signals)")
        print(f"  Executed Trades: {res['executed_trades']}")
        print(f"  Qualifying Hit Rate (>= +1.0R): {res['qualifying_hit_rate_pct']}% ({res['qualifying_r1_0_trades']} trades)")
        ptr = res["per_trade_rates"]
        print(f"  Net PnL: INR {res['net_pnl_inr']:,.2f} (PF: {res['net_profit_factor']})")
        print(f"  Rates / Trade: Avg Net: INR {ptr['expectancy_net_pnl_inr_per_trade']} ({ptr['expectancy_net_return_bps_per_trade']} bps, {ptr['expectancy_r_multiple_per_trade']}R)")
        print(f"  Win Rate: {res['nominal_win_rate_pct']}% | Avg Win: +INR {ptr['avg_net_winner_inr_per_win']} (+{ptr['avg_net_winner_bps_per_win']} bps) | Avg Loss: -INR {abs(ptr['avg_net_loser_inr_per_loss'])} (-{abs(ptr['avg_net_loser_bps_per_loss'])} bps)")
        print(f"  Payoff Ratio: {ptr['realized_payoff_ratio']} (Break-even: {ptr['breakeven_payoff_ratio']}) | Max DD: INR {res['max_drawdown_inr']:,.2f}\n")

    summary_file = out_dir / "rp608_h3_regime_audit_results.json"
    with open(summary_file, "w", encoding="utf-8") as f:
        json.dump(results, f, indent=2)

    print(f"[✓] Gate 3A Regime Audit Complete. Saved to {summary_file}")


if __name__ == "__main__":
    main()
