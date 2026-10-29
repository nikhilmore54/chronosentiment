import os
import json
import numpy as np
import pandas as pd
from datetime import datetime, timedelta

import sys
sys.path.append(os.path.abspath(os.path.join(os.path.dirname(__file__), '..')))
from apps.crypto_24h.observation_store import ObservationStore
from apps.crypto_24h.rolling_state_engine import RollingStateEngine
from scripts.e9_fetch_binance import fetch_binance_klines

DATA_DIR = os.path.join(os.path.dirname(__file__), '../datasets/e9/crypto')

def generate_fresh_moga_datasets():
    # 60 days ago to 30 days ago (completely unseen)
    now = datetime.now()
    end_dt = now - timedelta(days=30)
    start_dt = end_dt - timedelta(days=21) # 21 days total (14 discovery, 7 validation)
    
    end_ms = int(end_dt.timestamp() * 1000)
    start_ms = int(start_dt.timestamp() * 1000)
    
    print(f"Fetching fresh MOGA dataset from {start_dt} to {end_dt}")
    klines = fetch_binance_klines("BTCUSDT", "1m", start_ms, end_ms)
    
    raw_data = []
    for d in klines:
        raw_data.append({
            "timestamp": int(d[0]),
            "open": float(d[1]),
            "high": float(d[2]),
            "low": float(d[3]),
            "close": float(d[4]),
            "volume": float(d[5])
        })
        
    print(f"Loaded {len(raw_data)} raw observations.")
    
    obs_store = ObservationStore()
    state_engine = RollingStateEngine()
    
    data_rows = []
    
    closes = np.array([float(d['close']) for d in raw_data])
    
    for i, raw_obs in enumerate(raw_data):
        ts = int(raw_obs['timestamp'])
        
        obs_store.add(
            timestamp=ts,
            open_px=float(raw_obs['open']),
            high_px=float(raw_obs['high']),
            low_px=float(raw_obs['low']),
            close_px=float(raw_obs['close']),
            volume=float(raw_obs['volume'])
        )
        state_snapshot = state_engine.update(obs_store)
        
        if not state_snapshot:
            continue
            
        if i + 300 >= len(raw_data):
            continue
            
        w = obs_store.get_recent(1440)
        
        vol_last_30m = sum(c['volume'] for c in w[-30:])
        vol_prev_30m = sum(c['volume'] for c in w[-60:-30])
        volume_acceleration_60m = vol_last_30m / vol_prev_30m if vol_prev_30m > 0 else 1.0
        
        t0_price = float(raw_obs['close'])
        if t0_price <= 0:
            continue
            
        f_closes = closes[i+1 : i+301]
        
        ret_15 = (f_closes[14] - t0_price) / t0_price
        ret_30 = (f_closes[29] - t0_price) / t0_price
        ret_60 = (f_closes[59] - t0_price) / t0_price
        ret_120 = (f_closes[119] - t0_price) / t0_price
        
        pb = int((ret_30 > ret_15) and (ret_60 > ret_30) and (ret_120 > ret_60))
        
        data_rows.append({
            'timestamp': ts,
            'volume_acceleration_60m': volume_acceleration_60m,
            'persistence_240m': state_snapshot.persistence_240m,
            'Persistent_Build': pb
        })
        
    df = pd.DataFrame(data_rows)
    print(f"Processed {len(df)} eligible T0 trajectories.")
    
    # Split into Discovery (first 14 days) and Validation (last 7 days)
    # The dataframe timestamps are in ms. 14 days = 14 * 24 * 60 * 60 * 1000 = 1209600000 ms
    first_ts = df['timestamp'].iloc[0]
    split_ts = first_ts + 1209600000
    
    discovery_df = df[df['timestamp'] < split_ts].copy()
    validation_df = df[df['timestamp'] >= split_ts].copy()
    
    print(f"Discovery Set: {len(discovery_df)} observations")
    print(f"Validation Set: {len(validation_df)} observations")
    
    discovery_path = os.path.join(DATA_DIR, "moga_discovery.csv")
    validation_path = os.path.join(DATA_DIR, "moga_validation.csv")
    
    discovery_df.to_csv(discovery_path, index=False)
    validation_df.to_csv(validation_path, index=False)
    
    print("MOGA datasets created successfully.")

if __name__ == "__main__":
    generate_fresh_moga_datasets()
