import json
import numpy as np

print(f"{'Universe':<42} | {'Baseline':<12} | {'T0 Max':<8} | {'T1 Max':<8} | {'T0 Persist':<12} | {'T1 Persist':<12}")
print("-" * 102)

universes = [
    "2024_etf_approval_1h", 
    "2023_liquidation_cascade_1h", 
    "2023_binance_outage_1h", 
    "2023_christmas_drift_1h",
    "2026_intraday_impulse_shock_0730_0800_utc",
    "2026_multi_stage_cascade_transition",
    "2026_crossfeed_state_disagreement"
]

for universe in universes:
    for cognition in ["rolling_50", "event_reset"]:
        t0_path = f"core/artifacts/phase2b/{universe}/tier0_tick/{cognition}/trace_v1.json"
        t1_path = f"core/artifacts/phase2b/{universe}/tier1_1m/{cognition}/trace_v1.json"
        try:
            with open(t0_path) as f:
                t0_data = json.load(f)
                t0_traces = t0_data['traces']
                t0_intensities = [t['occupancy'] for t in t0_traces]
                t0_max = max(t0_intensities) if t0_intensities else 0
                threshold = np.mean(t0_intensities) if t0_intensities else 0
                t0_persist = sum(1 for x in t0_intensities if x > threshold)
                
            with open(t1_path) as f:
                t1_data = json.load(f)
                t1_traces = t1_data['traces']
                t1_intensities = [t['occupancy'] for t in t1_traces]
                t1_max = max(t1_intensities) if t1_intensities else 0
                t1_persist = sum(1 for x in t1_intensities if x > threshold)
                
            baseline = "Baseline A" if cognition == "rolling_50" else "Baseline B"
            print(f"{universe:<42} | {baseline:<12} | {t0_max:<8.2f} | {t1_max:<8.2f} | {t0_persist:<12} | {t1_persist:<12}")
        except Exception as e:
            pass
