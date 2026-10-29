#!/usr/bin/env python3
"""
scripts/run_rp608_exp002_exit_falsification.py

RP-608-EXP-002: Controlled Exit-Policy Falsification Experiment.
Order Authority: STRICTLY 0.0% (Read-Only Offline Replay).

Hypothesis / Question:
Does an unconditional 60-minute holding period (or fixed 60m horizon with stop protection)
cure the negative expectancy of the RP-608 momentum strategy, or does it confirm that the
underlying entry signal itself has negative gross economics under realistic costs?

Variants:
  Variant A (Frozen Baseline):
    - Profit-conditioned 60m exit: bars_held >= 4 and curr_bar.close > entry_price
    - Stop loss: -1.00% (100 bps)
    - Target profit: +1.50% (150 bps)
    - Max hold horizon: 180m (12 bars)
    - Mandatory intraday square-off: Bar 24 close (15:15 IST)

  Variant B (Unconditional 60m Fixed Holding Period):
    - Strict 60m time exit: Close every position at the 60m boundary (bars_held >= 4)
      at curr_bar.close, regardless of profit or loss.
    - No early stop-loss or profit target.
    - Mandatory intraday square-off: Bar 24 close (15:15 IST).

  Variant C (Fixed 60m Horizon with Existing Stop Protection):
    - Exit at 60m boundary (bars_held >= 4) at curr_bar.close unconditionally,
      UNLESS stop-loss (-1.00%) or target (+1.50%) triggers earlier.
    - Mandatory intraday square-off: Bar 24 close (15:15 IST).

Full Portfolio Simulation Requirement:
  Both variants are simulated as complete dynamic portfolio replays with the
  10-position concurrency limit (Gate G2). Exiting positions earlier frees concurrency
  slots for subsequent signals, dynamically altering portfolio selection.
"""

from __future__ import annotations

import argparse
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
    bar_idx: int          # 1..25 in standard session
    interval_start_iso: str
    interval_close_iso: str


class ExitFalsificationHarness:
    def __init__(
        self,
        data_dir: Path,
        output_dir: Path,
        symbols: Optional[List[str]] = None,
        max_concurrent_trades: int = 10,
        half_spread_bps: float = DEFAULT_HALF_SPREAD_BPS,
        slippage_bps: float = DEFAULT_SLIPPAGE_BPS,
        target_bps: float = TARGET_PROFIT_BPS,
        stop_bps: float = STOP_LOSS_BPS
    ):
        self.data_dir = Path(data_dir)
        self.output_dir = Path(output_dir)
        self.output_dir.mkdir(parents=True, exist_ok=True)

        self.symbols = sorted(symbols or list(NSE_119_SYMBOLS))
        self.max_concurrent_trades = max_concurrent_trades
        self.half_spread_bps = half_spread_bps
        self.slippage_bps = slippage_bps
        self.target_bps = target_bps
        self.stop_bps = stop_bps

        # Preloaded and validated dataset
        self.symbol_bars: Dict[str, Dict[str, Dict[int, ReplayBar]]] = {}
        self.sorted_sessions: List[str] = []

    def load_and_validate_all_data(self) -> None:
        all_session_dates: Set[str] = set()

        for sym in self.symbols:
            json_path = self.data_dir / f"{sym}.json"
            if not json_path.exists():
                continue
            try:
                with open(json_path, "r", encoding="utf-8") as f:
                    raw_candles = json.load(f)
            except Exception:
                continue

            parsed: List[Dict[str, Any]] = []
            for c in raw_candles:
                ts = c.get("timestamp")
                if ts is None:
                    continue
                ts_ms = int(ts * 1000) if ts < 1e11 else int(ts)
                o = float(c.get("open", 0.0))
                h = float(c.get("high", 0.0))
                l = float(c.get("low", 0.0))
                cl = float(c.get("close", 0.0))
                v = float(c.get("volume", 0.0))
                if o <= 0 or h <= 0 or l <= 0 or cl <= 0:
                    continue
                parsed.append({"ts_ms": ts_ms, "open": o, "high": h, "low": l, "close": cl, "volume": v})

            parsed.sort(key=lambda x: x["ts_ms"])

            seen_ts = set()
            self.symbol_bars[sym] = {}

            for p in parsed:
                if p["ts_ms"] in seen_ts:
                    continue
                seen_ts.add(p["ts_ms"])

                dt_utc = datetime.fromtimestamp(p["ts_ms"] / 1000.0, tz=timezone.utc)
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
                if not (1 <= bar_idx <= 25):
                    continue

                c_start = open_ist + timedelta(minutes=(bar_idx - 1) * 15)
                c_close = open_ist + timedelta(minutes=bar_idx * 15)

                rb = ReplayBar(
                    timestamp_utc_ms=p["ts_ms"],
                    open=p["open"],
                    high=p["high"],
                    low=p["low"],
                    close=p["close"],
                    volume=p["volume"],
                    session_date=date_str,
                    bar_idx=bar_idx,
                    interval_start_iso=c_start.isoformat(),
                    interval_close_iso=c_close.isoformat()
                )

                all_session_dates.add(date_str)
                if date_str not in self.symbol_bars[sym]:
                    self.symbol_bars[sym][date_str] = {}
                self.symbol_bars[sym][date_str][bar_idx] = rb

        self.sorted_sessions = sorted(all_session_dates)

    def run_variant_simulation(self, variant_name: str) -> Dict[str, Any]:
        """
        Executes causal chronological replay under specified exit variant.
        Variant A: Baseline (profit-conditioned 60m exit, 180m max hold, -1.0% stop, +1.5% target)
        Variant B: Unconditional 60m exit (no stop/target, strictly exits at bars_held >= 4)
        Variant C: Fixed 60m horizon with existing stop protection (-1.0% stop, +1.5% target, strictly exits at bars_held >= 4)
        """
        active_trades: Dict[str, Dict[str, Any]] = {}
        pending_entries: List[Dict[str, Any]] = []
        closed_trades: List[Dict[str, Any]] = []

        session_history: Dict[str, List[float]] = {sym: [] for sym in self.symbols}
        current_active_session_date: Optional[str] = None

        gate_tallies = {
            "EVALUATED_BARS": 0,
            "GATE_G1_BOUNDARY_EXCLUDED": 0,
            "GATE_G2_EXISTING_POSITION_ACTIVE": 0,
            "GATE_G2_MAX_CONCURRENT_POSITIONS": 0,
            "GATE_G3_WARMUP_INSUFFICIENT": 0,
            "GATE_G3_MOMENTUM_BELOW_THRESHOLD": 0,
            "QUALIFIED_SIGNALS_ACCEPTED": 0,
            "EXECUTED_ENTRIES": 0
        }

        total_bars_processed = 0

        for session_date in self.sorted_sessions:
            if session_date != current_active_session_date:
                session_history = {sym: [] for sym in self.symbols}
                current_active_session_date = session_date
                pending_entries.clear()
                active_trades.clear()

            for bar_idx in range(1, 26):
                # ── Phase A: Causal Execution of Pending Orders at Bar k Open ──
                if pending_entries:
                    for entry in list(pending_entries):
                        sym = entry["symbol"]
                        curr_bar = self.symbol_bars.get(sym, {}).get(session_date, {}).get(bar_idx)
                        if not curr_bar:
                            continue

                        raw_open_p = curr_bar.open
                        entry_slip = raw_open_p * ((self.half_spread_bps + self.slippage_bps) / 10000.0)
                        effective_entry_p = round(raw_open_p + entry_slip, 2)
                        qty = max(1.0, math.floor(DEFAULT_NOTIONAL_PER_TRADE_INR / effective_entry_p))

                        target_p = round(effective_entry_p * (1.0 + self.target_bps / 10000.0), 2)
                        stop_p = round(effective_entry_p * (1.0 - self.stop_bps / 10000.0), 2)

                        active_trades[sym] = {
                            "trade_id": entry["signal_id"],
                            "symbol": sym,
                            "session_date": session_date,
                            "entry_bar_idx": bar_idx,
                            "entry_timestamp_iso": curr_bar.interval_start_iso,
                            "entry_price": effective_entry_p,
                            "quantity": qty,
                            "target_price": target_p,
                            "stop_price": stop_p,
                            "mfe_price": effective_entry_p,
                            "mae_price": effective_entry_p
                        }
                        gate_tallies["EXECUTED_ENTRIES"] += 1

                    pending_entries.clear()

                # ── Phase B: Process Exits for Active Trades During Bar k ──
                for sym in list(active_trades.keys()):
                    trade = active_trades[sym]
                    curr_bar = self.symbol_bars.get(sym, {}).get(session_date, {}).get(bar_idx)
                    if not curr_bar:
                        continue

                    if curr_bar.high > trade["mfe_price"]:
                        trade["mfe_price"] = curr_bar.high
                    if curr_bar.low < trade["mae_price"]:
                        trade["mae_price"] = curr_bar.low

                    bars_held = bar_idx - trade["entry_bar_idx"]
                    entry_p = trade["entry_price"]
                    tgt_p = trade["target_price"]
                    stp_p = trade["stop_price"]

                    exit_reason = None
                    raw_exit_p = None

                    # Mandatory intraday square-off at Bar 24 close (15:15 IST)
                    if bar_idx >= 24:
                        exit_reason = "MANDATORY_INTRADAY_SQUARE_OFF"
                        raw_exit_p = curr_bar.close

                    elif variant_name == "VARIANT_A_BASELINE":
                        # Conservative Resolution: simultaneous stop and target
                        if curr_bar.low <= stp_p and curr_bar.high >= tgt_p:
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

                    elif variant_name == "VARIANT_B_UNCONDITIONAL_60M":
                        # Close every surviving position at 60-minute mark unconditionally.
                        # No early stop loss or profit target.
                        if bars_held >= 4:
                            exit_reason = "UNCONDITIONAL_TIMED_EXIT_60M"
                            raw_exit_p = curr_bar.close

                    elif variant_name == "VARIANT_C_FIXED_60M_WITH_STOP":
                        # Close at 60m boundary UNLESS stop-loss or target triggers earlier.
                        if curr_bar.low <= stp_p and curr_bar.high >= tgt_p:
                            exit_reason = "CONSERVATIVE_STOP_LOSS_SIMULTANEOUS"
                            raw_exit_p = stp_p
                        elif curr_bar.low <= stp_p:
                            exit_reason = "STOP_LOSS_HIT"
                            raw_exit_p = stp_p
                        elif curr_bar.high >= tgt_p:
                            exit_reason = "TARGET_PROFIT_HIT"
                            raw_exit_p = tgt_p
                        elif bars_held >= 4:
                            exit_reason = "FIXED_HORIZON_TIMED_EXIT_60M"
                            raw_exit_p = curr_bar.close

                    if exit_reason and raw_exit_p:
                        exit_slip = raw_exit_p * ((self.half_spread_bps + self.slippage_bps) / 10000.0)
                        effective_exit_p = round(raw_exit_p - exit_slip, 2)

                        qty = trade["quantity"]
                        friction = calc_indian_equity_friction(
                            buy_price=entry_p,
                            sell_price=effective_exit_p,
                            quantity=qty,
                            half_spread_bps=self.half_spread_bps
                        )
                        gross_pnl = (effective_exit_p - entry_p) * qty
                        net_pnl = gross_pnl - friction["total_friction"]
                        net_ret_bps = ((effective_exit_p - entry_p) / entry_p) * 10000.0 - friction["effective_friction_bps"]

                        mfe_bps = ((trade["mfe_price"] - entry_p) / entry_p) * 10000.0
                        mae_bps = ((entry_p - trade["mae_price"]) / entry_p) * 10000.0

                        record = {
                            "trade_id": trade["trade_id"],
                            "symbol": sym,
                            "session_date": session_date,
                            "entry_bar_idx": trade["entry_bar_idx"],
                            "exit_bar_idx": bar_idx,
                            "bars_held": bars_held,
                            "entry_timestamp_iso": trade["entry_timestamp_iso"],
                            "exit_timestamp_iso": curr_bar.interval_close_iso,
                            "entry_price": entry_p,
                            "exit_price": effective_exit_p,
                            "quantity": qty,
                            "exit_reason": exit_reason,
                            "gross_pnl_inr": round(gross_pnl, 2),
                            "friction_breakdown": friction,
                            "total_friction_inr": round(friction["total_friction"], 2),
                            "effective_friction_bps": round(friction["effective_friction_bps"], 2),
                            "net_pnl_inr": round(net_pnl, 2),
                            "net_return_bps": round(net_ret_bps, 2),
                            "mfe_bps": round(mfe_bps, 2),
                            "mae_bps": round(mae_bps, 2),
                            "is_winner": bool(net_pnl > 0.0)
                        }

                        del active_trades[sym]
                        closed_trades.append(record)

                # ── Phase C: Causal Decision Evaluation at Bar k Close ──
                for sym in self.symbols:
                    curr_bar = self.symbol_bars.get(sym, {}).get(session_date, {}).get(bar_idx)
                    if not curr_bar:
                        continue

                    total_bars_processed += 1
                    gate_tallies["EVALUATED_BARS"] += 1
                    history = session_history[sym]
                    history.append(curr_bar.close)

                    # Compute momentum trajectory
                    if len(history) < 2:
                        traj15 = 0.0
                    else:
                        lookback = min(len(history), 15)
                        ret = (history[-1] - history[-lookback]) / (history[-lookback] + 1e-9)
                        diffs = [(history[i] - history[i-1]) / (history[i-1] + 1e-9) for i in range(-lookback + 1, 0)]
                        vol = (sum(d*d for d in diffs) / len(diffs))**0.5 if diffs else 1e-3
                        denom = (vol * (lookback**0.5) * 2.5) + 1e-9
                        traj15 = math.tanh(ret / denom)

                    # Gate G1: Intraday Boundary (new entries strictly bars 1..20)
                    if not (1 <= bar_idx <= 20):
                        gate_tallies["GATE_G1_BOUNDARY_EXCLUDED"] += 1
                    elif sym in active_trades or any(p["symbol"] == sym for p in pending_entries):
                        gate_tallies["GATE_G2_EXISTING_POSITION_ACTIVE"] += 1
                    elif len(active_trades) + len(pending_entries) >= self.max_concurrent_trades:
                        gate_tallies["GATE_G2_MAX_CONCURRENT_POSITIONS"] += 1
                    elif len(history) < 5:
                        gate_tallies["GATE_G3_WARMUP_INSUFFICIENT"] += 1
                    elif traj15 < 0.0500:
                        gate_tallies["GATE_G3_MOMENTUM_BELOW_THRESHOLD"] += 1
                    else:
                        gate_tallies["QUALIFIED_SIGNALS_ACCEPTED"] += 1
                        signal_id = f"exp002_{variant_name}_{session_date}_{bar_idx}_{sym}"
                        pending_entries.append({
                            "signal_id": signal_id,
                            "symbol": sym,
                            "signal_bar_idx": bar_idx,
                            "eval_timestamp_iso": curr_bar.interval_close_iso,
                            "traj15": traj15,
                            "prev_close": curr_bar.close
                        })

        # Save trades ledger
        trades_file = self.output_dir / f"{variant_name.lower()}_trades.jsonl"
        with open(trades_file, "w", encoding="utf-8") as f:
            for t in closed_trades:
                f.write(json.dumps(t) + "\n")

        # Compile metrics
        return self._compute_metrics(variant_name, closed_trades, gate_tallies, total_bars_processed)

    def _compute_metrics(
        self,
        variant_name: str,
        closed_trades: List[Dict[str, Any]],
        gate_tallies: Dict[str, int],
        total_bars_processed: int
    ) -> Dict[str, Any]:
        n = len(closed_trades)
        if n == 0:
            return {"variant": variant_name, "total_trades": 0}

        gross_wins = [t for t in closed_trades if t["gross_pnl_inr"] > 0]
        gross_losses = [t for t in closed_trades if t["gross_pnl_inr"] < 0]
        net_wins = [t for t in closed_trades if t["net_pnl_inr"] > 0]
        net_losses = [t for t in closed_trades if t["net_pnl_inr"] <= 0]

        gross_p = sum(t["gross_pnl_inr"] for t in gross_wins)
        gross_l = abs(sum(t["gross_pnl_inr"] for t in gross_losses))
        net_p = sum(t["net_pnl_inr"] for t in net_wins)
        net_l = abs(sum(t["net_pnl_inr"] for t in net_losses))

        gross_pnl = sum(t["gross_pnl_inr"] for t in closed_trades)
        net_pnl = sum(t["net_pnl_inr"] for t in closed_trades)
        total_friction = sum(t["total_friction_inr"] for t in closed_trades)

        # Max drawdown
        cum_pnl = 0.0
        peak = 0.0
        mdd = 0.0
        for t in closed_trades:
            cum_pnl += t["net_pnl_inr"]
            if cum_pnl > peak:
                peak = cum_pnl
            dd = peak - cum_pnl
            if dd > mdd:
                mdd = dd

        # In-sample vs Out-of-sample partition
        split_idx = int(len(self.sorted_sessions) * 0.6)
        is_sessions = set(self.sorted_sessions[:split_idx])
        oos_sessions = set(self.sorted_sessions[split_idx:])

        is_trades = [t for t in closed_trades if t["session_date"] in is_sessions]
        oos_trades = [t for t in closed_trades if t["session_date"] in oos_sessions]

        def part_stats(sub: List[Dict[str, Any]]) -> Dict[str, Any]:
            c = len(sub)
            if c == 0:
                return {}
            sw = [t for t in sub if t["net_pnl_inr"] > 0]
            sl = [t for t in sub if t["net_pnl_inr"] <= 0]
            gw = [t for t in sub if t["gross_pnl_inr"] > 0]
            gl = [t for t in sub if t["gross_pnl_inr"] < 0]
            gp = sum(t["gross_pnl_inr"] for t in gw)
            glo = abs(sum(t["gross_pnl_inr"] for t in gl))
            np_ = sum(t["net_pnl_inr"] for t in sw)
            nlo = abs(sum(t["net_pnl_inr"] for t in sl))
            return {
                "trade_count": c,
                "gross_pnl_inr": round(sum(t["gross_pnl_inr"] for t in sub), 2),
                "gross_profit_factor": round(gp / glo, 4) if glo > 0 else 0.0,
                "gross_win_rate": round(len(gw) / c, 4),
                "total_friction_inr": round(sum(t["total_friction_inr"] for t in sub), 2),
                "net_pnl_inr": round(sum(t["net_pnl_inr"] for t in sub), 2),
                "net_profit_factor": round(np_ / nlo, 4) if nlo > 0 else 0.0,
                "net_win_rate": round(len(sw) / c, 4),
                "avg_net_return_bps": round(sum(t["net_return_bps"] for t in sub) / c, 2)
            }

        # Exit reasons breakdown
        reasons_count: Dict[str, int] = {}
        reasons_pnl: Dict[str, float] = {}
        for t in closed_trades:
            r = t["exit_reason"]
            reasons_count[r] = reasons_count.get(r, 0) + 1
            reasons_pnl[r] = round(reasons_pnl.get(r, 0.0) + t["net_pnl_inr"], 2)

        return {
            "variant": variant_name,
            "total_trades": n,
            "gross_metrics": {
                "gross_pnl_inr": round(gross_pnl, 2),
                "gross_win_rate": round(len(gross_wins) / n, 4),
                "gross_profit_factor": round(gross_p / gross_l, 4) if gross_l > 0 else 0.0,
                "gross_profit_inr": round(gross_p, 2),
                "gross_loss_inr": round(-gross_l, 2),
                "avg_gross_winner_inr": round(gross_p / len(gross_wins), 2) if gross_wins else 0.0,
                "avg_gross_loser_inr": round(gross_l / len(gross_losses), 2) if gross_losses else 0.0,
                "gross_payoff_ratio": round((gross_p / len(gross_wins)) / (gross_l / len(gross_losses)), 4) if gross_wins and gross_losses else 0.0
            },
            "friction_metrics": {
                "total_friction_inr": round(total_friction, 2),
                "avg_friction_per_trade_inr": round(total_friction / n, 2)
            },
            "net_metrics": {
                "net_pnl_inr": round(net_pnl, 2),
                "net_win_rate": round(len(net_wins) / n, 4),
                "net_profit_factor": round(net_p / net_l, 4) if net_l > 0 else 0.0,
                "avg_net_return_bps": round(sum(t["net_return_bps"] for t in closed_trades) / n, 2),
                "avg_net_winner_inr": round(net_p / len(net_wins), 2) if net_wins else 0.0,
                "avg_net_loser_inr": round(net_l / len(net_losses), 2) if net_losses else 0.0,
                "net_payoff_ratio": round((net_p / len(net_wins)) / (net_l / len(net_losses)), 4) if net_wins and net_losses else 0.0,
                "max_drawdown_inr": round(mdd, 2)
            },
            "exit_reasons_breakdown": {
                "counts": reasons_count,
                "net_pnl_inr": reasons_pnl
            },
            "in_sample_dev_60pct": part_stats(is_trades),
            "out_of_sample_holdout_40pct": part_stats(oos_trades),
            "gate_tallies": gate_tallies
        }


def main():
    parser = argparse.ArgumentParser(description="RP-608-EXP-002: Exit Policy Falsification Replay")
    parser.add_argument("--data-dir", default=str(WORKSPACE_DIR / "intraday_capture" / "yahoo_cache_15m"))
    parser.add_argument("--output-dir", default=str(WORKSPACE_DIR / "artifacts" / "rp608_exp002_exit_falsification"))
    args = parser.parse_args()

    harness = ExitFalsificationHarness(
        data_dir=Path(args.data_dir),
        output_dir=Path(args.output_dir)
    )

    print("Loading and reconciling 15m dataset across NSE trading calendar...")
    harness.load_and_validate_all_data()
    print(f"Validated {len(harness.symbol_bars)} symbols across {len(harness.sorted_sessions)} trading sessions.")

    results = {}
    variants = [
        "VARIANT_A_BASELINE",
        "VARIANT_B_UNCONDITIONAL_60M",
        "VARIANT_C_FIXED_60M_WITH_STOP"
    ]

    for var in variants:
        print(f"\n--- Running Full Portfolio Replay for {var} ---")
        metrics = harness.run_variant_simulation(var)
        results[var] = metrics
        print(f"  Total Trades: {metrics['total_trades']}")
        print(f"  Gross PnL: ₹{metrics['gross_metrics']['gross_pnl_inr']:,.2f} (PF: {metrics['gross_metrics']['gross_profit_factor']:.4f})")
        print(f"  Friction: ₹{metrics['friction_metrics']['total_friction_inr']:,.2f}")
        print(f"  Net PnL: ₹{metrics['net_metrics']['net_pnl_inr']:,.2f} (PF: {metrics['net_metrics']['net_profit_factor']:.4f}, WinRate: {metrics['net_metrics']['net_win_rate']*100:.2f}%)")

    summary_file = harness.output_dir / "rp608_exp002_summary.json"
    with open(summary_file, "w", encoding="utf-8") as f:
        json.dump(results, f, indent=2)
    print(f"\nCompleted RP-608-EXP-002. Summary saved to {summary_file}")


if __name__ == "__main__":
    main()
