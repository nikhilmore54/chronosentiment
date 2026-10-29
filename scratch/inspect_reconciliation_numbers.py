import json
from pathlib import Path

path = Path("live_capture/shadow_validation/rp606_rp607_accumulating_evidence_snapshot.json")
with open(path) as f:
    data = json.load(f)

records = data["rp607_records"]
complete = [r for r in records if r.get("horizon_complete")]

print(f"Total complete records: {len(complete)}")

# Build matrix
matrix = []
for r in complete:
    matrix.append({
        "symbol": r["symbol"],
        "t_condition": r.get("t_condition") or r.get("t_exit"),
        "t_entry": r.get("t_entry"),
        "cohort": "STARTUP_SEEDING" if "04:50:50" in r.get("t_entry", "") else "STEADY_STATE"
    })

matrix.sort(key=lambda x: x["t_condition"])
split_idx = len(matrix) // 2
disc_cohort = matrix[:split_idx]
val_cohort = matrix[split_idx:]

counts_full = {}
for r in matrix:
    counts_full[r["symbol"]] = counts_full.get(r["symbol"], 0) + 1

counts_disc = {}
for r in disc_cohort:
    counts_disc[r["symbol"]] = counts_disc.get(r["symbol"], 0) + 1

counts_val = {}
for r in val_cohort:
    counts_val[r["symbol"]] = counts_val.get(r["symbol"], 0) + 1

print("\nTop 10 Tickers by Full Population Volume:")
top10_full = sorted(counts_full.items(), key=lambda x: x[1], reverse=True)[:10]
for sym, count in top10_full:
    print(f"  {sym}: Full={count}, Disc={counts_disc.get(sym,0)}, Val={counts_val.get(sym,0)}")

print("\nTop 10 Tickers by Stage 2 Validation Volume:")
top10_val = sorted(counts_val.items(), key=lambda x: x[1], reverse=True)[:10]
for sym, count in top10_val:
    print(f"  {sym}: Val={count}, Disc={counts_disc.get(sym,0)}, Full={counts_full.get(sym,0)}")
