#!/usr/bin/env python3
import json

with open("live_capture/shadow_validation/rp606_rp607_accumulating_evidence_snapshot.json") as f:
    data = json.load(f)

records = data.get("rp607_records", [])
print("Total records:", len(records))
complete = [r for r in records if r.get("horizon_complete")]
print("Complete horizon records:", len(complete))

total_ss = [r for r in records if "04:50:50" not in r.get("t_entry", "")]
total_su = [r for r in records if "04:50:50" in r.get("t_entry", "")]
print(f"Total Population (N={len(records)}): Steady-State = {len(total_ss)} | Startup = {len(total_su)}")

comp_ss = [r for r in complete if "04:50:50" not in r.get("t_entry", "")]
comp_su = [r for r in complete if "04:50:50" in r.get("t_entry", "")]
print(f"Complete Population (N={len(complete)}): Steady-State = {len(comp_ss)} | Startup = {len(comp_su)}")

# Chronological split of complete records
complete_sorted = sorted(complete, key=lambda x: x.get("t_condition") or x.get("t_exit"))
split_idx = len(complete_sorted) // 2
disc = complete_sorted[:split_idx]
val = complete_sorted[split_idx:]

disc_ss = [r for r in disc if "04:50:50" not in r.get("t_entry", "")]
disc_su = [r for r in disc if "04:50:50" in r.get("t_entry", "")]

val_ss = [r for r in val if "04:50:50" not in r.get("t_entry", "")]
val_su = [r for r in val if "04:50:50" in r.get("t_entry", "")]

split_ts = disc[-1].get("t_condition") or disc[-1].get("t_exit")

print(f"Discovery Cohort (N={len(disc)}): Steady-State = {len(disc_ss)} | Startup = {len(disc_su)}")
print(f"Validation Cohort (N={len(val)}): Steady-State = {len(val_ss)} | Startup = {len(val_su)}")
print(f"Chronological Split Boundary Timestamp: {split_ts}")
