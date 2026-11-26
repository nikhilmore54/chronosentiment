#!/usr/bin/env python3
"""
RP-608 NSE Estimand A — past-trade pattern library miner.

Two uses (books never pooled; no symbol in keys):
  1) ENTRY (Gate 2E): pre-entry candle + Coralys decision-time features →
     ALLOW / STANDASIDE_ADVERSE / ABSTAIN cohorts by historical W/L.
  2) EXIT  (Gate 2X observational): post-entry candle-path signatures →
     where matched paths historically reversed (hypothesis stamps only;
     frozen stops/targets unchanged without Gate 5).

Authority: 0%. Does not change OF ±0.30 / traj15≥0.05 / stops / targets.
"""

from __future__ import annotations

import argparse
import json
import math
import os
import sys
from collections import defaultdict
from datetime import datetime, timezone
from pathlib import Path
from typing import Any, Dict, Iterable, List, Optional, Sequence, Tuple

WORKSPACE = Path(__file__).resolve().parent.parent
sys.path.insert(0, str(WORKSPACE))

from scripts.rp608_nse_coralys_input_context import (  # noqa: E402
    IST,
    is_preopen_jittery,
    is_preopen_turbulent,
    long_trend_aligned,
    session_phase_ist,
)

PROTOCOL_ID = "RP-608-NSE-A-TRADE-PATTERN-V1"
LIBRARY_SCHEMA = "nse_a_trade_pattern_library_v2"

ADVERSE_MIN_N = 30
ADVERSE_MAX_WR = 0.35
EXIT_REVERSE_MIN_N = 20
HOLDOUT_SESSIONS = 2  # last N distinct IST sessions

# Entry candle axes (decision-time only).
TRAJ_BUCKETS = (
    ("LT_0_05", 0.05),
    ("0_05_0_10", 0.10),
    ("0_10_0_15", 0.15),
    ("0_15_0_30", 0.30),
    ("GE_0_30", float("inf")),
)


def _ist_from_ms(ts_ms: int) -> datetime:
    return datetime.fromtimestamp(int(ts_ms) / 1000.0, tz=timezone.utc).astimezone(IST)


def session_date_from_ms(ts_ms: int) -> str:
    return _ist_from_ms(ts_ms).date().isoformat()


def traj15_bucket(traj15: Optional[float]) -> str:
    if traj15 is None or (isinstance(traj15, float) and math.isnan(traj15)):
        return "UNK"
    t = float(traj15)
    if t < 0.05:
        return "LT_0_05"
    for name, upper in TRAJ_BUCKETS:
        if t < upper:
            return name
    return "GE_0_30"


def gap_bucket(gap_bps: Optional[float]) -> str:
    if gap_bps is None:
        return "UNK"
    g = float(gap_bps)
    if g <= -75:
        return "LE_M75"
    if g <= -35:
        return "M75_M35"
    if g < 0:
        return "M35_0"
    if g < 20:
        return "0_20"
    if g < 40:
        return "20_40"
    return "GE_40"


def bar_tertile(entry_bar_idx: Optional[int]) -> str:
    if entry_bar_idx is None:
        return "UNK"
    b = int(entry_bar_idx)
    if b <= 8:
        return "EARLY_BARS"
    if b <= 16:
        return "MID_BARS"
    return "LATE_BARS"


def _candle_dir(bar: Dict[str, float]) -> str:
    o, c = float(bar["open"]), float(bar["close"])
    if c > o * 1.0005:
        return "U"
    if c < o * 0.9995:
        return "D"
    return "F"


def _body_class(bar: Dict[str, float]) -> str:
    o, h, l, c = float(bar["open"]), float(bar["high"]), float(bar["low"]), float(bar["close"])
    rng = max(h - l, 1e-9)
    body = abs(c - o) / rng
    if body < 0.25:
        return "DOJI"
    if body < 0.55:
        return "MED"
    return "STRONG"


def _runup_bucket(bps: Optional[float]) -> str:
    if bps is None:
        return "UNK"
    x = float(bps)
    if x < -20:
        return "NEG"
    if x < 20:
        return "FLAT"
    if x < 60:
        return "UP_MED"
    return "UP_STRONG"


def _vol_surge_bucket(ratio: Optional[float]) -> str:
    if ratio is None:
        return "UNK"
    r = float(ratio)
    if r < 0.8:
        return "LOW"
    if r < 1.5:
        return "MED"
    return "HIGH"


def prior_support_token(v: Any) -> str:
    if v is True:
        return "T"
    if v is False:
        return "F"
    return "N"


def bool_token(v: Any) -> str:
    return "T" if bool(v) else "F"


def _nse_holidays() -> set:
    """Lazy holiday set — avoid importing heavy daemon (websockets) at mine time."""
    try:
        from scripts.run_rp608_nse_shadow_daemon import NSE_2026_HOLIDAYS

        return set(NSE_2026_HOLIDAYS)
    except Exception:
        return {
            datetime(2026, 1, 26).date(),
            datetime(2026, 3, 3).date(),
            datetime(2026, 3, 20).date(),
            datetime(2026, 3, 27).date(),
            datetime(2026, 3, 31).date(),
            datetime(2026, 4, 3).date(),
            datetime(2026, 4, 14).date(),
            datetime(2026, 5, 1).date(),
            datetime(2026, 5, 27).date(),
            datetime(2026, 6, 26).date(),
            datetime(2026, 8, 15).date(),
            datetime(2026, 10, 2).date(),
            datetime(2026, 10, 20).date(),
            datetime(2026, 11, 8).date(),
            datetime(2026, 11, 24).date(),
            datetime(2026, 12, 25).date(),
        }


def load_ohlcv_universe(data_dir: Path) -> Dict[str, Dict[str, Dict[int, Dict[str, float]]]]:
    """symbol → session_date → bar_idx → OHLCV (IST 15m grid)."""
    holidays = _nse_holidays()
    symbol_bars: Dict[str, Dict[str, Dict[int, Dict[str, float]]]] = {}
    for p in sorted(data_dir.glob("*.json")):
        sym = p.stem
        try:
            raw = json.loads(p.read_text(encoding="utf-8"))
        except (OSError, json.JSONDecodeError):
            continue
        if isinstance(raw, dict):
            raw = raw.get("usable") or raw.get("bars") or []
        if not isinstance(raw, list):
            continue
        by_day: Dict[str, Dict[int, Dict[str, float]]] = {}
        seen: set = set()
        for c in raw:
            ts = c.get("timestamp")
            if ts is None:
                continue
            ts_ms = int(ts * 1000) if float(ts) < 1e11 else int(ts)
            if ts_ms in seen:
                continue
            seen.add(ts_ms)
            o = float(c.get("open", 0) or 0)
            h = float(c.get("high", 0) or 0)
            l = float(c.get("low", 0) or 0)
            cl = float(c.get("close", 0) or 0)
            v = float(c.get("volume", 0) or 0)
            if min(o, h, l, cl) <= 0:
                continue
            dt_ist = datetime.fromtimestamp(ts_ms / 1000.0, tz=timezone.utc).astimezone(IST)
            if dt_ist.weekday() >= 5 or dt_ist.date() in holidays:
                continue
            open_ist = dt_ist.replace(hour=9, minute=15, second=0, microsecond=0)
            close_ist = dt_ist.replace(hour=15, minute=30, second=0, microsecond=0)
            if dt_ist < open_ist or dt_ist >= close_ist:
                continue
            el_min = (dt_ist - open_ist).total_seconds() / 60.0
            bar_idx = int(el_min // 15) + 1
            if not (1 <= bar_idx <= 25):
                continue
            day = dt_ist.date().isoformat()
            by_day.setdefault(day, {})[bar_idx] = {
                "ts_ms": float(ts_ms),
                "open": o,
                "high": h,
                "low": l,
                "close": cl,
                "volume": v,
            }
        if by_day:
            symbol_bars[sym] = by_day
    return symbol_bars


def extract_pre_entry_candle_feats(
    bars_dict: Dict[int, Dict[str, float]],
    entry_bar_idx: int,
) -> Dict[str, Any]:
    """Decision-time candle geometry ending at signal bar (entry_bar - 1)."""
    sig_idx = int(entry_bar_idx) - 1
    ordered = [bars_dict[i] for i in sorted(bars_dict) if i <= sig_idx]
    if not ordered:
        return {
            "candle_path3": "UNK",
            "signal_body": "UNK",
            "runup_3b": "UNK",
            "vol_surge": "UNK",
            "traj15_recon": None,
        }
    # traj15 recon (same spirit as analyze_rp608_trade_patterns)
    closes = [b["close"] for b in ordered]
    traj15 = 0.0
    if len(closes) >= 2:
        lookback = min(len(closes), 15)
        ret = (closes[-1] - closes[-lookback]) / (closes[-lookback] + 1e-9)
        diffs = [
            (closes[i] - closes[i - 1]) / (closes[i - 1] + 1e-9)
            for i in range(len(closes) - lookback + 1, len(closes))
        ]
        vol = (sum(d * d for d in diffs) / len(diffs)) ** 0.5 if diffs else 1e-3
        traj15 = math.tanh(ret / ((vol * (lookback**0.5) * 2.5) + 1e-9))

    path_dirs = "".join(_candle_dir(b) for b in ordered[-3:])
    while len(path_dirs) < 3:
        path_dirs = "X" + path_dirs
    sig = ordered[-1]
    body = _body_class(sig) + "_" + _candle_dir(sig)
    ret_3b = None
    if len(closes) >= 4:
        ret_3b = (closes[-1] - closes[-4]) / (closes[-4] + 1e-9) * 10000.0
    vols = [b["volume"] for b in ordered]
    vol_ratio = None
    if len(vols) >= 6:
        prior = vols[-6:-1]
        avg = sum(prior) / len(prior)
        vol_ratio = (vols[-1] / avg) if avg > 0 else 1.0
    return {
        "candle_path3": path_dirs,
        "signal_body": body,
        "runup_3b": _runup_bucket(ret_3b),
        "vol_surge": _vol_surge_bucket(vol_ratio),
        "traj15_recon": round(traj15, 6),
    }


def _vol_surge_at_bar(
    bars_dict: Dict[int, Dict[str, float]], bar_idx: int
) -> str:
    """Candle volume at bar_idx vs mean of prior 5 completed bars (same buckets as entry)."""
    bi = int(bar_idx)
    bar = bars_dict.get(bi)
    if not bar:
        return "UNK"
    prior = sorted(i for i in bars_dict.keys() if i < bi)[-5:]
    if len(prior) < 5:
        return "UNK"
    avg = sum(float(bars_dict[i].get("volume") or 0) for i in prior) / 5.0
    if avg <= 0:
        return "UNK"
    return _vol_surge_bucket(float(bar.get("volume") or 0) / avg)


def extract_exit_path_feats(
    bars_dict: Dict[int, Dict[str, float]],
    entry_bar_idx: int,
    exit_bar_idx: Optional[int],
    entry_price: float,
) -> Dict[str, Any]:
    """
    Post-entry path (Gate 2X diagnostic). Not used as entry cause.
    Captures early path + first reverse candle after a green excursion +
    volume surge on the reverse (or latest) path bar.
    """
    e = int(entry_bar_idx)
    x = int(exit_bar_idx) if exit_bar_idx is not None else e
    path_dirs: List[str] = []
    first_reverse_bar: Optional[int] = None
    peak_mfe_bps = 0.0
    saw_green = False
    last_path_bar: Optional[int] = None
    for bi in range(e, min(x, e + 8) + 1):
        bar = bars_dict.get(bi)
        if not bar:
            continue
        last_path_bar = bi
        d = _candle_dir(bar)
        path_dirs.append(d)
        mfe = max(0.0, (float(bar["high"]) - entry_price) / entry_price * 10000.0)
        peak_mfe_bps = max(peak_mfe_bps, mfe)
        close_ret = (float(bar["close"]) - entry_price) / entry_price * 10000.0
        if close_ret > 5.0:
            saw_green = True
        # Reverse: first strong down candle after having been green vs entry
        if saw_green and first_reverse_bar is None and d == "D" and _body_class(bar) in {"MED", "STRONG"}:
            first_reverse_bar = bi - e  # bars after entry
    path3 = "".join(path_dirs[:3]) if path_dirs else "UNK"
    while len(path3) < 3 and path3 != "UNK":
        path3 += "X"
    mfe_bucket = "NONE"
    if peak_mfe_bps >= 80:
        mfe_bucket = "MFE_GE80"
    elif peak_mfe_bps >= 40:
        mfe_bucket = "MFE_40_80"
    elif peak_mfe_bps >= 15:
        mfe_bucket = "MFE_15_40"
    else:
        mfe_bucket = "MFE_LT15"
    rev = "NONE"
    if first_reverse_bar is None:
        rev = "NONE"
    elif first_reverse_bar <= 1:
        rev = "REV_B1"
    elif first_reverse_bar <= 2:
        rev = "REV_B2"
    elif first_reverse_bar <= 4:
        rev = "REV_B3_4"
    else:
        rev = "REV_LATE"
    vol_bar = (
        (e + first_reverse_bar)
        if first_reverse_bar is not None
        else (last_path_bar if last_path_bar is not None else e)
    )
    exit_vol_surge = _vol_surge_at_bar(bars_dict, vol_bar)
    return {
        "exit_path3": path3 if path3 else "UNK",
        "mfe_bucket": mfe_bucket,
        "first_reverse": rev,
        "exit_vol_surge": exit_vol_surge,
        "peak_mfe_bps": round(peak_mfe_bps, 2),
        "first_reverse_bar_offset": first_reverse_bar,
    }


def build_entry_pattern_key(
    feats: Dict[str, Any], *, include_volume: bool = True
) -> str:
    """Canonical entry key — never includes symbol."""
    parts = [
        str(feats.get("session_phase") or "UNK"),
        str(feats.get("traj15_bucket") or "UNK"),
        str(feats.get("preopen_status") or "UNK"),
        str(feats.get("gap_bucket") or "UNK"),
        "J" + bool_token(feats.get("jittery")),
        "U" + bool_token(feats.get("turbulent")),
        "A" + prior_support_token(feats.get("trend_aligned_long")),
        "P" + prior_support_token(feats.get("prior_support")),
        "S" + bool_token(feats.get("same_session_prior_stop_in_similar_context")),
        str(feats.get("bar_tertile") or "UNK"),
        "C" + str(feats.get("candle_path3") or "UNK"),
        "B" + str(feats.get("signal_body") or "UNK"),
        "R" + str(feats.get("runup_3b") or "UNK"),
    ]
    if include_volume:
        parts.append("V" + str(feats.get("vol_surge") or "UNK"))
    key = "|".join(parts)
    assert "symbol" not in key.lower()
    return key


def build_exit_pattern_key(
    feats: Dict[str, Any], *, include_volume: bool = True
) -> str:
    """Exit/reverse cohort key — path geometry (+ optional path volume); no symbol."""
    parts = [
        str(feats.get("session_phase") or "UNK"),
        str(feats.get("traj15_bucket") or "UNK"),
        str(feats.get("exit_path3") or "UNK"),
        str(feats.get("mfe_bucket") or "UNK"),
        str(feats.get("first_reverse") or "UNK"),
    ]
    if include_volume:
        parts.append("V" + str(feats.get("exit_vol_surge") or "UNK"))
    return "|".join(parts)


ENTRY_BACKOFF_DROP_ORDER = (
    "vol_surge",
    "runup_3b",
    "signal_body",
    "prior_support",
    "same_session_prior_stop_in_similar_context",
    "bar_tertile",
    "trend_aligned_long",
    "jittery",
    "turbulent",
)


def entry_key_with_wildcards(feats: Dict[str, Any], dropped: Sequence[str]) -> str:
    f = dict(feats)
    for ax in dropped:
        if ax in {"vol_surge", "runup_3b", "signal_body", "candle_path3"}:
            f[ax] = "*"
        elif ax in {"prior_support", "trend_aligned_long"}:
            f[ax] = None  # → N via prior_support_token / handled below
        elif ax == "same_session_prior_stop_in_similar_context":
            f[ax] = False
            # mark wildcard via special: use "*" token by overriding after
        elif ax in {"jittery", "turbulent"}:
            f[ax] = False
    # Rebuild with wildcards for dropped axes
    parts = [
        str(f.get("session_phase") or "UNK"),
        str(f.get("traj15_bucket") or "UNK"),
        str(f.get("preopen_status") or "UNK"),
        str(f.get("gap_bucket") or "UNK"),
        "J" + ("*" if "jittery" in dropped else bool_token(f.get("jittery"))),
        "U" + ("*" if "turbulent" in dropped else bool_token(f.get("turbulent"))),
        "A"
        + (
            "*"
            if "trend_aligned_long" in dropped
            else prior_support_token(f.get("trend_aligned_long"))
        ),
        "P"
        + (
            "*"
            if "prior_support" in dropped
            else prior_support_token(f.get("prior_support"))
        ),
        "S"
        + (
            "*"
            if "same_session_prior_stop_in_similar_context" in dropped
            else bool_token(f.get("same_session_prior_stop_in_similar_context"))
        ),
        ("*" if "bar_tertile" in dropped else str(f.get("bar_tertile") or "UNK")),
        "C" + ("*" if "candle_path3" in dropped else str(f.get("candle_path3") or "UNK")),
        "B" + ("*" if "signal_body" in dropped else str(f.get("signal_body") or "UNK")),
        "R" + ("*" if "runup_3b" in dropped else str(f.get("runup_3b") or "UNK")),
        "V" + ("*" if "vol_surge" in dropped else str(f.get("vol_surge") or "UNK")),
    ]
    return "|".join(parts)


def load_jsonl(path: Path) -> List[Dict[str, Any]]:
    rows: List[Dict[str, Any]] = []
    if not path.exists():
        return rows
    with path.open("r", encoding="utf-8") as f:
        for line in f:
            line = line.strip()
            if not line:
                continue
            try:
                rows.append(json.loads(line))
            except json.JSONDecodeError:
                continue
    return rows


def index_evaluations(
    eval_paths: Iterable[Path],
) -> Dict[Tuple[str, int, int], Dict[str, Any]]:
    """(symbol, session_idx, bar_idx) → evaluation with coralys/score when present."""
    out: Dict[Tuple[str, int, int], Dict[str, Any]] = {}
    for path in eval_paths:
        if not path or not Path(path).exists():
            continue
        for r in load_jsonl(Path(path)):
            try:
                key = (
                    str(r.get("symbol") or ""),
                    int(r.get("session_idx") or -1),
                    int(r.get("bar_idx") or -1),
                )
            except (TypeError, ValueError):
                continue
            if not key[0]:
                continue
            out[key] = r
    return out


def join_coralys(
    trade: Dict[str, Any],
    eval_idx: Dict[Tuple[str, int, int], Dict[str, Any]],
) -> Dict[str, Any]:
    ctx = trade.get("coralys_input_context")
    if isinstance(ctx, dict) and ctx:
        return ctx
    sym = str(trade.get("symbol") or "")
    try:
        sess = int(trade.get("session_idx") or -1)
        bar = int(trade.get("entry_bar_idx") or -1)
    except (TypeError, ValueError):
        return {}
    ev = eval_idx.get((sym, sess, bar)) or eval_idx.get((sym, sess, bar - 1)) or {}
    ctx2 = ev.get("coralys_input_context")
    if isinstance(ctx2, dict):
        return ctx2
    # Minimal reconstruction from eval score + phase from entry time
    out: Dict[str, Any] = {}
    if ev.get("score") is not None:
        out["traj15"] = float(ev["score"])
    if trade.get("entry_timestamp_utc_ms"):
        out["session_phase"] = session_phase_ist(_ist_from_ms(int(trade["entry_timestamp_utc_ms"])))
    gap = ev.get("overnight_gap_bps")
    if gap is not None:
        out["opening_gap_bps"] = float(gap)
        out["preopen_status"] = "OK"
    return out


def enrich_trade_features(
    trade: Dict[str, Any],
    *,
    coralys: Dict[str, Any],
    candle: Dict[str, Any],
    exit_path: Dict[str, Any],
    same_session_prior_stop: bool,
) -> Dict[str, Any]:
    traj = coralys.get("traj15")
    if traj is None:
        traj = trade.get("traj15_at_entry")
    if traj is None:
        traj = candle.get("traj15_recon")
    gap = coralys.get("opening_gap_bps")
    imb = coralys.get("imbalance_ratio")
    phase = coralys.get("session_phase")
    if not phase and trade.get("entry_timestamp_utc_ms"):
        phase = session_phase_ist(_ist_from_ms(int(trade["entry_timestamp_utc_ms"])))
    preopen_status = coralys.get("preopen_status") or ("OK" if gap is not None else "MISSING")
    jittery = coralys.get("preopen_jittery")
    if jittery is None and gap is not None:
        jittery, _ = is_preopen_jittery(gap, imb)
    turbulent = coralys.get("preopen_turbulent")
    if turbulent is None and gap is not None:
        turbulent, _ = is_preopen_turbulent(gap, imb)
    aligned = coralys.get("trend_aligned_long")
    if aligned is None and gap is not None and traj is not None:
        aligned = long_trend_aligned(float(gap), float(traj))
    win = int(trade.get("win_label") or 0)
    if "win_label" not in trade:
        win = 1 if float(trade.get("net_pnl_inr") or 0) > 0 else 0
    feats = {
        "session_phase": phase or "UNK",
        "traj15_bucket": traj15_bucket(float(traj) if traj is not None else None),
        "preopen_status": preopen_status,
        "gap_bucket": gap_bucket(float(gap) if gap is not None else None),
        "jittery": bool(jittery) if jittery is not None else False,
        "turbulent": bool(turbulent) if turbulent is not None else False,
        "trend_aligned_long": aligned,
        "prior_support": coralys.get("prior_support"),
        "same_session_prior_stop_in_similar_context": bool(same_session_prior_stop),
        "bar_tertile": bar_tertile(trade.get("entry_bar_idx")),
        "candle_path3": candle.get("candle_path3") or "UNK",
        "signal_body": candle.get("signal_body") or "UNK",
        "runup_3b": candle.get("runup_3b") or "UNK",
        "vol_surge": candle.get("vol_surge") or "UNK",
        "exit_path3": exit_path.get("exit_path3") or "UNK",
        "mfe_bucket": exit_path.get("mfe_bucket") or "UNK",
        "first_reverse": exit_path.get("first_reverse") or "UNK",
        "exit_vol_surge": exit_path.get("exit_vol_surge") or "UNK",
        "peak_mfe_bps": exit_path.get("peak_mfe_bps"),
        "first_reverse_bar_offset": exit_path.get("first_reverse_bar_offset"),
        "win": win,
        "net_pnl_inr": float(trade.get("net_pnl_inr") or 0.0),
        "gross_pnl_inr": float(trade.get("gross_pnl_inr") or 0.0),
        "session_date": session_date_from_ms(int(trade["entry_timestamp_utc_ms"]))
        if trade.get("entry_timestamp_utc_ms")
        else None,
        # kept off key:
        "_symbol": str(trade.get("symbol") or ""),
        "_trade_id": str(trade.get("trade_id") or ""),
    }
    feats["entry_pattern_id"] = build_entry_pattern_key(feats)
    feats["exit_pattern_id"] = build_exit_pattern_key(feats)
    return feats


def _agg_metrics(rows: List[Dict[str, Any]]) -> Dict[str, Any]:
    n = len(rows)
    if n == 0:
        return {
            "n": 0,
            "wins": 0,
            "losses": 0,
            "wr": None,
            "avg_win": None,
            "avg_loss": None,
            "payoff": None,
            "net_sum": 0.0,
            "net_mean": None,
            "gross_mean": None,
        }
    wins = [r for r in rows if int(r.get("win") or 0) == 1]
    losses = [r for r in rows if int(r.get("win") or 0) != 1]
    avg_win = (
        sum(float(r["net_pnl_inr"]) for r in wins) / len(wins) if wins else None
    )
    avg_loss = (
        sum(abs(float(r["net_pnl_inr"])) for r in losses) / len(losses) if losses else None
    )
    payoff = None
    if avg_win is not None and avg_loss is not None and avg_loss > 0:
        payoff = avg_win / avg_loss
    net_sum = sum(float(r["net_pnl_inr"]) for r in rows)
    gross_sum = sum(float(r.get("gross_pnl_inr") or 0) for r in rows)
    return {
        "n": n,
        "wins": len(wins),
        "losses": len(losses),
        "wr": round(len(wins) / n, 4),
        "avg_win": round(avg_win, 2) if avg_win is not None else None,
        "avg_loss": round(avg_loss, 2) if avg_loss is not None else None,
        "payoff": round(payoff, 3) if payoff is not None else None,
        "net_sum": round(net_sum, 2),
        "net_mean": round(net_sum / n, 2),
        "gross_mean": round(gross_sum / n, 2),
    }


def classify_entry_label(m: Dict[str, Any]) -> str:
    n = int(m.get("n") or 0)
    wr = m.get("wr")
    if n < ADVERSE_MIN_N or wr is None:
        return "INSUFFICIENT"
    if wr <= ADVERSE_MAX_WR:
        return "ADVERSE"
    return "NEUTRAL"


def classify_exit_label(m: Dict[str, Any], rows: List[Dict[str, Any]]) -> str:
    """
    Exit cohort label from path reverse geometry.
    REVERSE_OFTEN: after green MFE, reverse candle historically precedes losses /
    giveback (W/L disclosed; hypothesis only).
    """
    n = int(m.get("n") or 0)
    if n < EXIT_REVERSE_MIN_N:
        return "INSUFFICIENT"
    # Among paths that saw a reverse stamp, loss rate
    wr = m.get("wr")
    rev_rows = [r for r in rows if str(r.get("first_reverse") or "NONE") != "NONE"]
    if len(rev_rows) < EXIT_REVERSE_MIN_N:
        return "INSUFFICIENT"
    # If cohort WR low AND reverse present → suggest early exit observation
    if wr is not None and wr <= 0.40:
        return "REVERSE_STANDASIDE_HOLD"
    if wr is not None and wr >= 0.55:
        return "HOLD_FAVORABLE"
    return "NEUTRAL"


def mine_library(
    trades: List[Dict[str, Any]],
    *,
    symbol_bars: Optional[Dict[str, Dict[str, Dict[int, Dict[str, float]]]]] = None,
    eval_idx: Optional[Dict[Tuple[str, int, int], Dict[str, Any]]] = None,
) -> Dict[str, Any]:
    symbol_bars = symbol_bars or {}
    eval_idx = eval_idx or {}

    # Chronological order
    closed = [
        t
        for t in trades
        if str(t.get("estimand") or "ESTIMAND_A") in ("ESTIMAND_A", "A", "")
        and t.get("entry_timestamp_utc_ms") is not None
    ]
    closed.sort(key=lambda t: int(t["entry_timestamp_utc_ms"]))

    sessions = []
    seen_s = set()
    for t in closed:
        d = session_date_from_ms(int(t["entry_timestamp_utc_ms"]))
        if d not in seen_s:
            seen_s.add(d)
            sessions.append(d)
    holdout_set = set(sessions[-HOLDOUT_SESSIONS:]) if len(sessions) > HOLDOUT_SESSIONS else set()
    train_sessions = [s for s in sessions if s not in holdout_set]

    enriched: List[Dict[str, Any]] = []
    # Session-level stop families (phase|traj) — not symbol bans
    session_stop_families: Dict[str, set] = defaultdict(set)

    for t in closed:
        sess = session_date_from_ms(int(t["entry_timestamp_utc_ms"]))
        coralys = join_coralys(t, eval_idx)
        sym = str(t.get("symbol") or "")
        day_bars = symbol_bars.get(sym, {}).get(sess, {})
        e_bar = int(t.get("entry_bar_idx") or 0)
        candle = (
            extract_pre_entry_candle_feats(day_bars, e_bar)
            if day_bars and e_bar > 0
            else {
                "candle_path3": "UNK",
                "signal_body": "UNK",
                "runup_3b": "UNK",
                "vol_surge": "UNK",
                "traj15_recon": None,
            }
        )
        if day_bars and e_bar > 0 and float(t.get("entry_price") or 0) > 0:
            exit_path = extract_exit_path_feats(
                day_bars,
                e_bar,
                t.get("exit_bar_idx"),
                float(t.get("entry_price") or 0) or 1.0,
            )
        else:
            # Journal fallback when 15m OHLCV does not cover the session
            # (candle path remains UNK; MFE + exit_reason still usable for Gate 2X).
            mfe = t.get("mfe_bps")
            mfe_f = float(mfe) if mfe is not None else None
            if mfe_f is None:
                mfe_bucket = "UNK"
            elif mfe_f >= 80:
                mfe_bucket = "MFE_GE80"
            elif mfe_f >= 40:
                mfe_bucket = "MFE_40_80"
            elif mfe_f >= 15:
                mfe_bucket = "MFE_15_40"
            else:
                mfe_bucket = "MFE_LT15"
            reason = str(t.get("exit_reason") or "")
            if mfe_f is not None and mfe_f >= 15 and reason == "STOP_LOSS":
                rev = "REV_AFTER_GREEN"
            elif reason in {"TAKE_PROFIT", "TARGET_HORIZON_60M"}:
                rev = "NONE"
            elif reason == "STOP_LOSS":
                rev = "REV_NO_MFE"
            else:
                rev = "UNK"
            exit_path = {
                "exit_path3": "UNK",
                "mfe_bucket": mfe_bucket,
                "first_reverse": rev,
                "exit_vol_surge": "UNK",
                "peak_mfe_bps": round(mfe_f, 2) if mfe_f is not None else None,
                "first_reverse_bar_offset": None,
            }
        phase = coralys.get("session_phase") or (
            session_phase_ist(_ist_from_ms(int(t["entry_timestamp_utc_ms"])))
        )
        traj_b = traj15_bucket(
            float(coralys["traj15"])
            if coralys.get("traj15") is not None
            else (candle.get("traj15_recon") if candle.get("traj15_recon") is not None else None)
        )
        family = f"{phase}|{traj_b}"
        prior_stop = family in session_stop_families[sess]
        feats = enrich_trade_features(
            t,
            coralys=coralys,
            candle=candle,
            exit_path=exit_path,
            same_session_prior_stop=prior_stop,
        )
        feats["_split"] = "holdout" if sess in holdout_set else "train"
        enriched.append(feats)
        if str(t.get("exit_reason") or "") == "STOP_LOSS":
            session_stop_families[sess].add(family)

    train_rows = [r for r in enriched if r["_split"] == "train"]
    holdout_rows = [r for r in enriched if r["_split"] == "holdout"]

    def _group(rows: List[Dict[str, Any]], key_field: str) -> Dict[str, List[Dict[str, Any]]]:
        g: Dict[str, List[Dict[str, Any]]] = defaultdict(list)
        for r in rows:
            g[str(r[key_field])].append(r)
        return g

    entry_train = _group(train_rows, "entry_pattern_id")
    exit_train = _group(train_rows, "exit_pattern_id")
    entry_hold = _group(holdout_rows, "entry_pattern_id")
    exit_hold = _group(holdout_rows, "exit_pattern_id")

    entry_patterns = {}
    for pid, rows in entry_train.items():
        m = _agg_metrics(rows)
        hrows = entry_hold.get(pid, [])
        hm = _agg_metrics(hrows)
        label = classify_entry_label(m)
        entry_patterns[pid] = {
            "pattern_id": pid,
            "kind": "ENTRY",
            "label": label,
            "train": m,
            "holdout": hm,
            "coverage_train_share": round(len(rows) / max(len(train_rows), 1), 4),
        }

    exit_patterns = {}
    for pid, rows in exit_train.items():
        m = _agg_metrics(rows)
        hrows = exit_hold.get(pid, [])
        hm = _agg_metrics(hrows)
        label = classify_exit_label(m, rows)
        exit_patterns[pid] = {
            "pattern_id": pid,
            "kind": "EXIT",
            "label": label,
            "train": m,
            "holdout": hm,
            "coverage_train_share": round(len(rows) / max(len(train_rows), 1), 4),
        }

    adverse_entry = [p for p in entry_patterns.values() if p["label"] == "ADVERSE"]
    reverse_exit = [
        p for p in exit_patterns.values() if p["label"] == "REVERSE_STANDASIDE_HOLD"
    ]

    return {
        "protocol_id": PROTOCOL_ID,
        "schema": LIBRARY_SCHEMA,
        "book": "NSE_ESTIMAND_A",
        "authority_pct": 0.0,
        "generated_at_utc": datetime.now(timezone.utc).isoformat(),
        "rules": {
            "adverse_min_n": ADVERSE_MIN_N,
            "adverse_max_wr": ADVERSE_MAX_WR,
            "exit_reverse_min_n": EXIT_REVERSE_MIN_N,
            "holdout_sessions": HOLDOUT_SESSIONS,
            "symbol_in_key": False,
            "entry_uses": "Gate2E decision-time standaside vs allow",
            "exit_uses": "Gate2X observational reverse timing (no Gate5 apply)",
        },
        "sessions": {
            "all": sessions,
            "train": train_sessions,
            "holdout": sorted(holdout_set),
        },
        "counts": {
            "trades_total": len(enriched),
            "trades_train": len(train_rows),
            "trades_holdout": len(holdout_rows),
            "entry_patterns": len(entry_patterns),
            "exit_patterns": len(exit_patterns),
            "adverse_entry_patterns": len(adverse_entry),
            "reverse_exit_patterns": len(reverse_exit),
        },
        "entry_patterns": entry_patterns,
        "exit_patterns": exit_patterns,
        "summary_wl": {
            "train": _agg_metrics(train_rows),
            "holdout": _agg_metrics(holdout_rows),
        },
    }


def default_library_path(nse_dir: Optional[Path] = None) -> Path:
    if nse_dir is None:
        nse_dir = WORKSPACE / "live_capture" / "shadow_validation" / "nse"
    return Path(nse_dir) / "rp608_nse_a_trade_pattern_library.json"


def main() -> int:
    ap = argparse.ArgumentParser(description="Mine NSE A entry+exit trade pattern library")
    ap.add_argument(
        "--journal",
        default=str(
            WORKSPACE / "live_capture" / "shadow_validation" / "nse" / "rp608_nse_trade_journal.jsonl"
        ),
    )
    ap.add_argument(
        "--ohlcv-dir",
        default=str(WORKSPACE / "intraday_capture" / "yahoo_cache_15m"),
    )
    ap.add_argument(
        "--evaluations",
        action="append",
        default=[],
        help="Optional evaluations.jsonl paths for Coralys/traj join",
    )
    ap.add_argument(
        "--output",
        default="",
        help="Library JSON path (default under journal's nse dir)",
    )
    args = ap.parse_args()

    journal = Path(args.journal)
    trades = load_jsonl(journal)
    eval_paths = [Path(p) for p in (args.evaluations or [])]
    if not eval_paths:
        # auto-discover beside journal
        parent = journal.parent
        for name in (
            "rp608_nse_evaluations.jsonl",
            "test_run/rp608_nse_evaluations.jsonl",
        ):
            cand = parent / name
            if cand.exists():
                eval_paths.append(cand)
    eval_idx = index_evaluations(eval_paths)
    ohlcv_dir = Path(args.ohlcv_dir)
    symbol_bars = load_ohlcv_universe(ohlcv_dir) if ohlcv_dir.exists() else {}
    print(
        f"[pattern_library] trades={len(trades)} evals_indexed={len(eval_idx)} "
        f"ohlcv_symbols={len(symbol_bars)}"
    )
    lib = mine_library(trades, symbol_bars=symbol_bars, eval_idx=eval_idx)
    out = Path(args.output) if args.output else default_library_path(journal.parent)
    out.parent.mkdir(parents=True, exist_ok=True)
    out.write_text(json.dumps(lib, indent=2, sort_keys=False) + "\n", encoding="utf-8")
    print(
        f"[pattern_library] wrote {out} "
        f"entry={lib['counts']['entry_patterns']} "
        f"adverse={lib['counts']['adverse_entry_patterns']} "
        f"exit={lib['counts']['exit_patterns']} "
        f"reverse={lib['counts']['reverse_exit_patterns']}"
    )
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
