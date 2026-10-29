import json
from pathlib import Path

FILES = [
    "datasets/shadow_assessment_20260909.json",
    "datasets/shadow_assessment_20260915.json",
    "datasets/shadow_assessment_20260918.json",
]

THRESHOLDS = [-0.0050, -0.0075, -0.0100]

def load(path):
    with open(path) as f:
        return json.load(f)

def realized(p):
    return p.get("realized_return", p.get("h300_ret"))

for path in FILES:
    run = load(path)
    date = run["date"]

    assessments = {}
    for a in run.get("tick_assessments", []):
        did = a.get("decision_id")
        if did:
            assessments.setdefault(did, []).append(a)

    rows = []

    for p in run.get("positions", []):
        did = p.get("decision_id")
        base = realized(p)

        if not did or base is None:
            continue

        path_rows = assessments.get(did, [])
        path_rows.sort(key=lambda x: x.get("bar_unix", 0))

        rows.append((did, p.get("ticker"), base, path_rows))

    print()
    print("=" * 100)
    print(f"{date} — POSITION CONTRIBUTION ANALYSIS")
    print("=" * 100)
    print(f"Eligible positions: {len(rows)}")

    for threshold in THRESHOLDS:
        result = []

        for did, ticker, base, path_rows in rows:
            trigger = None

            for a in path_rows:
                r = a.get("current_signed_return")
                if r is not None and r <= threshold:
                    trigger = a
                    break

            managed = (
                trigger["current_signed_return"]
                if trigger is not None
                else base
            )

            result.append({
                "ticker": ticker,
                "baseline": base,
                "managed": managed,
                "delta": managed - base,
                "triggered": trigger is not None,
                "bar": (
                    trigger.get("bars_since_entry")
                    if trigger is not None else None
                ),
            })

        result.sort(key=lambda x: x["delta"])

        print()
        print(f"THRESHOLD {threshold*100:.2f}%")
        print("-" * 100)

        print(
            "sum delta        : "
            f"{sum(x['delta'] for x in result)*100:.3f}%"
        )
        print(
            "median delta     : "
            f"{sorted(x['delta'] for x in result)[len(result)//2]*100:.3f}%"
        )
        print(
            "improved         : "
            f"{sum(x['delta'] > 0 for x in result)}"
        )
        print(
            "worsened         : "
            f"{sum(x['delta'] < 0 for x in result)}"
        )

        print()
        print("TOP 5 POSITIVE CONTRIBUTIONS")
        for x in sorted(result, key=lambda x: x["delta"], reverse=True)[:5]:
            print(
                f"{x['ticker']:<18} "
                f"base={x['baseline']*100:7.3f}% "
                f"managed={x['managed']*100:7.3f}% "
                f"delta={x['delta']*100:7.3f}% "
                f"bar={x['bar']}"
            )

        print()
        print("TOP 5 NEGATIVE CONTRIBUTIONS")
        for x in sorted(result, key=lambda x: x["delta"])[:5]:
            print(
                f"{x['ticker']:<18} "
                f"base={x['baseline']*100:7.3f}% "
                f"managed={x['managed']*100:7.3f}% "
                f"delta={x['delta']*100:7.3f}% "
                f"bar={x['bar']}"
            )
