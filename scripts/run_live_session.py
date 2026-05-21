#!/usr/bin/env python3
"""
Live synchronized observatory session — warm daemon + incremental barriers.

Phase 1: Warm Rust buffers from frozen substrate (no archive writes).
Phase 2: Live incremental commits (yfinance latest 5m bar) with delta persistence.

Usage:
  python3 scripts/run_live_session.py --batch-id 3
  python3 scripts/run_live_session.py --batch-id 3 --cycles 5 --live-only
  python3 scripts/run_live_session.py --batch-id 3 --warmup-only
"""

from __future__ import annotations

import argparse
import json
import math
import sys
import time
import urllib.request
from datetime import datetime, timedelta, timezone
from pathlib import Path

import pandas as pd

sys.path.insert(0, str(Path(__file__).resolve().parent))
from candle_substrate import load_frozen_cohort
from observatory_daemon import ObservatoryDaemon
from concurrent.futures import ThreadPoolExecutor

from run_nse_cohort import NSEIngestionEngine, resolve_archive_dir

# Delisted / poison symbols — skip fetch, do not fail the barrier
FORCE_SKIP_SYMBOLS = frozenset({"NEAGI.NS"})
DEFAULT_BAR_SEC = 300
DEFAULT_PROVIDER_LAG_SEC = 15

# ── Phase B: Contextual Retry Profiles ─────────────────────────────────────
# Empirically derived from APAC/NSE open observations.
# continuous_market: crypto 24/7, intraday, US overnight ETFs
# exchange_open    : NSE/TSE/ASX equilibrium zone (9:00–9:20 IST, etc.)
# auction_window   : explicit call auction / pre-open order collection
RETRY_PROFILES: dict[str, dict] = {
    "continuous_market": {"attempts": 3, "spacing_sec": 15, "stale_window_sec": 20},
    "exchange_open":     {"attempts": 6, "spacing_sec": 20, "stale_window_sec": 30},
    "auction_window":    {"attempts": 8, "spacing_sec": 30, "stale_window_sec": 45},
}
DEFAULT_RETRY_PROFILE = "continuous_market"

# ── Phase B: Secondary Provider Triangulation ─────────────────────────────────

# Crypto symbol mapping: yfinance style → Binance REST style
_BINANCE_SYMBOL_MAP: dict[str, str] = {
    "BTC-USD": "BTCUSDT", "ETH-USD": "ETHUSDT", "SOL-USD": "SOLUSDT",
    "BNB-USD": "BNBUSDT", "XRP-USD": "XRPUSDT", "ADA-USD": "ADAUSDT",
    "DOGE-USD": "DOGEUSDT", "AVAX-USD": "AVAXUSDT", "DOT-USD": "DOTUSDT",
    "LINK-USD": "LINKUSDT", "LTC-USD": "LTCUSDT", "BCH-USD": "BCHUSDT",
    "ATOM-USD": "ATOMUSDT", "UNI-USD": "UNIUSDT", "NEAR-USD": "NEARUSDT",
    "APT-USD": "APTUSDT",  "ARB-USD": "ARBUSDT",  "OP-USD":  "OPUSDT",
    "FIL-USD": "FILUSDT", "INJ-USD": "INJUSDT",
}
_BINANCE_KLINE_URL = "https://api.binance.com/api/v3/klines"
_BINANCE_TIMEOUT_SEC = 5

_NSE_COOKIES: str | None = None
_NSE_TIMEOUT_SEC = 5


def _to_nse_symbol(symbol: str) -> str | None:
    if symbol.endswith(".NS"):
        return symbol[:-3]
    if symbol.endswith(".BO"):
        return symbol[:-3]
    return None


def _binance_latest_ts(symbol: str, bar_sec: int) -> int | None:
    """
    Fetch the open_time of the most recent *closed* kline from Binance.
    Returns the bar's open-timestamp (UTC seconds), or None on failure.
    No API key required; rate limit 1200 req/min.
    """
    binance_sym = _BINANCE_SYMBOL_MAP.get(symbol)
    if not binance_sym:
        return None  # non-crypto symbol — no Binance mapping

    interval_map = {60: "1m", 300: "5m", 900: "15m", 3600: "1h"}
    interval = interval_map.get(bar_sec)
    if not interval:
        return None

    url = f"{_BINANCE_KLINE_URL}?symbol={binance_sym}&interval={interval}&limit=2"
    try:
        with urllib.request.urlopen(url, timeout=_BINANCE_TIMEOUT_SEC) as resp:
            data = json.loads(resp.read())
        if not data or len(data) < 1:
            return None
        # kline row: [open_time_ms, open, high, low, close, volume, close_time_ms, ...]
        # Use second-to-last row to guarantee the bar is fully closed
        row = data[-2] if len(data) >= 2 else data[-1]
        return int(row[0]) // 1000  # ms → seconds
    except Exception:
        return None


def _nse_india_latest_ts(symbol: str, bar_sec: int) -> int | None:
    """
    Fetch the live lastUpdateTime from NSE India's official API.
    Returns the UTC timestamp floored to the nearest `bar_sec` boundary.
    """
    global _NSE_COOKIES
    nse_sym = _to_nse_symbol(symbol)
    if not nse_sym:
        return None

    headers = {
        "User-Agent": "Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/537.36",
        "Accept": "application/json",
        "Accept-Language": "en-US,en;q=0.9",
        "Referer": "https://www.nseindia.com/",
    }

    try:
        # Fetch cookies once per process if missing
        if not _NSE_COOKIES:
            req = urllib.request.Request("https://www.nseindia.com/", headers=headers)
            with urllib.request.urlopen(req, timeout=_NSE_TIMEOUT_SEC) as resp:
                _NSE_COOKIES = resp.headers.get("Set-Cookie", "")
        
        headers["Cookie"] = _NSE_COOKIES
        url = f"https://www.nseindia.com/api/quote-equity?symbol={urllib.parse.quote(nse_sym)}"
        req = urllib.request.Request(url, headers=headers)
        with urllib.request.urlopen(req, timeout=_NSE_TIMEOUT_SEC) as resp:
            data = json.loads(resp.read())
        
        update_time_str = data.get("metadata", {}).get("lastUpdateTime")
        if not update_time_str:
            return None
            
        # Example format: "21-May-2026 10:11:09"
        IST = timezone(timedelta(hours=5, minutes=30))
        dt = datetime.strptime(update_time_str, "%d-%b-%Y %H:%M:%S").replace(tzinfo=IST)
        
        # Floor to nearest bar boundary
        epoch = int(dt.timestamp())
        return (epoch // bar_sec) * bar_sec

    except Exception:
        return None


def probe_secondary_ts(
    symbols: list[str],
    last_committed_ts: int,
    bar_sec: int = DEFAULT_BAR_SEC,
) -> dict:
    """
    Phase C — Triangulates timestamp via multiple secondary providers (Binance + NSE India).
    
    Polls a subset of symbols per provider to verify if the market has genuinely advanced.
    Returns a dict with fields logged to the TRL step.
    """
    import urllib.parse
    t_probe = time.time()
    probed: list[dict] = []
    advanced_globally = False

    # Cap to max 5 symbols per provider to stay fast and avoid rate limits
    crypto_symbols = [s for s in symbols if s in _BINANCE_SYMBOL_MAP and s not in FORCE_SKIP_SYMBOLS][:5]
    nse_symbols = [s for s in symbols if _to_nse_symbol(s) and s not in FORCE_SKIP_SYMBOLS][:5]
    
    symbols_to_probe = crypto_symbols + nse_symbols
    providers_used = set()

    for sym in symbols_to_probe:
        ts = None
        provider = "unknown"
        
        if sym in crypto_symbols:
            ts = _binance_latest_ts(sym, bar_sec)
            provider = "binance"
        elif sym in nse_symbols:
            ts = _nse_india_latest_ts(sym, bar_sec)
            provider = "nse_india"

        if ts is None:
            continue
            
        providers_used.add(provider)
        advanced = ts > last_committed_ts
        probed.append({"symbol": sym, "provider": provider, "latest_ts": ts, "advanced": advanced})
        if advanced:
            advanced_globally = True

    latency_ms = round((time.time() - t_probe) * 1000)
    provider_label = "+".join(sorted(providers_used)) if providers_used else "none"
    
    return {
        "provider": provider_label,
        "symbols_probed": len(probed),
        "advanced": advanced_globally,
        "latency_ms": latency_ms,
        "symbol_results": probed,
    }


def aligned_closed_bar_ts(now: float | None = None, bar_sec: int = DEFAULT_BAR_SEC) -> int:
    """Latest fully closed bar open-time (UTC), from wall clock."""
    t = int(now if now is not None else time.time())
    return (t // bar_sec) * bar_sec - bar_sec


def wait_for_barrier_target(
    target_ts: int,
    bar_sec: int,
    provider_lag_sec: float,
    max_wait_sec: float,
) -> float:
    """Wait until closed-bar clock reaches target_ts, then provider lag buffer."""
    t0 = time.time()
    while aligned_closed_bar_ts(bar_sec=bar_sec) < target_ts:
        if time.time() - t0 > max_wait_sec:
            raise TimeoutError(
                f"aligned bar ts {aligned_closed_bar_ts(bar_sec=bar_sec)} < target {target_ts} "
                f"after {max_wait_sec:.0f}s"
            )
        time.sleep(1)
    time.sleep(provider_lag_sec)
    return time.time() - t0


def _fetch_yfinance_anchor_ts(symbol: str, bar_sec: int) -> int | None:
    """Fetch the latest closed bar open-time from yfinance for an anchor symbol."""
    import yfinance as yf
    try:
        interval = {60: "1m", 300: "5m", 900: "15m", 3600: "1h"}.get(bar_sec, "5m")
        df = yf.Ticker(symbol).history(period="1d", interval=interval, auto_adjust=True)
        if df is None or df.empty:
            return None
        last_idx = df.index[-1]
        if hasattr(last_idx, "timestamp"):
            return int(last_idx.timestamp())
        return None
    except Exception:
        return None


def active_temporal_observatory(
    target_ts: int,
    bar_sec: int,
    symbols: list[str],
    max_wait_sec: float,
    session_id: str,
    trace_log: Path,
) -> dict:
    """
    Phase D — Active Temporal Propagation Observatory.
    Layer 1: Poll exchange publication for anchor symbols.
    Layer 2: Poll provider visibility (yfinance) for anchor symbols.
    """
    t0 = time.time()
    
    # Select anchors (1 crypto, 1 NSE)
    crypto_anchors = [s for s in symbols if s in _BINANCE_SYMBOL_MAP and s not in FORCE_SKIP_SYMBOLS][:1]
    nse_anchors = [s for s in symbols if _to_nse_symbol(s) and s not in FORCE_SKIP_SYMBOLS][:1]
    anchors = crypto_anchors + nse_anchors

    if not anchors:
        # Fallback to passive wait if no anchors available
        waited = wait_for_barrier_target(target_ts, bar_sec, DEFAULT_PROVIDER_LAG_SEC, max_wait_sec)
        return {"exchange_to_observer_latency_ms": int(waited * 1000)}

    print(f"   🔭 [Phase D] Active Propagation Mode enabled. Anchors: {', '.join(anchors)}")
    
    # Layer 1: Wait for Exchange Publication
    exchange_publish_ts = None
    exchange_advanced = False
    while not exchange_advanced:
        if time.time() - t0 > max_wait_sec:
            raise TimeoutError(f"Exchange anchors never reached target {target_ts} after {max_wait_sec}s")
            
        for sym in anchors:
            ts = None
            provider = "unknown"
            if sym in crypto_anchors:
                ts = _binance_latest_ts(sym, bar_sec)
                provider = "binance"
            elif sym in nse_anchors:
                ts = _nse_india_latest_ts(sym, bar_sec)
                provider = "nse_india"
                
            if ts is not None and ts >= target_ts:
                exchange_publish_ts = time.time()
                exchange_advanced = True
                print(f"   📡 [Exchange] {provider} published {sym} at {exchange_publish_ts - t0:.1f}s")
                with open(trace_log, "a") as f:
                    f.write(json.dumps({
                        "session_id": session_id, "layer": "exchange", "wall_clock": exchange_publish_ts,
                        "target_ts": target_ts, "provider": provider, "symbol": sym, "latest_ts": ts
                    }) + "\\n")
                break
        if not exchange_advanced:
            time.sleep(1)

    # Layer 2: Wait for Provider (yfinance) Visibility
    provider_visible_ts = None
    provider_advanced = False
    attempts = 0
    while not provider_advanced:
        if time.time() - t0 > max_wait_sec:
            raise TimeoutError(f"Provider anchors never reached target {target_ts} after {max_wait_sec}s")
            
        attempts += 1
        for sym in anchors:
            ts = _fetch_yfinance_anchor_ts(sym, bar_sec)
            if ts is not None and ts >= target_ts:
                provider_visible_ts = time.time()
                provider_advanced = True
                print(f"   📡 [Provider] yfinance published {sym} at {provider_visible_ts - t0:.1f}s")
                with open(trace_log, "a") as f:
                    f.write(json.dumps({
                        "session_id": session_id, "layer": "provider", "wall_clock": provider_visible_ts,
                        "target_ts": target_ts, "provider": "yfinance", "symbol": sym, "latest_ts": ts
                    }) + "\\n")
                break
        if not provider_advanced:
            time.sleep(1)
            
    # Derive propagation telemetry
    provider_lag_ms = int((provider_visible_ts - exchange_publish_ts) * 1000)
    exchange_to_observer_latency_ms = int((provider_visible_ts - target_ts) * 1000)
    
    print(f"   📊 [Propagation] τ_exchange→provider = {provider_lag_ms}ms | visibility_attempts = {attempts}")
    
    return {
        "exchange_publish_ts": exchange_publish_ts,
        "provider_visible_ts": provider_visible_ts,
        "provider_lag_ms": provider_lag_ms,
        "exchange_to_observer_latency_ms": exchange_to_observer_latency_ms,
        "anchor_visibility_attempts": attempts
    }




def sorted_timestamps_from_data(data: dict[str, pd.DataFrame]) -> list[int]:
    all_ts: set[int] = set()
    for df in data.values():
        for ts in df.index:
            all_ts.add(int(ts.timestamp()))
    return sorted(all_ts)


def build_batch_at_ts(data: dict[str, pd.DataFrame], ts: int) -> list[dict]:
    batch = []
    dt = pd.to_datetime(ts, unit="s", utc=True)
    for sym, df in data.items():
        if dt not in df.index:
            continue
        row = df.loc[dt]
        if isinstance(row, pd.DataFrame):
            row = row.iloc[0]

        def get_val(k: str) -> float:
            raw = row.get(k, row.get(k.lower(), 0.0))
            if hasattr(raw, "iloc"):
                raw = raw.iloc[0]
            return float(raw)

        batch.append(
            {
                "symbol": sym,
                "timestamp": ts,
                "open": get_val("Open"),
                "high": get_val("High"),
                "low": get_val("Low"),
                "close": get_val("Close"),
                "volume": get_val("Volume"),
            }
        )
    return batch


def fetch_symbol_frames(
    engine: NSEIngestionEngine,
    symbols: list[str],
    max_workers: int = 15,
) -> dict[str, pd.DataFrame]:
    """Download latest OHLC frames for diagnostics (no quorum gate)."""
    active = [s for s in symbols if s not in FORCE_SKIP_SYMBOLS]
    data: dict[str, pd.DataFrame] = {}

    def _one(sym: str) -> tuple[str, pd.DataFrame]:
        return sym, engine.download_ticker_data(sym)

    with ThreadPoolExecutor(max_workers=max_workers) as ex:
        for sym, df in ex.map(_one, active):
            if df is not None and not df.empty:
                data[sym] = df
    return data


def symbol_close_lag_report(
    data: dict[str, pd.DataFrame],
    symbols: list[str],
    target_ts: int | None,
    bar_sec: int,
) -> list[dict]:
    """
    Per-symbol latest closed bar vs barrier target (feed synchronization diagnostics).

    Does not alter chronology — measurement only for stalled/skipped barriers.
    """
    wall_aligned = aligned_closed_bar_ts(bar_sec=bar_sec)
    ref_ts = target_ts if target_ts is not None else wall_aligned
    rows: list[dict] = []
    for sym in symbols:
        if sym in FORCE_SKIP_SYMBOLS:
            rows.append(
                {
                    "symbol": sym,
                    "status": "force_skip",
                    "latest_closed_ts": None,
                    "lag_vs_target_sec": None,
                    "lag_bars": None,
                    "at_target": None,
                }
            )
            continue
        df = data.get(sym)
        if df is None or df.empty:
            rows.append(
                {
                    "symbol": sym,
                    "status": "no_data",
                    "latest_closed_ts": None,
                    "lag_vs_target_sec": None,
                    "lag_bars": None,
                    "at_target": False,
                }
            )
            continue
        latest = int(df.index[-1].timestamp())
        lag_sec = ref_ts - latest
        rows.append(
            {
                "symbol": sym,
                "status": "ok",
                "latest_closed_ts": latest,
                "lag_vs_target_sec": lag_sec,
                "lag_bars": round(lag_sec / bar_sec, 2) if bar_sec else None,
                "at_target": latest >= ref_ts,
            }
        )
    rows.sort(key=lambda r: (-(r.get("lag_vs_target_sec") or -10**9), r["symbol"]))
    return rows


def summarize_symbol_lags(rows: list[dict], target_ts: int) -> dict:
    ok = [r for r in rows if r.get("status") == "ok"]
    behind = [r for r in ok if not r.get("at_target")]
    return {
        "target_ts": target_ts,
        "symbols_ok": len(ok),
        "symbols_at_target": sum(1 for r in ok if r.get("at_target")),
        "symbols_behind_target": len(behind),
        "max_lag_sec": max((r["lag_vs_target_sec"] for r in behind), default=0),
        "poison_symbols": [r["symbol"] for r in behind[:5]],
    }


def sync_breakdown(
    data: dict[str, pd.DataFrame],
    live_ts: int,
    target_ts: int | None,
) -> dict:
    """
    Split fetched symbols by synchronized barrier membership.

    participation = symbols_at_live_ts (exact bar at global max close).
    symbols_at_target but not at live_ts = feed fragmentation (caught up to target
    but on a different closed bar than the cohort max).
    """
    at_live: list[str] = []
    at_target_not_live: list[str] = []
    behind_target: list[str] = []
    ref = target_ts if target_ts is not None else live_ts
    for sym, df in data.items():
        if df is None or df.empty:
            continue
        latest = int(df.index[-1].timestamp())
        if latest == live_ts:
            at_live.append(sym)
        elif latest >= ref:
            at_target_not_live.append(sym)
        else:
            behind_target.append(sym)
    return {
        "live_ts": live_ts,
        "target_ts": ref,
        "symbols_at_live_ts": len(at_live),
        "symbols_at_target_not_live_ts": len(at_target_not_live),
        "symbols_behind_target": len(behind_target),
        "fragmentation_symbols": at_target_not_live[:8],
    }


def compute_trl_metrics(
    cycle: int,
    barrier_committed: bool,
    skip_reason: str | None,
    expected_symbols: int,
    participating_symbols: int,
    fetch_stats: dict,
    duration_sec: float
) -> dict:
    """
    Temporal Reliability Layer (TRL) — Observability-quality chronology confidence instrumentation.
    Separates market chronology stalls, feed fragmentation, and network instability.

    Failure taxonomy (Phase B extended):
      PROVIDER_STALE          — primary provider lagged; secondary advanced
      PROVIDER_STALE_RESOLVED — primary caught up within retry window
      TEMPORAL_DIVERGENCE     — ts disagrees; retry in progress
      QUORUM_COLLAPSE         — insufficient symbol participation
      API_PROVIDER_DEGRADED   — transport/health instability
      TRUE_TEMPORAL_STALL     — all providers failed to advance after full retry horizon
      CAUSAL_INVALIDITY       — impossible chronology ordering; immediate abort
      LOCAL_NETWORK_DEGRADED  — zero symbols reachable
      TRUE_MARKET_STALL       — wall-clock did not advance past barrier target
      UNKNOWN_STALL           — unclassified
    """
    success = fetch_stats.get("success", 0)
    attempted = fetch_stats.get("attempted", 1) or 1
    sync = fetch_stats.get("sync_breakdown") or {}

    # 1. API Health & Network Jitter
    api_health_score = round(success / attempted, 4)
    network_stability = round(max(0.0, min(1.0, 1.0 - (duration_sec / 30.0))), 4)

    # 2. Provider Consensus & Fragmentation
    symbols_at_live_ts = sync.get("symbols_at_live_ts", participating_symbols)
    provider_consensus = round(symbols_at_live_ts / success, 4) if success > 0 else 1.0
    frag_count = sync.get("symbols_at_target_not_live_ts", 0)
    fragmentation_ratio = round(frag_count / success, 4) if success > 0 else 0.0

    # 3. Quorum & Barrier Confidence
    barrier_confidence = round(participating_symbols / expected_symbols, 4) if expected_symbols > 0 else 0.0

    # 4. Chronology Integrity Taxonomy
    if not barrier_committed:
        # ── Failed/Pending states ──
        if skip_reason in ("provider_stale_yfinance", "provider_stale_recovered",
                           "provider_stale_unresolved"):
            chronology_integrity = "DEGRADED"
            failure_type = "PROVIDER_STALE"
        elif skip_reason == "pending_advancement":
            # Transient state — retry horizon active
            chronology_integrity = "PENDING_ADVANCEMENT"
            failure_type = "TEMPORAL_DIVERGENCE"
        elif skip_reason == "true_temporal_stall":
            chronology_integrity = "FAILED"
            failure_type = "TRUE_TEMPORAL_STALL"
        elif skip_reason == "causal_invalidity":
            chronology_integrity = "FAILED"
            failure_type = "CAUSAL_INVALIDITY"
        elif skip_reason == "barrier_wait_timeout":
            lag_sum = fetch_stats.get("lag_summary") or {}
            chronology_integrity = "FAILED"
            if lag_sum.get("symbols_ok", 0) == 0:
                failure_type = "LOCAL_NETWORK_DEGRADED"
            elif lag_sum.get("symbols_at_target", 0) == 0:
                failure_type = "TRUE_MARKET_STALL"
            else:
                failure_type = "API_PROVIDER_DEGRADED"
        elif skip_reason == "quorum_not_met":
            chronology_integrity = "FAILED"
            failure_type = "LOCAL_NETWORK_DEGRADED" if success == 0 else "QUORUM_COLLAPSE"
        elif skip_reason == "non_advancing_ts":
            # Legacy path — only reached if triangulation was bypassed
            chronology_integrity = "FAILED"
            failure_type = "TEMPORAL_DIVERGENCE"
        else:
            chronology_integrity = "FAILED"
            failure_type = "UNKNOWN_STALL"
    else:
        failure_type = None
        if barrier_confidence >= 0.8 and provider_consensus >= 0.9:
            chronology_integrity = "TRUSTED"
        elif fragmentation_ratio > 0.2:
            chronology_integrity = "FRAGMENTED"
        elif api_health_score < 0.5:
            chronology_integrity = "UNSTABLE"
        else:
            chronology_integrity = "DEGRADED"

    return {
        "barrier_confidence": barrier_confidence,
        "provider_consensus": provider_consensus,
        "fragmentation_ratio": fragmentation_ratio,
        "api_health_score": api_health_score,
        "network_stability": network_stability,
        "chronology_integrity": chronology_integrity,
        "failure_type": failure_type
    }



def fetch_live_latest_batch(
    engine: NSEIngestionEngine,
    symbols: list[str],
    max_workers: int = 15,
    quorum_ratio: float = 0.15,
    target_ts: int | None = None,
    bar_sec: int = DEFAULT_BAR_SEC,
) -> tuple[list[dict], int | None, dict]:
    """Parallel yfinance fetch (proven path) — NEAGI etc. in FORCE_SKIP_SYMBOLS."""
    data = fetch_symbol_frames(engine, symbols, max_workers=max_workers)
    attempted = len([s for s in symbols if s not in FORCE_SKIP_SYMBOLS])
    success = len(data)
    min_required = max(1, int(attempted * quorum_ratio))
    lag_rows = symbol_close_lag_report(data, symbols, target_ts, bar_sec)
    stats = {
        "attempted": attempted,
        "success": success,
        "skipped": len(symbols) - attempted,
        "quorum_threshold": min_required,
        "quorum_met": success >= min_required,
        "acquisition_ratio": round(success / attempted, 4) if attempted else 0.0,
        "symbol_close_lags": lag_rows,
    }
    if target_ts is not None:
        stats["lag_summary"] = summarize_symbol_lags(lag_rows, target_ts)
    if not data or not stats["quorum_met"]:
        return [], None, stats

    latest_ts = max(int(df.index[-1].timestamp()) for df in data.values())
    stats["sync_breakdown"] = sync_breakdown(data, latest_ts, target_ts)
    return build_batch_at_ts(data, latest_ts), latest_ts, stats


def persist_telemetry(engine: NSEIngestionEngine, lines: list[str], persist: bool) -> tuple[int, int]:
    processed = corridors = 0
    for line in lines:
        if not line.startswith("[TELEMETRY]"):
            continue
        if persist:
            rec = engine.process_telemetry_line(line)
            if rec:
                processed += 1
                if rec.get("corridor"):
                    corridors += 1
    if persist:
        engine.dedupe.save()
        engine._gzip_pool.flush_all()
    return processed, corridors


def main():
    parser = argparse.ArgumentParser(description="Live warm observatory session")
    parser.add_argument("--batch-id", type=int, default=3)
    parser.add_argument("--run-label", default="live")
    parser.add_argument("--cycles", type=int, default=3, help="Live incremental cycles after warmup")
    parser.add_argument("--warmup-only", action="store_true", help="Only warm Rust from frozen substrate")
    parser.add_argument("--live-only", action="store_true", help="Skip warmup; live fetches only")
    parser.add_argument(
        "--sleep-sec",
        type=int,
        default=0,
        help="Fixed sleep between cycles (only if --no-bar-aligned)",
    )
    parser.add_argument(
        "--bar-aligned",
        action=argparse.BooleanOptionalAction,
        default=True,
        help="Wait for next closed 5m bar + provider lag before each fetch (default)",
    )
    parser.add_argument("--bar-sec", type=int, default=DEFAULT_BAR_SEC, help="Barrier interval seconds")
    parser.add_argument(
        "--provider-lag-sec",
        type=int,
        default=DEFAULT_PROVIDER_LAG_SEC,
        help="Extra seconds after bar close before fetch",
    )
    parser.add_argument(
        "--max-barrier-wait-sec",
        type=int,
        default=420,
        help="Abort if wall-clock aligned close < target within this window "
        "(try 600–900 for noisy 24/7 crypto feeds)",
    )
    parser.add_argument("--warmup-intervals", type=int, default=0, help="Cap warmup intervals (0=all)")
    parser.add_argument(
        "--quorum-ratio",
        type=float,
        default=0.15,
        help="Min fraction of active symbols with data required to commit barrier",
    )
    parser.add_argument(
        "--retry-profile",
        default=DEFAULT_RETRY_PROFILE,
        choices=list(RETRY_PROFILES),
        help=(
            "Retry horizon policy for non-advancing timestamps. "
            "'continuous_market' (3×15s) for crypto/intraday; "
            "'exchange_open' (6×20s) for NSE/TSE/ASX open turbulence; "
            "'auction_window' (8×30s) for pre-open call auctions."
        ),
    )
    parser.add_argument(
        "--temporal-observatory",
        action="store_true",
        help="Enable Phase D active propagation mode to dynamically poll anchor symbols."
    )
    args = parser.parse_args()

    cohort_file = Path(f"cohorts/batch_{args.batch_id:03d}.txt")
    archive_dir = resolve_archive_dir(args.batch_id, False, args.run_label)
    archive_dir.mkdir(parents=True, exist_ok=True)

    symbols = [
        s
        for s in cohort_file.read_text().splitlines()
        if s.strip() and s.strip() not in FORCE_SKIP_SYMBOLS
    ]
    frozen_data, frozen_manifest = load_frozen_cohort(args.batch_id, symbols)

    engine = NSEIngestionEngine(
        cohort_file=cohort_file,
        archive_dir=archive_dir,
        batch_id=args.batch_id,
        run_label=args.run_label,
        from_frozen=True,
    )
    engine.init_dedupe_index()

    steps_log = archive_dir / "metadata" / "live_session_steps.jsonl"
    steps_log.parent.mkdir(parents=True, exist_ok=True)

    print("=" * 60)
    print("CHRONOSENTIMENT — LIVE SYNCHRONIZED SESSION")
    print("=" * 60)
    print(f"  Batch              : {args.batch_id:03d}")
    print(f"  Archive            : {archive_dir}")
    print(f"  Frozen fingerprint : {frozen_manifest.get('timeline_fingerprint')}")
    print(f"  Frozen hash        : {frozen_manifest.get('substrate_hash')}")
    print("=" * 60)

    # Session identity — stamped on every TRL step for hermetic grouping
    IST = timezone(timedelta(hours=5, minutes=30))
    session_id = datetime.now(IST).strftime("%Y-%m-%dT%H:%M:%S%z")
    retry_profile = RETRY_PROFILES[args.retry_profile]
    print(f"  Session ID         : {session_id}")
    print(f"  Retry Profile      : {args.retry_profile} "
          f"(attempts={retry_profile['attempts']}, "
          f"spacing={retry_profile['spacing_sec']}s, "
          f"stale_window={retry_profile['stale_window_sec']}s)")
    print("=" * 60)

    with ObservatoryDaemon() as daemon:
        if not args.live_only:
            timestamps = sorted_timestamps_from_data(frozen_data)
            if args.warmup_intervals > 0:
                timestamps = timestamps[-args.warmup_intervals :]
            print(f"\n🔥 Warming observatory: {len(timestamps)} synchronized intervals (no persist)...")
            t0 = time.time()
            for i, ts in enumerate(timestamps, 1):
                batch = build_batch_at_ts(frozen_data, ts)
                if not batch:
                    continue
                daemon.send_batch(batch)
                if i % 50 == 0 or i == len(timestamps):
                    print(f"   Warmed {i}/{len(timestamps)} intervals ({time.time()-t0:.1f}s)")
            print(f"✅ Warmup complete in {time.time()-t0:.1f}s — Rust buffers hot\n")

        if args.warmup_only:
            return

        skip_note = f" (skip: {', '.join(sorted(FORCE_SKIP_SYMBOLS))})" if FORCE_SKIP_SYMBOLS else ""
        print(f"📡 Live incremental cycles: {args.cycles}{skip_note}")

        last_committed_ts: int | None = None

        for cycle in range(1, args.cycles + 1):
            propagation_telemetry = {}
            if last_committed_ts is not None:
                target_ts = last_committed_ts + args.bar_sec
                if args.bar_aligned:
                    try:
                        if args.temporal_observatory:
                            trace_log = archive_dir / "metadata" / "provider_propagation_trace.jsonl"
                            propagation_telemetry = active_temporal_observatory(
                                target_ts=target_ts,
                                bar_sec=args.bar_sec,
                                symbols=symbols,
                                max_wait_sec=float(args.max_barrier_wait_sec),
                                session_id=session_id,
                                trace_log=trace_log
                            )
                        else:
                            waited = wait_for_barrier_target(
                                target_ts,
                                args.bar_sec,
                                float(args.provider_lag_sec),
                                float(args.max_barrier_wait_sec),
                            )
                            print(
                                f"   ⏳ barrier-aligned wait {waited:.1f}s "
                                f"(target_ts={target_ts}, closed={aligned_closed_bar_ts(bar_sec=args.bar_sec)})"
                            )
                    except TimeoutError as e:
                        print(f"   ❌ {e}")
                        stall_data = fetch_symbol_frames(engine, symbols)
                        lag_rows = symbol_close_lag_report(
                            stall_data, symbols, target_ts, args.bar_sec
                        )
                        lag_sum = summarize_symbol_lags(lag_rows, target_ts)
                        
                        # Build mock fetch stats for wait timeout TRL calculation
                        fetch_stats_timeout = {
                            "success": lag_sum.get("symbols_at_target", 0),
                            "attempted": len(symbols),
                            "lag_summary": lag_sum,
                        }
                        trl = compute_trl_metrics(
                            cycle=cycle,
                            barrier_committed=False,
                            skip_reason="barrier_wait_timeout",
                            expected_symbols=len(symbols),
                            participating_symbols=0,
                            fetch_stats=fetch_stats_timeout,
                            duration_sec=float(args.max_barrier_wait_sec),
                        )
                        stall_step = {
                            "cycle": cycle,
                            "barrier_committed": False,
                            "skip_reason": "barrier_wait_timeout",
                            "target_ts": target_ts,
                            "aligned_wall_ts": aligned_closed_bar_ts(bar_sec=args.bar_sec),
                            "last_committed_ts": last_committed_ts,
                            "max_barrier_wait_sec": args.max_barrier_wait_sec,
                            "lag_summary": lag_sum,
                            "symbol_close_lags": lag_rows,
                            "completed_at_utc": datetime.now(timezone.utc).isoformat(),
                            **trl
                        }
                        with open(steps_log, "a") as f:
                            f.write(json.dumps(stall_step) + "\n")
                        print(
                            f"   📊 Feed lag: {lag_sum['symbols_at_target']}/"
                            f"{lag_sum['symbols_ok']} at target | "
                            f"behind={lag_sum['symbols_behind_target']} | "
                            f"max_lag_sec={lag_sum['max_lag_sec']}"
                        )
                        if lag_sum.get("poison_symbols"):
                            print(
                                f"   📊 Slowest vs target: {lag_sum['poison_symbols']}"
                            )
                        print("❌ Chronology stall — no new closed bar (feed sync or lag exceeded)")
                        sys.exit(1)
                elif args.sleep_sec and cycle > 1:
                    time.sleep(args.sleep_sec)

            target_for_lag = (
                (last_committed_ts + args.bar_sec) if last_committed_ts is not None else None
            )
            t0 = time.time()
            batch, live_ts, fetch_stats = fetch_live_latest_batch(
                engine,
                symbols,
                quorum_ratio=args.quorum_ratio,
                target_ts=target_for_lag,
                bar_sec=args.bar_sec,
            )
            expected_symbols = len(symbols)
            if not batch:
                trl = compute_trl_metrics(
                    cycle=cycle,
                    barrier_committed=False,
                    skip_reason="quorum_not_met",
                    expected_symbols=expected_symbols,
                    participating_symbols=0,
                    fetch_stats=fetch_stats,
                    duration_sec=round(time.time() - t0, 3),
                )
                step = {
                    "session_id": session_id,
                    "cycle": cycle,
                    "expected_symbols": expected_symbols,
                    "participating_symbols": 0,
                    "quorum_ratio": 0.0,
                    "barrier_committed": False,
                    "skip_reason": "quorum_not_met",
                    "target_ts": target_for_lag,
                    "fetch_stats": fetch_stats,
                    "lag_summary": fetch_stats.get("lag_summary"),
                    "symbol_close_lags": fetch_stats.get("symbol_close_lags"),
                    "duration_sec": round(time.time() - t0, 3),
                    "completed_at_utc": datetime.now(timezone.utc).isoformat(),
                    "propagation_telemetry": propagation_telemetry,
                    **trl
                }
                with open(steps_log, "a") as f:
                    f.write(json.dumps(step) + "\n")
                print(
                    f"   Cycle {cycle}: barrier skipped "
                    f"(quorum={fetch_stats.get('quorum_met')} success={fetch_stats.get('success')}/"
                    f"{fetch_stats.get('attempted')})"
                )
                if args.sleep_sec:
                    time.sleep(args.sleep_sec)
                continue

            if last_committed_ts is not None and live_ts <= last_committed_ts:
                # ── Phase B: Full Chronology Escalation Ladder ────────────────
                #
                # Ontology: non_advancing_ts is NOT a terminal condition.
                # It enters PENDING_ADVANCEMENT and works through a tiered
                # escalation before reaching TRUE_TEMPORAL_STALL (abort).
                #
                # Ladder:
                #   1. Probe secondary (Binance) immediately
                #   2a. Secondary advanced → PROVIDER_STALE
                #       → bounded yfinance retry (20s)
                #       → recovered: commit normally
                #       → unresolved: log DEGRADED, continue to next cycle
                #   2b. Secondary also not advanced → PENDING_ADVANCEMENT
                #       → multi-attempt retry horizon (3 attempts, 15s spacing)
                #       → at each attempt: re-probe Binance AND re-fetch yfinance
                #       → if any provider advances: recover and commit
                #       → horizon exhausted: TRUE_TEMPORAL_STALL → abort

                _RETRY_ATTEMPTS          = retry_profile["attempts"]
                _RETRY_SPACING_S         = retry_profile["spacing_sec"]
                _PROVIDER_STALE_WINDOW_S = retry_profile["stale_window_sec"]

                print(
                    f"   ⚠️  Non-advancing ts={live_ts} (last={last_committed_ts}) "
                    f"— probing secondary provider..."
                )
                probe = probe_secondary_ts(symbols, last_committed_ts, args.bar_sec)
                print(
                    f"   🔭 Secondary probe: provider={probe['provider']} "
                    f"probed={probe['symbols_probed']} advanced={probe['advanced']} "
                    f"latency={probe['latency_ms']}ms"
                )

                # ── Arm shared step-log helper ────────────────────────────────
                def _log_stall_step(
                    skip_reason: str,
                    probe_result: dict,
                    extra_fields: dict | None = None,
                ) -> None:
                    trl = compute_trl_metrics(
                        cycle=cycle,
                        barrier_committed=False,
                        skip_reason=skip_reason,
                        expected_symbols=expected_symbols,
                        participating_symbols=len(batch),
                        fetch_stats=fetch_stats,
                        duration_sec=round(time.time() - t0, 3),
                    )
                    s = {
                        "session_id": session_id,
                        "cycle": cycle,
                        "ts": live_ts,
                        "expected_symbols": expected_symbols,
                        "participating_symbols": len(batch),
                        "quorum_ratio": round(len(batch) / expected_symbols, 4) if expected_symbols else 0.0,
                        "barrier_committed": False,
                        "skip_reason": skip_reason,
                        "target_ts": target_for_lag,
                        "last_committed_ts": last_committed_ts,
                        "fetch_stats": fetch_stats,
                        "lag_summary": fetch_stats.get("lag_summary"),
                        "symbol_close_lags": fetch_stats.get("symbol_close_lags"),
                        "provider_triangulation": probe_result,
                        "duration_sec": round(time.time() - t0, 3),
                        "completed_at_utc": datetime.now(timezone.utc).isoformat(),
                        "propagation_telemetry": propagation_telemetry,
                        **(extra_fields or {}),
                        **trl,
                    }
                    with open(steps_log, "a") as f:
                        f.write(json.dumps(s) + "\n")

                # ── Branch A: secondary advanced → PROVIDER_STALE ─────────────
                if probe["advanced"]:
                    probe["outcome"] = "PROVIDER_STALE"
                    recovered = False
                    retry_deadline = time.time() + _PROVIDER_STALE_WINDOW_S
                    print(f"   🔄 PROVIDER_STALE: Binance advanced. Retrying yfinance for up to {_PROVIDER_STALE_WINDOW_S}s...")
                    while time.time() < retry_deadline:
                        time.sleep(2)
                        retry_batch, retry_ts, retry_stats = fetch_live_latest_batch(
                            engine, symbols,
                            quorum_ratio=args.quorum_ratio,
                            target_ts=target_for_lag,
                            bar_sec=args.bar_sec,
                        )
                        if retry_batch and retry_ts is not None and retry_ts > last_committed_ts:
                            batch = retry_batch
                            live_ts = retry_ts
                            fetch_stats = retry_stats
                            probe["outcome"] = "PROVIDER_STALE_RECOVERED"
                            recovered = True
                            print(f"   ✅ PROVIDER_STALE_RECOVERED: yfinance advanced to ts={live_ts}")
                            break

                    if not recovered:
                        probe["outcome"] = "PROVIDER_STALE_UNRESOLVED"
                        _log_stall_step("provider_stale_unresolved", probe)
                        print(f"   ⚠️  PROVIDER_STALE_UNRESOLVED — yfinance still stale after {_PROVIDER_STALE_WINDOW_S}s. Continuing to next cycle.")
                        continue  # do NOT abort; next cycle may succeed
                    # else: fall through to normal barrier commit

                # ── Branch B: secondary also not advanced → full retry horizon ─
                else:
                    probe["outcome"] = "PENDING_ADVANCEMENT"
                    _log_stall_step("pending_advancement", probe)
                    print(f"   ⏳ PENDING_ADVANCEMENT: both providers at ts={live_ts}. Entering retry horizon ({_RETRY_ATTEMPTS} attempts, {_RETRY_SPACING_S}s spacing)...")

                    recovered = False
                    for attempt in range(1, _RETRY_ATTEMPTS + 1):
                        time.sleep(_RETRY_SPACING_S)

                        # Re-probe secondary
                        re_probe = probe_secondary_ts(symbols, last_committed_ts, args.bar_sec)
                        # Re-fetch primary
                        retry_batch, retry_ts, retry_stats = fetch_live_latest_batch(
                            engine, symbols,
                            quorum_ratio=args.quorum_ratio,
                            target_ts=target_for_lag,
                            bar_sec=args.bar_sec,
                        )

                        primary_advanced = retry_batch and retry_ts is not None and retry_ts > last_committed_ts
                        secondary_advanced = re_probe["advanced"]

                        print(
                            f"   🔁 Retry {attempt}/{_RETRY_ATTEMPTS}: "
                            f"yfinance_ts={retry_ts} primary_adv={primary_advanced} "
                            f"binance_adv={secondary_advanced}"
                        )

                        if primary_advanced:
                            # Primary caught up — commit normally
                            batch = retry_batch
                            live_ts = retry_ts
                            fetch_stats = retry_stats
                            probe["outcome"] = "TEMPORAL_DIVERGENCE_RECOVERED"
                            probe["recovered_on_attempt"] = attempt
                            recovered = True
                            print(f"   ✅ TEMPORAL_DIVERGENCE_RECOVERED: yfinance advanced to ts={live_ts} on attempt {attempt}")
                            break
                        elif secondary_advanced:
                            # Secondary advanced but primary still stale — switch to
                            # PROVIDER_STALE path with a short final yfinance retry
                            final_deadline = time.time() + 10
                            while time.time() < final_deadline:
                                time.sleep(2)
                                rb, rt, rs = fetch_live_latest_batch(
                                    engine, symbols,
                                    quorum_ratio=args.quorum_ratio,
                                    target_ts=target_for_lag,
                                    bar_sec=args.bar_sec,
                                )
                                if rb and rt is not None and rt > last_committed_ts:
                                    batch = rb
                                    live_ts = rt
                                    fetch_stats = rs
                                    probe["outcome"] = "PROVIDER_STALE_RECOVERED"
                                    probe["recovered_on_attempt"] = attempt
                                    recovered = True
                                    print(f"   ✅ PROVIDER_STALE_RECOVERED (late): yfinance advanced to ts={live_ts}")
                                    break
                            if recovered:
                                break

                    if not recovered:
                        # Full horizon exhausted — TRUE_TEMPORAL_STALL
                        probe["outcome"] = "TRUE_TEMPORAL_STALL"
                        _log_stall_step("true_temporal_stall", probe)
                        print(
                            f"   ❌ TRUE_TEMPORAL_STALL: no provider advanced after "
                            f"{_RETRY_ATTEMPTS} attempts × {_RETRY_SPACING_S}s. "
                            f"Genuine chronology stall confirmed. Aborting."
                        )
                        sys.exit(1)
                    # else: fall through to normal barrier commit

                # ── Execution falls through here only on recovery ──────────────
                # (all non-recovered paths either `continue` or `sys.exit` above)


            lines = daemon.send_batch(batch)
            processed, corridors = persist_telemetry(engine, lines, persist=True)

            participating = len(batch)
            participation_ratio = round(participating / expected_symbols, 4) if expected_symbols else 0.0

            # Compute fragmentation entropy (Shannon entropy of timestamps)
            lags = fetch_stats.get("symbol_close_lags") or []
            ts_counts = {}
            for item in lags:
                ts_val = item.get("latest_closed_ts")
                ts_counts[ts_val] = ts_counts.get(ts_val, 0) + 1
            
            fragmentation_entropy = 0.0
            total_lag_symbols = sum(ts_counts.values())
            if total_lag_symbols > 0:
                for c in ts_counts.values():
                    p = c / total_lag_symbols
                    fragmentation_entropy -= p * math.log2(p)
            fragmentation_entropy = round(fragmentation_entropy, 4)

            propagation_state = "UNKNOWN"
            if fragmentation_entropy > 0.0:
                propagation_state = "PARTIAL_FRAGMENTATION"
            elif participation_ratio >= 0.99:
                propagation_state = "FULL_PROPAGATION"
            else:
                propagation_state = "COHORT_SYNCHRONIZED"

            if args.temporal_observatory:
                propagation_telemetry["cohort_sync_ratio"] = participation_ratio
                propagation_telemetry["fragmentation_entropy"] = fragmentation_entropy
                propagation_telemetry["propagation_state"] = propagation_state
                
                # ── Layer 3: Fragmentation Decay Curve ──────────────────────
                decay_curve = [{"t": 0, "sync": participation_ratio}]
                half_life_ms = None
                
                if participation_ratio < 0.99:
                    print(f"   🔭 [Layer 3] Observing fragmentation decay (start={participation_ratio:.1%})...")
                    t_start_decay = time.time()
                    target_sync_half = participation_ratio + (1.0 - participation_ratio) / 2.0
                    
                    while time.time() - t_start_decay < 60:
                        time.sleep(10)
                        elapsed = int(time.time() - t_start_decay)
                        
                        _, _, temp_stats = fetch_live_latest_batch(
                            engine, symbols, quorum_ratio=0.0, target_ts=target_for_lag, bar_sec=args.bar_sec
                        )
                        
                        synced_count = temp_stats.get("lag_summary", {}).get("symbols_at_target", 0)
                        new_sync = round(synced_count / expected_symbols, 4) if expected_symbols else 0.0
                        
                        decay_curve.append({"t": elapsed, "sync": new_sync})
                        print(f"      decay t={elapsed}s | sync={new_sync:.1%}")
                        
                        if half_life_ms is None and new_sync >= target_sync_half:
                            half_life_ms = elapsed * 1000
                            
                        if new_sync >= 0.99:
                            break
                            
                propagation_telemetry["cohort_sync_half_life_ms"] = half_life_ms
                propagation_telemetry["fragmentation_decay_curve"] = decay_curve

            trl = compute_trl_metrics(
                cycle=cycle,
                barrier_committed=True,
                skip_reason=None,
                expected_symbols=expected_symbols,
                participating_symbols=participating,
                fetch_stats=fetch_stats,
                duration_sec=round(time.time() - t0, 3),
            )
            step = {
                "session_id": session_id,
                "cycle": cycle,
                "ts": live_ts,
                "expected_symbols": expected_symbols,
                "participating_symbols": participating,
                "quorum_ratio": participation_ratio,
                "symbols": participating,
                "processed_ticks": processed,
                "corridors": corridors,
                "telemetry_lines": len(lines),
                "duration_sec": round(time.time() - t0, 3),
                "dedupe_skipped": engine.dedupe.skipped,
                "barrier_committed": True,
                "target_ts": target_for_lag,
                "fetch_stats": fetch_stats,
                "lag_summary": fetch_stats.get("lag_summary"),
                "symbol_close_lags": fetch_stats.get("symbol_close_lags"),
                "sync_breakdown": fetch_stats.get("sync_breakdown"),
                "fetch_success": fetch_stats.get("success"),
                "completed_at_utc": datetime.now(timezone.utc).isoformat(),
                "propagation_telemetry": propagation_telemetry,
                **trl
            }
            with open(steps_log, "a") as f:
                f.write(json.dumps(step) + "\n")
            last_committed_ts = live_ts

            lag_sum = fetch_stats.get("lag_summary") or {}
            sync = fetch_stats.get("sync_breakdown") or {}
            lag_note = ""
            if lag_sum:
                lag_note = (
                    f" | fetched={fetch_stats.get('success', '?')}"
                    f" at_live_ts={sync.get('symbols_at_live_ts', participating)}"
                    f" at_target={lag_sum.get('symbols_at_target', '?')}"
                    f"/{lag_sum.get('symbols_ok', '?')}"
                )
                frag = sync.get("symbols_at_target_not_live_ts", 0)
                if frag:
                    lag_note += f" fragmented={frag}"
            print(
                f"   ⚡ Cycle {cycle}/{args.cycles} | ts={live_ts} | "
                f"participation={participating}/{expected_symbols} ({participation_ratio:.1%}) | "
                f"ticks={processed} | corridors={corridors} | "
                f"dedupe_skip={engine.dedupe.skipped} | {step['duration_sec']}s{lag_note}"
            )

            if not args.bar_aligned and args.sleep_sec and cycle < args.cycles:
                time.sleep(args.sleep_sec)

    engine._gzip_pool.close_all()
    print("\n✅ Live session complete")
    print(f"   Steps log: {steps_log}")


if __name__ == "__main__":
    main()
