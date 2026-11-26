import json
import statistics
from collections import defaultdict

PATH = "datasets/shadow_assessment_20260918.json"

THRESHOLDS = [
    -0.0025,   # -0.25%
    -0.0050,   # -0.50%
    -0.0075,   # -0.75%
    -0.0100,   # -1.00%
    -0.0125,   # -1.25%
]

with open(PATH) as f:
    run = json.load(f)

assessments = run["tick_assessments"]
positions = run["positions"]

# ---------------------------------------------------------------------
# Build one chronological assessment path per position
# ---------------------------------------------------------------------

by_position = defaultdict(list)

for a in assessments:
    decision_id = a.get("decision_id")
    if decision_id:
        by_position[decision_id].append(a)

for rows in by_position.values():
    rows.sort(key=lambda x: x["bar_unix"])

# ---------------------------------------------------------------------
# Get baseline H300 / realized outcome
# ---------------------------------------------------------------------

position_map = {}

for p in positions:
    decision_id = p.get("decision_id")
    if not decision_id:
        continue

    realized = p.get("realized_return")

    # Some reports expose realized return through trajectory.
    if realized is None:
        realized = None

    position_map[decision_id] = {
        "ticker": p.get("ticker"),
        "direction": p.get("direction"),
        "entry_price": p.get("entry_price"),
        "opened_at": p.get("opened_at"),
        "realized_return": realized,
    }

# If realized_return is absent, recover it from trajectory.
for p in positions:
    decision_id = p.get("decision_id")
    if not decision_id:
        continue

    if position_map[decision_id]["realized_return"] is not None:
        continue

    # Try common trajectory representations.
    h300 = p.get("h300_ret")
    if h300 is not None:
        position_map[decision_id]["realized_return"] = h300

# ---------------------------------------------------------------------
# Only positions with a known terminal outcome are eligible.
# ---------------------------------------------------------------------

eligible = []

for decision_id, p in position_map.items():
    baseline = p["realized_return"]

    if baseline is None:
        continue

    path = by_position.get(decision_id, [])

    eligible.append({
        "decision_id": decision_id,
        "ticker": p["ticker"],
        "baseline": baseline,
        "path": path,
    })

print("=" * 90)
print("CHRONOSENTIMENT — P&L EXIT COUNTERFACTUAL")
print("=" * 90)

print(f"Positions in report : {len(positions)}")
print(f"Positions with path: {len(by_position)}")
print(f"Eligible positions : {len(eligible)}")
print()

# ---------------------------------------------------------------------
# Counterfactual
# ---------------------------------------------------------------------

results = {}

for threshold in THRESHOLDS:
    rows = []

    for p in eligible:
        baseline = p["baseline"]
        path = p["path"]

        trigger = None

        for a in path:
            if a["current_signed_return"] <= threshold:
                trigger = a
                break

        if trigger is None:
            managed = baseline
            triggered = False
            trigger_bar = None
            trigger_return = None
        else:
            managed = trigger["current_signed_return"]
            triggered = True
            trigger_bar = trigger["bars_since_entry"]
            trigger_return = trigger["current_signed_return"]

        delta = managed - baseline

        rows.append({
            "decision_id": p["decision_id"],
            "ticker": p["ticker"],
            "baseline": baseline,
            "managed": managed,
            "delta": delta,
            "triggered": triggered,
            "trigger_bar": trigger_bar,
            "trigger_return": trigger_return,
        })

    results[threshold] = rows

# ---------------------------------------------------------------------
# Summary
# ---------------------------------------------------------------------

for threshold, rows in results.items():

    n = len(rows)
    triggered = [r for r in rows if r["triggered"]]

    baseline_returns = [r["baseline"] for r in rows]
    managed_returns = [r["managed"] for r in rows]
    deltas = [r["delta"] for r in rows]

    improved = [r for r in rows if r["delta"] > 1e-12]
    worsened = [r for r in rows if r["delta"] < -1e-12]
    unchanged = [r for r in rows if abs(r["delta"]) <= 1e-12]

    baseline_wins = sum(r["baseline"] > 0 for r in rows)
    managed_wins = sum(r["managed"] > 0 for r in rows)

    print("-" * 90)
    print(f"THRESHOLD {threshold * 100:.2f}%")
    print("-" * 90)

    print(f"N positions              : {n}")
    print(
        f"Triggered                : {len(triggered)} "
        f"({len(triggered)/n*100:.1f}%)"
    )

    if triggered:
        bars = [r["trigger_bar"] for r in triggered]
        print(f"Median trigger bar       : {statistics.median(bars):.1f}")
        print(f"Mean trigger bar         : {statistics.mean(bars):.1f}")

    print()
    print(
        f"Baseline mean            : "
        f"{statistics.mean(baseline_returns)*100:.3f}%"
    )
    print(
        f"Managed mean             : "
        f"{statistics.mean(managed_returns)*100:.3f}%"
    )
    print(
        f"Mean incremental value   : "
        f"{statistics.mean(deltas)*100:.3f}%"
    )
    print(
        f"Aggregate incremental    : "
        f"{sum(deltas)*100:.3f}%"
    )

    print()
    print(
        f"Baseline win rate        : "
        f"{baseline_wins/n*100:.1f}%"
    )
    print(
        f"Managed win rate         : "
        f"{managed_wins/n*100:.1f}%"
    )

    print()
    print(f"Improved positions       : {len(improved)}")
    print(f"Worsened positions       : {len(worsened)}")
    print(f"Unchanged positions      : {len(unchanged)}")

    if triggered:
        trigger_returns = [r["trigger_return"] for r in triggered]

        print()
        print(
            f"Mean trigger return      : "
            f"{statistics.mean(trigger_returns)*100:.3f}%"
        )
        print(
            f"Median trigger return   : "
            f"{statistics.median(trigger_returns)*100:.3f}%"
        )

        triggered_delta = [r["delta"] for r in triggered]

        print(
            f"Mean triggered delta     : "
            f"{statistics.mean(triggered_delta)*100:.3f}%"
        )

    worst_baseline = min(rows, key=lambda r: r["baseline"])
    worst_managed = min(rows, key=lambda r: r["managed"])

    print()
    print(
        f"Worst baseline           : "
        f"{worst_baseline['ticker']} "
        f"{worst_baseline['baseline']*100:.3f}%"
    )
    print(
        f"Worst managed            : "
        f"{worst_managed['ticker']} "
        f"{worst_managed['managed']*100:.3f}%"
    )

print()
print("=" * 90)
print("END")
print("=" * 90)
