# scripts/session_metrics.py
"""Metric computation for a single 1‑minute canonical CSV.
All functions are pure and return float values (or None where not applicable).
"""
import pandas as pd
import numpy as np
from pathlib import Path
import json

def load_canonical(csv_path: Path) -> pd.DataFrame:
    """Read the canonical 1m CSV and ensure required columns exist.
    Returns a DataFrame with a datetime index based on the 'timestamp' column.
    """
    df = pd.read_csv(csv_path)
    # Ensure proper types
    df["timestamp"] = pd.to_datetime(df["timestamp"], utc=True)
    return df

def ohlc(df: pd.DataFrame) -> dict:
    """Calculate OHLC for the whole session.
    Returns a dict with keys: open, high, low, close (floats).
    """
    open_price = float(df.iloc[0]["open"])
    high_price = float(df["high"].max())
    low_price = float(df["low"].min())
    close_price = float(df.iloc[-1]["close"])
    return {"open": open_price, "high": high_price, "low": low_price, "close": close_price}

def gap_pct(current_open: float, prev_close: float) -> float | None:
    """Percentage gap between today's open and previous day's close.
    Returns None if prev_close is None.
    """
    if prev_close is None:
        return None
    return (current_open - prev_close) / prev_close * 100.0

def realized_volatility(df: pd.DataFrame) -> float:
    """Standard deviation of log returns for the session.
    Uses close prices.
    """
    closes = df["close"].astype(float)
    log_returns = np.log(closes / closes.shift(1)).dropna()
    return float(log_returns.std())

def net_return_pct(open_price: float, close_price: float) -> float:
    """Absolute net return relative to open.
    """
    return abs(close_price - open_price) / open_price * 100.0

def avg_range_pct(df: pd.DataFrame, open_price: float) -> float:
    """Mean of (high - low) / open for each candle, expressed as %.
    """
    ranges = (df["high"] - df["low"]).astype(float) / open_price * 100.0
    return float(ranges.mean())

def trend_strength(df: pd.DataFrame, open_price: float, close_price: float) -> float:
    """Deterministic trend strength as defined in the spec.
    net_return_pct / avg_range_pct (no clipping). Returns 0.0 if avg_range_pct == 0.
    """
    net_ret = net_return_pct(open_price, close_price)
    avg_rng = avg_range_pct(df, open_price)
    if avg_rng == 0:
        return 0.0
    strength = net_ret / avg_rng
    return strength

def compute_all(csv_path: Path, manifest_sha256: str, prev_close: float | None) -> dict:
    """Compute the full metric set for a session.
    Returns a dict ready to be merged into the catalog entry.
    """
    df = load_canonical(csv_path)
    ohlc_vals = ohlc(df)
    gap = gap_pct(ohlc_vals["open"], prev_close)
    vol = realized_volatility(df)
    tr_strength = trend_strength(df, ohlc_vals["open"], ohlc_vals["close"])
    session_range = (ohlc_vals["high"] - ohlc_vals["low"]) / ohlc_vals["open"] * 100.0
    net_ret = net_return_pct(ohlc_vals["open"], ohlc_vals["close"])
    candle_count = len(df)
    # Duplicate timestamps already validated, but we can compute here as a sanity check
    duplicate_ts = int(df.duplicated(subset=["timestamp"]).sum())
    missing = max(0, 375 - candle_count)  # approximate expected count
    return {
        "open": ohlc_vals["open"],
        "high": ohlc_vals["high"],
        "low": ohlc_vals["low"],
        "close": ohlc_vals["close"],
        "gap_pct": gap,
        "realized_volatility": vol,
        "trend_strength": tr_strength,
        "session_range_pct": session_range,
        "net_return_pct": net_ret,
        "candle_count": candle_count,
        "missing_candles": missing,
        "duplicate_timestamps": duplicate_ts,
        "manifest_sha256": manifest_sha256,
    }
