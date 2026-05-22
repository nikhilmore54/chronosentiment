#!/usr/bin/env python3
"""
Provider Propagation Timing Study — Phase C precursor.

Samples yfinance and Stooq in parallel every N seconds and records
the exact wall-clock time each provider first publishes a new 5m bar.

Produces:
  - live console output showing first-observed timestamps per provider
  - state_archive/provider_propagation/{date}/report.json
    with per-bar provider synchronization:
      τ_yfinance, τ_stooq, Δτ (convergence lag), consensus_agreement

Usage:
  python3 scripts/probe_provider_propagation.py --symbols RELIANCE.NS HDFCBANK.NS TCS.NS --sample-interval 3 --duration 600
  python3 scripts/probe_provider_propagation.py --duration 1200  # full 20-min NSE open window
"""

from __future__ import annotations

import argparse
import csv
import io
import json
import time
import urllib.request
from concurrent.futures import ThreadPoolExecutor, as_completed
from datetime import datetime, timezone
from pathlib import Path

import yfinance as yf

# ── Constants ────────────────────────────────────────────────────────────────

DEFAULT_SYMBOLS = ["RELIANCE.NS", "HDFCBANK.NS", "TCS.NS", "INFY.NS", "SBIN.NS"]
DEFAULT_BAR_SEC = 300
DEFAULT_SAMPLE_INTERVAL = 3   # seconds between polls
DEFAULT_DURATION = 600        # total observation window in seconds
STOOQ_TIMEOUT = 5
YFINANCE_TIMEOUT = 10

# NSE → Stooq suffix mapping: RELIANCE.NS → RELIANCE.IN
def _to_stooq(symbol: str) -> str:
    if symbol.endswith(".NS"):
        return symbol[:-3] + ".IN"
    if symbol.endswith(".BO"):
        return symbol[:-3] + ".IN"  # BSE — Stooq uses .IN for both
    return symbol  # pass through for others

# ── Provider fetch functions ─────────────────────────────────────────────────

def _fetch_yfinance_ts(symbol: str, bar_sec: int) -> int | None:
    """Latest closed 5m bar open-time from yfinance (UTC seconds)."""
    try:
        interval_map = {60: "1m", 300: "5m", 900: "15m", 3600: "1h"}
        interval = interval_map.get(bar_sec, "5m")
        ticker = yf.Ticker(symbol)
        df = ticker.history(period="1d", interval=interval, auto_adjust=True)
        if df is None or df.empty:
            return None
        last_idx = df.index[-1]
        if hasattr(last_idx, "timestamp"):
            return int(last_idx.timestamp())
        return None
    except Exception:
        return None


def _fetch_stooq_ts(symbol: str, bar_sec: int) -> int | None:
    """Latest 5m bar open-time from Stooq CSV (UTC seconds)."""
    stooq_sym = _to_stooq(symbol)
    interval_map = {60: "1", 300: "5", 900: "15", 3600: "60"}
    interval = interval_map.get(bar_sec, "5")
    url = f"https://stooq.com/q/d/l/?s={stooq_sym}&i={interval}"
    try:
        req = urllib.request.Request(url, headers={"User-Agent": "Mozilla/5.0"})
        with urllib.request.urlopen(req, timeout=STOOQ_TIMEOUT) as resp:
            content = resp.read().decode("utf-8", errors="replace")
        reader = csv.DictReader(io.StringIO(content))
        rows = list(reader)
        if not rows:
            return None
        last = rows[-1]
        date_str = last.get("Date", "")
        time_str = last.get("Time", "")
        if not date_str:
            return None
        if time_str:
            dt_str = f"{date_str} {time_str}"
            fmt = "%Y-%m-%d %H:%M:%S"
        else:
            dt_str = date_str
            fmt = "%Y-%m-%d"
        # Stooq returns IST — convert to UTC
        from datetime import timedelta
        IST = timezone(timedelta(hours=5, minutes=30))
        dt = datetime.strptime(dt_str, fmt).replace(tzinfo=IST)
        return int(dt.timestamp())
    except Exception:
        return None


# ── Core sampling loop ───────────────────────────────────────────────────────

def run_propagation_study(
    symbols: list[str],
    bar_sec: int,
    sample_interval: float,
    duration: float,
    output_dir: Path,
) -> dict:
    """
    Polls both providers every `sample_interval` seconds for `duration` seconds.
    Records first-observed wall-clock time for each new bar per provider.
    """
    IST_offset = 5 * 3600 + 30 * 60

    # State: {symbol: {provider: {bar_ts: first_observed_wall_clock}}}
    first_seen: dict[str, dict[str, dict[int, float]]] = {
        sym: {"yfinance": {}, "stooq": {}} for sym in symbols
    }

    t_start = time.time()
    t_end = t_start + duration
    samples_taken = 0
    bar_advances: list[dict] = []  # recorded events

    print(f"  Symbols  : {', '.join(symbols)}")
    print(f"  Interval : {sample_interval}s | Duration: {duration}s")
    print(f"  Started  : {datetime.now().strftime('%H:%M:%S')}")
    print("─" * 80)
    print(f"  {'Time':^8} | {'Symbol':^16} | {'Provider':^10} | {'Bar TS':^12} | {'First Seen':^10}")
    print("─" * 80)

    while time.time() < t_end:
        wall_now = time.time()
        samples_taken += 1

        with ThreadPoolExecutor(max_workers=len(symbols) * 2) as pool:
            yf_futures = {
                pool.submit(_fetch_yfinance_ts, sym, bar_sec): ("yfinance", sym)
                for sym in symbols
            }
            st_futures = {
                pool.submit(_fetch_stooq_ts, sym, bar_sec): ("stooq", sym)
                for sym in symbols
            }
            all_futures = {**yf_futures, **st_futures}

            for future in as_completed(all_futures):
                provider, sym = all_futures[future]
                try:
                    ts = future.result()
                except Exception:
                    ts = None

                if ts is None:
                    continue

                seen = first_seen[sym][provider]
                if ts not in seen:
                    # New bar observed — record
                    seen[ts] = wall_now
                    ist_time = datetime.fromtimestamp(wall_now + IST_offset, tz=timezone.utc)
                    bar_ist = datetime.fromtimestamp(ts + IST_offset, tz=timezone.utc)
                    print(
                        f"  {ist_time.strftime('%H:%M:%S'):^8} | {sym:^16} | "
                        f"{provider:^10} | {bar_ist.strftime('%H:%M'):^12} | ✅ FIRST"
                    )
                    bar_advances.append({
                        "symbol": sym,
                        "provider": provider,
                        "bar_ts": ts,
                        "bar_ist": bar_ist.strftime("%Y-%m-%d %H:%M:%S"),
                        "first_observed_wall": wall_now,
                        "first_observed_ist": ist_time.isoformat(),
                    })

        # Sleep remainder of interval
        elapsed = time.time() - wall_now
        sleep_rem = max(0.0, sample_interval - elapsed)
        if sleep_rem > 0:
            time.sleep(sleep_rem)

    # ── Compute convergence metrics per bar ──────────────────────────────────
    bar_stats: dict = {}
    for sym in symbols:
        for bar_ts, yf_t in first_seen[sym]["yfinance"].items():
            st_t = first_seen[sym]["stooq"].get(bar_ts)
            bar_stats.setdefault(sym, {})[bar_ts] = {
                "bar_ts": bar_ts,
                "tau_yfinance": yf_t - t_start,        # seconds from study start
                "tau_stooq": st_t - t_start if st_t else None,
                "delta_tau": (st_t - yf_t) if st_t else None,  # + means stooq slower
                "consensus": st_t is not None,
            }

    report = {
        "schema_version": 1,
        "study_start_utc": datetime.fromtimestamp(t_start, tz=timezone.utc).isoformat(),
        "study_end_utc": datetime.fromtimestamp(time.time(), tz=timezone.utc).isoformat(),
        "symbols": symbols,
        "bar_sec": bar_sec,
        "sample_interval_sec": sample_interval,
        "duration_sec": duration,
        "samples_taken": samples_taken,
        "bar_advance_events": len(bar_advances),
        "per_bar_convergence": bar_stats,
        "raw_events": bar_advances,
    }

    # ── Print convergence summary ────────────────────────────────────────────
    print("\n" + "=" * 80)
    print("  PROVIDER PROPAGATION CONVERGENCE SUMMARY")
    print("=" * 80)
    for sym in symbols:
        for bar_ts, metrics in bar_stats.get(sym, {}).items():
            ist_bar = datetime.fromtimestamp(bar_ts + IST_offset, tz=timezone.utc)
            tau_yf = f"{metrics['tau_yfinance']:.1f}s" if metrics['tau_yfinance'] is not None else "—"
            tau_st = f"{metrics['tau_stooq']:.1f}s" if metrics['tau_stooq'] is not None else "NOT SEEN"
            delta = f"{metrics['delta_tau']:+.1f}s" if metrics['delta_tau'] is not None else "—"
            consensus = "✅" if metrics['consensus'] else "❌"
            print(
                f"  {sym:>18} | Bar {ist_bar.strftime('%H:%M')} | "
                f"τ_yf={tau_yf:>8} | τ_stooq={tau_st:>10} | Δτ={delta:>7} | {consensus}"
            )
    print("=" * 80)

    return report


# ── Entry point ──────────────────────────────────────────────────────────────

def main():
    parser = argparse.ArgumentParser(description="Provider propagation timing study")
    parser.add_argument(
        "--symbols", nargs="+", default=DEFAULT_SYMBOLS,
        help="NSE symbols to probe (e.g. RELIANCE.NS HDFCBANK.NS)"
    )
    parser.add_argument("--bar-sec", type=int, default=DEFAULT_BAR_SEC)
    parser.add_argument(
        "--sample-interval", type=float, default=DEFAULT_SAMPLE_INTERVAL,
        help="Seconds between provider polls (default: 3)"
    )
    parser.add_argument(
        "--duration", type=float, default=DEFAULT_DURATION,
        help="Total observation window in seconds (default: 600 = 10 min)"
    )
    args = parser.parse_args()

    output_dir = Path("state_archive/provider_propagation") / datetime.now().strftime("%Y-%m-%d")
    output_dir.mkdir(parents=True, exist_ok=True)

    print("=" * 80)
    print("  CHRONOSENTIMENT — PROVIDER PROPAGATION TIMING STUDY")
    print("=" * 80)

    report = run_propagation_study(
        symbols=args.symbols,
        bar_sec=args.bar_sec,
        sample_interval=args.sample_interval,
        duration=args.duration,
        output_dir=output_dir,
    )

    report_path = output_dir / f"report_{datetime.now().strftime('%H%M%S')}.json"
    with open(report_path, "w") as f:
        json.dump(report, f, indent=4)

    print(f"\n💾 Propagation report saved to {report_path}")
    print(f"   Bar advance events: {report['bar_advance_events']}")
    print(f"   Samples taken     : {report['samples_taken']}")


if __name__ == "__main__":
    main()
