#!/usr/bin/env python3
"""
Freeze the 364-trade RP-609 export and replicate entry-time discrimination
on the closed NSE Ledger A session of 2026-09-29.

Research only. No entry filter. No multivariate model.
RP-605, RP-608, PFT, SPL, VAT, and PPR are not modified.

The 29 September trades are a separate session. They are not appended to
the frozen 364-row export. Trade IDs restart across days, so this session
is selected by entry date, not by trade ID alone.

Entry features used here are restricted to information at or before entry.
The RP-606 freshness helper can read cards up to 15 seconds after entry.
This script recomputes freshness on cards with timestamp <= entry and
records whether that window changes any value.
"""

from __future__ import annotations

import csv
import hashlib
import json
import subprocess
import sys
from collections import Counter, defaultdict
from datetime import datetime, timedelta, timezone
from pathlib import Path

from build_rp609_entry_time_discrimination_dataset import (
    HORIZONS_MIN,
    POST_HORIZON_MIN,
    cards_between,
    favourable_mfe,
    last_card_at_or_before,
    load_cards,
    parse_ts,
    signed_pnl,
)
from run_rp609_entry_discrimination_evaluation import (
    BOOTSTRAP_SEED,
    BOOTSTRAPS,
    FROZEN_F15_GATE,
    aligned_score,
    cluster_auc_ci,
)

WORKSPACE = Path(__file__).resolve().parent.parent
LOG_DIR = WORKSPACE / "live_capture" / "shadow_validation"
CSV_PATH = LOG_DIR / "rp609_entry_time_discrimination_dataset.csv"
INTEGRITY_MD = LOG_DIR / "rp609_entry_time_discrimination_integrity.md"
INTEGRITY_JSON = LOG_DIR / "rp609_entry_time_discrimination_integrity.json"
BUILDER = WORKSPACE / "scripts" / "build_rp609_entry_time_discrimination_dataset.py"
SNAPSHOT = LOG_DIR / "rp606_rp607_accumulating_evidence_snapshot.json"
JOURNAL = LOG_DIR / "rp605_nse_baseline_journal.jsonl"
REPORT = LOG_DIR / "rp609_temporal_validation_report.json"
MANIFEST = LOG_DIR / "rp609_entry_discrimination_freeze_manifest.json"
OUT_JSON = LOG_DIR / "rp609_nse_20260929_session_replication.json"
OUT_MD = LOG_DIR / "rp609_nse_20260929_session_replication.md"
OUT_CSV = LOG_DIR / "rp609_nse_20260929_entry_discrimination_dataset.csv"

FROZEN_SNAPSHOT_SHA = "0190c56603ce59b9db2f695b536a751ac70371e010621c04633e2366982bb0b5"
SESSION_DAY = "2026-09-29"
IST = timezone(timedelta(hours=5, minutes=30))
FREEZE_ID = "RP609-ENTRY-EXPORT-V1"


def sha256(path: Path) -> str:
    digest = hashlib.sha256()
    with path.open("rb") as handle:
        for chunk in iter(lambda: handle.read(1024 * 1024), b""):
            digest.update(chunk)
    return digest.hexdigest()


def fnum(value):
    if value is None or value == "":
        return None
    return float(value)


def median(values):
    if not values:
        return None
    ordered = sorted(values)
    mid = len(ordered) // 2
    if len(ordered) % 2:
        return ordered[mid]
    return (ordered[mid - 1] + ordered[mid]) / 2.0


def freshness_from_cards(cards, side):
    if not cards:
        return None, None, 0
    scores_15 = [float(card["15m_score_trajectory"]) for card in cards]
    scores_30 = [float(card["30m_score_trajectory"]) for card in cards]
    if side != "LONG":
        scores_15 = [-value for value in scores_15]
        scores_30 = [-value for value in scores_30]
    churn = 0
    last_state = None
    for card in cards:
        state = card.get("decision_state")
        if state and state != last_state:
            churn += 1
            last_state = state
    return scores_15[-1] - max(scores_15), scores_30[-1] - max(scores_30), churn


def lifecycle_mfe(side, entry_price, quantity, prices):
    if side == "LONG":
        excursion = (max(prices) - entry_price) * quantity
    else:
        excursion = (entry_price - min(prices)) * quantity
    return max(0.0, excursion)


def load_sep29_journal():
    rows = []
    with JOURNAL.open(encoding="utf-8") as handle:
        for line in handle:
            line = line.strip()
            if not line:
                continue
            row = json.loads(line)
            entry_dt = parse_ts(row["entry_timestamp"])
            if entry_dt.astimezone(timezone.utc).date().isoformat() != SESSION_DAY:
                continue
            row["_entry_dt"] = entry_dt
            row["_exit_dt"] = parse_ts(row["exit_timestamp"])
            rows.append(row)
    return rows


def build_sep29(cards):
    journal_rows = load_sep29_journal()
    built = []
    extra_15s = 0
    for row in journal_rows:
        symbol = row["symbol"]
        side = row["side"]
        entry_dt = row["_entry_dt"]
        exit_dt = row["_exit_dt"]
        index = cards.get(symbol)
        entry_price = float(row["entry_price"])
        quantity = float(row["quantity"])
        entry_card = last_card_at_or_before(index, entry_dt) if index else None
        entry_lag = None if entry_card is None else (entry_dt - entry_card["_dt"]).total_seconds()
        window_loose = []
        window_strict = []
        if index:
            window_loose = [
                card for card in index["cards"]
                if entry_dt - timedelta(minutes=10) <= card["_dt"] <= entry_dt + timedelta(seconds=15)
            ]
            window_strict = [card for card in window_loose if card["_dt"] <= entry_dt]
            if any(card["_dt"] > entry_dt for card in window_loose):
                extra_15s += 1
        fresh_15, fresh_30, churn = freshness_from_cards(window_strict, side)
        loose_15, loose_30, loose_churn = freshness_from_cards(window_loose, side)
        holding_prices = []
        if index:
            holding_prices = [
                card["current_price"]
                for card in cards_between(index, entry_dt, exit_dt, start_exclusive=False)
            ]
        if not holding_prices:
            holding_prices = [entry_price, float(row["exit_price"])]
        mfe = lifecycle_mfe(side, entry_price, quantity, holding_prices)
        record = {
            "trade_id": row["trade_id"],
            "symbol": symbol,
            "side": side,
            "t_entry_utc": entry_dt.astimezone(timezone.utc).isoformat(),
            "t_exit_utc": exit_dt.astimezone(timezone.utc).isoformat(),
            "feature_f15_at_entry": row.get("f15_at_entry"),
            "feature_is_fresh": row.get("is_fresh"),
            "feature_pre_freshness_15": fresh_15,
            "feature_pre_freshness_30": fresh_30,
            "feature_state_churn": churn,
            "feature_entry_15m_score": None if entry_card is None else entry_card.get("15m_score_trajectory"),
            "feature_entry_30m_score": None if entry_card is None else entry_card.get("30m_score_trajectory"),
            "feature_entry_1d_score": None if entry_card is None else entry_card.get("1d_score_trajectory"),
            "feature_entry_decision_state": None if entry_card is None else entry_card.get("decision_state"),
            "feature_entry_card_lag_sec": entry_lag,
            "feature_entry_price": entry_price,
            "feature_quantity": quantity,
            "context_entry_minute_ist": entry_dt.astimezone(IST).hour * 60 + entry_dt.astimezone(IST).minute,
            "label_mfe_pnl": round(mfe, 4),
            "label_realized_pnl": round(float(row["pnl"]), 4),
            "label_zero_mfe": int(round(mfe, 4) == 0.0),
            "strict_matches_loose_freshness": (
                fresh_15 == loose_15 and fresh_30 == loose_30 and churn == loose_churn
            ),
        }
        if index:
            for horizon in HORIZONS_MIN:
                horizon_dt = entry_dt + timedelta(minutes=horizon)
                observed = last_card_at_or_before(index, horizon_dt)
                window = cards_between(index, entry_dt, horizon_dt, start_exclusive=False)
                later = cards_between(
                    index,
                    horizon_dt,
                    horizon_dt + timedelta(minutes=POST_HORIZON_MIN),
                    start_exclusive=True,
                )
                if observed is None or observed["_dt"] < entry_dt:
                    record[f"obs_{horizon}m_mtm_pnl"] = None
                    record[f"obs_{horizon}m_mfe_pnl"] = None
                    record[f"obs_{horizon}m_favourable_seen"] = None
                    record[f"obs_{horizon}m_lag_sec"] = None
                else:
                    record[f"obs_{horizon}m_lag_sec"] = round((horizon_dt - observed["_dt"]).total_seconds(), 3)
                    record[f"obs_{horizon}m_mtm_pnl"] = round(
                        signed_pnl(side, entry_price, observed["current_price"], quantity), 4
                    )
                    mfe_h = favourable_mfe(side, entry_price, quantity, window)
                    record[f"obs_{horizon}m_mfe_pnl"] = None if mfe_h is None else round(mfe_h, 4)
                    record[f"obs_{horizon}m_favourable_seen"] = None if mfe_h is None else int(mfe_h > 0.0)
                later_mfe = favourable_mfe(side, entry_price, quantity, later)
                record[f"outcome_{horizon}m_mfe_next_60m"] = None if later_mfe is None else round(later_mfe, 4)
                record[f"outcome_{horizon}m_later_favourable"] = None if later_mfe is None else int(later_mfe > 0.0)
        built.append(record)
    return built, {
        "journal_rows": len(journal_rows),
        "cards_in_open_interval_after_entry_15s": extra_15s,
    }


def population(rows):
    zero = [row for row in rows if int(row["label_zero_mfe"]) == 1]
    positive = [row for row in rows if int(row["label_zero_mfe"]) == 0]
    return {
        "rows": len(rows),
        "zero_mfe": len(zero),
        "positive_mfe": len(positive),
        "zero_mfe_rate": round(len(zero) / len(rows), 4) if rows else None,
        "zero_mfe_realized_pnl": round(sum(float(row["label_realized_pnl"]) for row in zero), 2),
        "positive_mfe_realized_pnl": round(sum(float(row["label_realized_pnl"]) for row in positive), 2),
        "realized_pnl": round(sum(float(row["label_realized_pnl"]) for row in rows), 2),
        "symbols": len({row["symbol"] for row in rows}),
        "f15_present": sum(1 for row in rows if fnum(row.get("feature_f15_at_entry")) is not None),
        "f15_nonzero": sum(1 for row in rows if fnum(row.get("feature_f15_at_entry")) not in (None, 0.0)),
        "entry_card_missing": sum(1 for row in rows if row.get("feature_entry_card_lag_sec") in (None, "")),
    }


def univariate(rows):
    specs = [
        ("F15 at entry", "feature_f15_at_entry", lambda row, raw: -raw),
        ("F30 pre-entry freshness", "feature_pre_freshness_30", lambda row, raw: -raw),
        ("F15 card freshness", "feature_pre_freshness_15", lambda row, raw: -raw),
        ("State churn", "feature_state_churn", lambda row, raw: raw),
        ("Side-aligned 15m score", "feature_entry_15m_score", lambda row, raw: -aligned_score(row, raw)),
        ("Side-aligned 30m score", "feature_entry_30m_score", lambda row, raw: -aligned_score(row, raw)),
        ("Side-aligned 1d score", "feature_entry_1d_score", lambda row, raw: -aligned_score(row, raw)),
    ]
    reports = []
    for name, column, transform in specs:
        kept = []
        labels = []
        scores = []
        raw_zero = []
        raw_positive = []
        for row in rows:
            raw = fnum(row.get(column))
            if raw is None:
                continue
            score = transform(row, raw)
            kept.append(row)
            labels.append(int(row["label_zero_mfe"]))
            scores.append(score)
            if int(row["label_zero_mfe"]) == 1:
                raw_zero.append(raw)
            else:
                raw_positive.append(raw)
        estimate = cluster_auc_ci(kept, labels, scores)
        reports.append({
            "feature": name,
            "column": column,
            "n": len(kept),
            "missing": len(rows) - len(kept),
            "nonzero": sum(1 for value in raw_zero + raw_positive if value != 0.0),
            "median_zero_mfe": None if not raw_zero else round(median(raw_zero), 4),
            "median_positive_mfe": None if not raw_positive else round(median(raw_positive), 4),
            "auc": estimate,
        })
    return reports


def gate_metrics(rows):
    eligible = [row for row in rows if fnum(row.get("feature_f15_at_entry")) is not None]
    flagged = [row for row in eligible if fnum(row["feature_f15_at_entry"]) < FROZEN_F15_GATE]
    zero = [row for row in eligible if int(row["label_zero_mfe"]) == 1]
    positive = [row for row in eligible if int(row["label_zero_mfe"]) == 0]
    zero_flagged = [row for row in flagged if int(row["label_zero_mfe"]) == 1]
    positive_flagged = [row for row in flagged if int(row["label_zero_mfe"]) == 0]
    return {
        "rule": "existing freshness gate, F15 < -0.01",
        "eligible": len(eligible),
        "flagged": len(flagged),
        "precision_zero_mfe": round(len(zero_flagged) / len(flagged), 4) if flagged else None,
        "recall_zero_mfe": round(len(zero_flagged) / len(zero), 4) if zero else None,
        "profitable_movers_incorrectly_filtered": round(len(positive_flagged) / len(positive), 4) if positive else None,
        "base_rate_zero_mfe": round(len(zero) / len(eligible), 4) if eligible else None,
    }


def early_exit(rows):
    reports = []
    for horizon in HORIZONS_MIN:
        seen = f"obs_{horizon}m_favourable_seen"
        later = f"outcome_{horizon}m_later_favourable"
        mtm = f"obs_{horizon}m_mtm_pnl"
        eligible = [
            row for row in rows
            if row.get(seen) not in (None, "") and row.get(later) not in (None, "") and fnum(row.get(mtm)) is not None
        ]
        flagged = [row for row in eligible if int(float(row[seen])) == 0]
        quiet = [row for row in eligible if int(float(row[later])) == 0]
        flagged_quiet = [row for row in flagged if int(float(row[later])) == 0]
        flagged_later = [row for row in flagged if int(float(row[later])) == 1]
        y_true = [1 - int(float(row[later])) for row in eligible]
        scores = [-fnum(row[mtm]) for row in eligible]
        estimate = cluster_auc_ci(eligible, y_true, scores) if eligible else {"auc": None, "ci95": None}
        reports.append({
            "horizon_min": horizon,
            "n_with_post_horizon_price": len(eligible),
            "unobserved_post_horizon": len(rows) - len(eligible),
            "flagged_flat": len(flagged),
            "precision_no_later_favourable": round(len(flagged_quiet) / len(flagged), 4) if flagged else None,
            "recall_no_later_favourable": round(len(flagged_quiet) / len(quiet), 4) if quiet else None,
            "share_of_flat_trades_that_later_move": round(len(flagged_later) / len(flagged), 4) if flagged else None,
            "auc_adverse_mark": estimate,
        })
    return reports


def hour_table(rows):
    buckets = defaultdict(lambda: {"n": 0, "zero": 0})
    for row in rows:
        minute = int(float(row["context_entry_minute_ist"]))
        hour = minute // 60
        buckets[hour]["n"] += 1
        buckets[hour]["zero"] += int(row["label_zero_mfe"])
    table = []
    for hour in sorted(buckets):
        item = buckets[hour]
        table.append({
            "hour_ist": f"{hour:02d}:00",
            "n": item["n"],
            "zero_mfe": item["zero"],
            "zero_mfe_rate": round(item["zero"] / item["n"], 4),
        })
    return table


def audit_holdout_timing(cards):
    with CSV_PATH.open(encoding="utf-8") as handle:
        frozen = list(csv.DictReader(handle))
    compared = 0
    freshness_diff = 0
    churn_diff = 0
    lag_not_zero = 0
    decision_states = Counter()
    for row in frozen:
        decision_states[row.get("feature_entry_decision_state")] += 1
        if fnum(row.get("feature_entry_card_lag_sec")) != 0.0:
            lag_not_zero += 1
        entry_dt = parse_ts(row["t_entry_utc"])
        index = cards.get(row["symbol"])
        if index is None:
            continue
        window = [
            card for card in index["cards"]
            if entry_dt - timedelta(minutes=10) <= card["_dt"] <= entry_dt
        ]
        fresh_15, fresh_30, churn = freshness_from_cards(window, row["side"])
        compared += 1
        stored_30 = fnum(row.get("feature_pre_freshness_30"))
        stored_churn = fnum(row.get("feature_state_churn"))
        if stored_30 is None or fresh_30 is None or abs(stored_30 - fresh_30) > 1e-9:
            freshness_diff += 1
        if stored_churn is None or churn != int(stored_churn):
            churn_diff += 1
    f15_values = [fnum(row["feature_f15_at_entry"]) for row in frozen]
    card_fresh = [fnum(row["feature_pre_freshness_15"]) for row in frozen]
    identical = sum(1 for left, right in zip(f15_values, card_fresh) if left is not None and right is not None and abs(left - right) < 1e-9)
    return {
        "rows": len(frozen),
        "strict_entry_recompute_compared": compared,
        "pre_freshness_30_differs_from_strict_entry": freshness_diff,
        "state_churn_differs_from_strict_entry": churn_diff,
        "entry_card_lag_not_zero": lag_not_zero,
        "decision_states": dict(decision_states),
        "f15_identical_to_card_freshness_15": identical,
        "f15_nonzero": sum(1 for value in f15_values if value not in (None, 0.0)),
        "code_note": (
            "RP-606 builds pre-entry freshness from cards in [entry-10m, entry+15s]. "
            "On this frozen cohort the strict cutoff entry timestamp reproduces the stored values."
        ),
    }


def write_sep29_csv(rows):
    fieldnames = list(rows[0].keys())
    with OUT_CSV.open("w", encoding="utf-8", newline="") as handle:
        writer = csv.DictWriter(handle, fieldnames=fieldnames)
        writer.writeheader()
        writer.writerows(rows)


def markdown(report):
    frozen = report["frozen_holdout"]
    session = report["session_20260929"]
    lines = [
        "# RP-609 NSE Session Replication — 29 September 2026",
        "",
        "**Disposition:** Research only. No promotion. No entry filter.",
        "",
        "The frozen export remains the original 364 trades from 28 September. The closed 29 September NSE Ledger A session is scored separately. A classifier was not fit.",
        "",
        "## Freeze",
        "",
        f"- Freeze ID: `{report['freeze']['freeze_id']}`",
        f"- Frozen CSV SHA256: `{report['freeze']['sha256']['dataset_csv']}`",
        f"- Builder SHA256: `{report['freeze']['sha256']['builder_script']}`",
        f"- Integrity note SHA256: `{report['freeze']['sha256']['integrity_md']}`",
        f"- Population report SHA256: `{report['freeze']['sha256']['frozen_population_snapshot_sha']}`",
        f"- Current snapshot file SHA256: `{report['freeze']['sha256']['current_snapshot_file']}`",
        "",
        "The current snapshot file has grown past the population hash. The builder still selects the original 364 composite IDs.",
        "",
        "## Timing audit on the frozen 364",
        "",
        f"- Entry-card lag not zero: {report['timing_audit']['entry_card_lag_not_zero']}",
        f"- Stored F30 differs from a strict at-or-before-entry recompute: {report['timing_audit']['pre_freshness_30_differs_from_strict_entry']}",
        f"- Stored state churn differs from that recompute: {report['timing_audit']['state_churn_differs_from_strict_entry']}",
        f"- Journal F15 identical to card freshness 15: {report['timing_audit']['f15_identical_to_card_freshness_15']} / {report['timing_audit']['rows']}",
        f"- F15 nonzero, an availability count: {report['timing_audit']['f15_nonzero']}",
        f"- Decision states: {report['timing_audit']['decision_states']}",
        "",
        "Decision state on these entries is BUY or SHORT and repeats the trade side. It is not a regime feature. ATR, volatility, market regime, and signal strength remain absent. `obs_*`, `outcome_*`, and `label_*` are not entry features.",
        "",
        "## Populations",
        "",
        "| Session | Trades | Zero-MFE | Positive-MFE | Zero-MFE P&L | Positive-MFE P&L | Combined P&L |",
        "| :--- | ---: | ---: | ---: | ---: | ---: | ---: |",
    ]
    for label, block in (("28 Sep frozen holdout", frozen["population"]), ("29 Sep closed session", session["population"])):
        lines.append(
            f"| {label} | {block['rows']} | {block['zero_mfe']} | {block['positive_mfe']} | {block['zero_mfe_realized_pnl']:.2f} | {block['positive_mfe_realized_pnl']:.2f} | {block['realized_pnl']:.2f} |"
        )
    lines.extend([
        "",
        f"29 September entries run from {session['entry_span_ist']['first']} to {session['entry_span_ist']['last']}. Exits run through {session['exit_span_ist']['last']}. Cards continue through {session['last_card_ist']}.",
        "",
        "## Entry-time discrimination",
        "",
        "AUC uses a fixed risk direction: staler freshness and weaker side-aligned scores rank as higher zero-MFE risk; higher state churn ranks as higher risk. A 95% interval is a ticker-cluster bootstrap with 1,000 draws and seed 42. An interval that includes 0.50 does not separate the classes.",
        "",
        "| Feature | 28 Sep AUC | 29 Sep AUC |",
        "| :--- | :--- | :--- |",
    ])
    later_by_name = {item["feature"]: item for item in session["univariate"]}
    for item in frozen["univariate"]:
        other = later_by_name[item["feature"]]
        def cell(block):
            estimate = block["auc"]
            ci = estimate.get("ci95")
            ci_text = "n/a" if not ci else f"[{ci[0]:.3f}, {ci[1]:.3f}]"
            return f"{estimate['auc']:.3f} {ci_text} (n={block['n']})"
        lines.append(f"| {item['feature']} | {cell(item)} | {cell(other)} |")
    lines.extend([
        "",
        "F15 nonzero counts are availability, not predictive evidence: "
        f"{frozen['population']['f15_nonzero']} of {frozen['population']['rows']} on 28 Sep, "
        f"{session['population']['f15_nonzero']} of {session['population']['rows']} on 29 Sep.",
        "",
        "## Existing F15 gate, not a new threshold",
        "",
        "| Session | Flagged | Precision | Recall | Positive-MFE removed | Base rate |",
        "| :--- | ---: | ---: | ---: | ---: | ---: |",
    ])
    for label, block in (("28 Sep", frozen["frozen_f15_gate"]), ("29 Sep", session["frozen_f15_gate"])):
        lines.append(
            f"| {label} | {block['flagged']} / {block['eligible']} | {block['precision_zero_mfe']} | {block['recall_zero_mfe']} | {block['profitable_movers_incorrectly_filtered']} | {block['base_rate_zero_mfe']} |"
        )
    lines.extend([
        "",
        "## Clock time by session",
        "",
        "These rates describe each session. They are not a filter.",
        "",
        "| Hour IST | 28 Sep trades | 28 Sep zero-MFE rate | 29 Sep trades | 29 Sep zero-MFE rate |",
        "| :--- | ---: | ---: | ---: | ---: |",
    ])
    hours = sorted(set([row["hour_ist"] for row in frozen["hour_table"]] + [row["hour_ist"] for row in session["hour_table"]]))
    frozen_hours = {row["hour_ist"]: row for row in frozen["hour_table"]}
    session_hours = {row["hour_ist"]: row for row in session["hour_table"]}
    for hour in hours:
        left = frozen_hours.get(hour)
        right = session_hours.get(hour)
        left_n = "" if left is None else str(left["n"])
        left_rate = "" if left is None else f"{left['zero_mfe_rate']:.3f}"
        right_n = "" if right is None else str(right["n"])
        right_rate = "" if right is None else f"{right['zero_mfe_rate']:.3f}"
        lines.append(f"| {hour} | {left_n} | {left_rate} | {right_n} | {right_rate} |")
    lines.extend([
        "",
        "## Early-horizon observations on 29 September",
        "",
        "This block uses prices at or before the horizon and the next 60 minutes after it. A missing post-horizon price, including after the cash close, stays unobserved. Lifecycle zero-MFE is not an input.",
        "",
        "| Horizon | Observed | Unobserved | Flat trades that later move | Precision | AUC of adverse mark |",
        "| :--- | ---: | ---: | ---: | ---: | :--- |",
    ])
    for item in session["early_exit"]:
        estimate = item["auc_adverse_mark"]
        ci = estimate.get("ci95")
        auc_text = "n/a" if estimate.get("auc") is None else f"{estimate['auc']:.3f}"
        if ci:
            auc_text += f" [{ci[0]:.3f}, {ci[1]:.3f}]"
        lines.append(
            f"| {item['horizon_min']}m | {item['n_with_post_horizon_price']} | {item['unobserved_post_horizon']} | {item['share_of_flat_trades_that_later_move']} | {item['precision_no_later_favourable']} | {auc_text} |"
        )
    lines.extend([
        "",
        f"Cards inside the 15 seconds after entry on 29 September: {session['timing']['cards_in_open_interval_after_entry_15s']}.",
        "",
        "No entry mechanism is authorized from these two sessions.",
        "",
    ])
    return "\n".join(lines)


def holdout_rows_from_csv():
    with CSV_PATH.open(encoding="utf-8") as handle:
        return list(csv.DictReader(handle))


def main():
    subprocess.check_call([sys.executable, str(BUILDER)], cwd=WORKSPACE)
    cards = load_cards()
    sep_rows, sep_meta = build_sep29(cards)
    write_sep29_csv(sep_rows)
    frozen_rows = holdout_rows_from_csv()
    if len(frozen_rows) != 364:
        raise SystemExit(f"Frozen export has {len(frozen_rows)} rows")
    entries = [parse_ts(row["t_entry_utc"]) for row in sep_rows]
    exits = [parse_ts(row["t_exit_utc"]) for row in sep_rows]
    last_card = max(card["_dt"] for seq in cards.values() for card in seq["cards"] if card["_dt"].date().isoformat() == SESSION_DAY)
    report_sha = json.loads(REPORT.read_text(encoding="utf-8")).get("snapshot_sha256")
    if report_sha != FROZEN_SNAPSHOT_SHA:
        raise SystemExit("Population report hash changed")
    freeze = {
        "freeze_id": FREEZE_ID,
        "population": "NSE Ledger A frozen Stage 2 holdout, 364 composite IDs, 2026-09-28",
        "sep29_included_in_frozen_csv": False,
        "sha256": {
            "dataset_csv": sha256(CSV_PATH),
            "integrity_md": sha256(INTEGRITY_MD),
            "integrity_json": sha256(INTEGRITY_JSON),
            "builder_script": sha256(BUILDER),
            "current_snapshot_file": sha256(SNAPSHOT),
            "frozen_population_snapshot_sha": FROZEN_SNAPSHOT_SHA,
        },
        "bootstrap": {"replicates": BOOTSTRAPS, "seed": BOOTSTRAP_SEED, "cluster": "symbol"},
    }
    report = {
        "disposition": "RESEARCH ONLY — no promotion",
        "freeze": freeze,
        "timing_audit": audit_holdout_timing(cards),
        "frozen_holdout": {
            "population": population(frozen_rows),
            "univariate": univariate(frozen_rows),
            "frozen_f15_gate": gate_metrics(frozen_rows),
            "hour_table": hour_table(frozen_rows),
        },
        "session_20260929": {
            "session": "NSE Ledger A baseline, closed cash session",
            "entry_span_ist": {
                "first": min(entries).astimezone(IST).isoformat(),
                "last": max(entries).astimezone(IST).isoformat(),
            },
            "exit_span_ist": {
                "first": min(exits).astimezone(IST).isoformat(),
                "last": max(exits).astimezone(IST).isoformat(),
            },
            "last_card_ist": last_card.astimezone(IST).isoformat(),
            "timing": sep_meta,
            "population": population(sep_rows),
            "univariate": univariate(sep_rows),
            "frozen_f15_gate": gate_metrics(sep_rows),
            "hour_table": hour_table(sep_rows),
            "early_exit": early_exit(sep_rows),
            "dataset_csv": str(OUT_CSV.relative_to(WORKSPACE)),
        },
    }
    MANIFEST.write_text(json.dumps(freeze, indent=2), encoding="utf-8")
    OUT_JSON.write_text(json.dumps(report, indent=2), encoding="utf-8")
    OUT_MD.write_text(markdown(report), encoding="utf-8")
    print(json.dumps({
        "freeze_csv_sha256": freeze["sha256"]["dataset_csv"],
        "frozen_population": report["frozen_holdout"]["population"],
        "sep29_population": report["session_20260929"]["population"],
        "sep29_gate": report["session_20260929"]["frozen_f15_gate"],
        "sep29_auc": [
            {"feature": item["feature"], "auc": item["auc"]["auc"], "ci": item["auc"]["ci95"]}
            for item in report["session_20260929"]["univariate"]
        ],
        "sep29_early": report["session_20260929"]["early_exit"],
        "timing": report["timing_audit"],
    }, indent=2))


if __name__ == "__main__":
    main()
