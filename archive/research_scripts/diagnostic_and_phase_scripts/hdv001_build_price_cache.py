#!/usr/bin/env python3
"""
HDV-001-B — Raw Price Surface + Integrity
==========================================

Builds `datasets/hdv001/hdv001_price_cache_v1/` — the canonical OHLCV cache
for the 52-instrument NSE universe used in Historical Decision Validation v1.

**This script does NOT compute MAE, MFE, or any outcome.**
Its sole responsibility is to acquire, validate, and hash a trustworthy
historical price surface. MAE/MFE computation belongs to HDV-001-C/D.

Session rule (from HDV_001_PERIODS.md §8, §15):
    The first eligible evaluation bar for a decision is the first NSE trading
    session whose open is strictly after decision_time (IST). This means:
        decision_date = decision_time.astimezone('Asia/Kolkata').date()
        first_eval_bar_date > decision_date
    This prevents look-ahead from the same-session bar.

Coverage required:
    2026-07-14 → 2026-08-27
    (2026-07-14 = earliest decision; 2026-08-27 = 10 sessions after 2026-08-13)

Data source: Yahoo Finance (yfinance), auto_adjust=True, interval='1d'
"""

import hashlib
import json
import os
import sys
import time
from datetime import date, datetime, timezone
from pathlib import Path
from typing import Any

import pandas as pd
import yfinance as yf

# ─── Constants ────────────────────────────────────────────────────────────────

CACHE_VERSION = "hdv001_price_cache_v1"
CACHE_ROOT = Path("datasets/hdv001") / CACHE_VERSION
MANIFEST_PATH = Path("datasets/hdv001/cache_manifest.json")
HASH_PATH = Path("datasets/hdv001/cache_hash.txt")
REPORT_PATH = Path("datasets/hdv001/HDV_001_B_DATA_REPORT.md")

# Coverage window: earliest decision → today (2026-08-17).
# The 10-session observation window for the last decision (2026-08-13) extends
# to approximately 2026-08-27, but those bars do not yet exist. The cache is
# built with data available today and will be refreshed as sessions complete.
# REQUIRED_END is set to the last available trading day (2026-08-13) so that
# the integrity check does not flag future dates as missing.
FETCH_START  = "2026-07-01"   # buffer before earliest decision
FETCH_END    = "2026-08-18"   # today + 1 day (yfinance end is exclusive)

# Required coverage: earliest decision → last decision date.
# Future bars (2026-08-14 onward) are not yet available and must not be required.
REQUIRED_START = date(2026, 7, 14)
REQUIRED_END   = date(2026, 8, 13)

# NSE universe — exactly the 52 instruments in stop_research_dataset_v01.json.
# Derived from: sorted(set(r['instrument'] for r in records))
# Do NOT modify this list — it is the frozen HDV-001 universe.
NSE_UNIVERSE = [
    "ADANIENT.NS", "ADANIPORTS.NS", "ASIANPAINT.NS", "AXISBANK.NS",
    "BAJAJFINSV.NS", "BAJFINANCE.NS", "BHARTIARTL.NS", "BPCL.NS",
    "CIPLA.NS", "COALINDIA.NS", "DIVISLAB.NS", "DRREDDY.NS",
    "EICHERMOT.NS", "GRASIM.NS", "HCLTECH.NS", "HDFCBANK.NS",
    "HDFCLIFE.NS", "HEROMOTOCO.NS", "HINDALCO.NS", "HINDUNILVR.NS",
    "ICICIBANK.NS", "IDEA.NS", "INDUSINDBK.NS", "INFY.NS",
    "ITC.NS", "JSWSTEEL.NS", "KOTAKBANK.NS", "LT.NS",
    "M&M.NS", "MAHABANK.NS", "MARUTI.NS", "NESTLEIND.NS",
    "NTPC.NS", "ONGC.NS", "PIDILITIND.NS", "POWERGRID.NS",
    "RELIANCE.NS", "SBILIFE.NS", "SBIN.NS", "SHREECEM.NS",
    "SUNPHARMA.NS", "TATACONSUM.NS", "TATASTEEL.NS", "TCS.NS",
    "TECHM.NS", "TITAN.NS", "TMPV.NS", "ULTRACEMCO.NS",
    "UNITDSPR.NS", "UPL.NS", "VEDL.NS", "WIPRO.NS",
]

# ─── Helpers ──────────────────────────────────────────────────────────────────

def sha256_file(path: Path) -> str:
    h = hashlib.sha256()
    with open(path, "rb") as f:
        for chunk in iter(lambda: f.read(65536), b""):
            h.update(chunk)
    return h.hexdigest()


def sha256_string(s: str) -> str:
    return hashlib.sha256(s.encode()).hexdigest()


def is_nse_trading_day(d: date) -> bool:
    """
    Returns True if `d` is a weekday (Mon–Fri).
    NSE holiday calendar for 2026 is not embedded here; missing bars are
    detected and reported rather than pre-filtered.
    """
    return d.weekday() < 5  # 0=Mon, 4=Fri


def expected_trading_days(start: date, end: date) -> list[date]:
    """Return all weekdays in [start, end] inclusive."""
    days = []
    d = start
    while d <= end:
        if is_nse_trading_day(d):
            days.append(d)
        d = date.fromordinal(d.toordinal() + 1)
    return days


# ─── Fetch ────────────────────────────────────────────────────────────────────

def fetch_instrument(symbol: str) -> tuple[pd.DataFrame | None, str]:
    """
    Fetch daily OHLCV for `symbol` from Yahoo Finance.
    Returns (DataFrame, error_message). DataFrame is None on failure.
    """
    try:
        ticker = yf.Ticker(symbol)
        df = ticker.history(
            start=FETCH_START,
            end=FETCH_END,
            interval="1d",
            auto_adjust=True,
            actions=True,   # include dividends/splits for corporate-action check
        )
        if df.empty:
            return None, "empty response from yfinance"
        # Normalize index to date (remove time component).
        df.index = pd.to_datetime(df.index).normalize().date
        df.index.name = "date"
        # Keep only required columns.
        cols = [c for c in ["Open", "High", "Low", "Close", "Volume", "Dividends", "Stock Splits"] if c in df.columns]
        df = df[cols].copy()
        df.columns = [c.lower().replace(" ", "_") for c in df.columns]
        return df, ""
    except Exception as e:
        return None, str(e)


# ─── Integrity checks ─────────────────────────────────────────────────────────

def check_coverage(df: pd.DataFrame, symbol: str) -> dict[str, Any]:
    """
    Verify that the DataFrame covers the required date range and has no
    unexpected gaps. Returns a dict of findings.
    """
    available_dates = set(df.index)
    expected = expected_trading_days(REQUIRED_START, REQUIRED_END)

    missing = [d for d in expected if d not in available_dates]
    extra   = [d for d in available_dates if d < REQUIRED_START or d > REQUIRED_END]

    # Duplicate index check.
    duplicates = [d for d in available_dates if list(df.index).count(d) > 1]

    # NaN check in OHLC.
    nan_rows = int(df[["open", "high", "low", "close"]].isnull().any(axis=1).sum())

    # Corporate-action events in the required window.
    ca_events = []
    if "dividends" in df.columns:
        divs = df.loc[
            (df.index >= REQUIRED_START) & (df.index <= REQUIRED_END),
            "dividends"
        ]
        ca_events += [
            {"date": str(d), "type": "dividend", "value": float(v)}
            for d, v in divs.items() if v > 0
        ]
    if "stock_splits" in df.columns:
        splits = df.loc[
            (df.index >= REQUIRED_START) & (df.index <= REQUIRED_END),
            "stock_splits"
        ]
        ca_events += [
            {"date": str(d), "type": "split", "value": float(v)}
            for d, v in splits.items() if v not in (0, 1)
        ]

    return {
        "symbol": symbol,
        "first_available_bar": str(min(available_dates)) if available_dates else None,
        "last_available_bar": str(max(available_dates)) if available_dates else None,
        "bars_in_required_window": len([d for d in available_dates if REQUIRED_START <= d <= REQUIRED_END]),
        "expected_trading_days": len(expected),
        "missing_sessions": [str(d) for d in missing],
        "missing_session_count": len(missing),
        "duplicate_sessions": [str(d) for d in duplicates],
        "nan_ohlc_rows": nan_rows,
        "corporate_action_events": ca_events,
        "corporate_action_count": len(ca_events),
        "adjustment_status": "auto_adjust=True (yfinance back-adjusted)",
        "coverage_ok": len(missing) == 0 and len(duplicates) == 0 and nan_rows == 0,
    }


# ─── Persist ──────────────────────────────────────────────────────────────────

def save_instrument(symbol: str, df: pd.DataFrame) -> Path:
    """Save OHLCV to JSON. Returns the file path."""
    CACHE_ROOT.mkdir(parents=True, exist_ok=True)
    safe_name = symbol.replace(".", "_").replace("&", "AND").replace("-", "_")
    path = CACHE_ROOT / f"{safe_name}.json"
    records = []
    for d, row in df.iterrows():
        rec = {"date": str(d)}
        for col in df.columns:
            v = row[col]
            rec[col] = None if pd.isna(v) else float(v)
        records.append(rec)
    with open(path, "w") as f:
        json.dump({"symbol": symbol, "source": "yfinance", "auto_adjust": True, "bars": records}, f, indent=2)
    return path


# ─── Main ─────────────────────────────────────────────────────────────────────

def main() -> None:
    print(f"HDV-001-B: Building {CACHE_VERSION}")
    print(f"  Universe: {len(NSE_UNIVERSE)} instruments")
    print(f"  Fetch window: {FETCH_START} → {FETCH_END}")
    print(f"  Required coverage: {REQUIRED_START} → {REQUIRED_END}")
    print()

    CACHE_ROOT.mkdir(parents=True, exist_ok=True)

    manifest_entries: list[dict] = []
    retrieval_ts = datetime.now(timezone.utc).isoformat()

    ok_count = 0
    fail_count = 0
    warn_count = 0

    for i, symbol in enumerate(NSE_UNIVERSE, 1):
        print(f"  [{i:02d}/{len(NSE_UNIVERSE)}] {symbol} ... ", end="", flush=True)

        df, err = fetch_instrument(symbol)
        if df is None:
            print(f"FAIL — {err}")
            manifest_entries.append({
                "symbol": symbol,
                "status": "FAILED",
                "error": err,
                "retrieval_timestamp": retrieval_ts,
            })
            fail_count += 1
            continue

        integrity = check_coverage(df, symbol)
        path = save_instrument(symbol, df)
        file_hash = sha256_file(path)

        entry = {
            **integrity,
            "file": str(path.relative_to(Path("datasets/hdv001"))),
            "file_hash": file_hash,
            "source": "yfinance",
            "auto_adjust": True,
            "retrieval_timestamp": retrieval_ts,
            "schema_version": "hdv001_price_cache_v1",
            "status": "OK" if integrity["coverage_ok"] else "WARN",
        }
        manifest_entries.append(entry)

        if integrity["coverage_ok"]:
            print(f"OK  ({integrity['bars_in_required_window']} bars, {integrity['corporate_action_count']} CA events)")
            ok_count += 1
        else:
            issues = []
            if integrity["missing_session_count"]:
                issues.append(f"{integrity['missing_session_count']} missing sessions")
            if integrity["duplicate_sessions"]:
                issues.append(f"{len(integrity['duplicate_sessions'])} duplicates")
            if integrity["nan_ohlc_rows"]:
                issues.append(f"{integrity['nan_ohlc_rows']} NaN rows")
            print(f"WARN — {'; '.join(issues)}")
            warn_count += 1

        # Rate-limit to avoid Yahoo Finance throttling.
        time.sleep(0.5)

    # ── Write manifest ────────────────────────────────────────────────────────
    manifest = {
        "cache_version": CACHE_VERSION,
        "built_at": retrieval_ts,
        "fetch_start": FETCH_START,
        "fetch_end": FETCH_END,
        "required_start": str(REQUIRED_START),
        "required_end": str(REQUIRED_END),
        "universe_size": len(NSE_UNIVERSE),
        "ok_count": ok_count,
        "warn_count": warn_count,
        "fail_count": fail_count,
        "instruments": manifest_entries,
    }
    with open(MANIFEST_PATH, "w") as f:
        json.dump(manifest, f, indent=2)

    # ── Compute cache hash (hash of all instrument file hashes, sorted) ───────
    instrument_hashes = sorted(
        e["file_hash"] for e in manifest_entries if "file_hash" in e
    )
    cache_hash = sha256_string("\n".join(instrument_hashes))
    with open(HASH_PATH, "w") as f:
        f.write(f"{cache_hash}\n")

    # ── Write data report ─────────────────────────────────────────────────────
    write_report(manifest, cache_hash)

    print()
    print(f"Results: {ok_count} OK, {warn_count} WARN, {fail_count} FAIL")
    print(f"Cache hash: {cache_hash}")
    print(f"Manifest:   {MANIFEST_PATH}")
    print(f"Report:     {REPORT_PATH}")

    if fail_count > 0:
        print(f"\nWARNING: {fail_count} instruments failed. Review {REPORT_PATH}.")
        sys.exit(1)
    if warn_count > 0:
        print(f"\nWARNING: {warn_count} instruments have coverage gaps. Review {REPORT_PATH}.")
        # Do not exit 1 — warnings are expected for NSE holidays.


def write_report(manifest: dict, cache_hash: str) -> None:
    instruments = manifest["instruments"]
    ok = [e for e in instruments if e.get("status") == "OK"]
    warn = [e for e in instruments if e.get("status") == "WARN"]
    fail = [e for e in instruments if e.get("status") == "FAILED"]

    total_ca = sum(e.get("corporate_action_count", 0) for e in instruments)
    ca_instruments = [e for e in instruments if e.get("corporate_action_count", 0) > 0]

    lines = [
        "# HDV-001-B Data Report",
        f"",
        f"**Cache version:** `{manifest['cache_version']}`  ",
        f"**Built at:** {manifest['built_at']}  ",
        f"**Cache hash:** `{cache_hash}`  ",
        f"",
        "---",
        "",
        "## Coverage Summary",
        "",
        f"| Dimension | Value |",
        f"|-----------|-------|",
        f"| Universe size | {manifest['universe_size']} |",
        f"| OK | {manifest['ok_count']} |",
        f"| WARN (coverage gaps) | {manifest['warn_count']} |",
        f"| FAILED | {manifest['fail_count']} |",
        f"| Required window | {manifest['required_start']} → {manifest['required_end']} |",
        f"| Fetch window | {manifest['fetch_start']} → {manifest['fetch_end']} |",
        f"| Corporate-action events | {total_ca} across {len(ca_instruments)} instruments |",
        "",
        "---",
        "",
        "## Instrument Coverage",
        "",
        "| Symbol | Status | Bars | Missing Sessions | CA Events |",
        "|--------|--------|------|-----------------|-----------|",
    ]

    for e in sorted(instruments, key=lambda x: x["symbol"]):
        status = e.get("status", "FAILED")
        bars = e.get("bars_in_required_window", 0)
        missing = e.get("missing_session_count", "—")
        ca = e.get("corporate_action_count", "—")
        lines.append(f"| {e['symbol']} | {status} | {bars} | {missing} | {ca} |")

    if warn:
        lines += [
            "",
            "---",
            "",
            "## Coverage Warnings",
            "",
            "Missing sessions are expected for NSE holidays. Verify against the",
            "NSE 2026 holiday calendar before declaring HDV-001-G.",
            "",
        ]
        for e in warn:
            lines.append(f"### {e['symbol']}")
            lines.append(f"Missing sessions: {e.get('missing_sessions', [])}")
            lines.append("")

    if ca_instruments:
        lines += [
            "",
            "---",
            "",
            "## Corporate-Action Events",
            "",
            "These events occurred within the required coverage window.",
            "Verify that yfinance back-adjustment correctly handles each event.",
            "",
        ]
        for e in ca_instruments:
            lines.append(f"### {e['symbol']}")
            for ev in e.get("corporate_action_events", []):
                lines.append(f"- {ev['date']}: {ev['type']} = {ev['value']}")
            lines.append("")

    if fail:
        lines += [
            "",
            "---",
            "",
            "## Failed Instruments",
            "",
        ]
        for e in fail:
            lines.append(f"- **{e['symbol']}**: {e.get('error', 'unknown error')}")

    lines += [
        "",
        "---",
        "",
        "## Integrity Assertions",
        "",
        f"- [ ] {manifest['ok_count'] + manifest['warn_count']}/{manifest['universe_size']} instruments retrieved",
        f"- [ ] All missing sessions verified against NSE 2026 holiday calendar",
        f"- [ ] 0 duplicate sessions",
        f"- [ ] 0 NaN OHLC rows",
        f"- [ ] Corporate-action back-adjustment spot-checked (≥ 5 instruments)",
        f"- [ ] Cache hash recorded: `{cache_hash}`",
        "",
        "**HDV-001-G prerequisite:** All assertions above must be checked before",
        "the freeze gate is declared.",
        "",
        "---",
        "",
        "*This report is generated by `scripts/hdv001_build_price_cache.py`.*",
        "*Do not edit manually.*",
    ]

    with open(REPORT_PATH, "w") as f:
        f.write("\n".join(lines) + "\n")


if __name__ == "__main__":
    main()