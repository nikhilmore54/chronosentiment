import json
from datetime import datetime
from pathlib import Path
from typing import Any

import pandas as pd
import streamlit as st

REGISTRY_PATH = "data/experiments.jsonl"

st.set_page_config(page_title="ChronoSentiment Demo", layout="wide")


@st.cache_data(ttl=5)
def load_registry(path: str) -> list[dict[str, Any]]:
    records: list[dict[str, Any]] = []
    registry = Path(path)
    if not registry.exists():
        return records

    with registry.open("r", encoding="utf-8") as f:
        for line in f:
            line = line.strip()
            if not line:
                continue
            try:
                records.append(json.loads(line))
            except json.JSONDecodeError:
                continue

    return records


def latest_per_hypothesis(records: list[dict[str, Any]]) -> list[dict[str, Any]]:
    latest: dict[str, dict[str, Any]] = {}

    for record in records:
        hypothesis_id = record.get("hypothesis_id")
        ts_raw = record.get("timestamp")
        if not hypothesis_id or not isinstance(ts_raw, str) or not ts_raw:
            continue

        try:
            ts = datetime.fromisoformat(ts_raw.replace("Z", "+00:00"))
        except ValueError:
            continue

        prev = latest.get(hypothesis_id)
        if prev is None or ts > prev["_ts"]:
            row = dict(record)
            row["_ts"] = ts
            latest[hypothesis_id] = row

    return list(latest.values())


def to_dataframe(records: list[dict[str, Any]]) -> pd.DataFrame:
    rows: list[dict[str, Any]] = []
    for record in records:
        summary = record.get("batch_summary", {})
        rows.append(
            {
                "hypothesis_id": record.get("hypothesis_id"),
                "decision": record.get("decision"),
                "confidence": record.get("confidence"),
                "state": record.get("state", "active"),
                "avg_pnl_delta": summary.get("avg_delta_avg_pnl", 0.0),
                "hit_rate_delta": summary.get("avg_delta_hit_rate", 0.0),
                "drawdown_delta": summary.get("avg_delta_max_dd", 0.0),
                "retained_pct": summary.get("avg_retained_pct", 0.0),
                "positive_ratio": summary.get("positive_ratio", 0.0),
            }
        )
    return pd.DataFrame(rows)


records = load_registry(REGISTRY_PATH)
latest = latest_per_hypothesis(records)
df = to_dataframe(latest)

if df.empty:
    st.warning("No experiment data found.")
    st.stop()

st.sidebar.header("Filters")
state_filter = st.sidebar.multiselect(
    "State",
    options=["active", "validated", "archived"],
    default=["active", "validated"],
)
decision_filter = st.sidebar.multiselect(
    "Decision",
    options=["PROMOTE", "HOLD", "REJECT"],
    default=["PROMOTE", "HOLD", "REJECT"],
)
min_retention = st.sidebar.slider("Min Retention %", 0, 100, 0)
min_positive_ratio = st.sidebar.slider("Min Positive Ratio", 0.0, 1.0, 0.0)

df_filtered = df[
    (df["state"].isin(state_filter))
    & (df["decision"].isin(decision_filter))
    & (df["retained_pct"] >= min_retention)
    & (df["positive_ratio"] >= min_positive_ratio)
]

df_filtered = df_filtered.sort_values(
    by=["avg_pnl_delta", "hit_rate_delta", "drawdown_delta"],
    ascending=[False, False, True],
)

st.title("ChronoSentiment - Decision Validation Engine")
st.markdown("**We don't predict trades. We prevent bad decisions.**")

col1, col2, col3, col4 = st.columns(4)
if df_filtered.empty:
    avg_pnl_delta = 0.0
    hit_rate_delta = 0.0
    drawdown_delta = 0.0
    retention_pct = 0.0
else:
    avg_pnl_delta = float(df_filtered["avg_pnl_delta"].mean())
    hit_rate_delta = float(df_filtered["hit_rate_delta"].mean())
    drawdown_delta = float(df_filtered["drawdown_delta"].mean())
    retention_pct = float(df_filtered["retained_pct"].mean())

col1.metric("Avg PnL Delta", f"{avg_pnl_delta:+.5f}")
col2.metric("Hit Rate Delta", f"{hit_rate_delta:+.2f}")
col3.metric("Drawdown Delta", f"{drawdown_delta:+.5f}")
col4.metric("Retention %", f"{retention_pct:.2f}")

st.subheader("Hypothesis Leaderboard")
if df_filtered.empty:
    st.info("No records match current filters. Broaden filters to view hypotheses.")
st.dataframe(df_filtered, use_container_width=True, height=500)

if not df_filtered.empty:
    best = df_filtered.iloc[0]
    st.subheader("Top Hypothesis")
    st.markdown(
        f"""
**Hypothesis:** `{best['hypothesis_id']}`  
**Decision:** {best['decision']}  
**Confidence:** {best['confidence']}  
**State:** {best['state']}

**Impact:**
- Avg PnL Delta: `{best['avg_pnl_delta']:+.5f}`
- Hit Rate Delta: `{best['hit_rate_delta']:+.2f}`
- Drawdown Delta: `{best['drawdown_delta']:+.5f}`
- Retention: `{best['retained_pct']:.2f}%`
"""
    )

st.caption("Deterministic - Read-only - No strategy mutation")
