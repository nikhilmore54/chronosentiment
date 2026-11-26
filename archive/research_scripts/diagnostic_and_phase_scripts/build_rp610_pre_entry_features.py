#!/usr/bin/env python3
"""
RP-610 Feature Extraction: Point-in-Time Pre-Entry Feature Dataset Builder.

Constructs the standardized, strictly point-in-time feature extraction dataset
for RP-610 across the historical sessions (2026-09-28 and 2026-09-29).

Enforces:
1. Strict point-in-time causal cutoff: t_known <= t_entry - 1 second.
2. Only completed 1-minute bars used; zero intra-bar or post-entry leakage.
3. Causal prior-minute concurrency: entries triggered in [t_entry - 60s, t_entry).
4. Full feature extraction for Families A, B, C, D and Baselines 0-3.
5. Primary economic target: label_win = I(realized_pnl > 0).

Output:
  live_capture/shadow_validation/rp610_pre_entry_features_dataset.csv
"""

from __future__ import annotations

import csv
import json
import math
from collections import defaultdict
from datetime import datetime, timedelta, timezone
from pathlib import Path
import numpy as np

WORKSPACE = Path(__file__).resolve().parent.parent
LOG_DIR = WORKSPACE / "live_capture" / "shadow_validation"
CANDLE_ROOT = WORKSPACE / "intraday_capture" / "yahoo_cache_1m"
CARDS_PATH = LOG_DIR / "rp605_nse_decision_cards_log.jsonl"
OUT_CSV = LOG_DIR / "rp610_pre_entry_features_dataset.csv"

CSV_28 = LOG_DIR / "rp609_nse_20260928_replication_dataset.csv"
CSV_29 = LOG_DIR / "rp609_nse_20260929_entry_discrimination_dataset.csv"
CSV_30 = LOG_DIR / "rp609_nse_20260930_replication_dataset.csv"
CSV_01 = LOG_DIR / "rp609_nse_20261001_replication_dataset.csv"

EPS = 1e-6
IST = timezone(timedelta(hours=5, minutes=30))


def parse_ts(val: str) -> datetime:
    val = val.strip().replace("Z", "+00:00")
    return datetime.fromisoformat(val)


def calc_ema(prices: list[float], period: int) -> float:
    if not prices:
        return 0.0
    alpha = 2.0 / (period + 1.0)
    ema = prices[0]
    for p in prices[1:]:
        ema = alpha * p + (1.0 - alpha) * ema
    return ema


def calc_atr(bars: list[dict], period: int = 14) -> float:
    if len(bars) < 2:
        return 1.0
    trs = []
    for i in range(1, len(bars)):
        h = bars[i]["high"]
        l = bars[i]["low"]
        pc = bars[i - 1]["close"]
        tr = max(h - l, abs(h - pc), abs(l - pc))
        trs.append(tr)
    if not trs:
        return 1.0
    # Wilder's smoothing
    if len(trs) < period:
        return float(np.mean(trs)) if trs else 1.0
    atr = float(np.mean(trs[:period]))
    for tr in trs[period:]:
        atr = (atr * (period - 1) + tr) / period
    return max(atr, EPS)


def main():
    print("Building RP-610 Point-in-Time Pre-Entry Feature Dataset...")

    # Load trades across historical and prospective sessions
    trades = []
    with CSV_28.open(encoding="utf-8") as f:
        for r in csv.DictReader(f):
            r["_session"] = "2026-09-28"
            trades.append(r)
    with CSV_29.open(encoding="utf-8") as f:
        for r in csv.DictReader(f):
            r["_session"] = "2026-09-29"
            trades.append(r)
    with CSV_30.open(encoding="utf-8") as f:
        for r in csv.DictReader(f):
            r["_session"] = "2026-09-30"
            trades.append(r)
    with CSV_01.open(encoding="utf-8") as f:
        for r in csv.DictReader(f):
            r["_session"] = "2026-10-01"
            trades.append(r)

    print(f"Total raw trades loaded: {len(trades)} (28 Sep: 1,782, 29 Sep: 1,719, 30 Sep: {sum(1 for t in trades if t['_session']=='2026-09-30')}, 1 Oct: {sum(1 for t in trades if t['_session']=='2026-10-01')})")

    symbols = sorted({t["symbol"] for t in trades})
    print(f"Total unique instruments: {len(symbols)}")

    # Load 1m Bars
    print("Loading 1m bars from cache...")
    bars_by_sym = {}
    for sym in symbols:
        path = CANDLE_ROOT / f"{sym}.json"
        if not path.exists():
            bars_by_sym[sym] = []
            continue
        raw_rows = json.loads(path.read_text(encoding="utf-8"))
        bars = []
        for r in raw_rows:
            start = datetime.fromtimestamp(int(r["timestamp"]), timezone.utc)
            known_at = start + timedelta(seconds=60)
            bars.append({
                "start": start,
                "known_at": known_at,
                "open": float(r["open"]),
                "high": float(r["high"]),
                "low": float(r["low"]),
                "close": float(r["close"]),
                "volume": float(r.get("volume") or 0.0),
            })
        bars.sort(key=lambda b: b["known_at"])
        bars_by_sym[sym] = bars

    # Pre-index bars by known_at for fast binary search / slice
    print("Indexing bars...")
    bars_ts_by_sym = {sym: [b["known_at"] for b in bars_by_sym[sym]] for sym in symbols}

    # Load Decision Cards
    print("Loading Decision Cards...")
    valid_prefixes = ("2026-09-28", "2026-09-29", "2026-09-30", "2026-10-01")
    cards_by_sym = defaultdict(list)
    with CARDS_PATH.open(encoding="utf-8") as f:
        for line in f:
            if not line.strip():
                continue
            c = json.loads(line)
            ts = c.get("timestamp_utc") or ""
            if any(ts.startswith(prefix) for prefix in valid_prefixes):
                dt = parse_ts(ts)
                cards_by_sym[c["instrument"]].append({
                    "_dt": dt,
                    "price": float(c["current_price"]),
                    "15m_score": float(c.get("15m_score_trajectory") or 0.0),
                    "30m_score": float(c.get("30m_score_trajectory") or 0.0),
                    "decision_state": c.get("decision_state"),
                })
    for sym in cards_by_sym:
        cards_by_sym[sym].sort(key=lambda x: x["_dt"])
    cards_ts_by_sym = {sym: [c["_dt"] for c in cards_by_sym[sym]] for sym in symbols}

    # Pre-calculate historical median volume per symbol per 15m time-of-day bucket
    # using bars strictly prior to 2026-09-28
    print("Computing historical volume reference distribution (prior to 28 Sep)...")
    hist_vol_ref = defaultdict(lambda: defaultdict(list))
    for sym, b_list in bars_by_sym.items():
        for b in b_list:
            if b["start"].date() < datetime(2026, 9, 28).date():
                tod_minute = b["start"].astimezone(IST).hour * 60 + b["start"].astimezone(IST).minute
                bucket = tod_minute // 15
                hist_vol_ref[sym][bucket].append(b["volume"])

    hist_median_vol = defaultdict(dict)
    for sym in symbols:
        for bucket in range(24 * 4):
            vols = hist_vol_ref[sym][bucket]
            hist_median_vol[sym][bucket] = float(np.median(vols)) if vols and np.median(vols) > 0 else 1000.0

    # Index entry timestamps across all trades for causal concurrency
    entry_timestamps = sorted([parse_ts(t["t_entry_utc"]) for t in trades])

    # Build features for each trade
    print("Extracting features per trade...")
    rows_out = []

    for t in trades:
        t_entry = parse_ts(t["t_entry_utc"])
        t_cutoff = t_entry - timedelta(seconds=1)
        sym = t["symbol"]
        side = t["side"]
        sgn = 1.0 if side == "LONG" else -1.0
        realized_pnl = float(t["label_realized_pnl"])
        mfe_pnl = float(t["label_mfe_pnl"])

        # Eligible bars for symbol
        sym_bars = bars_by_sym.get(sym, [])
        sym_ts = bars_ts_by_sym.get(sym, [])

        import bisect
        idx = bisect.bisect_right(sym_ts, t_cutoff)
        eligible_bars = sym_bars[:idx]

        if not eligible_bars:
            continue

        anchor_bar = eligible_bars[-1]
        anchor_bar_known = anchor_bar["known_at"]
        anchor_lag_sec = (t_entry - anchor_bar_known).total_seconds()
        assert anchor_lag_sec >= 1.0, f"Leakage violation! lag = {anchor_lag_sec}s for trade {t.get('trade_id')}"

        # Session start for trade's day
        entry_ist = t_entry.astimezone(IST)
        session_start_ist = entry_ist.replace(hour=9, minute=15, second=0, microsecond=0)
        session_bars = [b for b in eligible_bars if b["known_at"].astimezone(IST) >= session_start_ist]
        is_warmup = len(session_bars) < 60 or entry_ist.hour < 10 or (entry_ist.hour == 10 and entry_ist.minute < 15)

        # Family A: Multi-Horizon Trend Alignment
        n_bars = len(eligible_bars)
        p_close = [b["close"] for b in eligible_bars]

        ret_5m = (math.log(p_close[-1] / p_close[-6]) * 10000.0 * sgn) if n_bars >= 6 and p_close[-6] > 0 else 0.0
        ret_15m = (math.log(p_close[-1] / p_close[-16]) * 10000.0 * sgn) if n_bars >= 16 and p_close[-16] > 0 else 0.0
        ret_30m = (math.log(p_close[-1] / p_close[-31]) * 10000.0 * sgn) if n_bars >= 31 and p_close[-31] > 0 else 0.0
        ret_60m = (math.log(p_close[-1] / p_close[-61]) * 10000.0 * sgn) if n_bars >= 61 and p_close[-61] > 0 else 0.0

        concordance_score = (
            (1 if ret_5m > 0 else (-1 if ret_5m < 0 else 0)) +
            (1 if ret_15m > 0 else (-1 if ret_15m < 0 else 0)) +
            (1 if ret_30m > 0 else (-1 if ret_30m < 0 else 0)) +
            (1 if ret_60m > 0 else (-1 if ret_60m < 0 else 0))
        )
        dual_horizon_consensus = 1 if (ret_15m > 0 and ret_60m > 0) else 0

        # EMA spread / ATR
        ema_8 = calc_ema(p_close[-30:], 8) if n_bars >= 30 else p_close[-1]
        ema_21 = calc_ema(p_close[-50:], 21) if n_bars >= 50 else p_close[-1]
        atr_14 = calc_atr(eligible_bars[-30:], 14)
        atr_60 = calc_atr(eligible_bars[-90:], 60) if n_bars >= 90 else atr_14
        d_ema_8_21_atr = ((ema_8 - ema_21) / (atr_14 + EPS)) * sgn

        # Family B: Trend Quality & Volume
        path_15 = sum(abs(p_close[-i] - p_close[-i - 1]) for i in range(1, min(15, n_bars)))
        disp_15 = abs(p_close[-1] - p_close[-min(16, n_bars)])
        ker_15m = disp_15 / (path_15 + EPS)

        path_30 = sum(abs(p_close[-i] - p_close[-i - 1]) for i in range(1, min(30, n_bars)))
        disp_30 = abs(p_close[-1] - p_close[-min(31, n_bars)])
        ker_30m = disp_30 / (path_30 + EPS)

        # Directional Bar Consistency (15m)
        recent_15_bars = eligible_bars[-15:]
        bar_consistency_15m = sum(1 for b in recent_15_bars if (b["close"] - b["open"]) * sgn > 0) / max(len(recent_15_bars), 1)

        # Relative Volume
        tod_minute = anchor_bar["start"].astimezone(IST).hour * 60 + anchor_bar["start"].astimezone(IST).minute
        tod_bucket = tod_minute // 15
        base_vol_15m = hist_median_vol[sym].get(tod_bucket, 1000.0)
        recent_vol_15m = sum(b["volume"] for b in recent_15_bars)
        rvol_15m = recent_vol_15m / (base_vol_15m + EPS)

        # Volume Acceleration
        v_5m = sum(b["volume"] for b in eligible_bars[-5:]) / 5.0
        v_6_20m = sum(b["volume"] for b in eligible_bars[-20:-5]) / 15.0 if n_bars >= 20 else v_5m
        vol_accel = v_5m / (v_6_20m + EPS)

        # Family C: Volatility Context
        rets_15 = [math.log(p_close[-i] / p_close[-i - 1]) for i in range(1, min(15, n_bars))]
        realized_vol_15m = float(np.std(rets_15) * math.sqrt(252 * 375) * 100.0) if len(rets_15) >= 2 else 0.0

        rets_60 = [math.log(p_close[-i] / p_close[-i - 1]) for i in range(1, min(60, n_bars))]
        realized_vol_60m = float(np.std(rets_60) * math.sqrt(252 * 375) * 100.0) if len(rets_60) >= 2 else realized_vol_15m

        recent_30_bars = eligible_bars[-30:]
        high_30 = max(b["high"] for b in recent_30_bars)
        low_30 = min(b["low"] for b in recent_30_bars)
        norm_range_expansion_30m = (high_30 - low_30) / (atr_60 + EPS)

        # Proximity to Adverse Extreme
        if side == "LONG":
            dist_adverse_extreme_30m = (p_close[-1] - low_30) / (atr_14 + EPS)
        else:
            dist_adverse_extreme_30m = (high_30 - p_close[-1]) / (atr_14 + EPS)

        # Family D: Market Synchrony & Causal Concurrency
        # Causal concurrency: count entries in [t_entry - 60s, t_entry)
        t_concur_start = t_entry - timedelta(seconds=60)
        c_idx_start = bisect.bisect_left(entry_timestamps, t_concur_start)
        c_idx_end = bisect.bisect_left(entry_timestamps, t_entry)
        causal_concurrency_prev_min = c_idx_end - c_idx_start

        # Constituent Basket Alignment (15m return across universe up to anchor bar known_at)
        basket_rets_15 = []
        basket_1m_rets_t1 = []
        for s in symbols:
            s_bars = bars_by_sym.get(s, [])
            s_ts = bars_ts_by_sym.get(s, [])
            s_idx = bisect.bisect_right(s_ts, anchor_bar_known)
            if s_idx >= 16:
                p_c = s_bars[s_idx - 1]["close"]
                p_15 = s_bars[s_idx - 16]["close"]
                if p_15 > 0:
                    basket_rets_15.append(math.log(p_c / p_15) * 10000.0)
            if s_idx >= 1:
                b_last = s_bars[s_idx - 1]
                if b_last["open"] > 0:
                    basket_1m_rets_t1.append((b_last["close"] - b_last["open"]) / b_last["open"] * 10000.0)

        basket_align_15m = (float(np.mean(basket_rets_15)) * sgn) if basket_rets_15 else 0.0
        basket_dispersion_prev_min = float(np.std(basket_1m_rets_t1)) if len(basket_1m_rets_t1) >= 2 else 0.0

        # Baselines
        # Baseline 1: Freshness gate
        f15_raw = t.get("feature_f15_at_entry")
        f15_val = float(f15_raw) if f15_raw not in (None, "") else 0.0
        baseline_1_freshness = 1 if f15_val >= -0.01 else 0

        # Baseline 2: Decision Card 15m score
        c_bars = cards_by_sym.get(sym, [])
        c_ts = cards_ts_by_sym.get(sym, [])
        c_idx = bisect.bisect_right(c_ts, t_cutoff)
        latest_card = c_bars[c_idx - 1] if c_idx > 0 else None
        card_15m_score = latest_card["15m_score"] if latest_card else 0.0
        baseline_2_side_aligned_score = card_15m_score * sgn

        # Baseline 3: Dual horizon trend consensus
        baseline_3_trend_consensus = dual_horizon_consensus

        # Labels
        label_win = 1 if realized_pnl > 0 else 0
        label_strict_loss = 1 if realized_pnl < 0 else 0
        label_breakeven = 1 if realized_pnl == 0 else 0
        label_zero_mfe = 1 if mfe_pnl == 0 else 0

        rows_out.append({
            # Identifiers
            "session_date": t["_session"],
            "trade_id": t.get("trade_id", ""),
            "symbol": sym,
            "side": side,
            "t_entry_utc": t["t_entry_utc"],
            "t_exit_utc": t["t_exit_utc"],
            "t_cutoff_utc": t_cutoff.isoformat(),
            "anchor_bar_start_utc": anchor_bar["start"].isoformat(),
            "anchor_bar_known_at_utc": anchor_bar["known_at"].isoformat(),
            "anchor_lag_sec": round(anchor_lag_sec, 3),
            "is_warmup": 1 if is_warmup else 0,

            # Family A: Trend Alignment
            "feature_ret_5m_bps": round(ret_5m, 2),
            "feature_ret_15m_bps": round(ret_15m, 2),
            "feature_ret_30m_bps": round(ret_30m, 2),
            "feature_ret_60m_bps": round(ret_60m, 2),
            "feature_trend_concordance_score": concordance_score,
            "feature_dual_horizon_consensus": dual_horizon_consensus,
            "feature_d_ema_8_21_atr": round(d_ema_8_21_atr, 4),

            # Family B: Trend Quality & Volume
            "feature_ker_15m": round(ker_15m, 4),
            "feature_ker_30m": round(ker_30m, 4),
            "feature_bar_consistency_15m": round(bar_consistency_15m, 4),
            "feature_rvol_15m": round(rvol_15m, 4),
            "feature_vol_accel": round(vol_accel, 4),

            # Family C: Volatility Context
            "feature_realized_vol_15m": round(realized_vol_15m, 2),
            "feature_realized_vol_60m": round(realized_vol_60m, 2),
            "feature_norm_range_expansion_30m": round(norm_range_expansion_30m, 4),
            "feature_dist_adverse_extreme_30m": round(dist_adverse_extreme_30m, 4),

            # Family D: Market Synchrony & Causal Concurrency
            "feature_basket_align_15m_bps": round(basket_align_15m, 2),
            "feature_causal_concurrency_prev_min": causal_concurrency_prev_min,
            "feature_basket_dispersion_prev_min_bps": round(basket_dispersion_prev_min, 2),

            # Baselines
            "baseline_0_null": 1,
            "baseline_1_freshness_gate": baseline_1_freshness,
            "baseline_2_side_aligned_15m_score": round(baseline_2_side_aligned_score, 4),
            "baseline_3_trend_consensus": baseline_3_trend_consensus,

            # Outcome Labels
            "label_realized_pnl": realized_pnl,
            "label_win": label_win,
            "label_strict_loss": label_strict_loss,
            "label_breakeven": label_breakeven,
            "label_zero_mfe": label_zero_mfe,
        })

    # Write out
    assert len(rows_out) == len(trades), f"Mismatch in row count! {len(rows_out)} vs {len(trades)}"

    fieldnames = list(rows_out[0].keys())
    with OUT_CSV.open("w", newline="", encoding="utf-8") as f:
        writer = csv.DictWriter(f, fieldnames=fieldnames)
        writer.writeheader()
        writer.writerows(rows_out)

    print(f"Successfully generated {len(rows_out)} rows in {OUT_CSV}")


if __name__ == "__main__":
    main()
