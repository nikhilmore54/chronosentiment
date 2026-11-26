#!/usr/bin/env python3
"""
Build the RP-609 entry-time discrimination dataset for the Stage 2 NSE holdout.

Governance
- Exploratory export only. No entry filter, no candidate promotion.
- RP-605 Ledger A stays the frozen control. This script only reads it.
- PFT, SPL, VAT, and PPR are not modified.
- Zero-MFE is a retrospective label. It is not an entry feature.
- Entry discrimination and early-exit discrimination are separate column groups.
- Horizon columns use decision-card prices at or before the horizon.
  They do not use the milestone trajectory (Entry, Initial Move, Peak MFE).

Population
- The 364 composite IDs in the frozen temporal-validation report
  (snapshot SHA 0190c56603ce59b9..., baseline realized P&L -5188.90).
- The live snapshot has since been appended. Rows after that frozen
  slice are not part of this holdout.

Join
- Journal rows are matched on (trade_id, symbol, side, entry timestamp).
- The dashboard sample in rp605_nse_live_shadow_results.json is not used.
"""

from __future__ import annotations

import csv
import hashlib
import json
from bisect import bisect_right
from collections import defaultdict
from datetime import datetime, timedelta, timezone
from pathlib import Path

WORKSPACE = Path(__file__).resolve().parent.parent
LOG_DIR = WORKSPACE / "live_capture" / "shadow_validation"
SNAPSHOT_PATH = LOG_DIR / "rp606_rp607_accumulating_evidence_snapshot.json"
JOURNAL_PATH = LOG_DIR / "rp605_nse_baseline_journal.jsonl"
CARDS_PATH = LOG_DIR / "rp605_nse_decision_cards_log.jsonl"
HOLDOUT_REPORT_PATH = LOG_DIR / "rp609_temporal_validation_report.json"

OUT_CSV = LOG_DIR / "rp609_entry_time_discrimination_dataset.csv"
OUT_JSON = LOG_DIR / "rp609_entry_time_discrimination_integrity.json"
OUT_MD = LOG_DIR / "rp609_entry_time_discrimination_integrity.md"

FROZEN_SNAPSHOT_SHA = "0190c56603ce59b9db2f695b536a751ac70371e010621c04633e2366982bb0b5"
EXPECTED_N = 364
EXPECTED_ZERO_MFE = 139
EXPECTED_REALIZED = -5188.90
EXPECTED_ZERO_PNL = -13189.66
EXPECTED_POS_PNL = 8000.76
HORIZONS_MIN = (5, 10, 15)
POST_HORIZON_MIN = 60
IST = timezone(timedelta(hours=5, minutes=30))

ABSENT_ENTRY_FEATURES = (
    "feature_atr_at_entry",
    "feature_volatility_at_entry",
    "feature_regime_at_entry",
    "feature_signal_strength_at_entry",
)


def parse_ts(value: str) -> datetime:
    return datetime.fromisoformat(value.replace("Z", "+00:00"))


def iso_utc(dt: datetime) -> str:
    return dt.astimezone(timezone.utc).isoformat()


def round4(value):
    if value is None:
        return None
    return round(float(value), 4)


def signed_pnl(side: str, entry_price: float, price: float, quantity: float) -> float:
    if side == "LONG":
        return (price - entry_price) * quantity
    return (entry_price - price) * quantity


def load_snapshot():
    raw = SNAPSHOT_PATH.read_bytes()
    data = json.loads(raw.decode("utf-8"))
    return data, hashlib.sha256(raw).hexdigest()


def load_journal():
    grouped = defaultdict(list)
    with JOURNAL_PATH.open(encoding="utf-8") as handle:
        for line in handle:
            line = line.strip()
            if not line:
                continue
            row = json.loads(line)
            row["_entry_dt"] = parse_ts(row["entry_timestamp"])
            grouped[row["trade_id"]].append(row)
    return grouped


def load_cards():
    by_symbol = defaultdict(list)
    with CARDS_PATH.open(encoding="utf-8") as handle:
        for line in handle:
            line = line.strip()
            if not line:
                continue
            card = json.loads(line)
            card["_dt"] = parse_ts(card["timestamp_utc"])
            by_symbol[card["instrument"]].append(card)
    indexes = {}
    for symbol, cards in by_symbol.items():
        cards.sort(key=lambda card: card["_dt"])
        indexes[symbol] = {
            "cards": cards,
            "times": [card["_dt"] for card in cards],
        }
    return indexes


def last_card_at_or_before(index, target: datetime):
    times = index["times"]
    pos = bisect_right(times, target) - 1
    if pos < 0:
        return None
    return index["cards"][pos]


def cards_between(index, start: datetime, end: datetime, start_exclusive: bool = False):
    times = index["times"]
    cards = index["cards"]
    left = bisect_right(times, start) if start_exclusive else bisect_right(times, start - timedelta(microseconds=1))
    right = bisect_right(times, end)
    return cards[left:right]


def composite_key(record):
    return (record["symbol"], record["side"], record["t_entry"], record["trade_id"])


def load_frozen_holdout(snapshot):
    """Return the frozen 364-trade holdout in report order.

    The current snapshot is a later append. Selecting 'everything after
    row 506' would pull in trades that were not in the forensic profile.
    """
    if not HOLDOUT_REPORT_PATH.exists():
        raise FileNotFoundError(f"Frozen holdout report missing: {HOLDOUT_REPORT_PATH}")
    report = json.loads(HOLDOUT_REPORT_PATH.read_text(encoding="utf-8"))
    if report.get("snapshot_sha256") != FROZEN_SNAPSHOT_SHA:
        raise RuntimeError("Holdout report is not the frozen RP-609 snapshot")
    trades = report["candidate_replay_results"]["PPR-v1.0-FROZEN"]["trade_level_results"]
    nse_a = [
        row for row in snapshot.get("rp607_records", [])
        if row.get("session") == "NSE Ledger A (Baseline)"
    ]
    by_key = {composite_key(row): row for row in nse_a}
    selected = []
    missing = []
    pnl_mismatches = []
    for trade in trades:
        key = tuple(trade["composite_id"])
        record = by_key.get(key)
        if record is None:
            missing.append(key[3])
            continue
        if abs(float(record["realized_pnl"]) - float(trade["baseline_realized_pnl"])) > 0.02:
            pnl_mismatches.append(key[3])
        if abs(float(record["mfe_pnl"]) - float(trade["mfe_pnl"])) > 0.02:
            pnl_mismatches.append(key[3])
        selected.append(record)
    return nse_a, selected, missing, pnl_mismatches


def rp606_index(snapshot):
    indexed = defaultdict(list)
    for row in snapshot.get("rp606_records", []):
        if row.get("session") != "NSE Ledger A (Baseline)":
            continue
        indexed[row["trade_id"]].append(row)
    return indexed


def favourable_mfe(side, entry_price, quantity, cards):
    if not cards:
        return None
    peak = max(signed_pnl(side, entry_price, card["current_price"], quantity) for card in cards)
    return max(0.0, peak)


def materialize_records(records, journal=None, freshness=None, cards=None):
    """Join entry features and fixed-horizon prices for an explicit record list."""
    if journal is None or freshness is None or cards is None:
        snapshot, _snapshot_sha = load_snapshot()
        journal = load_journal() if journal is None else journal
        freshness = rp606_index(snapshot) if freshness is None else freshness
        cards = load_cards() if cards is None else cards

    rows = []
    integrity = {
        "journal_trade_ids": len(journal),
        "journal_rows": sum(len(v) for v in journal.values()),
        "duplicate_journal_ids": sum(1 for rows_ in journal.values() if len(rows_) > 1),
        "identity_mismatches": [],
        "ambiguous_journal_matches": [],
        "missing_journal": [],
        "missing_rp606": [],
        "ambiguous_rp606": [],
        "missing_entry_card": [],
        "horizon_unobserved": {str(h): 0 for h in HORIZONS_MIN},
        "post_horizon_unobserved": {str(h): 0 for h in HORIZONS_MIN},
    }

    for record in records:
        trade_id = record["trade_id"]
        symbol = record["symbol"]
        side = record["side"]
        entry_dt = parse_ts(record["t_entry"])
        exit_dt = parse_ts(record["t_exit"]) if record.get("t_exit") else None
        composite = composite_key(record)

        candidates = [
            row for row in journal.get(trade_id, [])
            if row["symbol"] == symbol
            and row["side"] == side
            and row["_entry_dt"] == entry_dt
        ]
        if not candidates:
            integrity["missing_journal"].append(trade_id)
            journal_row = None
        elif len(candidates) > 1:
            integrity["ambiguous_journal_matches"].append(trade_id)
            journal_row = candidates[0]
        else:
            journal_row = candidates[0]

        freshness_matches = [
            row for row in freshness.get(trade_id, [])
            if row.get("symbol") == symbol
            and row.get("side") == side
            and parse_ts(row["entry_dt"]) == entry_dt
        ]
        if not freshness_matches:
            integrity["missing_rp606"].append(trade_id)
            freshness_row = None
        elif len(freshness_matches) > 1:
            integrity["ambiguous_rp606"].append(trade_id)
            freshness_row = freshness_matches[0]
        else:
            freshness_row = freshness_matches[0]

        index = cards.get(symbol)
        entry_card = last_card_at_or_before(index, entry_dt) if index else None
        entry_lag = None
        if entry_card is None:
            integrity["missing_entry_card"].append(trade_id)
        else:
            entry_lag = (entry_dt - entry_card["_dt"]).total_seconds()
            if entry_card["_dt"] > entry_dt:
                integrity["identity_mismatches"].append(trade_id)

        entry_price = float(journal_row["entry_price"]) if journal_row else None
        quantity = float(journal_row["quantity"]) if journal_row else None

        row = {
            "trade_id": trade_id,
            "symbol": symbol,
            "side": side,
            "t_entry_utc": iso_utc(entry_dt),
            "t_exit_utc": iso_utc(exit_dt) if exit_dt else None,
            "composite_key": "|".join(composite),
            "feature_f15_at_entry": journal_row.get("f15_at_entry") if journal_row else None,
            "feature_is_fresh": journal_row.get("is_fresh") if journal_row else None,
            "feature_pre_freshness_15": None if not freshness_row else freshness_row.get("pre_freshness_15"),
            "feature_pre_freshness_30": None if not freshness_row else freshness_row.get("pre_freshness_30"),
            "feature_state_churn": None if not freshness_row else freshness_row.get("state_churn"),
            "feature_entry_15m_score": None if not entry_card else entry_card.get("15m_score_trajectory"),
            "feature_entry_30m_score": None if not entry_card else entry_card.get("30m_score_trajectory"),
            "feature_entry_1d_score": None if not entry_card else entry_card.get("1d_score_trajectory"),
            "feature_entry_cross_resolution": None if not entry_card else entry_card.get("cross_resolution_relationship"),
            "feature_entry_decision_state": None if not entry_card else entry_card.get("decision_state"),
            "feature_entry_card_lag_sec": entry_lag,
            "feature_entry_price": entry_price,
            "feature_quantity": quantity,
            "context_entry_minute_ist": entry_dt.astimezone(IST).hour * 60 + entry_dt.astimezone(IST).minute,
            "recon_journal_pnl": None if not journal_row else round4(journal_row.get("pnl")),
            "recon_journal_copies": len(journal.get(trade_id, [])),
            "label_mfe_pnl": record.get("mfe_pnl"),
            "label_mae_pnl": record.get("mae_pnl"),
            "label_realized_pnl": record.get("realized_pnl"),
            "label_zero_mfe": int(float(record.get("mfe_pnl") or 0.0) == 0.0),
            "label_holding_duration_mins": record.get("holding_duration_mins"),
            "label_post_exit_mark_pnl_15m": record.get("pnl_plus_15m"),
            "label_post_exit_mark_pnl_30m": record.get("pnl_plus_30m"),
            "label_post_exit_mark_pnl_60m": record.get("pnl_plus_60m"),
            "label_diagnostic_bucket": record.get("diagnostic_bucket"),
        }
        for name in ABSENT_ENTRY_FEATURES:
            row[name] = None

        if index and entry_price is not None and quantity is not None:
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
                    integrity["horizon_unobserved"][str(horizon)] += 1
                    row[f"obs_{horizon}m_price"] = None
                    row[f"obs_{horizon}m_lag_sec"] = None
                    row[f"obs_{horizon}m_mtm_pnl"] = None
                else:
                    row[f"obs_{horizon}m_price"] = observed["current_price"]
                    row[f"obs_{horizon}m_lag_sec"] = round((horizon_dt - observed["_dt"]).total_seconds(), 3)
                    row[f"obs_{horizon}m_mtm_pnl"] = round4(
                        signed_pnl(side, entry_price, observed["current_price"], quantity)
                    )
                mfe_h = favourable_mfe(side, entry_price, quantity, window)
                row[f"obs_{horizon}m_mfe_pnl"] = round4(mfe_h)
                row[f"obs_{horizon}m_favourable_seen"] = None if mfe_h is None else int(mfe_h > 0.0)
                later_mfe = favourable_mfe(side, entry_price, quantity, later)
                if later_mfe is None:
                    integrity["post_horizon_unobserved"][str(horizon)] += 1
                row[f"outcome_{horizon}m_mfe_next_60m"] = round4(later_mfe)
                row[f"outcome_{horizon}m_later_favourable"] = None if later_mfe is None else int(later_mfe > 0.0)
        else:
            for horizon in HORIZONS_MIN:
                integrity["horizon_unobserved"][str(horizon)] += 1
                integrity["post_horizon_unobserved"][str(horizon)] += 1
                for suffix in (
                    "price", "lag_sec", "mtm_pnl", "mfe_pnl", "favourable_seen",
                ):
                    row[f"obs_{horizon}m_{suffix}"] = None
                row[f"outcome_{horizon}m_mfe_next_60m"] = None
                row[f"outcome_{horizon}m_later_favourable"] = None

        rows.append(row)

    return rows, integrity


def build_rows():
    snapshot, snapshot_sha = load_snapshot()
    nse_a, stage2, missing_holdout, pnl_mismatches = load_frozen_holdout(snapshot)
    freshness = rp606_index(snapshot)
    journal = load_journal()
    cards = load_cards()
    rows, integrity = materialize_records(stage2, journal, freshness, cards)
    integrity["population_nse_ledger_a_current_snapshot"] = len(nse_a)
    integrity["population_stage2"] = len(stage2)
    integrity["missing_frozen_ids"] = missing_holdout
    integrity["snapshot_vs_report_pnl_mismatches"] = pnl_mismatches
    summary = summarize(rows, snapshot_sha, integrity)
    return rows, summary


def summarize(rows, snapshot_sha, integrity):
    realized = [float(row["label_realized_pnl"] or 0.0) for row in rows]
    zero = [row for row in rows if row["label_zero_mfe"] == 1]
    positive = [row for row in rows if row["label_zero_mfe"] == 0]
    zero_pnl = sum(float(row["label_realized_pnl"] or 0.0) for row in zero)
    pos_pnl = sum(float(row["label_realized_pnl"] or 0.0) for row in positive)

    def missing_rate(column):
        missing = sum(1 for row in rows if row.get(column) is None or row.get(column) == "")
        return {"missing": missing, "present": len(rows) - missing}

    feature_columns = [
        key for key in rows[0].keys()
        if key.startswith("feature_")
    ] if rows else []
    availability = {column: missing_rate(column) for column in feature_columns}

    f15_values = [row["feature_f15_at_entry"] for row in rows if row["feature_f15_at_entry"] is not None]
    nonzero_f15 = sum(1 for value in f15_values if float(value) != 0.0)
    entry_lags = [float(row["feature_entry_card_lag_sec"]) for row in rows if row["feature_entry_card_lag_sec"] is not None]
    horizon_descriptive = {}
    for horizon in HORIZONS_MIN:
        seen_col = f"obs_{horizon}m_favourable_seen"
        later_col = f"outcome_{horizon}m_later_favourable"
        zero_seen = sum(1 for row in zero if row.get(seen_col) == 1)
        zero_later = sum(1 for row in zero if row.get(later_col) == 1)
        pos_unseen = sum(1 for row in positive if row.get(seen_col) == 0)
        pos_unseen_later = sum(
            1 for row in positive
            if row.get(seen_col) == 0 and row.get(later_col) == 1
        )
        horizon_descriptive[f"{horizon}m"] = {
            "zero_mfe_label_but_favourable_by_horizon": zero_seen,
            "zero_mfe_label_favourable_in_next_60m": zero_later,
            "positive_mfe_label_still_flat_at_horizon": pos_unseen,
            "flat_at_horizon_but_favourable_in_next_60m_among_positive_mfe": pos_unseen_later,
        }

    return {
        "artifact": "RP-609 entry-time discrimination dataset",
        "disposition": "EXPLORATORY — no promotion",
        "governance": {
            "rp605": "frozen control, read only",
            "rp608": "shadow only, unchanged",
            "pft_spl_vat_ppr": "unchanged",
            "entry_filter": "not designed",
            "time_of_day": "context column only; not a filter",
        },
        "population": {
            "definition": (
                "Frozen Stage 2 NSE holdout from temporal-validation report "
                f"SHA {FROZEN_SNAPSHOT_SHA}. Current snapshot rows appended "
                "after that slice are excluded."
            ),
            "rows": len(rows),
            "unique_trade_ids": len({row["trade_id"] for row in rows}),
            "zero_mfe_trades": len(zero),
            "positive_mfe_trades": len(positive),
            "realized_pnl": round(sum(realized), 2),
            "zero_mfe_realized_pnl": round(zero_pnl, 2),
            "positive_mfe_realized_pnl": round(pos_pnl, 2),
        },
        "reconciliation_checks": {
            "row_count_is_364": len(rows) == EXPECTED_N,
            "zero_mfe_count_is_139": len(zero) == EXPECTED_ZERO_MFE,
            "realized_pnl_matches_baseline": abs(sum(realized) - EXPECTED_REALIZED) < 0.05,
            "zero_mfe_pnl_matches": abs(zero_pnl - EXPECTED_ZERO_PNL) < 0.05,
            "positive_mfe_pnl_matches": abs(pos_pnl - EXPECTED_POS_PNL) < 0.05,
            "cohorts_sum_to_baseline": abs((zero_pnl + pos_pnl) - sum(realized)) < 0.001,
            "missing_frozen_ids": len(integrity["missing_frozen_ids"]),
            "snapshot_vs_report_pnl_mismatches": len(integrity["snapshot_vs_report_pnl_mismatches"]),
            "missing_journal": len(integrity["missing_journal"]),
            "ambiguous_journal_matches": len(integrity["ambiguous_journal_matches"]),
            "missing_rp606": len(integrity["missing_rp606"]),
            "ambiguous_rp606": len(integrity["ambiguous_rp606"]),
            "missing_entry_card": len(integrity["missing_entry_card"]),
        },
        "sources": {
            "snapshot": str(SNAPSHOT_PATH.relative_to(WORKSPACE)),
            "snapshot_sha256": snapshot_sha,
            "journal": str(JOURNAL_PATH.relative_to(WORKSPACE)),
            "decision_cards": str(CARDS_PATH.relative_to(WORKSPACE)),
            "journal_duplicate_policy": (
                "Match journal rows on trade_id + symbol + side + entry timestamp. "
                "Duplicate trade ids exist because the journal was appended more than once; "
                "copies that do not match the holdout timestamp are not used."
            ),
        },
        "join_integrity": {
            "journal_rows": integrity["journal_rows"],
            "journal_trade_ids": integrity["journal_trade_ids"],
            "duplicate_journal_ids": integrity["duplicate_journal_ids"],
            "horizon_unobserved": integrity["horizon_unobserved"],
            "post_horizon_unobserved_next_60m": integrity["post_horizon_unobserved"],
        },
        "feature_availability": availability,
        "f15_nonzero_count": nonzero_f15,
        "entry_card_lag_sec_max": max(entry_lags) if entry_lags else None,
        "horizon_path_counts": horizon_descriptive,
        "absent_by_design": {
            "feature_atr_at_entry": "No ATR series is stored on the journal or the decision card.",
            "feature_volatility_at_entry": "No volatility measure is stored at entry.",
            "feature_regime_at_entry": (
                "No volatility or market regime is stored. "
                "feature_entry_decision_state is the Coralys card state and is not a regime label."
            ),
            "feature_signal_strength_at_entry": (
                "No field is named signal strength. "
                "15m, 30m, and 1d scores are kept under their own names."
            ),
        },
        "experiments": {
            "entry_discrimination": {
                "features": "columns prefixed feature_",
                "label": "label_zero_mfe, known only after the holding path is observed",
                "model_fit": "not run",
            },
            "early_exit_discrimination": {
                "observations": "columns prefixed obs_{5,10,15}m_, using cards with timestamp <= entry+h",
                "outcomes": "columns prefixed outcome_{5,10,15}m_, using cards strictly after the horizon through +60m",
                "excluded_inputs": "Initial Move, Peak MFE, and label_zero_mfe are not horizon features",
                "model_fit": "not run",
            },
        },
        "chronological_validation": {
            "status": "not run",
            "reason": "The 364 holdout rows are one NSE session. A later unseen session is not in this export.",
        },
        "leakage_controls": [
            "label_ and outcome_ columns are retrospective relative to entry.",
            "obs_* columns ignore cards timestamped after the horizon.",
            "Milestone trajectory points from the RP-609 replay are not copied into this file.",
            "Post-exit marks (label_post_exit_mark_pnl_*) are the RP-607 exit-clock marks, not the post-entry horizons.",
        ],
    }


def write_csv(rows):
    fieldnames = list(rows[0].keys())
    with OUT_CSV.open("w", encoding="utf-8", newline="") as handle:
        writer = csv.DictWriter(handle, fieldnames=fieldnames)
        writer.writeheader()
        writer.writerows(rows)


def write_markdown(summary):
    population = summary["population"]
    checks = summary["reconciliation_checks"]
    lines = [
        "# RP-609 Entry-Time Discrimination Dataset",
        "",
        "**Disposition:** Exploratory. No promotion. No entry filter.",
        "",
        "Aligned with the frozen RP-605 control and the RP-609 separation of profit retention from entry quality. Deterministic read of the existing snapshot, journal, and decision cards. Same inputs reproduce this file.",
        "",
        "## Population reconciliation",
        "",
        f"- Rows: {population['rows']}",
        f"- Zero-MFE label: {population['zero_mfe_trades']} trades, realized {population['zero_mfe_realized_pnl']:.2f}",
        f"- Positive-MFE label: {population['positive_mfe_trades']} trades, realized {population['positive_mfe_realized_pnl']:.2f}",
        f"- Combined realized P&L: {population['realized_pnl']:.2f}",
        "",
        "## Checks",
        "",
    ]
    for name, value in checks.items():
        lines.append(f"- `{name}`: {value}")
    lines.extend([
        "",
        "## What is joined",
        "",
        "- `feature_f15_at_entry` and `feature_is_fresh` come from the full NSE baseline journal, matched on trade id, symbol, side, and entry timestamp.",
        "- `feature_pre_freshness_15`, `feature_pre_freshness_30`, and `feature_state_churn` come from the RP-606 pre-entry card summary.",
        "- Entry-card scores, cross-resolution, and decision state are the card at or before entry.",
        "- `obs_5m_*`, `obs_10m_*`, and `obs_15m_*` use decision-card prices at or before entry plus that many minutes.",
        "- `outcome_*_mfe_next_60m` uses only cards after that horizon, through 60 minutes later.",
        "",
        "## Clock-time path counts",
        "",
        "These counts describe the price path. They are not a fitted rule. A lifecycle zero-MFE label can still show a favourable price after the baseline exit, because the horizon clock continues.",
        "",
    ])
    for horizon, counts in summary["horizon_path_counts"].items():
        lines.append(f"- `{horizon}`: {json.dumps(counts)}")
    lines.extend([
        "",
        "## What is still missing",
        "",
        "- ATR at entry.",
        "- A volatility measure at entry.",
        "- A market-regime label. Decision state is not that label.",
        "- A field named signal strength. The 15m, 30m, and 1d scores stay under their own names.",
        "- A later trading session, so chronological validation is not available from this holdout alone.",
        "",
        "## Experiments kept separate",
        "",
        "- Entry discrimination may use `feature_*` to study `label_zero_mfe`. That label is retrospective.",
        "- Early-exit discrimination may use `obs_*` at a fixed horizon to study `outcome_*`. It must not use Initial Move or Peak MFE.",
        "- `context_entry_minute_ist` records the clock time. It is not a filter.",
        "",
        "No classifier was fit.",
        "",
    ])
    OUT_MD.write_text("\n".join(lines), encoding="utf-8")


def main():
    rows, summary = build_rows()
    failed = []
    for name, value in summary["reconciliation_checks"].items():
        if value is False or (type(value) is int and value != 0):
            failed.append(f"{name}={value}")
    write_csv(rows)
    OUT_JSON.write_text(json.dumps(summary, indent=2), encoding="utf-8")
    write_markdown(summary)
    print(json.dumps({
        "csv": str(OUT_CSV),
        "rows": summary["population"]["rows"],
        "checks": summary["reconciliation_checks"],
        "failed": failed,
        "horizon_unobserved": summary["join_integrity"]["horizon_unobserved"],
        "f15_nonzero": summary["f15_nonzero_count"],
    }, indent=2))
    if failed:
        raise SystemExit(1)


if __name__ == "__main__":
    main()
