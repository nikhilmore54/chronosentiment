#!/usr/bin/env python3
import json

with open("live_capture/shadow_validation/rp606_rp607_accumulating_evidence_snapshot.json") as f:
    data = json.load(f)

records = data.get("rp607_records", [])
complete = [r for r in records if r.get("horizon_complete")]
complete_sorted = sorted(complete, key=lambda x: x.get("t_condition") or x.get("t_exit"))

split_idx = len(complete_sorted) // 2
disc = complete_sorted[:split_idx]
val = complete_sorted[split_idx:]

disc_tickers = set(r.get("symbol") for r in disc)
val_tickers = set(r.get("symbol") for r in val)
all_tickers = set(r.get("symbol") for r in complete)

print(f"Total Complete Exits (N={len(complete)}): {len(all_tickers)} Distinct Ticker Clusters")
print(f"Stage 1 Discovery Cohort (N={len(disc)}): {len(disc_tickers)} Distinct Ticker Clusters")
print(f"Stage 2 Temporal Validation Cohort (N={len(val)}): {len(val_tickers)} Distinct Ticker Clusters")

# Ticker distribution top 10
from collections import Counter
disc_counts = Counter(r.get("symbol") for r in disc)
val_counts = Counter(r.get("symbol") for r in val)

print("\nTop 5 Tickers in Discovery:", disc_counts.most_common(5))
print("Top 5 Tickers in Validation:", val_counts.most_common(5))
