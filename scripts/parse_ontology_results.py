import json
import os

print(f"{'Universe':<42} | {'Baseline':<12} | {'T0 Max':<8} | {'T1 Max':<8} | {'T0 Persist':<12} | {'T1 Persist':<12}")
print("-" * 102)

universes = [
    "2026_recent_crossfeed_1h_ontology",
    "2026_recent_discontinuity_1h_ontology"
]

for universe in universes:
    for cognition in ["rolling_50", "event_reset"]:
        t0_path = f"core/artifacts/phase2c/{universe}/tier0_tick/{cognition}/trace_summary.json"
        t1_path = f"core/artifacts/phase2c/{universe}/tier1_1m/{cognition}/trace_summary.json"
        try:
            with open(t0_path) as f:
                t0_data = json.load(f)
                t0_max = t0_data['max']
                t0_persist = t0_data['persistence']
                
            with open(t1_path) as f:
                t1_data = json.load(f)
                t1_max = t1_data['max']
                t1_persist = t1_data['persistence']
                
            baseline = "Baseline A" if cognition == "rolling_50" else "Baseline B"
            print(f"{universe:<42} | {baseline:<12} | {t0_max:<8.2f} | {t1_max:<8.2f} | {t0_persist:<12} | {t1_persist:<12}")
        except Exception as e:
            print(f"Error for {universe} {cognition}: {e}")
