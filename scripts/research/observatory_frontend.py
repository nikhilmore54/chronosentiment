import streamlit as st
import pandas as pd
import numpy as np
import os

st.set_page_config(page_title="ChronoSentiment Observatory", layout="wide")

ARCHIVE_PATH = "archive/physics_divergence.csv"

st.title("🔭 ChronoSentiment: Execution Physics Observatory")
st.markdown("### Phase 2B.6: Statistical Hardening & Controlled Ablation")
st.markdown("*(The architecture no longer assumes its own validity. It competes for existence against null baselines.)*")

if not os.path.exists(ARCHIVE_PATH):
    st.warning(f"Archive not found at {ARCHIVE_PATH}. Waiting for data accumulation...")
    st.stop()

# --- DATA INGESTION ---
cols = ["timestamp", "symbol", "regime", "vol_bucket", "half_life", 
        "legacy_exp", "gross_move", "noise_floor", "micro_exp", "divergence"]
try:
    df = pd.read_csv(ARCHIVE_PATH, names=cols, header=None)
except Exception as e:
    st.error(f"Error reading archive: {e}")
    st.stop()

df = df[df["regime"].isin(["MeanReversion", "HighVolatilityNoise", "DirectionalTrend", "Breakout", "Unknown"])]
df = df.sort_values(by=["symbol", "timestamp"]).copy()

# Date conversions
df['datetime'] = pd.to_datetime(df['timestamp'], unit='s')

# --- EPOCH GATE CONDITIONS (DIVERSITY) ---
st.header("1. Epoch Gate Conditions (Diversity Accumulation)")

col1, col2, col3, col4 = st.columns(4)
col1.metric("Evaluated Ticks (Target: 1000+)", f"{len(df)}")
unique_regimes = df['regime'].nunique()
col2.metric("Regimes Captured", f"{unique_regimes}")
unique_assets = df['symbol'].nunique()
col3.metric("Assets Monitored", f"{unique_assets}")

# Determine if we can proceed to Phase 2B.7
if len(df) < 1000:
    st.info("⏳ **Status:** Accumulating reality. Epoch 2B.7 is locked until minimum 1,000 ticks.")
else:
    st.success("✅ **Status:** Archive size meets Phase 2B.7 threshold. Awaiting regime/shock diversity checks.")

# --- THE NULL HYPOTHESIS (MODEL B) ---
st.header("2. The Null Hypothesis vs. Provisional Abstractions")

if len(df) < 50:
    st.warning("Insufficient data for ablation calculations.")
    st.stop()

# Calculate Derivatives
df['hostility_accel'] = df.groupby('symbol')['divergence'].diff(periods=2).fillna(0)
df['noise_ratio'] = np.where(df['gross_move'] > 0, df['noise_floor'] / df['gross_move'], 0)

COLLAPSE_THRESHOLD = 0.06
df['is_collapsed'] = df['divergence'] >= COLLAPSE_THRESHOLD
df['future_collapse'] = df.groupby('symbol')['is_collapsed'].shift(-1) | \
                        df.groupby('symbol')['is_collapsed'].shift(-2) | \
                        df.groupby('symbol')['is_collapsed'].shift(-3)

valid_df = df.dropna(subset=['future_collapse']).copy()
valid_df['future_collapse'] = valid_df['future_collapse'].astype(bool)

baseline_exposure = valid_df['divergence'].sum()

# Model B (Null)
size_B = np.where(valid_df['divergence'] > 0.05, 0.0, 1.0)
exposure_B = (valid_df['divergence'] * size_B).sum()
survival_gain_B = 1.0 - (exposure_B / baseline_exposure) if baseline_exposure > 0 else 0

# Model C (Derivatives)
size_C = np.where((valid_df['divergence'] > 0.05) | (valid_df['hostility_accel'] > 0.02), 0.0, 1.0)
exposure_C = (valid_df['divergence'] * size_C).sum()
survival_gain_C = 1.0 - (exposure_C / baseline_exposure) if baseline_exposure > 0 else 0

mcol1, mcol2, mcol3 = st.columns(3)
mcol1.metric("Baseline Exposure", f"{baseline_exposure:.4f}")
mcol2.metric("Model B (Null) Survival Gain", f"{survival_gain_B:+.1%}")
mcol3.metric("Model C (Deriv) Survival Gain", f"{survival_gain_C:+.1%}", delta=f"{survival_gain_C - survival_gain_B:+.1%} Incremental Lift")

# False positive vs True positive
fp_B = len(valid_df[(size_B == 0.0) & (valid_df['future_collapse'] == False)])
fp_C = len(valid_df[(size_C == 0.0) & (valid_df['future_collapse'] == False)])

tp_B = len(valid_df[(size_B == 0.0) & (valid_df['future_collapse'] == True)])
tp_C = len(valid_df[(size_C == 0.0) & (valid_df['future_collapse'] == True)])

st.markdown(f"**Predictive Precision:** Model C dodged **{tp_C}** true collapses (vs Model B's {tp_B}) while suffering **{fp_C}** false suppressions (vs Model B's {fp_B}).")

# --- VISUALIZING THE REALITY STREAM ---
st.header("3. Divergence Physics Stream")

sym_filter = st.selectbox("Select Asset to Visualize", df['symbol'].unique())
asset_df = df[df['symbol'] == sym_filter].copy()

if not asset_df.empty:
    st.line_chart(asset_df.set_index('datetime')[['divergence', 'micro_exp', 'noise_floor']])
    
    st.markdown("### Hostility Acceleration (Model C's Core Metric)")
    st.line_chart(asset_df.set_index('datetime')['hostility_accel'])

st.markdown("---")
st.caption("ChronoSentiment Phase 2B.6 | Standing by for Epoch 2B.7 triggers.")
