import json
import statistics
from collections import defaultdict
from pathlib import Path

FILES = [
    "datasets/shadow_assessment_20260909.json",
    "datasets/shadow_assessment_20260915.json",
    "datasets/shadow_assessment_20260917.json",
    "datasets/shadow_assessment_20260918.json",
]

THRESHOLDS = [
    -0.0025,
    -0.0050,
    -0.0075,
    -0.0100,
    -0.0125,
]


def load_run(path):
    with open(path) as f:
        return json.load(f)


def extract_realized_return(p):
    # Current report format
    value = p.get("realized_return")
    if value is not None:
        return value

    # Defensive fallback for trajectory-style reports
    value = p.get("h300_ret")
    if value is not None:
        return value

    return None


def prepare_run(path):
    run = load_run(path)

    assessments = run.get("tick_assessments", [])
    positions = run.get("positions", [])

    by_position = defaultdict(list)

    for a in assessments:
        decision_id = a.get("decision_id")
        if decision_id:
            by_position[decision_id].append(a)

    for rows in by_position.values():
        rows.sort(key=lambda x: (x.get("bar_unix", 0),
                                 x.get("bars_since_entry", 0)))

    eligible = []

    for p in positions:
        decision_id = p.get("decision_id")
        if not decision_id:
            continue

        baseline = extract_realized_return(p)

        if baseline is None:
            continue

        path_rows = by_position.get(decision_id, [])

        if not path_rows:
            continue

        eligible.append({
            "decision_id": decision_id,
            "ticker": p.get("ticker"),
            "baseline": baseline,
            "path": path_rows,
        })

    return run, eligible


def evaluate(positions, threshold):
    rows = []

    for p in positions:
        trigger = None

        for a in p["path"]:
            current_return = a.get("current_signed_return")

            if current_return is None:
                continue

            if current_return <= threshold:
                trigger = a
                break

        if trigger is None:
            managed = p["baseline"]
            triggered = False
            trigger_bar = None
            trigger_return = None
        else:
            managed = trigger["current_signed_return"]
            triggered = True
            trigger_bar = trigger.get("bars_since_entry")
            trigger_return = trigger["current_signed_return"]

        rows.append({
            "decision_id": p["decision_id"],
            "ticker": p["ticker"],
            "baseline": p["baseline"],
            "managed": managed,
            "delta": managed - p["baseline"],
            "triggered": triggered,
            "trigger_bar": trigger_bar,
            "trigger_return": trigger_return,
        })

    return rows


def summarize(rows):
    n = len(rows)

    if n == 0:
        return None

    baseline = [r["baseline"] for r in rows]
    managed = [r["managed"] for r in rows]
    delta = [r["delta"] for r in rows]

    triggered = [r for r in rows if r["triggered"]]
    improved = [r for r in rows if r["delta"] > 1e-12]
    worsened = [r for r in rows if r["delta"] < -1e-12]
    unchanged = [r for r in rows if abs(r["delta"]) <= 1e-12]

    trigger_bars = [
        r["trigger_bar"]
        for r in triggered
        if r["trigger_bar"] is not None
    ]

    return {
        "n": n,
        "triggered": len(triggered),
        "trigger_pct": len(triggered) / n * 100,
        "baseline_mean": statistics.mean(baseline),
        "managed_mean": statistics.mean(managed),
        "delta_mean": statistics.mean(delta),
        "delta_sum": sum(delta),
        "baseline_wr": sum(x > 0 for x in baseline) / n * 100,
        "managed_wr": sum(x > 0 for x in managed) / n * 100,
        "improved": len(improved),
        "worsened": len(worsened),
        "unchanged": len(unchanged),
        "median_trigger_bar": (
            statistics.median(trigger_bars)
            if trigger_bars else None
        ),
        "mean_trigger_bar": (
            statistics.mean(trigger_bars)
            if trigger_bars else None
        ),
    }


# ---------------------------------------------------------------------
# Load all runs
# ---------------------------------------------------------------------

runs = []
all_positions = []

print("=" * 110)
print("CHRONOSENTIMENT — MULTI-DATE P&L EXIT COUNTERFACTUAL")
print("=" * 110)

for path in FILES:
    if not Path(path).exists():
        print(f"WARNING: missing {path}")
        continue

    run, eligible = prepare_run(path)

    date = run.get("date", Path(path).stem[-8:])

    runs.append((date, run, eligible))
    all_positions.extend(eligible)

    print(
        f"{date}: "
        f"positions={len(run.get('positions', []))} "
        f"assessments={len(run.get('tick_assessments', []))} "
        f"eligible={len(eligible)}"
    )

print()
print(f"Total eligible positions: {len(all_positions)}")
print()

# ---------------------------------------------------------------------
# Per-date results
# ---------------------------------------------------------------------

for threshold in THRESHOLDS:

    print("=" * 110)
    print(f"THRESHOLD {threshold * 100:.2f}%")
    print("=" * 110)

    pooled = []

    print(
        f"{'DATE':<12}"
        f"{'N':>6}"
        f"{'TRIG%':>9}"
        f"{'BASE':>10}"
        f"{'MANAGED':>10}"
        f"{'DELTA':>10}"
        f"{'IMPROVED':>10}"
        f"{'WORSED':>9}"
    )

    print("-" * 110)

    for date, run, eligible in runs:
        rows = evaluate(eligible, threshold)
        pooled.extend(rows)

        s = summarize(rows)

        if s is None:
            continue

        print(
            f"{date:<12}"
            f"{s['n']:>6}"
            f"{s['trigger_pct']:>8.1f}%"
            f"{s['baseline_mean']*100:>9.3f}%"
            f"{s['managed_mean']*100:>9.3f}%"
            f"{s['delta_mean']*100:>9.3f}%"
            f"{s['improved']:>10}"
            f"{s['worsened']:>9}"
        )

    print("-" * 110)

    s = summarize(pooled)

    if s:
        print(
            f"{'POOLED':<12}"
            f"{s['n']:>6}"
            f"{s['trigger_pct']:>8.1f}%"
            f"{s['baseline_mean']*100:>9.3f}%"
            f"{s['managed_mean']*100:>9.3f}%"
            f"{s['delta_mean']*100:>9.3f}%"
            f"{s['improved']:>10}"
            f"{s['worsened']:>9}"
        )

        print()
        print(
            f"Aggregate delta       : "
            f"{s['delta_sum']*100:.3f}%"
        )
        print(
            f"Baseline win rate     : "
            f"{s['baseline_wr']:.1f}%"
        )
        print(
            f"Managed win rate      : "
            f"{s['managed_wr']:.1f}%"
        )
        print(
            f"Median trigger bar    : "
            f"{s['median_trigger_bar']}"
        )
        print(
            f"Mean trigger bar      : "
            f"{s['mean_trigger_bar']:.1f}"
            if s["mean_trigger_bar"] is not None
            else "Mean trigger bar      : N/A"
        )

print()
print("=" * 110)
print("END")
print("=" * 110)
