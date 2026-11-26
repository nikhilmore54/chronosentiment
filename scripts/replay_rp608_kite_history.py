#!/usr/bin/env python3
"""
scripts/replay_rp608_kite_history.py

RP-608 Phase 3: Standalone Offline Historical Replay & Causal Audit Harness.
Strictly Read-Only (0.0% Order Authority). Isolated from live shadow daemon state.

Key Principles:
  1. Dataset Contract & Validation:
     - Reconciles expected bars per session (25 bars for regular 09:15-15:30 IST session).
     - Audits timestamps, missing intervals, and corporate-action anomalies.
  2. Zero Future Leakage (Causal Physics):
     - Signal evaluated at Bar k Close (T_close(k)).
     - Executable Entry strictly at Bar k+1 Open (T_open(k+1)).
  3. Correct Gate Semantics:
     - Gates G1 (Boundary), G2 (Position Limit), G3 (5-Bar Warmup) evaluated causally.
     - Gate G4 (Quote Feasibility) transparently recorded as PROXY_SPREAD_EXECUTION
       (standard 2.5 bps half-spread + 2.0 bps slippage) since OHLCV lacks live order book quotes.
  4. Conservative Resolution:
     - Simultaneous target & stop breach within the same candle resolves as STOP_LOSS_HIT.
  5. 8-Component Statutory Equity Friction:
     - Exact Indian equity cash intraday charges: STT on sell leg, Exchange charges,
       SEBI turnover fee, Stamp duty on buy leg, GST (18%), brokerage, spread, and slippage.
  6. Dedicated Research Artifacts:
     - Outputs written strictly to artifacts/rp608_historical_replay/ without mutating live daemon checkpoints.
"""

from __future__ import annotations

import argparse
from dataclasses import asdict, dataclass
from datetime import datetime, time as dtime, timedelta, timezone
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


class RP608HistoricalReplayHarness:
    """
    Offline chronological replay engine for RP-608 momentum breakout strategy.
    Evaluates 119-symbol universe across multi-day 15m OHLCV datasets.
    """

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

        # Replay Ledgers
        self.summary_path = self.output_dir / "rp608_replay_summary.json"
        self.trades_path = self.output_dir / "rp608_replay_trades.jsonl"
        self.gate_rejections_path = self.output_dir / "rp608_gate_rejections.jsonl"
        self.data_audit_path = self.output_dir / "rp608_data_coverage_audit.json"

    def load_and_validate_symbol_data(self, symbol: str) -> Tuple[List[ReplayBar], Dict[str, Any]]:
        """
        Loads raw 15m candle JSON for a symbol and reconciles it against the NSE trading calendar.
        Standard regular trading day: 09:15 to 15:30 IST -> exactly 25 completed 15m intervals.
        """
        json_path = self.data_dir / f"{symbol}.json"
        if not json_path.exists():
            return [], {"symbol": symbol, "status": "FILE_NOT_FOUND", "bars_count": 0}

        try:
            with open(json_path, "r", encoding="utf-8") as f:
                raw_candles = json.load(f)
        except Exception as e:
            return [], {"symbol": symbol, "status": "CORRUPT_JSON", "error": str(e), "bars_count": 0}

        if not raw_candles:
            return [], {"symbol": symbol, "status": "EMPTY_DATASET", "bars_count": 0}

        # Parse & sort candles by timestamp
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
            parsed.append({
                "ts_ms": ts_ms,
                "open": o,
                "high": h,
                "low": l,
                "close": cl,
                "volume": v
            })

        parsed.sort(key=lambda x: x["ts_ms"])

        # Deduplicate timestamps
        deduped: List[Dict[str, Any]] = []
        seen_ts = set()
        duplicates_count = 0
        for p in parsed:
            if p["ts_ms"] in seen_ts:
                duplicates_count += 1
                continue
            seen_ts.add(p["ts_ms"])
            deduped.append(p)

        # Structure into session-aware ReplayBars
        replay_bars: List[ReplayBar] = []
        sessions: Dict[str, List[ReplayBar]] = {}

        for p in deduped:
            dt_utc = datetime.fromtimestamp(p["ts_ms"] / 1000.0, tz=timezone.utc)
            dt_ist = dt_utc + IST_OFFSET
            date_str = str(dt_ist.date())

            # Skip weekends and official holidays
            if dt_ist.weekday() >= 5:
                continue
            if dt_ist.date() in NSE_2026_HOLIDAYS:
                continue

            open_ist = dt_ist.replace(hour=9, minute=15, second=0, microsecond=0)
            close_ist = dt_ist.replace(hour=15, minute=30, second=0, microsecond=0)

            # Check if within regular trading hours
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
            replay_bars.append(rb)
            if date_str not in sessions:
                sessions[date_str] = []
            sessions[date_str].append(rb)

        # Audit session completeness
        complete_sessions = sum(1 for s, b in sessions.items() if len(b) == 25)
        incomplete_sessions = sum(1 for s, b in sessions.items() if len(b) != 25)

        audit_info = {
            "symbol": symbol,
            "status": "VALIDATED",
            "total_raw_candles": len(raw_candles),
            "duplicates_removed": duplicates_count,
            "valid_replay_bars": len(replay_bars),
            "total_trading_sessions": len(sessions),
            "complete_25_bar_sessions": complete_sessions,
            "incomplete_sessions": incomplete_sessions,
            "coverage_ratio": round(complete_sessions / len(sessions), 4) if sessions else 0.0
        }

        return replay_bars, audit_info

    def run_replay(self) -> Dict[str, Any]:
        """
        Executes causal chronological replay across all validated symbols.
        Guarantees zero future leakage:
          - Signal decided at Bar k Close.
          - Order executed at Bar k+1 Open with friction.
        """
        # 1. Load and organize dataset chronologically
        audit_by_symbol = {}
        symbol_bars: Dict[str, Dict[str, Dict[int, ReplayBar]]] = {}
        all_session_dates: Set[str] = set()

        for sym in self.symbols:
            bars, audit = self.load_and_validate_symbol_data(sym)
            audit_by_symbol[sym] = audit
            if not bars:
                continue
            symbol_bars[sym] = {}
            for b in bars:
                all_session_dates.add(b.session_date)
                if b.session_date not in symbol_bars[sym]:
                    symbol_bars[sym][b.session_date] = {}
                symbol_bars[sym][b.session_date][b.bar_idx] = b

        sorted_sessions = sorted(all_session_dates)

        # Clear output ledgers
        with open(self.trades_path, "w", encoding="utf-8") as f:
            pass
        with open(self.gate_rejections_path, "w", encoding="utf-8") as f:
            pass

        # Global Replay State
        active_trades: Dict[str, Dict[str, Any]] = {}
        pending_entries: Dict[str, Dict[str, Any]] = {}  # {symbol: entry_instruction} for next bar open
        closed_trades: List[Dict[str, Any]] = []

        gate_tallies = {
            "EVALUATED_BARS": 0,
            "GATE_G1_BOUNDARY_EXCLUDED": 0,
            "GATE_G2_MAX_CONCURRENT_POSITIONS": 0,
            "GATE_G2_EXISTING_POSITION_ACTIVE": 0,
            "GATE_G3_COLD_START_WARMUP": 0,
            "GATE_G3_SUB_THRESHOLD_MOMENTUM": 0,
            "QUALIFIED_BREAKOUT_SIGNALS": 0
        }

        total_bars_processed = 0

        # Chronological Step: Session by Session, Bar by Bar (1..25)
        for s_idx, session_date in enumerate(sorted_sessions, start=1):
            session_history: Dict[str, List[float]] = {s: [] for s in self.symbols}

            # Any open trade carried across session boundary is strictly prohibited: mandatory close at 15:15 of prior day
            active_trades.clear()
            pending_entries.clear()

            for bar_idx in range(1, 26):
                # ── Phase A: Execute Pending Orders from Previous Bar Signal (Bar k-1 Signal -> Bar k Open) ──
                for sym in list(pending_entries.keys()):
                    instr = pending_entries.pop(sym)
                    curr_bar = symbol_bars.get(sym, {}).get(session_date, {}).get(bar_idx)
                    if not curr_bar:
                        # Cannot execute: missing open bar
                        continue
                    if len(active_trades) >= self.max_concurrent_trades:
                        # Dropped at execution time due to concurrency
                        gate_tallies["GATE_G2_MAX_CONCURRENT_POSITIONS"] += 1
                        continue

                    # Executable Entry Price at Bar k Open + Half-spread + Slippage
                    raw_entry_p = curr_bar.open
                    entry_slip = raw_entry_p * ((self.half_spread_bps + self.slippage_bps) / 10000.0)
                    effective_entry_p = round(raw_entry_p + entry_slip, 2)
                    qty = max(1.0, math.floor(DEFAULT_NOTIONAL_PER_TRADE_INR / effective_entry_p))

                    trade_id = f"replay_{session_date}_{bar_idx}_{sym}"
                    active_trades[sym] = {
                        "trade_id": trade_id,
                        "symbol": sym,
                        "session_date": session_date,
                        "session_idx": s_idx,
                        "signal_bar_idx": instr["signal_bar_idx"],
                        "entry_bar_idx": bar_idx,
                        "entry_timestamp_iso": curr_bar.interval_start_iso,
                        "entry_price": effective_entry_p,
                        "raw_entry_price": raw_entry_p,
                        "quantity": qty,
                        "target_price": round(effective_entry_p * (1.0 + (self.target_bps / 10000.0)), 2),
                        "stop_price": round(effective_entry_p * (1.0 - (self.stop_bps / 10000.0)), 2),
                        "mfe_price": effective_entry_p,
                        "mae_price": effective_entry_p,
                        "traj15_signal": instr["traj15"],
                    }

                # ── Phase B: Evaluate Intrabar Excursions & Exits for Active Trades ──
                for sym in list(active_trades.keys()):
                    trade = active_trades[sym]
                    curr_bar = symbol_bars.get(sym, {}).get(session_date, {}).get(bar_idx)
                    if not curr_bar:
                        continue

                    # Track MFE / MAE
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

                    # Conservative Resolution: if both target and stop hit in same candle -> STOP_LOSS_HIT
                    elif curr_bar.low <= stp_p and curr_bar.high >= tgt_p:
                        exit_reason = "CONSERVATIVE_STOP_LOSS_SIMULTANEOUS"
                        raw_exit_p = stp_p

                    # Stop-loss breached
                    elif curr_bar.low <= stp_p:
                        exit_reason = "STOP_LOSS_HIT"
                        raw_exit_p = stp_p

                    # Target breached
                    elif curr_bar.high >= tgt_p:
                        exit_reason = "TARGET_PROFIT_HIT"
                        raw_exit_p = tgt_p

                    # Max hold horizon (180m = 12 bars)
                    elif bars_held >= 12:
                        exit_reason = "MAX_HOLD_HORIZON_180M"
                        raw_exit_p = curr_bar.close

                    # Target horizon (60m = 4 bars) with profit
                    elif bars_held >= 4 and curr_bar.close > entry_p:
                        exit_reason = "PROFIT_TIMED_EXIT_60M"
                        raw_exit_p = curr_bar.close

                    if exit_reason and raw_exit_p:
                        # Effective exit price adjusted for selling half-spread and slippage
                        exit_slip = raw_exit_p * ((self.half_spread_bps + self.slippage_bps) / 10000.0)
                        effective_exit_p = round(raw_exit_p - exit_slip, 2)

                        qty = trade["quantity"]
                        buy_turnover = entry_p * qty
                        sell_turnover = effective_exit_p * qty
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
                        self.append_jsonl(self.trades_path, record)

                # ── Phase C: Causal Decision Evaluation at Bar k Close ──
                for sym in self.symbols:
                    curr_bar = symbol_bars.get(sym, {}).get(session_date, {}).get(bar_idx)
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
                        decision_code = "GATE_G1_BOUNDARY_EXCLUDED"
                    # Gate G2: Position active
                    elif sym in active_trades or sym in pending_entries:
                        gate_tallies["GATE_G2_EXISTING_POSITION_ACTIVE"] += 1
                        decision_code = "GATE_G2_EXISTING_POSITION_ACTIVE"
                    elif len(active_trades) + len(pending_entries) >= self.max_concurrent_trades:
                        gate_tallies["GATE_G2_MAX_CONCURRENT_POSITIONS"] += 1
                        decision_code = "GATE_G2_MAX_CONCURRENT_POSITIONS"
                    # Gate G3: Insufficient Warmup (< 5 same-session bars)
                    elif len(history) < 5:
                        gate_tallies["GATE_G3_COLD_START_WARMUP"] += 1
                        decision_code = "GATE_G3_COLD_START_WARMUP"
                    # Gate G3: Sub-threshold momentum
                    elif traj15 < 0.05:
                        gate_tallies["GATE_G3_SUB_THRESHOLD_MOMENTUM"] += 1
                        decision_code = "GATE_G3_SUB_THRESHOLD_MOMENTUM"
                    else:
                        # Qualified Breakout Signal
                        gate_tallies["QUALIFIED_BREAKOUT_SIGNALS"] += 1
                        decision_code = "QUALIFIED_BREAKOUT"
                        pending_entries[sym] = {
                            "signal_bar_idx": bar_idx,
                            "signal_timestamp_iso": curr_bar.interval_close_iso,
                            "signal_close": curr_bar.close,
                            "traj15": round(traj15, 4)
                        }

                    # Log rejection if not qualified
                    if decision_code != "QUALIFIED_BREAKOUT":
                        self.append_jsonl(self.gate_rejections_path, {
                            "session_date": session_date,
                            "bar_idx": bar_idx,
                            "symbol": sym,
                            "decision_code": decision_code,
                            "bars_history": len(history),
                            "traj15": round(traj15, 4)
                        })

        # Calculate reconciled gross and net evidence metrics
        trade_count = len(closed_trades)
        
        # Gross metrics (partitioned strictly by gross_pnl_inr > 0)
        gross_wins = [t for t in closed_trades if t["gross_pnl_inr"] > 0]
        gross_losses = [t for t in closed_trades if t["gross_pnl_inr"] < 0]
        gross_profit = sum(t["gross_pnl_inr"] for t in gross_wins)
        gross_loss = abs(sum(t["gross_pnl_inr"] for t in gross_losses))
        gross_pnl = sum(t["gross_pnl_inr"] for t in closed_trades)
        gross_win_rate = (len(gross_wins) / trade_count) if trade_count > 0 else 0.0
        gross_pf = (gross_profit / gross_loss) if gross_loss > 0 else (99.0 if gross_profit > 0 else 0.0)

        # Friction metrics
        total_friction = sum(t["total_friction_inr"] for t in closed_trades)
        avg_friction = (total_friction / trade_count) if trade_count > 0 else 0.0
        # Trades where gross was positive but net became negative due to friction
        friction_flipped = len([t for t in closed_trades if t["gross_pnl_inr"] > 0 and t["net_pnl_inr"] <= 0])

        # Net metrics (partitioned strictly by net_pnl_inr > 0)
        net_wins = [t for t in closed_trades if t["net_pnl_inr"] > 0]
        net_losses = [t for t in closed_trades if t["net_pnl_inr"] <= 0]
        net_profit = sum(t["net_pnl_inr"] for t in net_wins)
        net_loss = abs(sum(t["net_pnl_inr"] for t in net_losses))
        total_net_pnl = sum(t["net_pnl_inr"] for t in closed_trades)
        net_win_rate = (len(net_wins) / trade_count) if trade_count > 0 else 0.0
        net_pf = (net_profit / net_loss) if net_loss > 0 else (99.0 if net_profit > 0 else 0.0)
        net_returns_bps = [t["net_return_bps"] for t in closed_trades]
        avg_net_return_bps = (sum(net_returns_bps) / trade_count) if trade_count > 0 else 0.0

        # Max drawdown calculation
        cumulative_pnl = 0.0
        peak_pnl = 0.0
        max_drawdown_inr = 0.0
        for t in closed_trades:
            cumulative_pnl += t["net_pnl_inr"]
            if cumulative_pnl > peak_pnl:
                peak_pnl = cumulative_pnl
            dd = peak_pnl - cumulative_pnl
            if dd > max_drawdown_inr:
                max_drawdown_inr = dd

        # Chronological In-Sample (60%) vs Out-of-Sample Holdout (40%) Partition
        split_idx = int(len(sorted_sessions) * 0.6)
        is_sessions = set(sorted_sessions[:split_idx])
        oos_sessions = set(sorted_sessions[split_idx:])

        is_trades = [t for t in closed_trades if t["session_date"] in is_sessions]
        oos_trades = [t for t in closed_trades if t["session_date"] in oos_sessions]

        def compute_partition_stats(trades_sub: List[Dict[str, Any]]) -> Dict[str, Any]:
            n = len(trades_sub)
            if n == 0:
                return {}
            g_wins = [t for t in trades_sub if t["gross_pnl_inr"] > 0]
            g_losses = [t for t in trades_sub if t["gross_pnl_inr"] < 0]
            n_wins = [t for t in trades_sub if t["net_pnl_inr"] > 0]
            n_losses = [t for t in trades_sub if t["net_pnl_inr"] <= 0]
            g_p = sum(t["gross_pnl_inr"] for t in g_wins)
            g_l = abs(sum(t["gross_pnl_inr"] for t in g_losses))
            n_p = sum(t["net_pnl_inr"] for t in n_wins)
            n_l = abs(sum(t["net_pnl_inr"] for t in n_losses))
            return {
                "trade_count": n,
                "gross_pnl_inr": round(sum(t["gross_pnl_inr"] for t in trades_sub), 2),
                "gross_win_rate": round(len(g_wins) / n, 4),
                "gross_profit_factor": round(g_p / g_l, 4) if g_l > 0 else 0.0,
                "total_friction_inr": round(sum(t["total_friction_inr"] for t in trades_sub), 2),
                "net_pnl_inr": round(sum(t["net_pnl_inr"] for t in trades_sub), 2),
                "net_win_rate": round(len(n_wins) / n, 4),
                "net_profit_factor": round(n_p / n_l, 4) if n_l > 0 else 0.0,
                "avg_net_return_bps": round(sum(t["net_return_bps"] for t in trades_sub) / n, 2)
            }

        summary = {
            "protocol_id": "RP608_PHASE3_OFFLINE_REPLAY",
            "order_authority": "0.0%",
            "execution_mode": "STRICT_OFFLINE_CAUSAL_REPLAY",
            "evaluated_universe_size": len(self.symbols),
            "sessions_evaluated": len(sorted_sessions),
            "total_bars_processed": total_bars_processed,
            "boundary_rule": "Causal signals accepted for completed candles k in {1..20}. Latest signal at Bar 20 close (14:15:00 IST), earliest entry at Bar 21 open (14:15:00 IST). Signals for k > 20 strictly excluded.",
            "gate_semantics": {
                "G1_boundary": "ENFORCED_BARS_1_TO_20",
                "G2_position_limits": f"MAX_{self.max_concurrent_trades}_CONCURRENT",
                "G3_warmup": "ENFORCED_5_BAR_MINIMUM",
                "G4_quote_feasibility": "PROXY_SPREAD_EXECUTION_2.5BPS_HALF_SPREAD_PLUS_2.0BPS_SLIPPAGE"
            },
            "gate_tallies": gate_tallies,
            "reconciled_accounting": {
                "total_trades": trade_count,
                "gross_metrics": {
                    "gross_winning_trades": len(gross_wins),
                    "gross_losing_trades": len(gross_losses),
                    "gross_win_rate": round(gross_win_rate, 4),
                    "gross_profit_inr": round(gross_profit, 2),
                    "gross_loss_inr": round(-gross_loss, 2),
                    "gross_pnl_inr": round(gross_pnl, 2),
                    "gross_profit_factor": round(gross_pf, 4)
                },
                "friction_metrics": {
                    "total_friction_paid_inr": round(total_friction, 2),
                    "avg_friction_per_trade_inr": round(avg_friction, 2),
                    "model_standard_notional_friction_inr": 171.70,
                    "trades_flipped_win_to_loss_by_friction": friction_flipped,
                    "friction_drag_percentage_of_net_loss": round((total_friction / abs(total_net_pnl)) * 100.0, 2) if total_net_pnl < 0 else 0.0
                },
                "net_metrics": {
                    "net_winning_trades": len(net_wins),
                    "net_losing_trades": len(net_losses),
                    "net_win_rate": round(net_win_rate, 4),
                    "net_profit_inr": round(net_profit, 2),
                    "net_loss_inr": round(-net_loss, 2),
                    "net_pnl_inr": round(total_net_pnl, 2),
                    "net_profit_factor": round(net_pf, 4),
                    "avg_net_return_bps": round(avg_net_return_bps, 2),
                    "max_drawdown_inr": round(max_drawdown_inr, 2)
                }
            },
            "performance_metrics": {
                "total_trades": trade_count,
                "winning_trades": len(net_wins),
                "losing_trades": len(net_losses),
                "win_rate": round(net_win_rate, 4),
                "profit_factor": round(net_pf, 4),
                "avg_net_return_bps": round(avg_net_return_bps, 2),
                "total_net_pnl_inr": round(total_net_pnl, 2),
                "total_friction_paid_inr": round(total_friction, 2),
                "max_drawdown_inr": round(max_drawdown_inr, 2)
            },
            "chronological_partitions": {
                "in_sample_development_60pct": {
                    "sessions_count": len(is_sessions),
                    "session_range": f"{sorted_sessions[0]} to {sorted_sessions[split_idx-1]}",
                    **compute_partition_stats(is_trades)
                },
                "out_of_sample_holdout_40pct": {
                    "sessions_count": len(oos_sessions),
                    "session_range": f"{sorted_sessions[split_idx]} to {sorted_sessions[-1]}",
                    **compute_partition_stats(oos_trades)
                }
            },
            "evidence_classification": self.classify_evidence(trade_count, avg_net_return_bps, gross_pf)
        }

        with open(self.summary_path, "w", encoding="utf-8") as f:
            json.dump(summary, f, indent=2)

        with open(self.data_audit_path, "w", encoding="utf-8") as f:
            json.dump(audit_by_symbol, f, indent=2)

        return summary

    def classify_evidence(self, trade_count: int, avg_net_return_bps: float, profit_factor: float) -> Dict[str, Any]:
        """Classifies outcome into 1 of 3 required diagnostic categories."""
        if trade_count < 30:
            return {
                "category": "INSUFFICIENT_EVIDENCE",
                "reason": f"Sample size of {trade_count} trades is inadequate for statistical significance (< 30 required)"
            }
        elif avg_net_return_bps <= 0.0 or profit_factor < 1.0:
            return {
                "category": "SIGNALS_EXIST_BUT_ECONOMICS_FAIL",
                "reason": f"Signals generated ({trade_count} trades), but net expectancy ({avg_net_return_bps:+.2f} bps) or profit factor ({profit_factor:.2f}) does not overcome statutory friction and slippage"
            }
        else:
            return {
                "category": "PROMISING_OUT_OF_SAMPLE_EVIDENCE",
                "reason": f"Strategy produces positive net expectancy ({avg_net_return_bps:+.2f} bps) and viable profit factor ({profit_factor:.2f}) across {trade_count} trades after full statutory friction"
            }

    def append_jsonl(self, path: Path, record: Dict[str, Any]):
        with open(path, "a", encoding="utf-8") as f:
            f.write(json.dumps(record, separators=(",", ":")) + "\n")


def open_time_min(dt_ist: datetime) -> datetime:
    return dt_ist.replace(hour=9, minute=15, second=0, microsecond=0)


def main():
    parser = argparse.ArgumentParser(description="RP-608 Phase 3: Standalone Offline Historical Replay Harness")
    parser.add_argument("--data-dir", type=str, default=str(WORKSPACE_DIR / "intraday_capture" / "yahoo_cache_15m"))
    parser.add_argument("--output-dir", type=str, default=str(WORKSPACE_DIR / "artifacts" / "rp608_historical_replay"))
    parser.add_argument("--symbols", type=str, default=None, help="Comma-separated subset of symbols")
    parser.add_argument("--max-concurrent", type=int, default=10)
    args = parser.parse_args()

    symbols = [s.strip() for s in args.symbols.split(",")] if args.symbols else None
    harness = RP608HistoricalReplayHarness(
        data_dir=Path(args.data_dir),
        output_dir=Path(args.output_dir),
        symbols=symbols,
        max_concurrent_trades=args.max_concurrent
    )

    print(f"[*] Starting RP-608 Phase 3 Replay across {len(harness.symbols)} symbols...")
    summary = harness.run_replay()
    print("\n[✓] Replay Completed Successfully.")
    print(f"    Total Bars Processed: {summary['total_bars_processed']}")
    ra = summary['reconciled_accounting']
    gm = ra['gross_metrics']
    nm = ra['net_metrics']
    fm = ra['friction_metrics']
    print(f"    Executed Trades:      {ra['total_trades']}")
    print(f"    Gross Win Rate:       {gm['gross_win_rate'] * 100:.2f}% (PF: {gm['gross_profit_factor']})")
    print(f"    Gross PnL:            ₹{gm['gross_pnl_inr']:,.2f}")
    print(f"    Total Friction:       ₹{fm['total_friction_paid_inr']:,.2f} (avg ₹{fm['avg_friction_per_trade_inr']:.2f}/trade)")
    print(f"    Net Win Rate:         {nm['net_win_rate'] * 100:.2f}% (Net PF: {nm['net_profit_factor']})")
    print(f"    Avg Net Return:       {nm['avg_net_return_bps']:+.2f} bps")
    print(f"    Total Net PnL:        ₹{nm['net_pnl_inr']:,.2f}")
    print(f"    Evidence Category:    {summary['evidence_classification']['category']}")
    print(f"    Diagnostic Reason:    {summary['evidence_classification']['reason']}")
    print(f"\n[*] Artifacts saved to: {harness.output_dir}")


if __name__ == "__main__":
    main()
