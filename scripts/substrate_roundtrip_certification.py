#!/usr/bin/env python3
import argparse
import json
import logging
from pathlib import Path
import pandas as pd
import numpy as np

# Adjust path to import local scripts
import sys
sys.path.append(str(Path(__file__).parent.parent))

from scripts.csv_to_replay_substrate import process_csv
from scripts.session_metrics import (
    load_canonical, ohlc, net_return_pct, realized_volatility, trend_strength
)

logging.basicConfig(level=logging.INFO, format="%(levelname)s: %(message)s")

def reconstruct_dataframe(jsonl_path: Path) -> pd.DataFrame:
    """Read a tick-level JSONL substrate and reconstruct 1-minute OHLC candles."""
    ticks = []
    with open(jsonl_path, "r", encoding="utf-8") as f:
        for line in f:
            if not line.strip():
                continue
            ticks.append(json.loads(line))
            
    df_ticks = pd.DataFrame(ticks)
    # timestamp is in ms, convert to datetime
    df_ticks["timestamp"] = pd.to_datetime(df_ticks["timestamp"], unit="ms", utc=True)
    
    # Group by 1-minute intervals (the base timestamps were at minute boundaries)
    # The 4 ticks are at 0, 15s, 30s, 45s.
    # Resample to 1Min
    df_ticks = df_ticks.set_index("timestamp")
    
    # Reconstruct OHLC
    df_ohlc = df_ticks["price"].resample("1Min").agg(
        open="first",
        high="max",
        low="min",
        close="last"
    )
    
    # Reconstruct Volume
    df_vol = df_ticks["volume"].resample("1Min").sum()
    df_ohlc["volume"] = df_vol
    
    # Drop NaNs (minutes with no data)
    df_ohlc = df_ohlc.dropna()
    
    # Reset index to match canonical CSV format
    df_ohlc = df_ohlc.reset_index()
    
    # Ensure types match canonical
    df_ohlc["open"] = df_ohlc["open"].astype(float)
    df_ohlc["high"] = df_ohlc["high"].astype(float)
    df_ohlc["low"] = df_ohlc["low"].astype(float)
    df_ohlc["close"] = df_ohlc["close"].astype(float)
    
    return df_ohlc

def certify_roundtrip(csv_path: Path, output_dir: Path):
    """Run the complete certification check on a single CSV."""
    logging.info(f"--- Certifying {csv_path.name} ---")
    
    # 1. Load Original CSV and Compute Metrics
    df_orig = load_canonical(csv_path)
    orig_ohlc = ohlc(df_orig)
    orig_range_pct = (orig_ohlc["high"] - orig_ohlc["low"]) / orig_ohlc["open"] * 100.0
    orig_net_ret = net_return_pct(orig_ohlc["open"], orig_ohlc["close"])
    orig_vol = realized_volatility(df_orig)
    orig_trend = trend_strength(df_orig, orig_ohlc["open"], orig_ohlc["close"])

    # 2. Convert to Synthetic Substrate
    jsonl_path = process_csv(csv_path, output_dir)
    
    # 3. Reconstruct from JSONL
    df_recon = reconstruct_dataframe(jsonl_path)
    recon_ohlc = ohlc(df_recon)
    recon_range_pct = (recon_ohlc["high"] - recon_ohlc["low"]) / recon_ohlc["open"] * 100.0
    recon_net_ret = net_return_pct(recon_ohlc["open"], recon_ohlc["close"])
    recon_vol = realized_volatility(df_recon)
    recon_trend = trend_strength(df_recon, recon_ohlc["open"], recon_ohlc["close"])

    # 4. Verify Exact Matches
    try:
        assert np.isclose(orig_ohlc["open"], recon_ohlc["open"]), f"Open mismatch: {orig_ohlc['open']} != {recon_ohlc['open']}"
        assert np.isclose(orig_ohlc["high"], recon_ohlc["high"]), f"High mismatch: {orig_ohlc['high']} != {recon_ohlc['high']}"
        assert np.isclose(orig_ohlc["low"], recon_ohlc["low"]), f"Low mismatch: {orig_ohlc['low']} != {recon_ohlc['low']}"
        assert np.isclose(orig_ohlc["close"], recon_ohlc["close"]), f"Close mismatch: {orig_ohlc['close']} != {recon_ohlc['close']}"
        assert np.isclose(orig_range_pct, recon_range_pct), f"Range % mismatch: {orig_range_pct} != {recon_range_pct}"
        assert np.isclose(orig_net_ret, recon_net_ret), f"Net Return % mismatch: {orig_net_ret} != {recon_net_ret}"
        assert np.isclose(orig_vol, recon_vol), f"Realized Volatility mismatch: {orig_vol} != {recon_vol}"
        assert np.isclose(orig_trend, recon_trend), f"Trend Strength mismatch: {orig_trend} != {recon_trend}"
        
        # Check DataFrame shapes
        assert len(df_orig) == len(df_recon), f"Length mismatch: {len(df_orig)} != {len(df_recon)}"
        
        logging.info("✅ ROUNDTRIP CERTIFICATION PASSED: Substrate flawlessly preserves ecological geometry.")
        return True
    except AssertionError as e:
        logging.error(f"❌ ROUNDTRIP CERTIFICATION FAILED: {e}")
        return False

def main():
    parser = argparse.ArgumentParser(description="Certify that CSV->Substrate->OHLC reconstruction is mathematically lossless.")
    parser.add_argument("--csv", required=True, help="Path to a canonical 1m CSV to test")
    parser.add_argument("--tmp-dir", default="/tmp/substrate_test", help="Temporary directory for the JSONL")
    
    args = parser.parse_args()
    csv_path = Path(args.csv)
    tmp_dir = Path(args.tmp_dir)
    tmp_dir.mkdir(parents=True, exist_ok=True)
    
    if not csv_path.exists():
        logging.error(f"CSV not found: {csv_path}")
        return
        
    success = certify_roundtrip(csv_path, tmp_dir)
    if not success:
        sys.exit(1)

if __name__ == "__main__":
    main()
