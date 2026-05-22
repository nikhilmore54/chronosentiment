import streamlit as st
import json
import pandas as pd
from pathlib import Path

st.set_page_config(page_title="Ecological Survivability Surface", layout="wide")

st.title("🔬 Ecological Survivability Surface")
st.markdown("""
This viewer projects the deterministic Execution-Qualified economic outcomes under topological stress.
**NOTE**: Governed by `SURFACE_INTERPRETATION_CONTRACT_v1`. Scalar compression, ranking, and optimization inference are strictly prohibited.
""")

artifact_path = Path("SURVIVABILITY_SURFACE_ARTIFACT_v1.json")

if not artifact_path.exists():
    st.error("Surface Artifact not found. Please compile the surface first.")
    st.stop()

with open(artifact_path, 'r') as f:
    artifact = json.load(f)

st.sidebar.header("Surface Substrate")
st.sidebar.code(f"Surface ID: {artifact['surface_id']}\nSubstrate: {artifact['substrate_id']}")

strategies = {s["strategy_id"]: s for s in artifact["strategies"]}
selected_strat = st.sidebar.selectbox("Select Strategy Cognition", list(strategies.keys()))

strat_data = strategies[selected_strat]
topologies = {t["topology_id"]: t for t in strat_data["topologies"]}
selected_topo = st.sidebar.selectbox("Select Topology Stress", list(topologies.keys()))

surface = topologies[selected_topo]

st.header(f"Projection: `{selected_strat}` × `{selected_topo}`")
st.code(f"Surface Hash: {surface['surface_hash']}")

col1, col2 = st.columns(2)

with col1:
    st.subheader("🌐 Topology Plane (Environmental Stress)")
    topo = surface["topology_plane"]
    st.json(topo)
    
    st.subheader("🧠 State Plane (Cognitive Integrity)")
    state = surface["state_plane"]
    st.json(state)

with col2:
    st.subheader("💵 Economic Plane (Realized Physics)")
    econ = surface["economic_plane"]
    st.json(econ)
    
    st.subheader("⏱️ Chronology Plane (Causal Truth)")
    chrono = surface["chronology_plane"]
    st.json(chrono)

st.markdown("---")
st.markdown("### Economic Asymmetry Visualization (Execution Realization)")
df_econ = pd.DataFrame([
    {"Type": "Canonical PnL", "Ticks": econ["canonical_pnl"]},
    {"Type": "Fragmented PnL", "Ticks": econ["fragmented_pnl"]}
])
st.bar_chart(df_econ.set_index("Type"))

st.info(f"**Economic Divergence**: {econ['economic_divergence']} ticks (Execution Non-Realization Rate: {econ['execution_non_realization_rate']*100}%)")
