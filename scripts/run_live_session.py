#!/usr/bin/env python3
"""
Live synchronized observatory session — Canonical execution kernel runner.

Phase 1: Active Temporal Propagation Observatory (waits for exchange/provider bar visibility).
Phase 2: Freeze mini-cohort (latest 5 days) via yfinance.
Phase 3: Run canonical `cs-ingest replay-step --resume` to process only the novel bars.
"""

import argparse
import time
import json
import urllib.request
import urllib.parse
from datetime import datetime, timedelta, timezone
from pathlib import Path

# Use the canonical frozen helpers
from candle_substrate import incremental_update_cohort
from run_nse_cohort import run_frozen_via_cs_ingest, resolve_archive_dir

# ─── Global Constants ───
FORCE_SKIP_SYMBOLS = frozenset({"NEAGI.NS"})
DEFAULT_BAR_SEC = 300
DEFAULT_PROVIDER_LAG_SEC = 15

# Crypto symbol mapping for Binance fast-ping
_BINANCE_SYMBOL_MAP = {
    "BTC-USD": "BTCUSDT", "ETH-USD": "ETHUSDT", "SOL-USD": "SOLUSDT",
    "BNB-USD": "BNBUSDT", "XRP-USD": "XRPUSDT", "ADA-USD": "ADAUSDT",
    "DOGE-USD": "DOGEUSDT", "AVAX-USD": "AVAXUSDT", "DOT-USD": "DOTUSDT",
    "LINK-USD": "LINKUSDT", "LTC-USD": "LTCUSDT", "BCH-USD": "BCHUSDT",
}
_BINANCE_KLINE_URL = "https://api.binance.com/api/v3/klines"

_NSE_COOKIES: str | None = None


def _to_nse_symbol(symbol: str) -> str | None:
    if symbol.endswith(".NS"):
        return symbol[:-3]
    if symbol.endswith(".BO"):
        return symbol[:-3]
    return None


def _binance_latest_ts(symbol: str, bar_sec: int) -> int | None:
    binance_sym = _BINANCE_SYMBOL_MAP.get(symbol)
    if not binance_sym: return None
    interval = {60: "1m", 300: "5m", 900: "15m", 3600: "1h"}.get(bar_sec)
    if not interval: return None

    try:
        url = f"{_BINANCE_KLINE_URL}?symbol={binance_sym}&interval={interval}&limit=2"
        with urllib.request.urlopen(url, timeout=5) as resp:
            data = json.loads(resp.read())
        if not data or len(data) < 1: return None
        row = data[-2] if len(data) >= 2 else data[-1]
        return int(row[0]) // 1000
    except Exception:
        return None


def _nse_india_latest_ts(symbol: str, bar_sec: int) -> int | None:
    global _NSE_COOKIES
    nse_sym = _to_nse_symbol(symbol)
    if not nse_sym: return None

    headers = {
        "User-Agent": "Mozilla/5.0",
        "Accept": "application/json",
        "Referer": "https://www.nseindia.com/",
    }

    try:
        if not _NSE_COOKIES:
            req = urllib.request.Request("https://www.nseindia.com/", headers=headers)
            with urllib.request.urlopen(req, timeout=5) as resp:
                _NSE_COOKIES = resp.headers.get("Set-Cookie", "")
        
        headers["Cookie"] = _NSE_COOKIES
        url = f"https://www.nseindia.com/api/quote-equity?symbol={urllib.parse.quote(nse_sym)}"
        req = urllib.request.Request(url, headers=headers)
        with urllib.request.urlopen(req, timeout=5) as resp:
            data = json.loads(resp.read())
        
        update_time_str = data.get("metadata", {}).get("lastUpdateTime")
        if not update_time_str: return None
            
        IST = timezone(timedelta(hours=5, minutes=30))
        dt = datetime.strptime(update_time_str, "%d-%b-%Y %H:%M:%S").replace(tzinfo=IST)
        return (int(dt.timestamp()) // bar_sec) * bar_sec
    except Exception:
        return None


def _fetch_yfinance_anchor_ts(symbol: str, bar_sec: int) -> int | None:
    import yfinance as yf
    try:
        interval = {60: "1m", 300: "5m", 900: "15m", 3600: "1h"}.get(bar_sec, "5m")
        df = yf.Ticker(symbol).history(period="1d", interval=interval, auto_adjust=True)
        if df is None or df.empty: return None
        return int(df.index[-1].timestamp())
    except Exception:
        return None


def wait_for_barrier_target(target_ts: int, bar_sec: int, provider_lag_sec: float) -> float:
    t0 = time.time()
    while ((int(time.time()) // bar_sec) * bar_sec - bar_sec) < target_ts:
        time.sleep(1)
    time.sleep(provider_lag_sec)
    return time.time() - t0


def active_temporal_observatory(target_ts: int, bar_sec: int, symbols: list[str], max_wait_sec: float) -> dict:
    t0 = time.time()
    crypto_anchors = [s for s in symbols if s in _BINANCE_SYMBOL_MAP and s not in FORCE_SKIP_SYMBOLS][:1]
    nse_anchors = [s for s in symbols if _to_nse_symbol(s) and s not in FORCE_SKIP_SYMBOLS][:1]
    anchors = crypto_anchors + nse_anchors

    if not anchors:
        waited = wait_for_barrier_target(target_ts, bar_sec, DEFAULT_PROVIDER_LAG_SEC)
        return {"exchange_to_observer_latency_ms": int(waited * 1000)}

    print(f"   🔭 [Phase A] Active Propagation Mode. Anchors: {', '.join(anchors)}")
    
    # Layer 1: Exchange Publication
    exchange_advanced = False
    exchange_publish_ts = None
    while not exchange_advanced:
        if time.time() - t0 > max_wait_sec:
            raise TimeoutError(f"Exchange anchors never reached {target_ts}")
        for sym in anchors:
            ts = _binance_latest_ts(sym, bar_sec) if sym in crypto_anchors else _nse_india_latest_ts(sym, bar_sec)
            if ts is not None and ts >= target_ts:
                exchange_publish_ts = time.time()
                exchange_advanced = True
                print(f"   📡 [Exchange] published {sym} at {exchange_publish_ts - t0:.1f}s")
                break
        if not exchange_advanced: time.sleep(1)

    # Layer 2: yfinance Visibility
    provider_advanced = False
    provider_visible_ts = None
    while not provider_advanced:
        if time.time() - t0 > max_wait_sec:
            raise TimeoutError(f"Provider anchors never reached {target_ts}")
        for sym in anchors:
            ts = _fetch_yfinance_anchor_ts(sym, bar_sec)
            if ts is not None and ts >= target_ts:
                provider_visible_ts = time.time()
                provider_advanced = True
                print(f"   📡 [Provider] yfinance published {sym} at {provider_visible_ts - t0:.1f}s")
                break
        if not provider_advanced: time.sleep(1)
            
    return {
        "provider_lag_ms": int((provider_visible_ts - exchange_publish_ts) * 1000),
        "exchange_to_observer_latency_ms": int((provider_visible_ts - target_ts) * 1000)
    }


def main():
    parser = argparse.ArgumentParser(description="Live session using canonical freeze+resume core engine.")
    parser.add_argument("--batch-id", type=int, default=3)
    parser.add_argument("--run-label", default="live")
    parser.add_argument("--cycles", type=int, default=3)
    parser.add_argument("--bar-sec", type=int, default=DEFAULT_BAR_SEC)
    parser.add_argument("--provider-lag-sec", type=int, default=DEFAULT_PROVIDER_LAG_SEC)
    parser.add_argument("--max-barrier-wait-sec", type=int, default=420)
    parser.add_argument("--temporal-observatory", action="store_true")
    args = parser.parse_args()

    cohort_file = Path(f"cohorts/batch_{args.batch_id:03d}.txt")
    archive_dir = resolve_archive_dir(args.batch_id, False, args.run_label)
    symbols = [s for s in cohort_file.read_text().splitlines() if s.strip() and s.strip() not in FORCE_SKIP_SYMBOLS]

    print("=" * 60)
    print("CHRONOSENTIMENT — LIVE SESSION (KERNEL ENGINE)")
    print("=" * 60)
    print(f"  Batch              : {args.batch_id:03d}")
    print(f"  Archive            : {archive_dir}")
    print(f"  Cycles             : {args.cycles}")
    print("=" * 60)

    last_target_ts = None

    for cycle in range(1, args.cycles + 1):
        if last_target_ts is not None:
            target_ts = last_target_ts + args.bar_sec
        else:
            # First cycle targets the currently closed bar
            target_ts = (int(time.time()) // args.bar_sec) * args.bar_sec - args.bar_sec

        print(f"\n[{datetime.now().strftime('%H:%M:%S')}] CYCLE {cycle}/{args.cycles} — Target TS: {target_ts}")
        
        # Phase 1: Temporal Propagation (Wait for bar close & provider visibility)
        obs_metrics = {}
        try:
            if args.temporal_observatory:
                obs_metrics = active_temporal_observatory(target_ts, args.bar_sec, symbols, float(args.max_barrier_wait_sec))
            else:
                waited = wait_for_barrier_target(target_ts, args.bar_sec, float(args.provider_lag_sec))
                obs_metrics = {"exchange_to_observer_latency_ms": int(waited * 1000)}
        except TimeoutError as e:
            print(f"❌ {e}")
            continue

        # Phase 2: Incremental Substrate Update (1d history to minimize bandwidth, preserves long-term history for PCA)
        print("   ❄️  [Phase B] Incrementally updating substrate (1d fetch)...")
        incremental_update_cohort(
            cohort_file=cohort_file,
            batch_id=args.batch_id,
            interval="5m",
            max_workers=15
        )

        # Phase 3: Canonical Replay (with dedupe to resume where we left off)
        print("   🚀 [Phase C] Executing canonical kernel (cs-ingest)...")
        ingest_stdout = run_frozen_via_cs_ingest(
            batch_id=args.batch_id,
            cohort_file=cohort_file,
            archive_dir=archive_dir,
            start_interval=0,
            max_intervals=None,
            fresh=False,
            resume=True,
            rebuild_dedupe=False,
        )

        # Phase 4: Chronology Ledger Append
        import re
        persisted = sum(int(x) for x in re.findall(r"persisted\s+(\d+)", ingest_stdout))
        dedupe_skip = sum(int(x) for x in re.findall(r"dedupe_skip\s+(\d+)", ingest_stdout))
        
        metadata_dir = archive_dir / "metadata"
        metadata_dir.mkdir(parents=True, exist_ok=True)
        ledger_path = metadata_dir / "live_session_steps.jsonl"
        
        ledger_entry = {
            "cycle": cycle,
            "barrier_ts": target_ts,
            "timeline_fingerprint": "pending_sha256",  # To be filled by replay engine
            "governor_state": "NOMINAL",               # Pending restoration of live governor
            "sync_ratio": 1.0,                         # Pending full cohort measurement 
            "dispersion": 1.0,                         # Pending full cohort measurement
            "provider_lag_ms": obs_metrics.get("provider_lag_ms", 0),
            "persisted": persisted,
            "dedupe_skip": dedupe_skip
        }
        
        with open(ledger_path, "a") as f:
            f.write(json.dumps(ledger_entry) + "\n")
        
        print(f"   📜 [Ledger] Appended canonical barrier step to {ledger_path.name}")

        last_target_ts = target_ts
        print(f"✅ Cycle {cycle} complete.")

if __name__ == "__main__":
    main()
