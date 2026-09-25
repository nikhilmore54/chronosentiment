import json
import statistics
from collections import defaultdict

PATH = "datasets/shadow_assessment_20260918.json"

with open(PATH) as f:
    run = json.load(f)

assessments = run["tick_assessments"]
positions = run["positions"]

print("=" * 80)
print("CHRONOSENTIMENT — TICK ASSESSMENT ANALYSIS")
print("=" * 80)

print(f"Positions     : {len(positions)}")
print(f"Assessments   : {len(assessments)}")

# ---------------------------------------------------------------------
# Position outcomes
# ---------------------------------------------------------------------

outcomes = {}

for p in positions:
    decision_id = p.get("decision_id")

    # CachedPositionReport fields can vary slightly, so inspect the
    # realized return from the trajectory if necessary.
    realized = p.get("realized_return")

    if realized is not None:
        outcomes[decision_id] = realized

# Fallback: trajectory realized return
for p in positions:
    decision_id = p.get("decision_id")

    if decision_id in outcomes:
        continue

    realized = p.get("realized_return")

    if realized is not None:
        outcomes[decision_id] = realized


# ---------------------------------------------------------------------
# Attach eventual outcome to every tick
# ---------------------------------------------------------------------

rows = []

for a in assessments:
    decision_id = a.get("decision_id")

    if decision_id not in outcomes:
        continue

    r = outcomes[decision_id]

    rows.append({
        "decision_id": decision_id,
        "ticker": a["ticker"],
        "bar": a["bars_since_entry"],
        "return": a["current_signed_return"],
        "momentum": a["recent_momentum"],
        "recent_change": a["recent_price_change"],
        "final_return": r,
        "final_win": r > 0,
    })

print(f"Joined ticks  : {len(rows)}")
print()


# ---------------------------------------------------------------------
# Helpers
# ---------------------------------------------------------------------

def summarize(label, rows):
    if not rows:
        print(f"{label}: N=0")
        return

    wins = sum(r["final_win"] for r in rows)
    rets = [r["final_return"] for r in rows]

    print(
        f"{label:<25}"
        f"N={len(rows):5d} "
        f"final_WR={wins/len(rows)*100:6.1f}% "
        f"mean={statistics.mean(rets)*100:7.3f}% "
        f"median={statistics.median(rets)*100:7.3f}%"
    )


# ---------------------------------------------------------------------
# Momentum buckets
# ---------------------------------------------------------------------

print("MOMENTUM → EVENTUAL H300 OUTCOME")
print("-" * 80)

momentum_buckets = [
    ("0.0–0.2", 0.0, 0.2),
    ("0.2–0.4", 0.2, 0.4),
    ("0.4–0.6", 0.4, 0.6),
    ("0.6–0.8", 0.6, 0.8),
    ("0.8–1.0", 0.8, 1.000001),
]

for label, lo, hi in momentum_buckets:
    bucket = [
        r for r in rows
        if lo <= r["momentum"] < hi
    ]
    summarize(label, bucket)

print()


# ---------------------------------------------------------------------
# Momentum by time since entry
# ---------------------------------------------------------------------

print("MOMENTUM × POSITION AGE")
print("-" * 80)

age_buckets = [
    ("1–5", 1, 5),
    ("6–10", 6, 10),
    ("11–20", 11, 20),
    ("21–30", 21, 30),
    ("31–60", 31, 60),
    ("61+", 61, 10_000),
]

for age_label, age_lo, age_hi in age_buckets:
    print()
    print(f"AGE {age_label}")

    for label, lo, hi in momentum_buckets:
        bucket = [
            r for r in rows
            if age_lo <= r["bar"] <= age_hi
            and lo <= r["momentum"] < hi
        ]
        summarize(label, bucket)


# ---------------------------------------------------------------------
# Recent price change
# ---------------------------------------------------------------------

print()
print("RECENT PRICE CHANGE → EVENTUAL H300 OUTCOME")
print("-" * 80)

change_buckets = [
    ("<-0.50%", -999, -0.005),
    ("-0.50 to -0.25%", -0.005, -0.0025),
    ("-0.25 to 0%", -0.0025, 0),
    ("0 to +0.25%", 0, 0.0025),
    ("+0.25 to +0.50%", 0.0025, 0.005),
    (">+0.50%", 0.005, 999),
]

for label, lo, hi in change_buckets:
    bucket = [
        r for r in rows
        if lo <= r["recent_change"] < hi
    ]
    summarize(label, bucket)


# ---------------------------------------------------------------------
# Current P&L
# ---------------------------------------------------------------------

print()
print("CURRENT UNREALIZED RETURN → EVENTUAL H300 OUTCOME")
print("-" * 80)

return_buckets = [
    ("<-1%", -999, -0.01),
    ("-1 to -0.5%", -0.01, -0.005),
    ("-0.5 to 0%", -0.005, 0),
    ("0 to +0.5%", 0, 0.005),
    ("+0.5 to +1%", 0.005, 0.01),
    (">+1%", 0.01, 999),
]

for label, lo, hi in return_buckets:
    bucket = [
        r for r in rows
        if lo <= r["return"] < hi
    ]
    summarize(label, bucket)

print()
print("=" * 80)
