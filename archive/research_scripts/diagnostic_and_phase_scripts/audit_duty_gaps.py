#!/usr/bin/env python3
"""
Audit inter-duty rest gaps in GERAD benchmark instances.
Answers: are there any gaps in the 8h–10h critical zone?
If yes, the layover threshold matters. If no, it cannot matter.
"""
import csv
from datetime import datetime
from collections import defaultdict
import os

def parse_utc(s):
    return datetime.fromisoformat(s.replace("Z", "+00:00"))

base = os.path.join(os.path.dirname(__file__), "..", "benchmarks", "gerad-g2014-22")
gaps_8_to_10 = []
gaps_under_8 = []
gaps_over_10 = []

for i in range(1, 8):
    idir = os.path.join(base, f"instance{i}")
    duties_by_crew = defaultdict(list)
    with open(os.path.join(idir, "duties.csv")) as f:
        for row in csv.DictReader(f):
            duties_by_crew[row["crew_id"]].append({
                "report": parse_utc(row["report_utc"]),
                "release": parse_utc(row["release_utc"]),
            })
    for crew_id, duties in duties_by_crew.items():
        duties.sort(key=lambda d: d["report"])
        for j in range(1, len(duties)):
            gap_h = (duties[j]["report"] - duties[j-1]["release"]).total_seconds() / 3600.0
            if 8.0 <= gap_h < 10.0:
                gaps_8_to_10.append((f"instance{i}", crew_id, round(gap_h, 2)))
            elif gap_h < 8.0:
                gaps_under_8.append((f"instance{i}", crew_id, round(gap_h, 2)))
            else:
                gaps_over_10.append(round(gap_h, 2))

print(f"Gaps < 8h (same FDP):        {len(gaps_under_8)}")
print(f"Gaps 8h-10h (critical zone): {len(gaps_8_to_10)}")
print(f"Gaps >= 10h (layover):       {len(gaps_over_10)}")
if gaps_8_to_10:
    print("\nSample gaps in 8-10h zone:")
    for g in gaps_8_to_10[:20]:
        print(f"  {g}")
else:
    print("\nNo gaps in 8-10h zone. Threshold change cannot affect pairing grouping.")
    print("The 0.0pp delta result is confirmed: the benchmark data has no inter-duty")
    print("gaps in the critical zone where the threshold would make a difference.")
