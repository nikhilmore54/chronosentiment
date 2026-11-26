#!/usr/bin/env python3
"""
Per-worker HC3 audit for CVD-001 Instance 1.
Prints: historical_credit, assigned_credit, HC3 limit, total, violation flag.
Also diagnoses the "Total credited h: 57h" discrepancy.

historical_workloads format: list of {"worker_id": int, "hours": float}
"""
import json

RESULT_PATH = "data/cvd001/CVD-001-INSTANCE1-RESULT-v2.0.json"
PAYLOAD_PATH = "data/cvd001/CVD-001-INSTANCE1-PAYLOAD-DRY-RUN.json"
HC3_LIMIT = 40.0

def main():
    with open(RESULT_PATH) as f:
        result = json.load(f)
    with open(PAYLOAD_PATH) as f:
        payload = json.load(f)

    # Schedule: str(shift_id) -> int(worker_id)
    schedule = result["api_response"]["schedule"]

    # Shift lookup: int(id) -> shift dict
    shifts_by_id = {s["id"]: s for s in payload["shifts"]}

    # Worker list
    workers_by_id = {w["id"]: w for w in payload["workers"]}

    # historical_workloads: list of {"worker_id": int, "hours": float}
    hist_wl_list = payload.get("historical_workloads", [])
    hist_wl = {entry["worker_id"]: float(entry["hours"]) for entry in hist_wl_list}

    # --- Shift duration stats ---
    all_durs = [s.get("duration_hours", 0.0) for s in payload["shifts"]]
    print("=== Shift duration_hours stats ===")
    print(f"  Count : {len(all_durs)}")
    print(f"  Min   : {min(all_durs):.4f}h")
    print(f"  Max   : {max(all_durs):.4f}h")
    print(f"  Mean  : {sum(all_durs)/len(all_durs):.4f}h")
    print(f"  Sum   : {sum(all_durs):.2f}h")
    print(f"  Zeros : {sum(1 for d in all_durs if d == 0.0)}")
    print()

    # --- historical_workloads sample ---
    print("=== historical_workloads sample (first 3) ===")
    for entry in hist_wl_list[:3]:
        print(f"  worker_id={entry['worker_id']}  hours={entry['hours']}")
    print(f"  Total workers with historical data: {len(hist_wl_list)}")
    print()

    # --- Compute assigned hours per worker ---
    assigned_hours = {}
    assigned_count = {}
    for sid_str, wid in schedule.items():
        shift = shifts_by_id.get(int(sid_str))
        if shift is None:
            print(f"WARNING: shift {sid_str} not found in payload")
            continue
        dur = float(shift.get("duration_hours", 0.0))
        assigned_hours[wid] = assigned_hours.get(wid, 0.0) + dur
        assigned_count[wid] = assigned_count.get(wid, 0) + 1

    # --- Per-worker table ---
    print(f"{'Worker':>8} {'Hist(h)':>10} {'Asgn(h)':>9} {'Total(h)':>10} {'Limit':>6} {'Viol?':>7} {'#Shifts':>8}")
    print("-" * 65)

    violations = 0
    total_asgn_all = 0.0
    total_hist_all = 0.0

    for wid in sorted(workers_by_id.keys()):
        hist = hist_wl.get(wid, 0.0)
        asgn = assigned_hours.get(wid, 0.0)
        total = hist + asgn
        viol = total > HC3_LIMIT
        if viol:
            violations += 1
        total_asgn_all += asgn
        total_hist_all += hist
        cnt = assigned_count.get(wid, 0)
        flag = "YES ***" if viol else "no"
        print(f"EMP{wid:03d}  {hist:>10.2f} {asgn:>9.2f} {total:>10.2f} {HC3_LIMIT:>6.1f} {flag:>7} {cnt:>8}")

    print("-" * 65)
    print(f"{'TOTAL':>8} {total_hist_all:>10.2f} {total_asgn_all:>9.2f} {'':>10} {'':>6} {violations:>7} violations")
    print()

    # --- Discrepancy analysis ---
    metrics = result["api_response"].get("metrics", {})
    api_credited = metrics.get("total_credited_hours", None)

    print("=== Discrepancy Analysis ===")
    print(f"  Shifts in schedule          : {len(schedule)}")
    print(f"  Script total assigned h     : {total_asgn_all:.2f}h")
    print(f"  API metrics total_credited_h: {api_credited}")
    print(f"  Avg assigned h per worker   : {total_asgn_all/33:.2f}h")
    print(f"  Avg assigned h per shift    : {total_asgn_all/len(schedule):.4f}h")
    if api_credited and float(api_credited) > 0:
        print(f"  Ratio (script / API)        : {total_asgn_all/float(api_credited):.2f}x")
    print()

    # --- Full API metrics block ---
    print("=== API metrics block ===")
    for k, v in metrics.items():
        print(f"  {k}: {v}")

if __name__ == "__main__":
    main()