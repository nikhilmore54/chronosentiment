#!/usr/bin/env python3
"""
RP-608 Successor Research Programme: Gate 3B (Hypothesis H1) Experiment Runner
Protocol Reference: RP-608-EXP-004-H1-BREAKOUT-ACCEPTANCE

Evaluates Confirmed Breakout Acceptance against strictly In-Sample Development data
(Sessions 1-40, 2026-06-22 to 2026-08-17).
Out-of-sample holdout (Sessions 41-68) remains SEALED.
Live order authority remains strictly 0.0%.
"""

import sys
import json
import math
from pathlib import Path
from datetime import datetime, timezone, timedelta, date
from dataclasses import dataclass, asdict
from typing import Dict, List, Any, Optional, Tuple, Set

# Standard Constants
WORKSPACE_DIR = Path("/Users/nikhil/ChronoSentiment_MEGA_FINAL")
IST_OFFSET = timedelta(hours=5, minutes=30)
DEFAULT_NOTIONAL_PER_TRADE_INR = 100_000.0
DEFAULT_HALF_SPREAD_BPS = 2.5
DEFAULT_SLIPPAGE_BPS = 2.0
TARGET_PROFIT_BPS = 150.0  # +1.50% (+1.5R)
STOP_LOSS_BPS = 100.0      # -1.00% (-1.0R)
MAX_HOLDING_BARS = 8       # 120 minutes
SESSION_END_BAR = 25       # 15:15 IST

NSE_2026_HOLIDAYS = {
    date(2026, 1, 26), date(2026, 3, 3), date(2026, 3, 26),
    date(2026, 4, 2), date(2026, 4, 14), date(2026, 5, 1),
    date(2026, 5, 28), date(2026, 6, 17), date(2026, 7, 2),
    date(2026, 8, 15), date(2026, 10, 2), date(2026, 10, 20),
    date(2026, 11, 8), date(2026, 11, 9), date(2026, 11, 24),
    date(2026, 12, 25)
}


def open_time_min(dt: datetime) -> datetime:
    return dt.replace(hour=9, minute=15, second=0, microsecond=0)


def calculate_statutory_friction(
    buy_px: float,
    sell_px: float,
    qty: float
) -> Dict[str, float]:
    buy_turnover = buy_px * qty
    sell_turnover = sell_px * qty
    total_turnover = buy_turnover + sell_turnover

    brokerage_buy = min(20.0, buy_turnover * 0.0003)
    brokerage_sell = min(20.0, sell_turnover * 0.0003)
    total_brokerage = round(brokerage_buy + brokerage_sell, 2)

    stt = round(sell_turnover * 0.00025, 2)
    exchange_charges = round(total_turnover * 0.0000297, 2)
    sebi_charges = round(total_turnover * 0.000001, 2)
    stamp_duty = round(buy_turnover * 0.00003, 2)

    gst_base = total_brokerage + exchange_charges + sebi_charges
    cgst = round(gst_base * 0.09, 2)
    sgst = round(gst_base * 0.09, 2)
    total_gst = cgst + sgst

    total_friction = round(
        total_brokerage + stt + exchange_charges + sebi_charges + stamp_duty + total_gst,
        2
    )
    return {
        "brokerage": total_brokerage,
        "stt": stt,
        "exchange_charges": exchange_charges,
        "sebi_charges": sebi_charges,
        "stamp_duty": stamp_duty,
        "gst": total_gst,
        "total_statutory_friction": total_friction
    }


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


class H1BreakoutHarness:
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

            for b in parsed:
                if b["ts_ms"] in seen_ts:
                    continue
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
                if not (1 <= bar_idx <= 25):
                    continue

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

                if date_str not in self.symbol_bars[sym]:
                    self.symbol_bars[sym][date_str] = {}
                self.symbol_bars[sym][date_str][bar_idx] = rb
                all_dates.add(date_str)

        self.all_sessions = sorted(list(all_dates))
        # Sessions 1-40 are strictly In-Sample Development
        self.is_sessions = self.all_sessions[:40]
        print(f"Loaded {len(self.symbol_bars)} symbols across {len(self.all_sessions)} total sessions.")
        print(f"Development Partition: {len(self.is_sessions)} sessions ({self.is_sessions[0]} to {self.is_sessions[-1]}).")
        print(f"Sealed Holdout Partition: {len(self.all_sessions) - len(self.is_sessions)} sessions (UNTOUCHED).")

    def run_h1_configuration(
        self,
        config_name: str,
        boundary_type: str,     # 'ORB' or 'SWING'
        formation_bars: int,    # 4 (60m) or 2 (30m) for ORB; 8 for SWING
        required_closes: int,   # 1 or 2
        volume_mult: float,     # 1.0 or 1.25
        max_pullback_pct: float = 0.0  # low >= boundary * (1 - pct)
    ) -> Dict[str, Any]:
        """
        Executes causal dynamic portfolio simulation of Confirmed Breakout Acceptance.
        """
        active_trades: Dict[str, Dict[str, Any]] = {}
        pending_entries: List[Dict[str, Any]] = []
        closed_trades: List[Dict[str, Any]] = []

        # Track intraday breakouts per symbol
        # symbol -> { 'breakout_bar': int, 'boundary_price': float, 'closes_above': int }
        candidate_state: Dict[str, Dict[str, Any]] = {}
        traded_today: Set[str] = set()

        eligible_breakouts = 0
        confirmed_signals = 0

        symbols = sorted(self.symbol_bars.keys())

        for session_date in self.is_sessions:
            # Session boundary reset
            active_trades.clear()
            pending_entries.clear()
            candidate_state.clear()
            traded_today.clear()

            # Precompute rolling/session boundaries
            for bar_idx in range(1, 26):
                # Phase A: Execute pending entries at Bar k Open
                if pending_entries:
                    for entry in list(pending_entries):
                        sym = entry["symbol"]
                        if len(active_trades) >= self.max_concurrent_trades:
                            # Concurrency limit reached; reject signal
                            continue

                        curr_bar = self.symbol_bars.get(sym, {}).get(session_date, {}).get(bar_idx)
                        if not curr_bar:
                            continue

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
                            "entry_timestamp_iso": curr_bar.interval_start_iso,
                            "raw_entry_price": raw_open_p,
                            "effective_entry_price": effective_entry_p,
                            "quantity": qty,
                            "target_price": target_p,
                            "stop_price": stop_p,
                            "bars_held": 0,
                            "boundary_price": entry["boundary_price"]
                        }
                    pending_entries.clear()

                # Phase B: Process active positions during Bar k
                for sym in list(active_trades.keys()):
                    tr = active_trades[sym]
                    curr_bar = self.symbol_bars.get(sym, {}).get(session_date, {}).get(bar_idx)
                    if not curr_bar:
                        continue

                    tr["bars_held"] += 1
                    exit_reason: Optional[str] = None
                    raw_exit_p: float = 0.0

                    # 1. Stop-Loss check (conservative: check stop first)
                    if curr_bar.low <= tr["stop_price"]:
                        exit_reason = "STOP_LOSS_1R"
                        raw_exit_p = min(curr_bar.open, tr["stop_price"])

                    # 2. Profit-Target check
                    elif curr_bar.high >= tr["target_price"]:
                        exit_reason = "PROFIT_TARGET_1_5R"
                        raw_exit_p = max(curr_bar.open, tr["target_price"])

                    # 3. Horizon Time-Stop (8 bars / 120m)
                    elif tr["bars_held"] >= MAX_HOLDING_BARS:
                        exit_reason = "HORIZON_TIME_EXIT_8BARS"
                        raw_exit_p = curr_bar.close

                    # 4. Mandatory EOD square-off at Bar 25
                    elif bar_idx >= SESSION_END_BAR:
                        exit_reason = "SESSION_SQUARE_OFF"
                        raw_exit_p = curr_bar.close

                    if exit_reason:
                        exit_slip = raw_exit_p * ((DEFAULT_HALF_SPREAD_BPS + DEFAULT_SLIPPAGE_BPS) / 10000.0)
                        effective_exit_p = round(raw_exit_p - exit_slip, 2)

                        # Accounting
                        qty = tr["quantity"]
                        gross_pnl = (raw_exit_p - tr["raw_entry_price"]) * qty
                        slip_inr = (
                            (tr["effective_entry_price"] - tr["raw_entry_price"]) +
                            (raw_exit_p - effective_exit_p)
                        ) * qty
                        stat_friction = calculate_statutory_friction(
                            tr["effective_entry_price"], effective_exit_p, qty
                        )
                        total_friction = round(slip_inr + stat_friction["total_statutory_friction"], 2)
                        net_pnl = round(gross_pnl - total_friction, 2)

                        notional = tr["effective_entry_price"] * qty
                        net_ret_bps = round((net_pnl / notional) * 10000.0, 2)
                        gross_ret_bps = round((gross_pnl / notional) * 10000.0, 2)

                        r_value = notional * (STOP_LOSS_BPS / 10000.0)  # ~1.0R = 1.00%
                        realized_r = round(net_pnl / r_value, 4)

                        closed_trades.append({
                            "trade_id": tr["trade_id"],
                            "symbol": sym,
                            "session_date": session_date,
                            "entry_bar_idx": tr["entry_bar_idx"],
                            "exit_bar_idx": bar_idx,
                            "bars_held": tr["bars_held"],
                            "exit_reason": exit_reason,
                            "raw_entry_price": tr["raw_entry_price"],
                            "effective_entry_price": tr["effective_entry_price"],
                            "raw_exit_price": raw_exit_p,
                            "effective_exit_price": effective_exit_p,
                            "quantity": qty,
                            "gross_pnl_inr": round(gross_pnl, 2),
                            "total_friction_inr": total_friction,
                            "net_pnl_inr": net_pnl,
                            "gross_return_bps": gross_ret_bps,
                            "net_return_bps": net_ret_bps,
                            "realized_r_multiple": realized_r,
                            "is_winner_r1_0": (net_pnl >= r_value),
                            "is_winner_breakeven": (net_pnl > 0.0),
                            "boundary_price": tr["boundary_price"]
                        })
                        del active_trades[sym]

                # Phase C: Evaluate Breakout Acceptance at Bar k Close
                # Determine eligibility based on boundary model
                for sym in symbols:
                    if sym in active_trades or sym in traded_today:
                        continue

                    curr_bar = self.symbol_bars.get(sym, {}).get(session_date, {}).get(bar_idx)
                    if not curr_bar:
                        continue

                    # Construct Boundary Causally
                    boundary_p: Optional[float] = None
                    if boundary_type == "ORB":
                        if bar_idx <= formation_bars:
                            # Still forming opening range
                            continue
                        # Boundary is max High of bars 1 to formation_bars
                        prior_highs = [
                            self.symbol_bars[sym][session_date][b].high
                            for b in range(1, formation_bars + 1)
                            if b in self.symbol_bars.get(sym, {}).get(session_date, {})
                        ]
                        if len(prior_highs) == formation_bars:
                            boundary_p = max(prior_highs)
                    elif boundary_type == "SWING":
                        if bar_idx <= formation_bars:
                            continue
                        # Trailing K bars: bar_idx - formation_bars to bar_idx - 1
                        trailing_highs = [
                            self.symbol_bars[sym][session_date][b].high
                            for b in range(bar_idx - formation_bars, bar_idx)
                            if b in self.symbol_bars.get(sym, {}).get(session_date, {})
                        ]
                        if len(trailing_highs) == formation_bars:
                            boundary_p = max(trailing_highs)

                    if boundary_p is None or boundary_p <= 0.0:
                        continue

                    # Volume Average (trailing 5 completed bars)
                    v_history = [
                        self.symbol_bars[sym][session_date][b].volume
                        for b in range(max(1, bar_idx - 5), bar_idx)
                        if b in self.symbol_bars.get(sym, {}).get(session_date, {})
                    ]
                    avg_v = sum(v_history) / len(v_history) if v_history else 1.0

                    # 1. Check Breakout Event
                    if curr_bar.close > boundary_p:
                        eligible_breakouts += 1

                        # Volume condition
                        vol_confirmed = (curr_bar.volume >= avg_v * volume_mult) if volume_mult > 1.0 else True

                        # State Tracking for Multi-Bar Acceptance
                        if sym not in candidate_state:
                            candidate_state[sym] = {
                                "first_breakout_bar": bar_idx,
                                "boundary_price": boundary_p,
                                "consecutive_closes": 1,
                                "volume_confirmed": vol_confirmed
                            }
                        else:
                            candidate_state[sym]["consecutive_closes"] += 1
                            if vol_confirmed:
                                candidate_state[sym]["volume_confirmed"] = True

                        st = candidate_state[sym]
                        pullback_ok = (curr_bar.low >= boundary_p * (1.0 - max_pullback_pct))

                        # Acceptance Trigger:
                        # Required closes met AND volume confirmed AND pullback held
                        if (
                            st["consecutive_closes"] >= required_closes and
                            st["volume_confirmed"] and
                            pullback_ok
                        ):
                            confirmed_signals += 1
                            sig_id = f"h1_{config_name}_{session_date}_{bar_idx}_{sym}"
                            pending_entries.append({
                                "signal_id": sig_id,
                                "symbol": sym,
                                "signal_bar_idx": bar_idx,
                                "boundary_price": boundary_p
                            })
                            traded_today.add(sym)
                            del candidate_state[sym]
                    else:
                        # Bar closed below boundary: reset acceptance state machine
                        if sym in candidate_state:
                            del candidate_state[sym]

        # Metric Calculations
        n = len(closed_trades)
        wins_r1 = [t for t in closed_trades if t["is_winner_r1_0"]]
        wins_be = [t for t in closed_trades if t["is_winner_breakeven"]]
        coverage = (confirmed_signals / eligible_breakouts) if eligible_breakouts > 0 else 0.0

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
            if cum > peak:
                peak = cum
            dd = peak - cum
            if dd > mdd:
                mdd = dd

        avg_win_inr = round(np_ / len(n_wins), 2) if n_wins else 0.0
        avg_loss_inr = round(nl / len(n_losses), 2) if n_losses else 0.0
        avg_net_pnl_inr = round(net_pnl / n, 2) if n > 0 else 0.0
        avg_gross_pnl_inr = round(gross_pnl / n, 2) if n > 0 else 0.0

        avg_ret_bps = round((net_pnl / (n * DEFAULT_NOTIONAL_PER_TRADE_INR)) * 10000.0, 2) if n > 0 else 0.0
        avg_gross_bps = round((gross_pnl / (n * DEFAULT_NOTIONAL_PER_TRADE_INR)) * 10000.0, 2) if n > 0 else 0.0
        avg_win_bps = round((np_ / (len(n_wins) * DEFAULT_NOTIONAL_PER_TRADE_INR)) * 10000.0, 2) if n_wins else 0.0
        avg_loss_bps = round((nl / (len(n_losses) * DEFAULT_NOTIONAL_PER_TRADE_INR)) * 10000.0, 2) if n_losses else 0.0

        avg_r_mult = round(sum(t["realized_r_multiple"] for t in closed_trades) / n, 4) if n > 0 else 0.0
        avg_win_r = round(sum(t["realized_r_multiple"] for t in n_wins) / len(n_wins), 4) if n_wins else 0.0
        avg_loss_r = round(sum(t["realized_r_multiple"] for t in n_losses) / len(n_losses), 4) if n_losses else 0.0

        payoff_ratio = round(avg_win_inr / avg_loss_inr, 4) if avg_loss_inr > 0 else 0.0
        p_win = len(n_wins) / n if n > 0 else 0.0
        be_payoff = round((1.0 - p_win) / p_win, 4) if p_win > 0 else 0.0

        # Exit distribution
        exit_dist = {}
        for t in closed_trades:
            reason = t["exit_reason"]
            exit_dist[reason] = exit_dist.get(reason, 0) + 1

        return {
            "config_name": config_name,
            "boundary_type": boundary_type,
            "formation_bars": formation_bars,
            "required_closes": required_closes,
            "volume_mult": volume_mult,
            "eligible_breakouts": eligible_breakouts,
            "confirmed_signals": confirmed_signals,
            "coverage_pct": round(coverage * 100.0, 2),
            "executed_trades": n,
            "qualifying_r1_0_trades": len(wins_r1),
            "qualifying_hit_rate_pct": round((len(wins_r1) / n * 100.0), 2) if n > 0 else 0.0,
            "nominal_win_rate_pct": round((len(wins_be) / n * 100.0), 2) if n > 0 else 0.0,
            "per_trade_rates": {
                "expectancy_gross_pnl_inr_per_trade": avg_gross_pnl_inr,
                "expectancy_gross_return_bps_per_trade": avg_gross_bps,
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
            "max_drawdown_inr": round(mdd, 2),
            "exit_distribution": exit_dist
        }


def main():
    data_dir = WORKSPACE_DIR / "intraday_capture" / "yahoo_cache_15m"
    out_dir = WORKSPACE_DIR / "artifacts" / "rp608_successor_charter"
    out_dir.mkdir(parents=True, exist_ok=True)

    harness = H1BreakoutHarness(data_dir=data_dir, output_dir=out_dir)
    harness.load_dataset()

    configurations = [
        # 1. 60m ORB with 1 close, no volume filter
        ("H1_ORB4_M1_VOL10", "ORB", 4, 1, 1.0),
        # 2. 60m ORB with 1 close + 1.25x volume filter
        ("H1_ORB4_M1_VOL125", "ORB", 4, 1, 1.25),
        # 3. 60m ORB with 2 confirmed closes, no volume filter
        ("H1_ORB4_M2_VOL10", "ORB", 4, 2, 1.0),
        # 4. 60m ORB with 2 confirmed closes + 1.25x volume filter
        ("H1_ORB4_M2_VOL125", "ORB", 4, 2, 1.25),
        # 5. 30m ORB with 1 close + 1.25x volume filter
        ("H1_ORB2_M1_VOL125", "ORB", 2, 1, 1.25),
        # 6. 30m ORB with 2 confirmed closes + 1.25x volume filter
        ("H1_ORB2_M2_VOL125", "ORB", 2, 2, 1.25),
        # 7. Trailing 8-bar (120m) swing high + 1 close + 1.25x volume
        ("H1_SWING8_M1_VOL125", "SWING", 8, 1, 1.25),
        # 8. Trailing 8-bar (120m) swing high + 2 closes + 1.25x volume
        ("H1_SWING8_M2_VOL125", "SWING", 8, 2, 1.25),
    ]

    results = []
    print("\nExecuting RP-608 Gate 3B (Hypothesis H1) Causal Simulations...")
    print(f"{'Config Name':<22} | {'Trades':<6} | {'Gross Exp/Trd':<15} | {'Net Exp/Trd':<15} | {'Payoff':<6} | {'Qual >=1R':<10} | {'Win Rate':<8}")
    print("-" * 92)

    for cfg_name, b_type, f_bars, req_c, v_m in configurations:
        res = harness.run_h1_configuration(
            config_name=cfg_name,
            boundary_type=b_type,
            formation_bars=f_bars,
            required_closes=req_c,
            volume_mult=v_m
        )
        results.append(res)
        ptr = res["per_trade_rates"]
        print(
            f"{cfg_name:<22} | {res['executed_trades']:<6} | "
            f"₹{ptr['expectancy_gross_pnl_inr_per_trade']:>7.2f} ({res['gross_profit_factor']:>4.2f}) | "
            f"₹{ptr['expectancy_net_pnl_inr_per_trade']:>7.2f} ({ptr['expectancy_net_return_bps_per_trade']:>5.1f}b) | "
            f"{ptr['realized_payoff_ratio']:>6.2f} | "
            f"{res['qualifying_hit_rate_pct']:>5.2f}% ({res['qualifying_r1_0_trades']:>2}) | "
            f"{res['nominal_win_rate_pct']:>5.2f}%"
        )

    # Save results to JSON
    json_path = out_dir / "rp608_h1_breakout_audit_results.json"
    with open(json_path, "w", encoding="utf-8") as f:
        json.dump(results, f, indent=2)
    print(f"\nAudit results successfully saved to: {json_path}")


if __name__ == "__main__":
    main()
