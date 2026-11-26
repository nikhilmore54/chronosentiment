#!/usr/bin/env python3
"""
Definition audit for the frozen RP-609 entry and early-horizon results.

Does not rebuild the frozen 364-trade CSV or the 29 September CSV.
Does not fit a classifier, flip a risk direction, or select a threshold.

The diagnostics below answer the implementation questions raised after the
29 September replication. They are secondary. They do not qualify a mechanism.
"""

from __future__ import annotations

import csv
import hashlib
import json
from collections import defaultdict
from datetime import datetime, timedelta, timezone
from pathlib import Path

from run_rp609_entry_discrimination_evaluation import cluster_auc_ci

WORKSPACE = Path(__file__).resolve().parent.parent
LOG_DIR = WORKSPACE / "live_capture" / "shadow_validation"
FROZEN_CSV = LOG_DIR / "rp609_entry_time_discrimination_dataset.csv"
SEP29_CSV = LOG_DIR / "rp609_nse_20260929_entry_discrimination_dataset.csv"
OUT_JSON = LOG_DIR / "rp609_replication_definition_audit.json"
OUT_MD = LOG_DIR / "rp609_replication_definition_audit.md"
IST = timezone(timedelta(hours=5, minutes=30))
CLOSE_MINUTE_IST = 15 * 60 + 30


def sha256(path: Path) -> str:
    digest = hashlib.sha256()
    with path.open("rb") as handle:
        for chunk in iter(lambda: handle.read(1024 * 1024), b""):
            digest.update(chunk)
    return digest.hexdigest()


def load(path: Path):
    with path.open(encoding="utf-8") as handle:
        return list(csv.DictReader(handle))


def num(value):
    if value is None or value == "":
        return None
    return float(value)


def parse(value):
    return datetime.fromisoformat(value.replace("Z", "+00:00"))


def minutes_between(start, end):
    return (end - start).total_seconds() / 60.0


def auc_block(rows, y, scores):
    if len(rows) < 20:
        return {"auc": None, "ci95": None, "n": len(rows), "note": "fewer than 20 rows"}
    estimate = cluster_auc_ci(rows, y, scores)
    estimate["n"] = len(rows)
    return estimate


def quintiles(rows, score_fn):
    paired = []
    for row in rows:
        score = score_fn(row)
        if score is None:
            continue
        paired.append((score, int(float(row["label_zero_mfe"]))))
    if len(paired) < 5:
        return []
    paired.sort(key=lambda item: item[0])
    size = len(paired)
    table = []
    for fold in range(5):
        start = round(fold * size / 5)
        stop = round((fold + 1) * size / 5)
        chunk = paired[start:stop]
        if not chunk:
            continue
        values = [item[0] for item in chunk]
        table.append({
            "bin": fold + 1,
            "n": len(chunk),
            "n_unique_risk_values": len({round(value, 10) for value in values}),
            "mean_risk": round(sum(values) / len(values), 4),
            "observed_zero_mfe_rate": round(sum(item[1] for item in chunk) / len(chunk), 4),
        })
    return {
        "status": "descriptive_only",
        "tie_treatment": (
            "Equal-count bins split identical scores across adjacent bins by stable sort order. "
            "A large mass at zero means neighbouring bins are not distinct risk levels."
        ),
        "bins": table,
    }


def f15_availability(rows):
    zero_score = [row for row in rows if num(row.get("feature_f15_at_entry")) == 0.0]
    nonzero = [row for row in rows if num(row.get("feature_f15_at_entry")) not in (None, 0.0)]
    def rate(group):
        if not group:
            return None
        return round(sum(int(float(row["label_zero_mfe"])) for row in group) / len(group), 4)
    kept = []
    labels = []
    scores = []
    for row in nonzero:
        raw = num(row["feature_f15_at_entry"])
        kept.append(row)
        labels.append(int(float(row["label_zero_mfe"])))
        scores.append(-raw)
    return {
        "f15_exactly_zero": {"n": len(zero_score), "zero_mfe_rate": rate(zero_score)},
        "f15_nonzero": {"n": len(nonzero), "zero_mfe_rate": rate(nonzero)},
        "auc_among_nonzero_prespecified_direction": auc_block(kept, labels, scores),
    }


def decomposition(rows):
    """Descriptive outcome groups. They do not replace label_zero_mfe."""
    groups = defaultdict(int)
    for row in rows:
        zero = int(float(row["label_zero_mfe"])) == 1
        realized = num(row["label_realized_pnl"])
        favourable_after_entry_clock = False
        observed_clock = False
        for horizon in (5, 10, 15):
            seen = row.get(f"obs_{horizon}m_favourable_seen")
            later = row.get(f"outcome_{horizon}m_later_favourable")
            if seen not in (None, "") or later not in (None, ""):
                observed_clock = True
            if seen not in (None, "") and int(float(seen)) == 1:
                favourable_after_entry_clock = True
            if later not in (None, "") and int(float(later)) == 1:
                favourable_after_entry_clock = True
        if zero and favourable_after_entry_clock:
            groups["delayed_opportunity_zero_lifecycle_mfe_but_later_favourable_vs_entry"] += 1
        elif zero and observed_clock:
            groups["no_favourable_vs_entry_in_observed_windows"] += 1
        elif zero:
            groups["zero_mfe_without_complete_clock_window"] += 1
        elif realized is not None and realized <= 0:
            groups["early_opportunity_positive_mfe_nonpositive_realized"] += 1
        else:
            groups["positive_mfe_and_positive_realized"] += 1
    groups["rows"] = len(rows)
    return dict(groups)


def duration_bands(rows):
    bands = {
        "holding_under_5m": lambda hold, remaining: hold < 5,
        "holding_5_to_15m": lambda hold, remaining: 5 <= hold < 15,
        "holding_15_to_30m": lambda hold, remaining: 15 <= hold < 30,
        "holding_30m_or_more": lambda hold, remaining: hold >= 30,
        "time_to_close_under_30m": lambda hold, remaining: remaining < 30,
        "time_to_close_30_to_60m": lambda hold, remaining: 30 <= remaining < 60,
        "time_to_close_60_to_180m": lambda hold, remaining: 60 <= remaining < 180,
        "time_to_close_180m_or_more": lambda hold, remaining: remaining >= 180,
    }
    counts = {name: {"n": 0, "zero_mfe": 0} for name in bands}
    last_hour_by_hold = {
        "last_hour_holding_under_15m": {"n": 0, "zero_mfe": 0},
        "last_hour_holding_15m_or_more": {"n": 0, "zero_mfe": 0},
    }
    for row in rows:
        entry = parse(row["t_entry_utc"]).astimezone(IST)
        exit_dt = parse(row["t_exit_utc"]).astimezone(IST)
        hold = minutes_between(entry, exit_dt)
        minute = entry.hour * 60 + entry.minute
        remaining = CLOSE_MINUTE_IST - minute
        zero = int(float(row["label_zero_mfe"]))
        for name, predicate in bands.items():
            if predicate(hold, remaining):
                counts[name]["n"] += 1
                counts[name]["zero_mfe"] += zero
        if remaining < 60:
            key = "last_hour_holding_under_15m" if hold < 15 else "last_hour_holding_15m_or_more"
            last_hour_by_hold[key]["n"] += 1
            last_hour_by_hold[key]["zero_mfe"] += zero
    def finish(table):
        out = {}
        for name, item in table.items():
            out[name] = {
                "n": item["n"],
                "zero_mfe": item["zero_mfe"],
                "zero_mfe_rate": None if item["n"] == 0 else round(item["zero_mfe"] / item["n"], 4),
            }
        return out
    return {"pre_specified_bands": finish(counts), "last_hour_split_by_holding": finish(last_hour_by_hold)}


def early_horizon_audit(rows):
    reports = []
    for horizon in (5, 10, 15):
        seen = f"obs_{horizon}m_favourable_seen"
        later = f"outcome_{horizon}m_later_favourable"
        mtm = f"obs_{horizon}m_mtm_pnl"
        eligible = []
        still_open = []
        already_exited = []
        for row in rows:
            if row.get(seen) in (None, "") or row.get(later) in (None, "") or num(row.get(mtm)) is None:
                continue
            entry = parse(row["t_entry_utc"])
            exit_dt = parse(row["t_exit_utc"])
            open_at_horizon = exit_dt > entry + timedelta(minutes=horizon)
            item = row
            eligible.append(item)
            (still_open if open_at_horizon else already_exited).append(item)
        def pack(group):
            y = [1 - int(float(row[later])) for row in group]
            scores = [-num(row[mtm]) for row in group]
            estimate = auc_block(group, y, scores)
            return estimate
        mtm_when_no_later = [num(row[mtm]) for row in eligible if int(float(row[later])) == 0]
        mtm_when_later = [num(row[mtm]) for row in eligible if int(float(row[later])) == 1]
        reports.append({
            "horizon_min": horizon,
            "eligible_with_complete_window": len(eligible),
            "already_exited_by_horizon": len(already_exited),
            "still_open_at_horizon": len(still_open),
            "auc_all_eligible_completed_trades": pack(eligible),
            "auc_still_open_only": pack(still_open),
            "auc_already_exited_only": pack(already_exited),
            "median_mtm_when_no_favourable_in_next_60m": None if not mtm_when_no_later else round(sorted(mtm_when_no_later)[len(mtm_when_no_later) // 2], 2),
            "median_mtm_when_favourable_in_next_60m": None if not mtm_when_later else round(sorted(mtm_when_later)[len(mtm_when_later) // 2], 2),
        })
    return reports


def session_block(rows):
    return {
        "rows": len(rows),
        "f15_availability": f15_availability(rows),
        "f15_risk_quintiles": quintiles(rows, lambda row: None if num(row.get("feature_f15_at_entry")) is None else -num(row["feature_f15_at_entry"])),
        "outcome_decomposition": decomposition(rows),
        "duration_and_time_to_close": duration_bands(rows),
        "early_horizon": early_horizon_audit(rows),
    }


def markdown(report):
    lines = [
        "# RP-609 Replication Definition Audit",
        "",
        "**Disposition:** Research only. The frozen 364-trade export and its hash are unchanged. No entry filter. No classifier. No reversed F15 direction.",
        "",
        "This note records what the reported AUCs measure. It does not replace the session replication report.",
        "",
        "## What the early-horizon AUC measures",
        "",
        "The score is the adverse mark-to-market P&L from the original entry to the last decision-card price at or before the horizon. Long P&L is (price − entry) × quantity. Short P&L is (entry − price) × quantity. The risk score is the negation of that P&L. It is not a change from the previous card, and it is not a distance from the stop.",
        "",
        "The outcome is whether any card strictly after the horizon, through 60 minutes later, has a favourable P&L against the original entry. It is not the change from the price at the horizon. A trade that is already far adverse has to travel back through the entry price to count as later favourable.",
        "",
        "The score and the outcome share the entry price. A trade that is far below entry must travel farther to count as recovered. The AUC ranks that distance-to-recovery question. It does not show a separate regime, and it does not show that an exit action improves realized P&L after costs and slippage.",
        "",
        "The all-eligible AUC uses every completed trade with both a horizon price and a post-horizon price, including trades that have already exited. The still-open AUC answers a conditional question: among trades RP-605 has not yet exited, how does the adverse mark rank a later cross of the entry price? That subset is selected by the baseline exit path. It is not the set of trades that would still be open under a different rule. A higher AUC at 15 minutes than at 5 minutes can come from a smaller, selected population. It is not evidence that waiting improves a decision.",
        "",
        "Incomplete windows stay out of the denominator. They are not filled in. No recovery outside the recorded window is inferred. The two sessions are not pooled into one AUC.",
        "",
        "## Bootstrap",
        "",
        "One thousand ticker-cluster resamples, seed 42. Each draw samples tickers with replacement until the draw has as many tickers as the original set, then keeps every trade in the sampled tickers. The interval is the 2.5 and 97.5 percentiles of draws that contain both outcome classes. Draws with one class are omitted, not imputed. A ticker with one class can still enter a draw. The interval is uncertainty inside that session under ticker resampling. It is not an estimate of how the AUC would vary on a new day. Sessions are not a resampling unit. The same ticker on 28 and 29 September is not linked across days.",
        "",
        "Feature values and directions were fixed in the replication script before the 29 September rows were scored. Missing feature values are dropped for that feature only. No feature was dropped after seeing its AUC. Six features were reported together. None is promoted. The 15-minute and 30-minute scores remain unqualified hypotheses for a later session.",
        "",
        "## Files",
        "",
        f"- Frozen CSV SHA256, unchanged: `{report['files']['frozen_csv_sha256']}`",
        f"- 29 September CSV SHA256: `{report['files']['sep29_csv_sha256']}`",
        "",
        "## Early horizon, all completed trades and trades still open",
        "",
        "| Session | Horizon | Eligible | Already exited | Still open | AUC, all eligible | AUC, still open |",
        "| :--- | ---: | ---: | ---: | ---: | :--- | :--- |",
    ]
    for session_name, block in (("28 Sep", report["frozen_holdout"]), ("29 Sep", report["session_20260929"])):
        for item in block["early_horizon"]:
            def cell(estimate):
                if estimate.get("auc") is None:
                    return "n/a"
                ci = estimate.get("ci95")
                text = f"{estimate['auc']:.3f}"
                if ci:
                    text += f" [{ci[0]:.3f}, {ci[1]:.3f}]"
                text += f" n={estimate.get('n')}"
                dropped = estimate.get("bootstraps_dropped_one_class")
                if dropped is not None:
                    text += f"; dropped {dropped}"
                return text
            lines.append(
                f"| {session_name} | {item['horizon_min']}m | {item['eligible_with_complete_window']} | {item['already_exited_by_horizon']} | {item['still_open_at_horizon']} | {cell(item['auc_all_eligible_completed_trades'])} | {cell(item['auc_still_open_only'])} |"
            )
    lines.extend([
        "",
        "Median mark-to-market P&L from entry, 29 September:",
        "",
    ])
    for item in report["session_20260929"]["early_horizon"]:
        lines.append(
            f"- {item['horizon_min']}m: median MTM when the next 60 minutes are not favourable vs entry = {item['median_mtm_when_no_favourable_in_next_60m']}; when they are favourable = {item['median_mtm_when_favourable_in_next_60m']}."
        )
    lines.extend([
        "",
        "## Outcome groups",
        "",
        "These groups describe the two sessions and sum to each session. They do not replace the frozen zero-MFE label. Later favourable means a favourable price versus entry inside the recorded clocks, not every later tick. No observed recovery means none was seen in those windows.",
        "",
    ])
    for session_name, block in (("28 Sep", report["frozen_holdout"]), ("29 Sep", report["session_20260929"])):
        lines.append(f"- {session_name}: {json.dumps(block['outcome_decomposition'])}")
    lines.extend([
        "",
        "## F15 value and F15 equal to zero",
        "",
        "A zero F15 is a computed score in this ledger, not a missing field. The rate below is the zero-MFE rate in each availability group. The AUC among nonzero values keeps the original direction, risk = −F15.",
        "",
    ])
    for session_name, block in (("28 Sep", report["frozen_holdout"]), ("29 Sep", report["session_20260929"])):
        avail = block["f15_availability"]
        lines.append(
            f"- {session_name}: F15 = 0, n={avail['f15_exactly_zero']['n']}, zero-MFE rate={avail['f15_exactly_zero']['zero_mfe_rate']}; "
            f"F15 ≠ 0, n={avail['f15_nonzero']['n']}, zero-MFE rate={avail['f15_nonzero']['zero_mfe_rate']}; "
            f"AUC on nonzero = {avail['auc_among_nonzero_prespecified_direction'].get('auc')}."
        )
    lines.extend([
        "",
        "The zero-versus-nonzero rate and the AUC among nonzero values are the F15 diagnostics. Equal-count quintiles are descriptive only. Identical zero scores are split across bins by stable sort order, so neighbouring bins with mean risk 0 are not separate risk levels. Each bin records how many distinct values it contains.",
        "",
    ])
    for session_name, block in (("28 Sep", report["frozen_holdout"]), ("29 Sep", report["session_20260929"])):
        lines.append(f"- {session_name}: {json.dumps(block['f15_risk_quintiles'])}")
    lines.extend([
        "",
        "## Holding time and time left in the session",
        "",
        "Cash close is taken as 15:30 IST. Realized holding time is known only after exit, so it is a lifecycle description and not an entry feature. Time remaining until the close is known at entry. It is recorded here as a diagnostic and is not a filter. A test of time-to-close waits for a separately specified multi-session protocol.",
        "",
    ])
    for session_name, block in (("28 Sep", report["frozen_holdout"]), ("29 Sep", report["session_20260929"])):
        lines.append(f"### {session_name}")
        lines.append("")
        for name, item in block["duration_and_time_to_close"]["pre_specified_bands"].items():
            lines.append(f"- `{name}`: n={item['n']}, zero-MFE rate={item['zero_mfe_rate']}")
        lines.append("")
        for name, item in block["duration_and_time_to_close"]["last_hour_split_by_holding"].items():
            lines.append(f"- `{name}`: n={item['n']}, zero-MFE rate={item['zero_mfe_rate']}")
        lines.append("")
    lines.extend([
        "## Causal cutoff",
        "",
        "The RP-606 helper previously allowed cards up to 15 seconds after entry. On these two sessions that window contained no card, and the frozen values matched a strict recompute. The helper now keeps only cards at or before entry. The frozen CSV was not regenerated.",
        "",
        "## Next closed session",
        "",
        "Score the next complete NSE Ledger A session with the same definitions. Do not change feature direction, the zero-MFE label, the entry reference, the 60-minute window, or the incomplete-window rule. Do not fit a classifier or choose a threshold. Report session population, strict as-of-entry audit, entry-feature AUCs, early-horizon results for all eligible trades and the still-open subset, horizon denominators, ticker-cluster counts, and dropped one-class draws. Keep outcome groups and holding duration descriptive. Leave both current session files and the frozen hash in place.",
        "",
        "If that ranking persists across further sessions, a management test is a separate shadow ledger against RP-605. It needs an action, the trades still open at that action, fill assumptions, costs, slippage, and paired realized P&L. The AUC is not that test.",
        "",
        "No mechanism is authorized.",
        "",
    ])
    return "\n".join(lines)


def main():
    frozen = load(FROZEN_CSV)
    sep29 = load(SEP29_CSV)
    if len(frozen) != 364:
        raise SystemExit(f"Frozen CSV row count is {len(frozen)}")
    report = {
        "disposition": "RESEARCH ONLY — definitions audit, no promotion",
        "files": {
            "frozen_csv_sha256": sha256(FROZEN_CSV),
            "sep29_csv_sha256": sha256(SEP29_CSV),
            "frozen_csv_unchanged": True,
        },
        "definitions": {
            "adverse_mark": "Negation of signed mark-to-market P&L from entry to the last card at or before the horizon.",
            "early_horizon_outcome": "Any card in (horizon, horizon+60m] with favourable signed P&L versus the original entry price.",
            "published_population": "Completed trades with both windows observed, including trades whose baseline exit is already before the horizon.",
            "bootstrap": "1000 ticker-cluster percentile intervals, seed 42. Not a cross-session interval.",
            "f15_direction": "Unchanged. Risk score remains -F15.",
        },
        "frozen_holdout": session_block(frozen),
        "session_20260929": session_block(sep29),
    }
    OUT_JSON.write_text(json.dumps(report, indent=2), encoding="utf-8")
    OUT_MD.write_text(markdown(report), encoding="utf-8")
    summary = {
        "frozen_sha": report["files"]["frozen_csv_sha256"],
        "sep29_sha": report["files"]["sep29_csv_sha256"],
        "sep29_early": report["session_20260929"]["early_horizon"],
        "sep29_decomp": report["session_20260929"]["outcome_decomposition"],
        "frozen_decomp": report["frozen_holdout"]["outcome_decomposition"],
        "sep29_f15": report["session_20260929"]["f15_availability"],
        "sep29_duration": report["session_20260929"]["duration_and_time_to_close"],
    }
    print(json.dumps(summary, indent=2))


if __name__ == "__main__":
    main()
