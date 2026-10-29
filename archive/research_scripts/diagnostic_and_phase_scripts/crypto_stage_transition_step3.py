import os
import json
import numpy as np
import pandas as pd
from datetime import datetime, timedelta
import scipy.stats as stats
from sklearn.linear_model import LogisticRegression
from sklearn.metrics import roc_auc_score, log_loss

import sys
sys.path.append(os.path.abspath(os.path.join(os.path.dirname(__file__), '..')))
from apps.crypto_24h.observation_store import ObservationStore
from apps.crypto_24h.rolling_state_engine import RollingStateEngine
from scripts.e9_fetch_binance import fetch_binance_klines

DATA_DIR = os.path.join(os.path.dirname(__file__), '../datasets/e9/crypto')
REPORT_FILE = os.path.join(os.path.dirname(__file__), '../transition_step3_report.md')

def fetch_fresh_transition_data():
    # To be absolutely sure we have unseen data, we'll fetch from 90 days ago to 60 days ago
    now = datetime.now()
    end_dt = now - timedelta(days=60)
    start_dt = end_dt - timedelta(days=21) # 14 discovery, 7 validation
    
    end_ms = int(end_dt.timestamp() * 1000)
    start_ms = int(start_dt.timestamp() * 1000)
    
    print(f"Fetching fresh Transition dataset from {start_dt} to {end_dt}")
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
        
    return raw_data

def process_transition_data(raw_data):
    obs_store = ObservationStore()
    state_engine = RollingStateEngine()
    
    data_rows = []
    
    closes = np.array([float(d['close']) for d in raw_data])
    
    # We need to maintain a history of valid states to compute Delta(t-60)
    state_history = {} # timestamp -> (VA, P)
    
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
            
        w = obs_store.get_recent(1440)
        vol_last_30m = sum(c['volume'] for c in w[-30:])
        vol_prev_30m = sum(c['volume'] for c in w[-60:-30])
        va_60m = vol_last_30m / vol_prev_30m if vol_prev_30m > 0 else 1.0
        p_240m = state_snapshot.persistence_240m
        
        # Store state for future reference
        state_history[ts] = (va_60m, p_240m)
        
        # Check eligibility (t-60m must be valid)
        ts_minus_60m = ts - (60 * 60 * 1000)
        if ts_minus_60m not in state_history:
            continue
            
        # Check forward trajectory
        if i + 300 >= len(raw_data):
            continue
            
        t0_price = float(raw_obs['close'])
        if t0_price <= 0:
            continue
            
        f_closes = closes[i+1 : i+301]
        
        ret_15 = (f_closes[14] - t0_price) / t0_price
        ret_30 = (f_closes[29] - t0_price) / t0_price
        ret_60 = (f_closes[59] - t0_price) / t0_price
        ret_120 = (f_closes[119] - t0_price) / t0_price
        
        pb = int((ret_30 > ret_15) and (ret_60 > ret_30) and (ret_120 > ret_60))
        
        va_past, p_past = state_history[ts_minus_60m]
        
        data_rows.append({
            'timestamp': ts,
            'VA': va_60m,
            'P': p_240m,
            'dVA60': va_60m - va_past,
            'dP60': p_240m - p_past,
            'Persistent_Build': pb
        })
        
    df = pd.DataFrame(data_rows)
    return df

def run_step3_test():
    raw_data = fetch_fresh_transition_data()
    if not raw_data:
        print("No data fetched.")
        return
        
    df = process_transition_data(raw_data)
    print(f"Processed {len(df)} fully eligible transition observations.")
    
    # Split chronologically
    first_ts = df['timestamp'].iloc[0]
    split_ts = first_ts + 1209600000 # 14 days
    
    disc = df[df['timestamp'] < split_ts].copy()
    val = df[df['timestamp'] >= split_ts].copy()
    
    print(f"Discovery: {len(disc)} | Validation: {len(val)}")
    
    # Rank transform (percentiles) fit on Discovery
    for col in ['VA', 'P', 'dVA60', 'dP60']:
        disc[col + '_pct'] = disc[col].rank(pct=True)
        # Apply discovery percentiles to validation (approximate via interpolation)
        # For simplicity and robustness, we can just use scipy's percentileofscore
        val[col + '_pct'] = val[col].apply(lambda x: stats.percentileofscore(disc[col], x) / 100.0)
        
    features_S = ['VA_pct', 'P_pct']
    features_T = ['VA_pct', 'P_pct', 'dVA60_pct', 'dP60_pct']
    
    # Train Models
    model_S = LogisticRegression(penalty=None, solver='lbfgs') # No regularization to get pure LRT
    model_T = LogisticRegression(penalty=None, solver='lbfgs')
    
    X_S_disc = disc[features_S]
    X_T_disc = disc[features_T]
    y_disc = disc['Persistent_Build']
    
    model_S.fit(X_S_disc, y_disc)
    model_T.fit(X_T_disc, y_disc)
    
    # Discovery Evaluation
    pred_S_disc = model_S.predict_proba(X_S_disc)[:, 1]
    pred_T_disc = model_T.predict_proba(X_T_disc)[:, 1]
    
    ll_S_disc = -log_loss(y_disc, pred_S_disc, normalize=False)
    ll_T_disc = -log_loss(y_disc, pred_T_disc, normalize=False)
    
    # Likelihood Ratio Test
    lrt_stat = 2 * (ll_T_disc - ll_S_disc)
    p_value = stats.chi2.sf(lrt_stat, df=2)
    
    disc_pass = p_value < 0.01
    
    # Validation Evaluation
    X_S_val = val[features_S]
    X_T_val = val[features_T]
    y_val = val['Persistent_Build']
    
    pred_S_val = model_S.predict_proba(X_S_val)[:, 1]
    pred_T_val = model_T.predict_proba(X_T_val)[:, 1]
    
    ll_S_val = -log_loss(y_val, pred_S_val, normalize=False)
    ll_T_val = -log_loss(y_val, pred_T_val, normalize=False)
    
    auc_S_val = roc_auc_score(y_val, pred_S_val)
    auc_T_val = roc_auc_score(y_val, pred_T_val)
    
    val_pass = (ll_T_val > ll_S_val) and (auc_T_val > auc_S_val)
    
    final_pass = disc_pass and val_pass
    
    with open(REPORT_FILE, 'w') as f:
        f.write("# Transition-Dynamics Step 3: Statistical Test Results\n\n")
        f.write("Testing H0: The transition displacement provides no incremental information over the static state.\n\n")
        
        f.write("## 1. Discovery Gate (Likelihood Ratio Test)\n")
        f.write(f"- **Static Log-Likelihood ($LL_S$)**: {ll_S_disc:.2f}\n")
        f.write(f"- **Transition Log-Likelihood ($LL_T$)**: {ll_T_disc:.2f}\n")
        f.write(f"- **Test Statistic ($D$)**: {lrt_stat:.2f}\n")
        f.write(f"- **p-value**: {p_value:.6e}\n")
        f.write(f"- **Result**: {'PASS' if disc_pass else 'FAIL'} (Threshold p < 0.01)\n\n")
        
        f.write("## 2. Validation Gate (Out-Of-Sample Generalization)\n")
        f.write(f"- **Static $LL_S$**: {ll_S_val:.2f}\n")
        f.write(f"- **Transition $LL_T$**: {ll_T_val:.2f}\n")
        f.write(f"- **Static AUC**: {auc_S_val:.4f}\n")
        f.write(f"- **Transition AUC**: {auc_T_val:.4f}\n")
        f.write(f"- **Result**: {'PASS' if val_pass else 'FAIL'} (Requires $LL_T > LL_S$ AND $AUC_T > AUC_S$)\n\n")
        
        f.write("## Final Assessment\n")
        if final_pass:
            f.write("**Status: HYPOTHESIS SURVIVED**. The transition representation adds statistically significant, out-of-sample validatable information about Persistent Build beyond the static snapshot.")
        else:
            f.write("**Status: REJECTED**. The transition representation failed to demonstrate robust incremental information over the static snapshot. H0 is retained.")

if __name__ == "__main__":
    run_step3_test()
