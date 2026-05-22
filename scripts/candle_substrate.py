#!/usr/bin/env python3
"""
Immutable OHLC substrate for deterministic cohort replay.

Layout:
  state_archive/candles/batch_NNN/
    manifest.json
    symbols/<SYMBOL>.jsonl.gz   # one JSON object per bar, sorted by ts
"""

from __future__ import annotations

import gzip
import hashlib
import json
import time
from datetime import datetime, timezone
from pathlib import Path

import pandas as pd
import yfinance as yf

CANDLE_ROOT = Path("state_archive/candles")


def frozen_batch_dir(batch_id: int, root: Path = CANDLE_ROOT) -> Path:
    return root / f"batch_{batch_id:03d}"


def symbol_path(batch_dir: Path, symbol: str) -> Path:
    safe = symbol.replace("/", "_")
    return batch_dir / "symbols" / f"{safe}.jsonl.gz"


def df_to_records(df: pd.DataFrame) -> list[dict]:
    records = []
    for ts, row in df.iterrows():
        if isinstance(row, pd.DataFrame):
            row = row.iloc[0]

        def get_val(k: str) -> float:
            raw = row.get(k, row.get(k.lower(), 0.0))
            if hasattr(raw, "iloc"):
                raw = raw.iloc[0]
            return float(raw)

        records.append(
            {
                "ts": int(pd.Timestamp(ts).timestamp()),
                "open": get_val("Open"),
                "high": get_val("High"),
                "low": get_val("Low"),
                "close": get_val("Close"),
                "volume": get_val("Volume"),
            }
        )
    records.sort(key=lambda r: r["ts"])
    return records


def write_symbol_candles(path: Path, records: list[dict]) -> None:
    path.parent.mkdir(parents=True, exist_ok=True)
    with gzip.open(path, "wt", encoding="utf-8") as f:
        for rec in records:
            f.write(json.dumps(rec, sort_keys=True) + "\n")


def read_symbol_candles(path: Path) -> pd.DataFrame:
    from archive_dedupe import iter_gzip_jsonl

    rows = []
    for _ln, rec, err in iter_gzip_jsonl(path):
        if err or rec is None:
            continue
        rows.append(rec)
    if not rows:
        return pd.DataFrame()
    df = pd.DataFrame(rows)
    df["datetime"] = pd.to_datetime(df["ts"], unit="s", utc=True)
    df = df.set_index("datetime").sort_index()
    df = df.rename(
        columns={
            "open": "Open",
            "high": "High",
            "low": "Low",
            "close": "Close",
            "volume": "Volume",
        }
    )
    return df[["Open", "High", "Low", "Close", "Volume"]]


def download_ticker(
    symbol: str, interval: str = "5m", period: str = "5d"
) -> pd.DataFrame:
    df, _stderr = download_ticker_with_stderr(symbol, interval, period)
    return df


def download_ticker_with_stderr(
    symbol: str, interval: str = "5m", period: str = "5d"
) -> tuple[pd.DataFrame, str]:
    """Download OHLC; return (dataframe, captured stderr) for symbol health classification."""
    import contextlib
    import io

    stderr_all = io.StringIO()
    for _attempt in range(3):
        buf = io.StringIO()
        try:
            with contextlib.redirect_stderr(buf):
                df = yf.download(
                    tickers=symbol,
                    period=period,
                    interval=interval,
                    auto_adjust=True,
                    progress=False,
                    threads=False,
                )
            stderr_all.write(buf.getvalue())
            if df is not None and not df.empty:
                if isinstance(df.columns, pd.MultiIndex):
                    df.columns = list(df.columns.get_level_values(0))
                else:
                    df.columns = list(df.columns)
                df = df.dropna()
                df = df[~df.index.duplicated(keep="first")]
                return df.sort_index(), stderr_all.getvalue()
        except BaseException as e:
            stderr_all.write(f"{buf.getvalue()}\n{type(e).__name__}: {e}\n")
            time.sleep(1.0)
    return pd.DataFrame(), stderr_all.getvalue()


def build_timeline_fingerprint(timestamps: list[int]) -> str:
    return hashlib.sha256(",".join(str(t) for t in sorted(timestamps)).encode()).hexdigest()[:16]


def compute_substrate_hash(batch_dir: Path, symbols: list[str]) -> str:
    h = hashlib.sha256()
    for sym in sorted(symbols):
        path = symbol_path(batch_dir, sym)
        if path.exists():
            h.update(path.read_bytes())
    return h.hexdigest()[:16]


def load_frozen_cohort(
    batch_id: int,
    cohort_symbols: list[str],
    root: Path = CANDLE_ROOT,
) -> tuple[dict[str, pd.DataFrame], dict]:
    batch_dir = frozen_batch_dir(batch_id, root)
    manifest_path = batch_dir / "manifest.json"
    if not manifest_path.exists():
        raise FileNotFoundError(
            f"No frozen substrate at {batch_dir}. Run: python3 scripts/freeze_cohort_candles.py --batch-id {batch_id}"
        )
    with open(manifest_path) as f:
        manifest = json.load(f)

    data: dict[str, pd.DataFrame] = {}
    for sym in cohort_symbols:
        path = symbol_path(batch_dir, sym)
        if not path.exists():
            continue
        df = read_symbol_candles(path)
        if not df.empty:
            data[sym] = df

    return data, manifest


def freeze_cohort(
    cohort_file: Path,
    batch_id: int,
    interval: str = "5m",
    period: str = "5d",
    max_workers: int = 15,
    root: Path = CANDLE_ROOT,
) -> Path:
    import concurrent.futures

    symbols = [line.strip() for line in cohort_file.read_text().splitlines() if line.strip()]
    batch_dir = frozen_batch_dir(batch_id, root)
    sym_dir = batch_dir / "symbols"
    if sym_dir.exists():
        import shutil

        shutil.rmtree(sym_dir)
    sym_dir.mkdir(parents=True, exist_ok=True)

    print(f"📥 Freezing {len(symbols)} symbols → {batch_dir}")

    all_ts: set[int] = set()
    frozen_count = 0
    total_bars = 0

    def _freeze_one(sym: str) -> tuple[str, int, list[int]]:
        df = download_ticker(sym, interval, period)
        if df.empty:
            return sym, 0, []
        recs = df_to_records(df)
        write_symbol_candles(symbol_path(batch_dir, sym), recs)
        ts_list = [r["ts"] for r in recs]
        return sym, len(recs), ts_list

    with concurrent.futures.ThreadPoolExecutor(max_workers=max_workers) as ex:
        futures = {ex.submit(_freeze_one, s): s for s in symbols}
        done = 0
        for fut in concurrent.futures.as_completed(futures):
            done += 1
            sym, n_bars, ts_list = fut.result()
            if n_bars > 0:
                frozen_count += 1
                total_bars += n_bars
                all_ts.update(ts_list)
            if done % 50 == 0 or done == len(symbols):
                print(f"   Frozen {done}/{len(symbols)} (with data: {frozen_count})...")

    sorted_ts = sorted(all_ts)
    manifest = {
        "batch_id": batch_id,
        "cohort_file": str(cohort_file),
        "interval": interval,
        "period": period,
        "symbols_cohort": len(symbols),
        "symbols_frozen": frozen_count,
        "total_bars": total_bars,
        "timeline_intervals": len(sorted_ts),
        "timeline_fingerprint": build_timeline_fingerprint(sorted_ts),
        "timeline_first_ts": sorted_ts[0] if sorted_ts else None,
        "timeline_last_ts": sorted_ts[-1] if sorted_ts else None,
        "substrate_hash": compute_substrate_hash(batch_dir, symbols),
        "frozen_at_utc": datetime.now(timezone.utc).isoformat(),
    }
    manifest_path = batch_dir / "manifest.json"
    with open(manifest_path, "w") as f:
        json.dump(manifest, f, indent=2)
    print(f"✅ Frozen substrate: {manifest_path}")
    print(f"   Timeline fingerprint: {manifest['timeline_fingerprint']}")
    print(f"   Bars: {total_bars:,} | Intervals: {len(sorted_ts)} | Hash: {manifest['substrate_hash']}")
    return manifest_path


def incremental_update_cohort(
    cohort_file: Path,
    batch_id: int,
    interval: str = "5m",
    max_workers: int = 15,
    root: Path = CANDLE_ROOT,
) -> Path:
    import concurrent.futures

    symbols = [line.strip() for line in cohort_file.read_text().splitlines() if line.strip()]
    batch_dir = frozen_batch_dir(batch_id, root)
    sym_dir = batch_dir / "symbols"
    if not sym_dir.exists():
        raise FileNotFoundError("Cannot incrementally update: Substrate does not exist.")

    print(f"📥 Incrementally updating {len(symbols)} symbols → {batch_dir}")

    all_ts: set[int] = set()
    frozen_count = 0
    total_bars = 0
    symbol_latest_ts = {}

    def _update_one(sym: str) -> tuple[str, int, list[int]]:
        # Fetch just 1d to capture the latest bars with minimal bandwidth
        df_new = download_ticker(sym, interval, period="1d")
        
        path = symbol_path(batch_dir, sym)
        if path.exists():
            df_old = read_symbol_candles(path)
            if not df_old.empty and not df_new.empty:
                df = pd.concat([df_old, df_new])
                df = df[~df.index.duplicated(keep="last")].sort_index()
            elif not df_old.empty:
                df = df_old
            else:
                df = df_new
        else:
            df = df_new

        if df.empty:
            return sym, 0, []
            
        recs = df_to_records(df)
        write_symbol_candles(path, recs)
        ts_list = [r["ts"] for r in recs]
        return sym, len(recs), ts_list

    with concurrent.futures.ThreadPoolExecutor(max_workers=max_workers) as ex:
        futures = {ex.submit(_update_one, s): s for s in symbols}
        done = 0
        for fut in concurrent.futures.as_completed(futures):
            done += 1
            try:
                sym, n_bars, ts_list = fut.result()
                if n_bars > 0:
                    frozen_count += 1
                    total_bars += n_bars
                    all_ts.update(ts_list)
                    symbol_latest_ts[sym] = max(ts_list) if ts_list else 0
            except Exception as e:
                print(f"   ❌ [Update Error] {futures[fut]}: {e}")
            if done % 50 == 0 or done == len(symbols):
                print(f"   Updated {done}/{len(symbols)} (with data: {frozen_count})...", flush=True)

    sorted_ts = sorted(all_ts)
    manifest = {
        "batch_id": batch_id,
        "cohort_file": str(cohort_file),
        "interval": interval,
        "period": "incremental",
        "symbols_cohort": len(symbols),
        "symbols_frozen": frozen_count,
        "total_bars": total_bars,
        "timeline_intervals": len(sorted_ts),
        "timeline_fingerprint": build_timeline_fingerprint(sorted_ts),
        "timeline_first_ts": sorted_ts[0] if sorted_ts else None,
        "timeline_last_ts": sorted_ts[-1] if sorted_ts else None,
        "substrate_hash": compute_substrate_hash(batch_dir, symbols),
        "frozen_at_utc": datetime.now(timezone.utc).isoformat(),
    }
    manifest_path = batch_dir / "manifest.json"
    with open(manifest_path, "w") as f:
        json.dump(manifest, f, indent=2)
    print(f"✅ Incremental substrate updated: {manifest_path}")
    print(f"   Timeline fingerprint: {manifest['timeline_fingerprint']}")
    return manifest_path, symbol_latest_ts
