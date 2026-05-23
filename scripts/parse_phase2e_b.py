import json

print(f"{'Universe':<38} | {'Baseline':<12} | {'T1 Max':<8} | {'T1 Persist':<12}")
print("-" * 75)

universes = [
    "2026_aapl_open_auction",
    "2026_reliance_open_auction"
]

for universe in universes:
    for cognition in ["rolling_50", "event_reset"]:
        t1_path = f"core/artifacts/phase2e_b/{universe}/tier1_1m/{cognition}/trace_summary.json"
        try:
            with open(t1_path) as f:
                t1_data = json.load(f)
                t1_max = t1_data['max']
                t1_persist = t1_data['persistence']
                
            baseline = "Baseline A" if cognition == "rolling_50" else "Baseline B"
            print(f"{universe:<38} | {baseline:<12} | {t1_max:<8.2f} | {t1_persist:<12}")
        except Exception as e:
            print(f"Error for {universe} {cognition}: {e}")
