import json
with open("phase1/collision_families.json") as f:
    families = json.load(f)
for rh, execs in families.items():
    lengths = set(ex["tick_count"] for ex in execs)
    if len(lengths) > 1:
        print(f"Hash: {rh}")
        for ex in execs:
            print(f"  {ex['substrate']} | {ex['topology']} | {ex['cognition']} | ticks: {ex['tick_count']}")
