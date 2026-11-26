import streamlit as st
import pandas as pd
import json
import os
import altair as alt

st.set_page_config(page_title="Temporal Observability Dashboard", layout="wide")
st.title("🔭 ChronoSentiment: Temporal Observability")
st.markdown("*(Visualizing canonical execution admissibility and market synchronization topology)*")

DEFAULT_LOG = "state_archive/batches/batch_9904/runs/live/metadata/live_session_steps.jsonl"
log_path = st.text_input("Ledger Path", DEFAULT_LOG)

if not os.path.exists(log_path):
    st.warning(f"Ledger not found at {log_path}. Waiting for data accumulation...")
    st.stop()

data = []
with open(log_path, 'r') as f:
    for line in f:
        if line.strip():
            try:
                row = json.loads(line)
                flat_row = {
                    "cycle": row.get("cycle"),
                    "barrier_ts": row.get("barrier_ts"),
                    "timeline_fingerprint": row.get("timeline_fingerprint"),
                    "symbols_attempted": row.get("symbols_attempted", 0),
                    "symbols_returned": row.get("symbols_returned", 0),
                    "symbols_accepted": row.get("symbols_accepted", 0),
                    "median_symbol_lag_sec": row.get("freshness", {}).get("median_symbol_lag_sec", 0),
                    "p90_lag_sec": row.get("freshness", {}).get("p90_lag_sec", 0),
                    "lag_stddev": row.get("freshness", {}).get("lag_stddev", 0.0),
                    "strict_ratio": row.get("observability", {}).get("strict_ratio", 0.0),
                    "acceptance_ratio": row.get("observability", {}).get("acceptance_ratio", 0.0),
                    "regime_state": row.get("observability", {}).get("regime_state", "UNKNOWN"),
                    "execution_admissible": row.get("admissibility", {}).get("execution_admissible", False),
                    "new_entries_allowed": row.get("admissibility", {}).get("new_entries_allowed", False),
                    "exits_allowed": row.get("admissibility", {}).get("exits_allowed", True),
                    "policy_version": row.get("admissibility", {}).get("classification_policy_version", "v1.0"),
                }
                data.append(flat_row)
            except Exception as e:
                pass

if not data:
    st.warning("No data found in ledger.")
    st.stop()

df = pd.DataFrame(data)
df['datetime'] = pd.to_datetime(df['barrier_ts'], unit='s')

st.header("1. Execution Admissibility Governor")
latest = df.iloc[-1]
m1, m2, m3, m4 = st.columns(4)
m1.metric("Current Regime", latest['regime_state'])
m2.metric("New Entries Allowed", str(latest['new_entries_allowed']))
m3.metric("Exits Allowed", str(latest['exits_allowed']))
m4.metric("Policy Version", str(latest['policy_version']))

st.header("2. Regime Timeline (Market Observability Tape)")
# Use Altair to plot a Gantt-like timeline or simple points
c = alt.Chart(df).mark_circle(size=100).encode(
    x=alt.X('datetime:T', title='Barrier Time'),
    y=alt.Y('regime_state:N', title='Regime'),
    color=alt.Color('regime_state:N', legend=None),
    tooltip=['cycle', 'datetime', 'regime_state', 'execution_admissible', 'timeline_fingerprint']
).properties(height=200)
st.altair_chart(c, use_container_width=True)

st.dataframe(df[['cycle', 'datetime', 'regime_state', 'execution_admissible', 'new_entries_allowed', 'timeline_fingerprint']], use_container_width=True)

st.header("3. Acceptance vs Strict Ratio")
st.markdown("Visually distinguishes synchronization vs fragmented but usable states.")
st.line_chart(df.set_index('datetime')[['strict_ratio', 'acceptance_ratio']])

st.header("4. Lag Distribution & Variance")
st.markdown("Monitoring the propagation wave width and synchronization variance.")
col1, col2 = st.columns(2)
with col1:
    st.line_chart(df.set_index('datetime')[['median_symbol_lag_sec', 'p90_lag_sec']])
with col2:
    st.line_chart(df.set_index('datetime')[['lag_stddev']])

st.header("5. Fingerprint Transition Overlay")
st.markdown("Validates chronology stability under observability turbulence.")
fingerprints = df[['datetime', 'timeline_fingerprint', 'regime_state']].drop_duplicates(subset=['timeline_fingerprint'])
st.dataframe(fingerprints, use_container_width=True)
