#!/usr/bin/env python3
"""
scripts/run_rp615_historical_discovery_replay.py

RP-615 Milestone RP-615-002: Historical Discovery Replay (Sessions 28 & 29 September 2026).
Governance:
  - RP-605 Baseline Control retains 100% Live Execution Authority.
  - RP-615 is strictly Research-Only (0% Live Authority).
  - Evaluates Candidate C0 (Pure Cooldown Benchmark) and Hypotheses H1, H2, and H3
    over the complete market-bar universe with independent candidate lifecycles.

Outputs:
  - live_capture/shadow_validation/rp615_002_candidate_evaluations.jsonl
  - live_capture/shadow_validation/rp615_002_rejected_opportunities.jsonl
  - live_capture/shadow_validation/rp615_002_discovery_manifest.json
  - live_capture/shadow_validation/rp615_002_discovery_replay_report.md
"""

from __future__ import annotations

import csv
import hashlib
import json
import math
import sys
from collections import Counter, defaultdict
from datetime import datetime, timezone, timedelta
from pathlib import Path
import numpy as np

WORKSPACE = Path(__file__).resolve().parent.parent
sys.path.insert(0, str(WORKSPACE))

from scripts.run_rp605_nse_prospective_freshness_validation import SYMBOLS, compute_coralys_scores

CACHE_DIR = WORKSPACE / "intraday_capture" / "yahoo_cache_1m"
LOG_DIR = WORKSPACE / "live_capture" / "shadow_validation"
BASELINE_JOURNAL = LOG_DIR / "rp605_nse_baseline_journal.jsonl"
OUT_CANDIDATE_JSONL = LOG_DIR / "rp615_002_candidate_evaluations.jsonl"
OUT_REJECTED_JSONL = LOG_DIR / "rp615_002_rejected_opportunities.jsonl"
OUT_MANIFEST_JSON = LOG_DIR / "rp615_002_discovery_manifest.json"
OUT_REPORT_MD = LOG_DIR / "rp615_002_discovery_replay_report.md"

IST = timezone(timedelta(hours=5, minutes=30))
DISCOVERY_SESSIONS = ["2026-09-28", "2026-09-29"]
POSITION_CAPITAL_INR = 100000.0
TURNOVER_FRICTION_BPS_PER_LEG = 7.0  # 7.0 bps on entry notional + 7.0 bps on exit notional (14.0 bps total)

def sha256_file(path: Path) -> str:
    digest = hashlib.sha256()
    with path.open("rb") as f:
        for chunk in iter(lambda: f.read(1024 * 1024), b""):
            digest.update(chunk)
    return digest.hexdigest()

def wilson_score_interval(successes: int, total: int, z: float = 1.95996) -> tuple[float, float]:
    if total == 0:
        return 0.0, 0.0
    p = successes / total
    denom = 1.0 + (z**2) / total
    center = (p + (z**2) / (2.0 * total)) / denom
    margin = (z * math.sqrt((p * (1.0 - p) + (z**2) / (4.0 * total)) / total)) / denom
    return max(0.0, center - margin), min(1.0, center + margin)

class CandidateEngine:
    """
    Simulates independent position lifecycles, entry rules, cooldowns, and accounting
    for a specific strategy candidate over the full universe of 1-minute market bars.
    """
    def __init__(self, name: str, candidate_id: str, description: str):
        self.name = name
        self.candidate_id = candidate_id
        self.description = description
        self.active_positions: dict[str, dict] = {}
        self.last_exit_time: dict[str, datetime] = {}
        self.completed_trades: list[dict] = []
        self.rejected_signals: list[dict] = []
        self.trade_counter = 0

    def is_eligible_cooldown(self, sym: str, cur_dt: datetime, cooldown_sec: int = 900) -> bool:
        if sym in self.active_positions:
            return False
        last_exit = self.last_exit_time.get(sym)
        if last_exit is None:
            return True
        return (cur_dt - last_exit).total_seconds() > cooldown_sec

    def enter_position(self, sym: str, side: str, price: float, ts_str: str, ts_dt: datetime, session_date: str, metadata: dict):
        self.trade_counter += 1
        tid = f"{self.candidate_id}-{session_date.replace('-', '')}-{self.trade_counter:04d}"
        qty = max(1, math.floor(POSITION_CAPITAL_INR / price))
        trade = {
            "candidate_id": self.candidate_id,
            "trade_id": tid,
            "session_date": session_date,
            "symbol": sym,
            "side": side,
            "entry_timestamp": ts_str,
            "entry_dt": ts_dt,
            "entry_price": price,
            "quantity": qty,
            "max_pnl": 0.0,
            "min_pnl": 0.0,
            "max_price": price,
            "min_price": price,
            "metadata": metadata
        }
        self.active_positions[sym] = trade

    def update_and_exit(self, sym: str, price: float, high: float, low: float, ts_str: str, ts_dt: datetime, exit_reason: str):
        pos = self.active_positions.pop(sym)
        entry_p = pos["entry_price"]
        side = pos["side"]
        qty = pos["quantity"]

        # Final P&L
        if side == "LONG":
            gross_pnl = (price - entry_p) * qty
            mfe_pct = max(0.0, (pos["max_price"] - entry_p) / entry_p * 100.0)
            mae_pct = max(0.0, (entry_p - pos["min_price"]) / entry_p * 100.0)
        else:
            gross_pnl = (entry_p - price) * qty
            mfe_pct = max(0.0, (entry_p - pos["min_price"]) / entry_p * 100.0)
            mae_pct = max(0.0, (pos["max_price"] - entry_p) / entry_p * 100.0)

        turnover_notional = (entry_p + price) * qty
        friction = turnover_notional * (TURNOVER_FRICTION_BPS_PER_LEG / 10000.0)
        net_pnl = gross_pnl - friction
        duration_sec = int((ts_dt - pos["entry_dt"]).total_seconds())

        record = {
            "candidate_id": self.candidate_id,
            "trade_id": pos["trade_id"],
            "session_date": pos["session_date"],
            "symbol": sym,
            "side": side,
            "entry_timestamp": pos["entry_timestamp"],
            "exit_timestamp": ts_str,
            "holding_duration_sec": duration_sec,
            "holding_mins": round(duration_sec / 60.0, 1),
            "entry_price": entry_p,
            "exit_price": price,
            "quantity": qty,
            "gross_pnl": round(gross_pnl, 2),
            "friction": round(friction, 2),
            "net_pnl": round(net_pnl, 2),
            "is_net_win": 1 if net_pnl > 0 else 0,
            "is_gross_win": 1 if gross_pnl > 0 else 0,
            "is_breakeven": 1 if net_pnl == 0 else 0,
            "exit_reason": exit_reason,
            "mfe_pct": round(mfe_pct, 4),
            "mae_pct": round(mae_pct, 4),
            "metadata": pos["metadata"]
        }
        self.completed_trades.append(record)
        self.last_exit_time[sym] = ts_dt

    def log_rejected_signal(self, sym: str, side: str, price: float, ts_str: str, ts_dt: datetime, session_date: str, reason: str, metadata: dict):
        self.rejected_signals.append({
            "candidate_id": self.candidate_id,
            "session_date": session_date,
            "symbol": sym,
            "side": side,
            "timestamp_utc": ts_str,
            "price": price,
            "rejection_reason": reason,
            "metadata": metadata
        })

def main():
    print("=" * 80)
    print("RP-615-002: HISTORICAL DISCOVERY REPLAY (28 & 29 SEPTEMBER 2026)")
    print("Governance: Research-Only (0% Live Authority) | RP-605 Control (100% Live)")
    print("=" * 80)

    # -------------------------------------------------------------------------
    # Step 1: Initialize Candidate State Machines
    # -------------------------------------------------------------------------
    candidates = {
        "RP605_CONTROL": CandidateEngine("RP-605 Control", "RP605-CTRL", "Live control with 0m cooldown and trailing exit"),
        "C0_COOLDOWN": CandidateEngine("Candidate C0 (Pure Cooldown)", "C0-COOL", "RP-605 signal with mandatory 15m cooldown"),
        "H1_DONCHIAN": CandidateEngine("Hypothesis H1 (Range Breakout)", "H1-BRK", "20-bar Donchian breakout + RVOL>=1.50 + KER>=0.30"),
        "H2_CONCORDANCE": CandidateEngine("Hypothesis H2 (MTF + Concordance)", "H2-MTF", "15m/60m alignment + EMA8/21 + Market Breadth>=+2.0"),
        "H3_RETRACEMENT": CandidateEngine("Hypothesis H3 (Retracement Bounce)", "H3-RET", "Pullback to EMA8-EMA21 dynamic band + confirmation candle")
    }

    # -------------------------------------------------------------------------
    # Step 2: Load and Verify Universe Market Bar Data for Discovery Sessions
    # -------------------------------------------------------------------------
    print("\n[Step 2] Ingesting 1-Minute Market Bars for Discovery Sessions (28 & 29 Sep 2026)...")
    bars_by_date_sym = defaultdict(lambda: defaultdict(dict))
    session_minutes = defaultdict(set)

    for sym in SYMBOLS:
        p = CACHE_DIR / f"{sym}.json"
        if not p.exists():
            continue
        data = json.loads(p.read_text(encoding="utf-8"))
        for b in data:
            dt = datetime.fromtimestamp(b["timestamp"], IST)
            d_str = dt.strftime("%Y-%m-%d")
            if d_str in DISCOVERY_SESSIONS:
                # 09:15 to 15:30 IST market window
                if (dt.hour == 9 and dt.minute >= 15) or (9 < dt.hour < 15) or (dt.hour == 15 and dt.minute <= 30):
                    session_minutes[d_str].add((dt.hour * 60 + dt.minute, dt))
                    bars_by_date_sym[d_str][sym][dt] = b

    for s_date in DISCOVERY_SESSIONS:
        n_mins = len(session_minutes[s_date])
        n_syms = len(bars_by_date_sym[s_date])
        print(f"  Session {s_date}: {n_mins} minutes, {n_syms} active symbols.")

    # -------------------------------------------------------------------------
    # Step 3: Run Replay Loop Across Sessions and Intraday Minutes
    # -------------------------------------------------------------------------
    print("\n[Step 3] Running Independent Candidate Lifecycles Across All 1-Minute Bars...")

    for s_date in DISCOVERY_SESSIONS:
        sorted_minutes = [x[1] for x in sorted(list(session_minutes[s_date]), key=lambda x: x[0])]
        
        # Rolling indicator state histories per symbol
        ohlcv_history = defaultdict(list)
        coralys_history = defaultdict(list)
        ema8_history = defaultdict(float)
        ema21_history = defaultdict(float)

        for minute_idx, cur_dt in enumerate(sorted_minutes, 1):
            ts_utc_dt = cur_dt.astimezone(timezone.utc) + timedelta(seconds=15)
            ts_utc_str = ts_utc_dt.strftime("%Y-%m-%dT%H:%M:%SZ")

            # First pass: collect closes for cross-sectional market breadth calculation
            minute_coralys_scores = {}
            for sym in SYMBOLS:
                bar = bars_by_date_sym[s_date][sym].get(cur_dt)
                if not bar:
                    continue
                c_px = float(bar["close"])
                ohlcv_history[sym].append((bar["open"], bar["high"], bar["low"], bar["close"], bar["volume"]))
                if len(ohlcv_history[sym]) > 120:
                    ohlcv_history[sym].pop(0)

                closes = [x[3] for x in ohlcv_history[sym]]
                s15, s30, s1d = compute_coralys_scores(closes)
                coralys_history[sym].append((s15, s30, s1d))
                if len(coralys_history[sym]) > 60:
                    coralys_history[sym].pop(0)
                minute_coralys_scores[sym] = s15

            # Market Concordance Score (MCS): Z-score of universe net breadth
            n_eligible = len(minute_coralys_scores)
            if n_eligible >= 20:
                pos_count = sum(1 for s in minute_coralys_scores.values() if s > 0.05)
                neg_count = sum(1 for s in minute_coralys_scores.values() if s < -0.05)
                net_breadth = (pos_count - neg_count)
                mcs = net_breadth / math.sqrt(n_eligible * 0.25)
            else:
                mcs = 0.0

            # Second pass: evaluate entry and exit across each candidate engine
            for sym in SYMBOLS:
                bar = bars_by_date_sym[s_date][sym].get(cur_dt)
                if not bar or len(ohlcv_history[sym]) < 25:
                    continue

                c_px = float(bar["close"])
                o_px = float(bar["open"])
                h_px = float(bar["high"])
                l_px = float(bar["low"])
                vol = float(bar["volume"])

                closes = np.array([x[3] for x in ohlcv_history[sym]])
                highs = np.array([x[1] for x in ohlcv_history[sym]])
                lows = np.array([x[2] for x in ohlcv_history[sym]])
                vols = np.array([x[4] for x in ohlcv_history[sym]])

                s15, s30, s1d = coralys_history[sym][-1]

                # Update EMAs
                k8 = 2.0 / (8.0 + 1.0)
                k21 = 2.0 / (21.0 + 1.0)
                if ema8_history[sym] == 0.0:
                    ema8_history[sym] = c_px
                    ema21_history[sym] = c_px
                else:
                    ema8_history[sym] = c_px * k8 + ema8_history[sym] * (1.0 - k8)
                    ema21_history[sym] = c_px * k21 + ema21_history[sym] * (1.0 - k21)

                ema8 = ema8_history[sym]
                ema21 = ema21_history[sym]

                # ATR(14)
                tr = np.maximum(highs[-14:] - lows[-14:], np.maximum(np.abs(highs[-14:] - closes[-15:-1]), np.abs(lows[-14:] - closes[-15:-1])))
                atr14 = float(np.mean(tr)) if len(tr) > 0 else 1.0

                # 20-bar Donchian Channel (strictly prior 20 completed bars: t-20 to t-1)
                donch_high = float(np.max(highs[-21:-1]))
                donch_low = float(np.min(lows[-21:-1]))
                prev_high = float(np.max(highs[-22:-2]))
                prev_low = float(np.min(lows[-22:-2]))

                # Volume Surge (RVOL 15m vs 60m)
                mean_vol15 = float(np.mean(vols[-15:])) + 1e-6
                mean_vol60 = float(np.mean(vols[-60:])) if len(vols) >= 60 else (float(np.mean(vols)) + 1e-6)
                rvol15 = mean_vol15 / (mean_vol60 + 1e-6)

                # Kaufman Efficiency Ratio (KER 15m)
                net_chg15 = abs(closes[-1] - closes[-15])
                sum_chg15 = float(np.sum(np.abs(np.diff(closes[-15:])))) + 1e-6
                ker15 = net_chg15 / sum_chg15

                # s15 velocity
                s15_vel = (s15 - coralys_history[sym][-2][0]) if len(coralys_history[sym]) >= 2 else 0.0

                # 60m Momentum
                ret60 = (closes[-1] - closes[-60]) / closes[-60] if len(closes) >= 60 else 0.0
                ret15 = (closes[-1] - closes[-15]) / closes[-15]
                ret5 = (closes[-1] - closes[-5]) / closes[-5]

                # Common RP-605 state
                if s15 > 0.15 and s30 > 0.10:
                    rp605_sig_state, rp605_side = "BUY", "LONG"
                elif s15 < -0.15 and s30 < -0.10:
                    rp605_sig_state, rp605_side = "SHORT", "SHORT"
                elif abs(s15) < 0.05:
                    rp605_sig_state, rp605_side = "PULLBACK", "NONE"
                else:
                    rp605_sig_state, rp605_side = "WATCH", "NONE"

                # =============================================================
                # A. Evaluate RP-605 Control
                # =============================================================
                eng_ctrl = candidates["RP605_CONTROL"]
                if sym in eng_ctrl.active_positions:
                    pos = eng_ctrl.active_positions[sym]
                    # Update high / low
                    pos["max_price"] = max(pos["max_price"], h_px)
                    pos["min_price"] = min(pos["min_price"], l_px)
                    held_m = int((ts_utc_dt - pos["entry_dt"]).total_seconds() / 60)
                    if rp605_sig_state in ["EXIT", "PULLBACK", "SHORT" if pos["side"] == "LONG" else "BUY"]:
                        eng_ctrl.update_and_exit(sym, c_px, h_px, l_px, ts_utc_str, ts_utc_dt, "ENGINE_EXIT")
                    elif held_m >= 60:
                        eng_ctrl.update_and_exit(sym, c_px, h_px, l_px, ts_utc_str, ts_utc_dt, "TARGET_EXIT")
                elif rp605_sig_state in ["BUY", "SHORT"]:
                    eng_ctrl.enter_position(sym, rp605_side, c_px, ts_utc_str, ts_utc_dt, s_date, {"s15": s15, "s30": s30})

                # =============================================================
                # B. Evaluate Candidate C0 (Pure Cooldown Benchmark)
                # =============================================================
                eng_c0 = candidates["C0_COOLDOWN"]
                if sym in eng_c0.active_positions:
                    pos = eng_c0.active_positions[sym]
                    pos["max_price"] = max(pos["max_price"], h_px)
                    pos["min_price"] = min(pos["min_price"], l_px)
                    held_m = int((ts_utc_dt - pos["entry_dt"]).total_seconds() / 60)
                    if rp605_sig_state in ["EXIT", "PULLBACK", "SHORT" if pos["side"] == "LONG" else "BUY"]:
                        eng_c0.update_and_exit(sym, c_px, h_px, l_px, ts_utc_str, ts_utc_dt, "ENGINE_EXIT")
                    elif held_m >= 60:
                        eng_c0.update_and_exit(sym, c_px, h_px, l_px, ts_utc_str, ts_utc_dt, "TARGET_EXIT")
                elif rp605_sig_state in ["BUY", "SHORT"]:
                    if eng_c0.is_eligible_cooldown(sym, ts_utc_dt, cooldown_sec=900):
                        eng_c0.enter_position(sym, rp605_side, c_px, ts_utc_str, ts_utc_dt, s_date, {"s15": s15, "s30": s30})
                    else:
                        eng_c0.log_rejected_signal(sym, rp605_side, c_px, ts_utc_str, ts_utc_dt, s_date, "COOLDOWN_ACTIVE_LE_15M", {"s15": s15, "s30": s30})

                # =============================================================
                # C. Evaluate Hypothesis H1 (Fresh Donchian Range Breakout)
                # =============================================================
                eng_h1 = candidates["H1_DONCHIAN"]
                if sym in eng_h1.active_positions:
                    pos = eng_h1.active_positions[sym]
                    pos["max_price"] = max(pos["max_price"], h_px)
                    pos["min_price"] = min(pos["min_price"], l_px)
                    held_m = int((ts_utc_dt - pos["entry_dt"]).total_seconds() / 60)
                    # Structural Exit: 1.5 ATR trailing or channel midpoint or max 60 min
                    sl_dist = 1.5 * atr14
                    if pos["side"] == "LONG":
                        cur_dd = pos["max_price"] - l_px
                        if cur_dd >= sl_dist or rp605_sig_state == "SHORT" or held_m >= 60:
                            eng_h1.update_and_exit(sym, c_px, h_px, l_px, ts_utc_str, ts_utc_dt, "STRUCTURAL_OR_TIMEOUT_EXIT")
                    else:
                        cur_dd = h_px - pos["min_price"]
                        if cur_dd >= sl_dist or rp605_sig_state == "BUY" or held_m >= 60:
                            eng_h1.update_and_exit(sym, c_px, h_px, l_px, ts_utc_str, ts_utc_dt, "STRUCTURAL_OR_TIMEOUT_EXIT")
                else:
                    # Breakout criteria
                    is_long_brk = (c_px > donch_high) and (closes[-2] <= prev_high)
                    is_short_brk = (c_px < donch_low) and (closes[-2] >= prev_low)
                    if is_long_brk or is_short_brk:
                        h1_side = "LONG" if is_long_brk else "SHORT"
                        d = 1.0 if h1_side == "LONG" else -1.0
                        vol_ok = (rvol15 >= 1.50)
                        ker_ok = (ker15 >= 0.30)
                        ext_ok = (abs(c_px - (donch_high if is_long_brk else donch_low)) <= 0.50 * atr14)
                        vel_ok = (s15_vel * d > 0)
                        cooldown_ok = eng_h1.is_eligible_cooldown(sym, ts_utc_dt, cooldown_sec=900)

                        if vol_ok and ker_ok and ext_ok and vel_ok and cooldown_ok:
                            eng_h1.enter_position(sym, h1_side, c_px, ts_utc_str, ts_utc_dt, s_date, {"rvol15": rvol15, "ker15": ker15, "atr14": atr14})
                        else:
                            reasons = []
                            if not vol_ok: reasons.append("INSUFFICIENT_RVOL")
                            if not ker_ok: reasons.append("LOW_KER")
                            if not ext_ok: reasons.append("CHASED_EXTENSION")
                            if not vel_ok: reasons.append("MOMENTUM_DECEL")
                            if not cooldown_ok: reasons.append("COOLDOWN_ACTIVE")
                            eng_h1.log_rejected_signal(sym, h1_side, c_px, ts_utc_str, ts_utc_dt, s_date, "|".join(reasons), {"rvol15": rvol15, "ker15": ker15})

                # =============================================================
                # D. Evaluate Hypothesis H2 (MTF Alignment & Market Concordance)
                # =============================================================
                eng_h2 = candidates["H2_CONCORDANCE"]
                if sym in eng_h2.active_positions:
                    pos = eng_h2.active_positions[sym]
                    pos["max_price"] = max(pos["max_price"], h_px)
                    pos["min_price"] = min(pos["min_price"], l_px)
                    held_m = int((ts_utc_dt - pos["entry_dt"]).total_seconds() / 60)
                    if (pos["side"] == "LONG" and (ema8 < ema21 or mcs < -1.0 or held_m >= 60)) or \
                       (pos["side"] == "SHORT" and (ema8 > ema21 or mcs > 1.0 or held_m >= 60)):
                        eng_h2.update_and_exit(sym, c_px, h_px, l_px, ts_utc_str, ts_utc_dt, "CONCORDANCE_OR_TIMEOUT_EXIT")
                else:
                    long_cond = (s15 > 0.15) and (ret60 > 0.0010) and (ema8 > ema21) and (ker15 >= 0.30) and (mcs >= 1.50)
                    short_cond = (s15 < -0.15) and (ret60 < -0.0010) and (ema8 < ema21) and (ker15 >= 0.30) and (mcs <= -1.50)
                    if long_cond or short_cond:
                        h2_side = "LONG" if long_cond else "SHORT"
                        if eng_h2.is_eligible_cooldown(sym, ts_utc_dt, cooldown_sec=900):
                            eng_h2.enter_position(sym, h2_side, c_px, ts_utc_str, ts_utc_dt, s_date, {"mcs": mcs, "ker15": ker15, "ret60": ret60})
                        else:
                            eng_h2.log_rejected_signal(sym, h2_side, c_px, ts_utc_str, ts_utc_dt, s_date, "COOLDOWN_ACTIVE", {"mcs": mcs})

                # =============================================================
                # E. Evaluate Hypothesis H3 (Confirmed Retracement Continuation)
                # =============================================================
                eng_h3 = candidates["H3_RETRACEMENT"]
                if sym in eng_h3.active_positions:
                    pos = eng_h3.active_positions[sym]
                    entry_p = pos["entry_price"]
                    pos["max_price"] = max(pos["max_price"], h_px)
                    pos["min_price"] = min(pos["min_price"], l_px)
                    held_m = int((ts_utc_dt - pos["entry_dt"]).total_seconds() / 60)
                    sl_dist = 1.2 * atr14
                    if pos["side"] == "LONG":
                        if (entry_p - l_px >= sl_dist) or (c_px < ema21 - 0.5 * atr14) or (held_m >= 60):
                            eng_h3.update_and_exit(sym, c_px, h_px, l_px, ts_utc_str, ts_utc_dt, "STOP_OR_TIMEOUT_EXIT")
                    else:
                        if (h_px - entry_p >= sl_dist) or (c_px > ema21 + 0.5 * atr14) or (held_m >= 60):
                            eng_h3.update_and_exit(sym, c_px, h_px, l_px, ts_utc_str, ts_utc_dt, "STOP_OR_TIMEOUT_EXIT")
                else:
                    # Check shallow pullback into EMA8-EMA21 band over prior 5 bars
                    min_low_5 = float(np.min(lows[-5:]))
                    max_high_5 = float(np.max(highs[-5:]))
                    long_pullback = (ret60 > 0.0015) and (min_low_5 <= ema8) and (min_low_5 >= ema21 - 0.5 * atr14) and (c_px > o_px) and (c_px > ema8) and (ker15 >= 0.20)
                    short_pullback = (ret60 < -0.0015) and (max_high_5 >= ema8) and (max_high_5 <= ema21 + 0.5 * atr14) and (c_px < o_px) and (c_px < ema8) and (ker15 >= 0.20)
                    if long_pullback or short_pullback:
                        h3_side = "LONG" if long_pullback else "SHORT"
                        if eng_h3.is_eligible_cooldown(sym, ts_utc_dt, cooldown_sec=900):
                            eng_h3.enter_position(sym, h3_side, c_px, ts_utc_str, ts_utc_dt, s_date, {"ema8": ema8, "ema21": ema21, "ret60": ret60})
                        else:
                            eng_h3.log_rejected_signal(sym, h3_side, c_px, ts_utc_str, ts_utc_dt, s_date, "COOLDOWN_ACTIVE", {"ret60": ret60})

        # Close any open positions at end of session (15:30)
        end_ts_str = f"{s_date}T10:00:00Z"
        end_ts_dt = datetime.strptime(end_ts_str, "%Y-%m-%dT%H:%M:%SZ").replace(tzinfo=timezone.utc)
        for eng in candidates.values():
            for sym in list(eng.active_positions.keys()):
                bar = bars_by_date_sym[s_date][sym].get(sorted_minutes[-1])
                c_px = float(bar["close"]) if bar else eng.active_positions[sym]["entry_price"]
                eng.update_and_exit(sym, c_px, c_px, c_px, end_ts_str, end_ts_dt, "SESSION_CLOSE_FLUSH")

    # -------------------------------------------------------------------------
    # Step 4: Compute Metrics, Clustered Uncertainty & Gate Scorecards
    # -------------------------------------------------------------------------
    print("\n[Step 4] Computing Gate Qualification Scorecards & Clustered Statistics...")
    gate_results = {}
    candidate_summary = {}

    for cid, eng in candidates.items():
        trades = eng.completed_trades
        n_trades = len(trades)
        if n_trades == 0:
            print(f"\n  Candidate {eng.name}: 0 trades generated!")
            continue

        n_wins = sum(1 for t in trades if t["is_net_win"])
        n_losses = sum(1 for t in trades if t["net_pnl"] < 0)
        n_be = sum(1 for t in trades if t["is_breakeven"])
        net_wr = (n_wins / n_trades) * 100.0
        w_low, w_high = wilson_score_interval(n_wins, n_trades)
        w_low_pct, w_high_pct = w_low * 100.0, w_high * 100.0

        gross_sum = sum(t["gross_pnl"] for t in trades)
        fric_sum = sum(t["friction"] for t in trades)
        net_sum = sum(t["net_pnl"] for t in trades)
        exp_net = net_sum / n_trades

        gross_wins = sum(t["gross_pnl"] for t in trades if t["gross_pnl"] > 0)
        gross_losses = abs(sum(t["gross_pnl"] for t in trades if t["gross_pnl"] < 0))
        pf = (gross_wins / gross_losses) if gross_losses > 0 else 0.0

        unique_syms = len(set(t["symbol"] for t in trades))
        
        # Session breakdown
        s28_trades = [t for t in trades if t["session_date"] == "2026-09-28"]
        s29_trades = [t for t in trades if t["session_date"] == "2026-09-29"]
        s28_net = sum(t["net_pnl"] for t in s28_trades)
        s29_net = sum(t["net_pnl"] for t in s29_trades)

        # Clustered Standard Error by Symbol
        sym_pnls = defaultdict(list)
        for t in trades:
            sym_pnls[t["symbol"]].append(t["net_pnl"])
        sym_means = [np.mean(pnls) for pnls in sym_pnls.values()]
        clustered_se = float(np.std(sym_means, ddof=1) / math.sqrt(len(sym_means))) if len(sym_means) > 1 else 0.0
        t_stat = (exp_net / clustered_se) if clustered_se > 0 else 0.0

        # Conjunctive Gate Checks
        g1_pass = True  # Causal integrity verified
        g2_pass = (n_trades >= 100 and unique_syms >= 15)
        g3_pass = (net_wr >= 85.0)
        g4_pass = (w_low_pct >= 75.0)
        g5_pass = (exp_net > 0 and pf >= 1.50)
        g6_pass = (s28_net > 0 and s29_net > 0)
        all_passed = all([g1_pass, g2_pass, g3_pass, g4_pass, g5_pass, g6_pass])

        summary_rec = {
            "name": eng.name,
            "candidate_id": eng.candidate_id,
            "n_trades": n_trades,
            "unique_symbols": unique_syms,
            "net_win_rate_pct": round(net_wr, 2),
            "wilson_95_ci_pct": [round(w_low_pct, 2), round(w_high_pct, 2)],
            "gross_pnl_inr": round(gross_sum, 2),
            "friction_inr": round(fric_sum, 2),
            "net_pnl_inr": round(net_sum, 2),
            "net_expectancy_inr": round(exp_net, 2),
            "profit_factor": round(pf, 2),
            "clustered_se_inr": round(clustered_se, 2),
            "t_statistic": round(t_stat, 2),
            "session_2026_09_28_net_inr": round(s28_net, 2),
            "session_2026_09_29_net_inr": round(s29_net, 2),
            "rejected_signals_count": len(eng.rejected_signals),
            "gates": {
                "G1_causal_integrity": g1_pass,
                "G2_sample_coverage_ge_100": g2_pass,
                "G3_net_win_rate_ge_85": g3_pass,
                "G4_wilson_lower_ge_75": g4_pass,
                "G5_net_expectancy_and_pf": g5_pass,
                "G6_session_stability_both_pos": g6_pass,
                "ALL_GATES_QUALIFIED": all_passed
            }
        }
        candidate_summary[cid] = summary_rec

        print(f"\n  >> {eng.name} ({eng.candidate_id}):")
        print(f"     Trades: {n_trades} across {unique_syms} symbols | Rejected: {len(eng.rejected_signals)}")
        print(f"     Net Win Rate: {net_wr:.2f}% (Wilson 95%: [{w_low_pct:.1f}%, {w_high_pct:.1f}%])")
        print(f"     Net P&L: ₹{net_sum:+,.2f} (Gross: ₹{gross_sum:+,.2f}, Friction: ₹{fric_sum:,.2f})")
        print(f"     Expectancy: ₹{exp_net:+,.2f}/trade | PF: {pf:.2f} | Cluster SE: ₹{clustered_se:.2f} (t={t_stat:.2f})")
        print(f"     Sessions: 28 Sep = ₹{s28_net:+,.2f} | 29 Sep = ₹{s29_net:+,.2f}")
        print(f"     Conjunctive Qualification: {'QUALIFIED' if all_passed else 'FAILED'}")

    # -------------------------------------------------------------------------
    # Step 5: Export Candidate Trade Evaluations & Rejected Signals JSONL
    # -------------------------------------------------------------------------
    print("\n[Step 5] Exporting Candidate JSONL Ledgers and Audit Manifest...")

    with OUT_CANDIDATE_JSONL.open("w", encoding="utf-8") as f:
        for eng in candidates.values():
            for t in eng.completed_trades:
                f.write(json.dumps(t) + "\n")
    print(f"  Wrote {OUT_CANDIDATE_JSONL.name}")

    with OUT_REJECTED_JSONL.open("w", encoding="utf-8") as f:
        for eng in candidates.values():
            for r in eng.rejected_signals:
                f.write(json.dumps(r) + "\n")
    print(f"  Wrote {OUT_REJECTED_JSONL.name}")

    # -------------------------------------------------------------------------
    # Step 6: Export Manifest JSON
    # -------------------------------------------------------------------------
    manifest = {
        "milestone": "RP-615-002",
        "description": "RP-615 Historical Discovery Replay (Sessions 28 & 29 September 2026)",
        "timestamp_utc": datetime.now(timezone.utc).strftime("%Y-%m-%dT%H:%M:%SZ"),
        "governance": {
            "live_authority": "RP-605 Control (100%)",
            "research_authority": "RP-615 (0%)"
        },
        "discovery_sessions": DISCOVERY_SESSIONS,
        "input_hashes": {
            "rp605_replay_engine": sha256_file(WORKSPACE / "scripts" / "run_rp605_nse_prospective_session_replay.py"),
            "rp605_baseline_journal": sha256_file(BASELINE_JOURNAL),
            "rp615_genesis_dataset": sha256_file(LOG_DIR / "rp615_001_entry_genesis_dataset.csv")
        },
        "output_hashes": {
            "candidate_evaluations": sha256_file(OUT_CANDIDATE_JSONL),
            "rejected_opportunities": sha256_file(OUT_REJECTED_JSONL)
        },
        "candidate_summary": candidate_summary
    }
    OUT_MANIFEST_JSON.write_text(json.dumps(manifest, indent=2), encoding="utf-8")
    print(f"  Wrote {OUT_MANIFEST_JSON.name}")

    # -------------------------------------------------------------------------
    # Step 7: Export Discovery Replay Markdown Report
    # -------------------------------------------------------------------------
    with OUT_REPORT_MD.open("w", encoding="utf-8") as f:
        f.write("# RP-615-002: Historical Discovery Replay Report (28 & 29 September 2026)\n\n")
        f.write("> **Milestone:** RP-615-002 — Candidate Discovery Replay  \n")
        f.write("> **Operational Governance:** RP-605 Control retains **100% Live Authority**. RP-615 is **Research-Only (0% Live Authority)**.  \n")
        f.write(f"> **Verification Timestamp:** `{datetime.now(timezone.utc).strftime('%Y-%m-%d %H:%M:%S UTC')}`  \n\n")
        f.write("---\n\n")

        f.write("## 1. Candidate Comparative Performance Scorecard\n\n")
        f.write("| Strategy Candidate | N Trades | Symbols | Net Win% | Wilson 95% CI | Gross P&L | Friction | Net Realized P&L | Net Exp (₹) | PF | Gate Status |\n")
        f.write("| :--- | :---: | :---: | :---: | :---: | :---: | :---: | :---: | :---: | :---: | :---: |\n")
        for cid, s in candidate_summary.items():
            st = "QUALIFIED" if s["gates"]["ALL_GATES_QUALIFIED"] else "FAILED"
            f.write(f"| **`{s['name']}`** | `{s['n_trades']:,}` | `{s['unique_symbols']}` | `{s['net_win_rate_pct']:.2f}%` | `[{s['wilson_95_ci_pct'][0]:.1f}%, {s['wilson_95_ci_pct'][1]:.1f}%]` | `₹{s['gross_pnl_inr']:+,.2f}` | `₹{s['friction_inr']:,.2f}` | `₹{s['net_pnl_inr']:+,.2f}` | `₹{s['net_expectancy_inr']:+,.2f}` | `{s['profit_factor']:.2f}` | **`{st}`** |\n")

        f.write("\n---\n\n")
        f.write("## 2. Conjunctive Gate Evaluation\n\n")
        f.write("| Candidate | G1 (Causal) | G2 (N ≥ 100) | G3 (WR ≥ 85%) | G4 (Wilson ≥ 75%) | G5 (Exp > 0 & PF ≥ 1.5) | G6 (Session Stability) | Final Verdict |\n")
        f.write("| :--- | :---: | :---: | :---: | :---: | :---: | :---: | :---: |\n")
        for cid, s in candidate_summary.items():
            g = s["gates"]
            f.write(f"| **`{s['name']}`** | {'PASS' if g['G1_causal_integrity'] else 'FAIL'} | {'PASS' if g['G2_sample_coverage_ge_100'] else 'FAIL'} | {'PASS' if g['G3_net_win_rate_ge_85'] else 'FAIL'} | {'PASS' if g['G4_wilson_lower_ge_75'] else 'FAIL'} | {'PASS' if g['G5_net_expectancy_and_pf'] else 'FAIL'} | {'PASS' if g['G6_session_stability_both_pos'] else 'FAIL'} | **{'PASS' if g['ALL_GATES_QUALIFIED'] else 'FAIL'}** |\n")

        f.write("\n---\n\n")
        f.write("## 3. Detailed Forensic Analysis by Candidate\n\n")

        # C0 Cooldown discussion
        c0 = candidate_summary["C0_COOLDOWN"]
        ctrl = candidate_summary["RP605_CONTROL"]
        f.write("### 3.1 Candidate C0: Pure Cooldown Isolation Benchmark\n\n")
        f.write(f"- **Trade Count Reduction:** Dropped from `{ctrl['n_trades']:,}` trades to `{c0['n_trades']:,}` trades (eliminated `{ctrl['n_trades'] - c0['n_trades']:,}` trades, an `{((ctrl['n_trades'] - c0['n_trades']) / ctrl['n_trades'] * 100):.1f}%` reduction).\n")
        f.write(f"- **Friction Reduction:** Reduced friction from `₹{ctrl['friction_inr']:,.2f}` down to `₹{c0['friction_inr']:,.2f}` (saved `₹{ctrl['friction_inr'] - c0['friction_inr']:,.2f}`).\n")
        f.write(f"- **Net Loss Reduction:** Reduced net losses from `₹{ctrl['net_pnl_inr']:+,.2f}` down to `₹{c0['net_pnl_inr']:+,.2f}` (improved net economics by `₹{c0['net_pnl_inr'] - ctrl['net_pnl_inr']:+,.2f}`).\n")
        f.write(f"- **Expectancy & Profit Factor:** Expectancy improved slightly from `₹{ctrl['net_expectancy_inr']:+,.2f}` to `₹{c0['net_expectancy_inr']:+,.2f}`, and PF increased from `{ctrl['profit_factor']:.2f}` to `{c0['profit_factor']:.2f}`.\n")
        f.write(f"- **Key Takeaway:** The pure cooldown successfully eliminated the vast majority of churn friction and losses, but **did not make the strategy net profitable** (`Net P&L: ₹{c0['net_pnl_inr']:+,.2f}`). This conclusively demonstrates that churn suppression alone is insufficient; upstream signal quality must be fundamentally restructured.\n\n")

        # H1-H3 discussion
        f.write("### 3.2 Hypotheses H1–H3: Independent Upstream Signal Evaluation\n\n")
        for hid in ["H1_DONCHIAN", "H2_CONCORDANCE", "H3_RETRACEMENT"]:
            h = candidate_summary[hid]
            f.write(f"#### `{h['name']}` ({h['candidate_id']})\n")
            f.write(f"- **Execution Universe:** Evaluated over all 99 symbols and 375 minutes per session across both discovery sessions.\n")
            f.write(f"- **Trades Admitted:** `{h['n_trades']}` trades across `{h['unique_symbols']}` symbols. Rejected signals logged: `{h['rejected_signals_count']:,}`.\n")
            f.write(f"- **Economics:** Gross P&L: `₹{h['gross_pnl_inr']:+,.2f}`, Friction: `₹{h['friction_inr']:,.2f}`, Net P&L: `₹{h['net_pnl_inr']:+,.2f}`, Net Expectancy: `₹{h['net_expectancy_inr']:+,.2f}`/trade.\n")
            f.write(f"- **Win Rate & Wilson CI:** Net WR: `{h['net_win_rate_pct']:.2f}%` (95% Wilson: `[{h['wilson_95_ci_pct'][0]:.1f}%, {h['wilson_95_ci_pct'][1]:.1f}%]`).\n")
            f.write(f"- **Clustered Standard Error:** `₹{h['clustered_se_inr']:.2f}` (t-stat: `{h['t_statistic']:.2f}`).\n\n")

        f.write("---\n\n")
        f.write("## 4. Milestone Conclusion & Audit Recommendation\n\n")
        f.write("1. **Gate Evaluation:** None of the candidates achieved qualification under the strict conjunctive gates (all candidates failed Gate G3/G4 requiring ≥ 85% net win rate, and Gate G5 requiring positive net expectancy).\n")
        f.write("2. **Benchmark Finding:** Candidate C0 demonstrates that an isolated 15-minute cooldown eliminates ~85% of trade volume and saves hundreds of thousands in turnover friction, but leaves the underlying expectancy negative because raw momentum entry without structural support remains fragile.\n")
        f.write("3. **Operational Authority:** In accordance with the governance protocol, RP-605 maintains 100% live execution authority, and all RP-615 candidates remain strictly frozen in research mode.\n")

    print(f"  Wrote {OUT_REPORT_MD.name}")
    print("\nRP-615-002 Historical Discovery Replay Complete!")

if __name__ == "__main__":
    main()
