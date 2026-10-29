#!/usr/bin/env python3
"""
RP-608-REGIME-BREAKOUT-001 Phase 0: In-sample Sessions 1–40 census.

Records completed sessions, eligible Arm A opportunity proxies, data gaps,
and whether eligibility floors are achievable. Does NOT relax thresholds.
Holdout Sessions 41+ are counted but never used for opportunity scanning.
Live order authority: 0.0%.
"""

from __future__ import annotations

import json
import math
from datetime import datetime, timezone, timedelta, date
from pathlib import Path
from typing import Any, Dict, List, Optional, Set, Tuple

WORKSPACE_DIR = Path(__file__).resolve().parent.parent
IST_OFFSET = timedelta(hours=5, minutes=30)

NSE_2026_HOLIDAYS = {
    date(2026, 1, 26), date(2026, 3, 3), date(2026, 3, 20), date(2026, 3, 26),
    date(2026, 3, 27), date(2026, 4, 2), date(2026, 4, 3), date(2026, 4, 14),
    date(2026, 5, 1), date(2026, 5, 28), date(2026, 6, 17), date(2026, 7, 2),
    date(2026, 8, 15), date(2026, 10, 2), date(2026, 10, 20),
    date(2026, 11, 8), date(2026, 11, 9), date(2026, 11, 24), date(2026, 12, 25),
}

# Eligibility floors from protocol (not power guarantees)
NSE_FLOOR_SESSIONS = 40
NSE_FLOOR_OPPORTUNITIES = 100
CRYPTO_FLOOR_BLOCKS = 20
CRYPTO_FLOOR_OPPORTUNITIES = 50


def _ts_to_ist_date(ts: float) -> Optional[date]:
    try:
        ts_ms = int(ts * 1000) if ts < 1e11 else int(ts)
        dt_utc = datetime.fromtimestamp(ts_ms / 1000.0, tz=timezone.utc)
        dt_ist = dt_utc + IST_OFFSET
        d = dt_ist.date()
        if dt_ist.weekday() >= 5 or d in NSE_2026_HOLIDAYS:
            return None
        return d
    except Exception:
        return None


def _bar_idx_ist(ts: float) -> Optional[int]:
    try:
        ts_ms = int(ts * 1000) if ts < 1e11 else int(ts)
        dt_utc = datetime.fromtimestamp(ts_ms / 1000.0, tz=timezone.utc)
        dt_ist = dt_utc + IST_OFFSET
        open_t = dt_ist.replace(hour=9, minute=15, second=0, microsecond=0)
        delta_min = (dt_ist - open_t).total_seconds() / 60.0
        if delta_min < 0 or delta_min >= 375:
            return None
        # Bar close indexing: first close at 09:30 = bar 1
        return int(delta_min // 15)
    except Exception:
        return None


def census_nse(cache_dir: Path) -> Dict[str, Any]:
    all_dates: Set[str] = set()
    symbol_sessions: Dict[str, Set[str]] = {}
    symbol_gaps: Dict[str, List[str]] = {}
    # Arm A opportunity proxy: traj15 >= 0.05, bar_idx in 1..20, >=5 closes in session
    opportunities = 0
    evaluation_census = 0
    symbols_loaded = 0
    corrupt_files = 0
    bars_is = 0

    paths = sorted(cache_dir.glob("*.json"))
    for p in paths:
        sym = p.stem
        try:
            candles = json.loads(p.read_text(encoding="utf-8"))
        except Exception:
            corrupt_files += 1
            continue
        if not isinstance(candles, list) or not candles:
            corrupt_files += 1
            continue
        symbols_loaded += 1
        by_session: Dict[str, List[dict]] = {}
        for c in candles:
            ts = c.get("timestamp")
            if ts is None:
                continue
            d = _ts_to_ist_date(float(ts))
            if d is None:
                continue
            ds = str(d)
            all_dates.add(ds)
            by_session.setdefault(ds, []).append(c)

        symbol_sessions[sym] = set(by_session.keys())

    sorted_dates = sorted(all_dates)
    is_sessions = sorted_dates[:40]
    holdout_sessions = sorted_dates[40:]
    is_set = set(is_sessions)

    # Gaps: symbols missing any IS session
    for sym, sess in symbol_sessions.items():
        missing = [s for s in is_sessions if s not in sess]
        if missing:
            symbol_gaps[sym] = missing[:10]  # sample

    # Opportunity scan on IS only
    for p in paths:
        try:
            candles = json.loads(p.read_text(encoding="utf-8"))
        except Exception:
            continue
        by_session: Dict[str, List[Tuple[int, float]]] = {}
        for c in candles:
            ts = c.get("timestamp")
            close = c.get("close")
            if ts is None or close is None:
                continue
            d = _ts_to_ist_date(float(ts))
            if d is None or str(d) not in is_set:
                continue
            bi = _bar_idx_ist(float(ts))
            if bi is None or bi < 1:
                continue
            by_session.setdefault(str(d), []).append((bi, float(close)))
            bars_is += 1

        for sess, bars in by_session.items():
            bars_sorted = sorted(bars, key=lambda x: x[0])
            closes: List[float] = []
            for bi, cl in bars_sorted:
                closes.append(cl)
                evaluation_census += 1
                if len(closes) < 5:
                    continue
                if not (1 <= bi <= 20):
                    continue
                lookback = min(len(closes), 15)
                ret = (closes[-1] - closes[-lookback]) / (closes[-lookback] + 1e-9)
                diffs = [
                    (closes[i] - closes[i - 1]) / (closes[i - 1] + 1e-9)
                    for i in range(-lookback + 1, 0)
                ]
                vol = (sum(d * d for d in diffs) / len(diffs)) ** 0.5 if diffs else 1e-3
                traj15 = math.tanh(ret / ((vol * (lookback ** 0.5) * 2.5) + 1e-9))
                if traj15 >= 0.05:
                    opportunities += 1

    floor_sessions_ok = len(is_sessions) >= NSE_FLOOR_SESSIONS
    floor_opp_ok = opportunities >= NSE_FLOOR_OPPORTUNITIES
    feasibility = "ACHIEVABLE" if (floor_sessions_ok and floor_opp_ok) else "INCONCLUSIVE"

    return {
        "market": "NSE",
        "cache_path": str(cache_dir),
        "symbols_loaded": symbols_loaded,
        "corrupt_files": corrupt_files,
        "total_sessions_observed": len(sorted_dates),
        "is_sessions_count": len(is_sessions),
        "is_session_start": is_sessions[0] if is_sessions else None,
        "is_session_end": is_sessions[-1] if is_sessions else None,
        "holdout_sessions_count": len(holdout_sessions),
        "holdout_session_start": holdout_sessions[0] if holdout_sessions else None,
        "holdout_session_end": holdout_sessions[-1] if holdout_sessions else None,
        "holdout_status": "SEALED_UNTOUCHED",
        "is_bars_scanned": bars_is,
        "evaluation_census_units": evaluation_census,
        "arm_a_opportunity_proxy_count": opportunities,
        "symbols_with_is_gaps": len(symbol_gaps),
        "gap_sample": {k: v for i, (k, v) in enumerate(symbol_gaps.items()) if i < 5},
        "eligibility_floors": {
            "min_sessions": NSE_FLOOR_SESSIONS,
            "min_arm_a_opportunities": NSE_FLOOR_OPPORTUNITIES,
            "sessions_floor_met": floor_sessions_ok,
            "opportunities_floor_met": floor_opp_ok,
        },
        "sample_feasibility": feasibility,
        "note": "Opportunity proxy uses frozen NSE traj15>=0.05 entry rule on bars 1..20; not live fills.",
    }


def census_crypto(path: Path) -> Dict[str, Any]:
    """
    Crypto development partition: first 40 UTC calendar days with bars from
    the 15m BTCUSDT archive (session-equivalent time-blocks = UTC days).
    Arm A opportunity proxy: Policy-B-like momentum — |ret_15m| proxy via
    close-to-close 1-bar return magnitude with directional continuation heuristic
    matching shadow feature readiness (need >= 20 completed bars warm-up for Donchian).
    """
    if not path.exists():
        return {
            "market": "CRYPTO_BTCUSDT",
            "cache_path": str(path),
            "error": "DATA_FILE_MISSING",
            "sample_feasibility": "INCONCLUSIVE",
            "eligibility_floors": {
                "min_time_blocks": CRYPTO_FLOOR_BLOCKS,
                "min_arm_a_opportunities": CRYPTO_FLOOR_OPPORTUNITIES,
                "blocks_floor_met": False,
                "opportunities_floor_met": False,
            },
            "holdout_status": "SEALED_UNTOUCHED",
        }

    candles = json.loads(path.read_text(encoding="utf-8"))
    if isinstance(candles, dict):
        candles = candles.get("candles") or candles.get("data") or []

    by_day: Dict[str, List[dict]] = {}
    for c in candles:
        ts = (
            c.get("open_time_ms")
            or c.get("close_time_ms")
            or c.get("timestamp")
            or c.get("open_time")
            or c.get("t")
            or c.get("close_time")
        )
        if ts is None:
            continue
        ts_ms = int(ts * 1000) if float(ts) < 1e11 else int(ts)
        day = datetime.fromtimestamp(ts_ms / 1000.0, tz=timezone.utc).strftime("%Y-%m-%d")
        by_day.setdefault(day, []).append(c)

    days = sorted(by_day.keys())
    # Prefer 2026 development window if present; else first 40 days of archive
    days_2026 = [d for d in days if d.startswith("2026-")]
    if len(days_2026) >= 40:
        is_days = days_2026[:40]
        holdout_days = days_2026[40:68]
    else:
        is_days = days[:40]
        holdout_days = days[40:68]

    is_set = set(is_days)
    opportunities = 0
    evaluation_census = 0
    bars_is = 0
    closes_rolling: List[float] = []

    # Chronological scan for opportunity proxy on IS days only
    chron = []
    for d in days:
        for c in sorted(
            by_day[d],
            key=lambda x: x.get("open_time_ms")
            or x.get("close_time_ms")
            or x.get("timestamp")
            or x.get("open_time")
            or 0,
        ):
            chron.append((d, c))

    for d, c in chron:
        close = c.get("close") or c.get("c")
        if close is None:
            continue
        close_f = float(close)
        closes_rolling.append(close_f)
        if d not in is_set:
            continue
        bars_is += 1
        evaluation_census += 1
        if len(closes_rolling) < 20:
            continue
        # Lightweight Policy-B style opportunity: positive 15m ret and above ATR noise
        ret = (closes_rolling[-1] - closes_rolling[-2]) / (closes_rolling[-2] + 1e-9)
        window = closes_rolling[-15:]
        rets = [
            (window[i] - window[i - 1]) / (window[i - 1] + 1e-9)
            for i in range(1, len(window))
        ]
        atr_proxy = sum(abs(r) for r in rets) / max(len(rets), 1)
        # Count as Arm A opportunity when directional move exceeds 0.5 * ATR proxy
        if abs(ret) >= max(0.0005, 0.5 * atr_proxy):
            opportunities += 1

    floor_blocks_ok = len(is_days) >= CRYPTO_FLOOR_BLOCKS
    floor_opp_ok = opportunities >= CRYPTO_FLOOR_OPPORTUNITIES
    feasibility = "ACHIEVABLE" if (floor_blocks_ok and floor_opp_ok) else "INCONCLUSIVE"

    return {
        "market": "CRYPTO_BTCUSDT",
        "cache_path": str(path),
        "total_utc_days_in_archive": len(days),
        "is_time_blocks_count": len(is_days),
        "is_block_start": is_days[0] if is_days else None,
        "is_block_end": is_days[-1] if is_days else None,
        "holdout_time_blocks_count": len(holdout_days),
        "holdout_block_start": holdout_days[0] if holdout_days else None,
        "holdout_block_end": holdout_days[-1] if holdout_days else None,
        "holdout_status": "SEALED_UNTOUCHED",
        "is_bars_scanned": bars_is,
        "evaluation_census_units": evaluation_census,
        "arm_a_opportunity_proxy_count": opportunities,
        "eligibility_floors": {
            "min_time_blocks": CRYPTO_FLOOR_BLOCKS,
            "min_arm_a_opportunities": CRYPTO_FLOOR_OPPORTUNITIES,
            "blocks_floor_met": floor_blocks_ok,
            "opportunities_floor_met": floor_opp_ok,
        },
        "sample_feasibility": feasibility,
        "note": (
            "Time-blocks are UTC calendar days. Opportunity proxy is a frozen "
            "magnitude filter for census only; Arm A live Policy B rules remain authoritative."
        ),
    }


def main() -> None:
    out_dir = WORKSPACE_DIR / "artifacts" / "rp608_regime_breakout"
    out_dir.mkdir(parents=True, exist_ok=True)

    nse = census_nse(WORKSPACE_DIR / "intraday_capture" / "yahoo_cache_15m")
    crypto = census_crypto(
        WORKSPACE_DIR / "datasets" / "crypto_15m" / "binance_btcusdt_2024_15m.json"
    )

    report = {
        "protocol": "RP-608-REGIME-BREAKOUT-001",
        "phase": 0,
        "artifact": "IS_CENSUS",
        "timestamp_utc": datetime.now(timezone.utc).isoformat(),
        "live_order_authority": "0.0%",
        "holdout_sessions_41_68": "SEALED_UNTOUCHED",
        "partition_rule": "Sessions/time-blocks indexed 1-40 development; 41+ sealed",
        "threshold_policy": "Do not relax floors; short sample => INCONCLUSIVE",
        "nse": nse,
        "crypto": crypto,
        "overall_is_feasibility": (
            "ACHIEVABLE"
            if nse.get("sample_feasibility") == "ACHIEVABLE"
            and crypto.get("sample_feasibility") == "ACHIEVABLE"
            else "INCONCLUSIVE_FOR_AT_LEAST_ONE_MARKET"
        ),
    }

    out = out_dir / "phase0_is_census.json"
    out.write_text(json.dumps(report, indent=2) + "\n", encoding="utf-8")
    print(json.dumps(report, indent=2))
    print(f"\nWrote {out}")


if __name__ == "__main__":
    main()
