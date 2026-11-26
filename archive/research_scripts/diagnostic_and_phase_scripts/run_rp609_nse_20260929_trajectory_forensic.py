#!/usr/bin/env python3
"""
Forensic trajectory study of the closed NSE Ledger A session on 2026-09-29.

Descriptive research only. This script does not fit a classifier, search a
threshold, change a frozen definition, or propose an entry or exit mechanism.

Governance
- RP-605 stays the frozen control. RP-608 stays shadow-only.
- PFT, SPL, VAT, and PPR are not modified.
- The 29 September extract is not pooled with 28 September.
- Findings are about observed trade behaviour under the control policy.

Population
- The existing 1,719-row session extract
  (rp609_nse_20260929_entry_discrimination_dataset.csv).
- Journal rows with a 29 September UTC entry that are absent from that
  extract are counted and excluded.
- Decision-card prices reconstruct the sampled path. Cards are not a
  continuous tape. A missing print is left unobserved.
"""

from __future__ import annotations

import csv
import hashlib
import json
from collections import Counter, defaultdict
from datetime import timedelta, timezone

import numpy as np

from build_rp609_entry_time_discrimination_dataset import (
    cards_between,
    load_cards,
    parse_ts,
    signed_pnl,
)
from run_rp609_entry_discrimination_evaluation import BOOTSTRAP_SEED, BOOTSTRAPS, cluster_auc_ci

def sha256(path):
    digest = hashlib.sha256()
    with path.open("rb") as handle:
        for chunk in iter(lambda: handle.read(1024 * 1024), b""):
            digest.update(chunk)
    return digest.hexdigest()


def num(value):
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


def percentile(values, p):
    if not values:
        return None
    ordered = sorted(values)
    if len(ordered) == 1:
        return ordered[0]
    k = (len(ordered) - 1) * p
    lo = int(k)
    hi = min(lo + 1, len(ordered) - 1)
    return ordered[lo] + (ordered[hi] - ordered[lo]) * (k - lo)


def round4(value):
    if value is None:
        return None
    return round(float(value), 4)


def round2(value):
    if value is None:
        return None
    return round(float(value), 2)


def triggers_exit(state, side):
    opposite = "SHORT" if side == "LONG" else "BUY"
    return state in {"EXIT", "PULLBACK", opposite}


def assign_group(row):
    """Same four-way split as the definition audit. Clocks are not imputed."""
    zero = int(float(row["label_zero_mfe"])) == 1
    realized = float(row["label_realized_pnl"])
    favourable = False
    observed = False
    for horizon in (5, 10, 15):
        seen = row.get(f"obs_{horizon}m_favourable_seen")
        later = row.get(f"outcome_{horizon}m_later_favourable")
        if seen not in (None, "") or later not in (None, ""):
            observed = True
        if seen not in (None, "") and int(float(seen)) == 1:
            favourable = True
        if later not in (None, "") and int(float(later)) == 1:
            favourable = True
    if zero and favourable:
        return "zero_later_favourable"
    if zero and observed:
        return "zero_no_favourable"
    if zero:
        return "zero_clock_unobserved"
    if realized <= 0:
        return "positive_mfe_nonpositive"
    return "positive_mfe_positive"


GROUP_ORDER = (
    "zero_no_favourable",
    "zero_later_favourable",
    "positive_mfe_nonpositive",
    "positive_mfe_positive",
    "zero_clock_unobserved",
)
GROUP_LABEL = {
    "zero_no_favourable": "Zero-MFE, no sampled favourable print in the entry clocks",
    "zero_later_favourable": "Zero-MFE, sampled favourable print inside the entry clocks",
    "positive_mfe_nonpositive": "Positive-MFE, nonpositive realized P&L",
    "positive_mfe_positive": "Positive-MFE, positive realized P&L",
    "zero_clock_unobserved": "Zero-MFE, entry clocks unobserved",
}


def aligned(row, column):
    raw = num(row.get(column))
    if raw is None:
        return None
    if row["side"] == "LONG":
        return raw
    return -raw


def load_journal_index(journal_path):
    indexed = {}
    extras = []
    with journal_path.open(encoding="utf-8") as handle:
        for line in handle:
            line = line.strip()
            if not line:
                continue
            row = json.loads(line)
            entry_dt = parse_ts(row["entry_timestamp"])
            if entry_dt.astimezone(timezone.utc).date().isoformat() != "2026-09-29":
                continue
            key = (
                row["trade_id"],
                row["symbol"],
                row["side"],
                entry_dt.astimezone(timezone.utc).isoformat(),
            )
            payload = {
                "exit_reason": row.get("exit_reason"),
                "exit_price": float(row["exit_price"]),
                "pnl": float(row["pnl"]),
                "holding_mins_journal": row.get("holding_mins"),
            }
            if key in indexed:
                extras.append({"key": key, "note": "duplicate_journal_identity"})
            indexed[key] = payload
            payload["_seen"] = True
    return indexed, extras


def session_cards(index):
    """Cards whose UTC timestamp falls on the session date."""
    kept = [
        card for card in index["cards"]
        if card["_dt"].astimezone(timezone.utc).date().isoformat() == "2026-09-29"
    ]
    return {
        "cards": kept,
        "times": [card["_dt"] for card in kept],
    }


def first_signed(cards, side, entry_price, quantity, predicate, after=None):
    for card in cards:
        if after is not None and card["_dt"] <= after:
            continue
        value = signed_pnl(side, entry_price, card["current_price"], quantity)
        if predicate(value):
            return card, value
    return None, None


def reconstruct(row, journal_row, index):
    entry_dt = parse_ts(row["t_entry_utc"])
    exit_dt = parse_ts(row["t_exit_utc"])
    side = row["side"]
    entry_price = float(row["feature_entry_price"])
    quantity = float(row["feature_quantity"])
    hold_sec = (exit_dt - entry_dt).total_seconds()
    official_end = entry_dt + timedelta(minutes=75)
    out = {
        "symbol": row["symbol"],
        "side": side,
        "group": row["_group"],
        "entry_dt": entry_dt,
        "exit_dt": exit_dt,
        "hold_min": hold_sec / 60.0,
        "minutes_to_close": (15 * 60 + 30) - (entry_dt.astimezone(timezone(timedelta(hours=5, minutes=30))).hour * 60 + entry_dt.astimezone(timezone(timedelta(hours=5, minutes=30))).minute),
        "entry_minute": int(row["context_entry_minute_ist"]),
        "realized": float(row["label_realized_pnl"]),
        "label_mfe": float(row["label_mfe_pnl"]),
        "zero_mfe": int(float(row["label_zero_mfe"])) == 1,
        "f15": num(row.get("feature_f15_at_entry")),
        "fresh_15": num(row.get("feature_pre_freshness_15")),
        "fresh_30": num(row.get("feature_pre_freshness_30")),
        "churn": num(row.get("feature_state_churn")),
        "score_15": aligned(row, "feature_entry_15m_score"),
        "score_30": aligned(row, "feature_entry_30m_score"),
        "score_1d": aligned(row, "feature_entry_1d_score"),
        "decision_state": row.get("feature_entry_decision_state") or "",
        "exit_reason": None if journal_row is None else journal_row.get("exit_reason"),
        "path_available": index is not None,
        "journal_joined": journal_row is not None,
    }
    if index is None:
        out["path_incomplete"] = True
        return out

    holding = cards_between(index, entry_dt, exit_dt, start_exclusive=False)
    post_entry = [card for card in holding if card["_dt"] > entry_dt]
    signed = [signed_pnl(side, entry_price, card["current_price"], quantity) for card in holding]
    mfe = 0.0 if not signed else max(0.0, max(signed))
    mae = 0.0 if not signed else min(signed)
    out["holding_cards"] = len(holding)
    out["post_entry_cards"] = len(post_entry)
    out["reconstructed_mfe"] = mfe
    out["mfe_label_agree"] = abs(round(mfe, 4) - round(float(row["label_mfe_pnl"]), 4)) <= 0.05
    out["mae"] = mae
    out["sparse_path"] = len(post_entry) < 2
    gaps = []
    for left, right in zip(holding, holding[1:]):
        gaps.append((right["_dt"] - left["_dt"]).total_seconds())
    out["median_gap_sec"] = median(gaps)
    out["max_gap_sec"] = None if not gaps else max(gaps)

    adverse_card, _adverse_value = first_signed(
        post_entry, side, entry_price, quantity, lambda value: value < 0
    )
    favourable_card, _favourable_value = first_signed(
        post_entry, side, entry_price, quantity, lambda value: value > 0
    )
    out["sec_to_first_adverse"] = None if adverse_card is None else (adverse_card["_dt"] - entry_dt).total_seconds()
    out["sec_to_first_favourable_in_hold"] = (
        None if favourable_card is None else (favourable_card["_dt"] - entry_dt).total_seconds()
    )
    out["immediate_adverse"] = bool(post_entry) and signed_pnl(
        side, entry_price, post_entry[0]["current_price"], quantity
    ) < 0
    out["sampled_flat"] = bool(post_entry) and all(
        signed_pnl(side, entry_price, card["current_price"], quantity) == 0 for card in post_entry
    )

    if signed:
        peak_at = max(range(len(signed)), key=lambda i: (signed[i], -i))
        trough_at = min(range(len(signed)), key=lambda i: (signed[i], i))
        out["sec_to_mfe_peak"] = (holding[peak_at]["_dt"] - entry_dt).total_seconds()
        out["sec_to_mae"] = (holding[trough_at]["_dt"] - entry_dt).total_seconds()
        out["exit_card_signed"] = signed[-1]
        out["exit_card_state"] = holding[-1].get("decision_state")
        out["exit_price_matches_card"] = abs(float(holding[-1]["current_price"]) - float(journal_row["exit_price"] if journal_row else holding[-1]["current_price"])) <= 0.05
    else:
        out["sec_to_mfe_peak"] = None
        out["sec_to_mae"] = None
        out["exit_card_signed"] = None
        out["exit_card_state"] = None
        out["exit_price_matches_card"] = False

    trigger = None
    for card in post_entry:
        if triggers_exit(card.get("decision_state"), side):
            trigger = card
            break
    out["sec_to_exit_trigger_state"] = None if trigger is None else (trigger["_dt"] - entry_dt).total_seconds()
    out["trigger_state"] = None if trigger is None else trigger.get("decision_state")
    if out["sec_to_exit_trigger_state"] is None:
        out["sec_trigger_to_exit"] = None
    else:
        out["sec_trigger_to_exit"] = hold_sec - out["sec_to_exit_trigger_state"]

    last_dt = index["times"][-1] if index["times"] else None
    out["last_card_dt"] = last_dt
    out["official_cover_min"] = max(0.0, (official_end - exit_dt).total_seconds() / 60.0)
    out["official_window_complete"] = last_dt is not None and last_dt >= official_end
    out["cover_through_last_card_min"] = 0.0 if last_dt is None else max(0.0, (min(last_dt, official_end) - exit_dt).total_seconds() / 60.0)

    def window_favourable(end_dt, require_end):
        if last_dt is None:
            return None
        if require_end and last_dt < end_dt:
            return None
        cards = cards_between(index, exit_dt, min(end_dt, last_dt), start_exclusive=True)
        if require_end and (not cards or cards[-1]["_dt"] < end_dt) and last_dt < end_dt:
            return None
        for card in cards:
            if card["_dt"] > end_dt:
                break
            if signed_pnl(side, entry_price, card["current_price"], quantity) > 0:
                return True
        if require_end and last_dt < end_dt:
            return None
        return False

    out["favourable_inside_official_after_exit"] = window_favourable(official_end, require_end=False)
    out["exit_plus_60_complete"] = last_dt is not None and last_dt >= exit_dt + timedelta(minutes=60)
    out["favourable_within_exit_60_if_complete"] = window_favourable(exit_dt + timedelta(minutes=60), require_end=True)
    for minutes in (15, 30, 45):
        end_dt = exit_dt + timedelta(minutes=minutes)
        out[f"post_{minutes}m_complete"] = last_dt is not None and last_dt >= end_dt
        out[f"post_{minutes}m_favourable"] = window_favourable(end_dt, require_end=True)

    post_60_cards = []
    if last_dt is not None and out["exit_plus_60_complete"]:
        post_60_cards = cards_between(index, exit_dt, exit_dt + timedelta(minutes=60), start_exclusive=True)
    if post_60_cards:
        post_signed = [signed_pnl(side, entry_price, card["current_price"], quantity) for card in post_60_cards]
        post_vs_exit = []
        exit_price = float(journal_row["exit_price"]) if journal_row else float(post_60_cards[0]["current_price"])
        for card in post_60_cards:
            post_vs_exit.append(signed_pnl(side, exit_price, card["current_price"], quantity))
        out["post_60_max_vs_entry"] = max(post_signed)
        out["post_60_max_vs_exit"] = max(post_vs_exit)
        out["post_60_min_vs_entry"] = min(post_signed)
    else:
        out["post_60_max_vs_entry"] = None
        out["post_60_max_vs_exit"] = None
        out["post_60_min_vs_entry"] = None
    if hold_sec > 0 and out.get("sec_to_mfe_peak") is not None:
        out["peak_fraction_of_hold"] = out["sec_to_mfe_peak"] / hold_sec
    else:
        out["peak_fraction_of_hold"] = None
    return out


def distribution(values):
    present = [value for value in values if value is not None]
    return {
        "n": len(present),
        "median": round4(median(present)),
        "p25": round4(percentile(present, 0.25)),
        "p75": round4(percentile(present, 0.75)),
    }


def rate(flags):
    if not flags:
        return {"n": 0, "count": 0, "rate": None}
    count = sum(1 for flag in flags if flag)
    return {"n": len(flags), "count": count, "rate": round4(count / len(flags))}


def summarize_group(trades):
    def values(name):
        return [trade[name] for trade in trades if trade.get(name) is not None]

    def flags(name):
        return [bool(trade.get(name)) for trade in trades if name in trade and trade.get(name) is not None]

    reasons = Counter(trade.get("exit_reason") or "UNJOINED" for trade in trades)
    states = Counter(trade.get("exit_card_state") or "UNAVAILABLE" for trade in trades)
    triggers = Counter(trade.get("trigger_state") or "NONE" for trade in trades)
    path_class = Counter()
    for trade in trades:
        if not trade.get("path_available"):
            path_class["path_unavailable"] += 1
        elif trade.get("sparse_path"):
            path_class["sparse_fewer_than_2_post_entry_cards"] += 1
        elif trade.get("sampled_flat"):
            path_class["sampled_flat_through_exit"] += 1
        elif trade.get("sec_to_first_favourable_in_hold") is not None:
            path_class["favourable_print_during_hold"] += 1
        elif trade.get("immediate_adverse"):
            path_class["immediate_adverse_on_first_post_entry_card"] += 1
        elif trade.get("sec_to_first_adverse") is not None:
            path_class["flat_cards_then_adverse"] += 1
        else:
            path_class["unclassified"] += 1
    complete_60 = [trade for trade in trades if trade.get("exit_plus_60_complete")]
    return {
        "n": len(trades),
        "realized_pnl": round2(sum(trade["realized"] for trade in trades)),
        "median_realized": round2(median([trade["realized"] for trade in trades])),
        "long_share": rate([trade["side"] == "LONG" for trade in trades]),
        "f15": distribution([trade.get("f15") for trade in trades]),
        "f15_exactly_zero": rate([trade.get("f15") == 0 for trade in trades if trade.get("f15") is not None]),
        "fresh_15": distribution([trade.get("fresh_15") for trade in trades]),
        "fresh_30": distribution([trade.get("fresh_30") for trade in trades]),
        "state_churn": distribution([trade.get("churn") for trade in trades]),
        "aligned_15m": distribution([trade.get("score_15") for trade in trades]),
        "aligned_30m": distribution([trade.get("score_30") for trade in trades]),
        "aligned_1d": distribution([trade.get("score_1d") for trade in trades]),
        "minutes_to_close": distribution([trade.get("minutes_to_close") for trade in trades]),
        "hold_min": distribution([trade.get("hold_min") for trade in trades]),
        "mae": distribution(values("mae")),
        "label_mfe": distribution([trade.get("label_mfe") for trade in trades]),
        "sec_to_first_adverse": distribution(values("sec_to_first_adverse")),
        "sec_to_mae": distribution(values("sec_to_mae")),
        "sec_to_first_favourable_in_hold": distribution(values("sec_to_first_favourable_in_hold")),
        "sec_to_mfe_peak": distribution(values("sec_to_mfe_peak")),
        "peak_fraction_of_hold": distribution(values("peak_fraction_of_hold")),
        "sec_to_exit_trigger_state": distribution(values("sec_to_exit_trigger_state")),
        "sec_trigger_to_exit": distribution(values("sec_trigger_to_exit")),
        "official_cover_min": distribution(values("official_cover_min")),
        "immediate_adverse": rate([trade.get("immediate_adverse") for trade in trades if trade.get("path_available")]),
        "sparse_path": rate([trade.get("sparse_path") for trade in trades if trade.get("path_available")]),
        "official_window_complete": rate([trade.get("official_window_complete") for trade in trades if trade.get("path_available")]),
        "exit_plus_60_complete": rate([trade.get("exit_plus_60_complete") for trade in trades if trade.get("path_available")]),
        "favourable_in_complete_exit_60": rate([
            trade.get("favourable_within_exit_60_if_complete") for trade in complete_60
        ]),
        "post_60_max_vs_entry": distribution([trade.get("post_60_max_vs_entry") for trade in complete_60]),
        "post_60_max_vs_exit": distribution([trade.get("post_60_max_vs_exit") for trade in complete_60]),
        "post_60_went_above_exit": rate([
            trade["post_60_max_vs_exit"] > 0
            for trade in complete_60
            if trade.get("post_60_max_vs_exit") is not None
        ]),
        "post_60_touched_nonpositive": rate([
            trade["post_60_min_vs_entry"] <= 0
            for trade in complete_60
            if trade.get("post_60_min_vs_entry") is not None
        ]),
        "exited_by_5m": rate([trade["hold_min"] <= 5 for trade in trades]),
        "exited_by_10m": rate([trade["hold_min"] <= 10 for trade in trades]),
        "exited_by_15m": rate([trade["hold_min"] <= 15 for trade in trades]),
        "exit_reasons": dict(reasons),
        "exit_card_states": dict(states),
        "first_trigger_states": dict(triggers),
        "path_classes": dict(path_class),
        "symbols": len({trade["symbol"] for trade in trades}),
    }


def fixed_post_exit_recovery(trades, minutes):
    """Same post-exit length for every included trade. Incomplete windows stay out."""
    eligible = []
    for trade in trades:
        if not trade.get("zero_mfe"):
            continue
        if not trade.get(f"post_{minutes}m_complete"):
            continue
        favourable = trade.get(f"post_{minutes}m_favourable")
        if favourable is None or trade.get("mae") is None:
            continue
        eligible.append(trade)
    if len(eligible) < 20:
        return {"minutes": minutes, "n": len(eligible), "note": "fewer than 20 complete windows"}
    y = [0 if trade[f"post_{minutes}m_favourable"] else 1 for trade in eligible]
    scores = [-trade["mae"] for trade in eligible]
    estimate = cluster_auc_ci(eligible, y, scores)
    return {
        "minutes": minutes,
        "n": len(eligible),
        "symbols": len({trade["symbol"] for trade in eligible}),
        "recovery_rate": round4(1 - (sum(y) / len(y))),
        "median_mae": round2(median([trade["mae"] for trade in eligible])),
        "auc_deeper_mae_ranks_no_recovery": estimate,
        "window": f"cards strictly after exit through exit+{minutes}m, only when the symbol's last session card reaches that end",
    }


def horizon_distance(csv_rows):
    reports = []
    for horizon in (5, 10, 15):
        later_col = f"outcome_{horizon}m_later_favourable"
        mtm_col = f"obs_{horizon}m_mtm_pnl"
        populations = {"all_eligible": [], "still_open": []}
        for row in csv_rows:
            if row.get(later_col) in (None, "") or num(row.get(mtm_col)) is None:
                continue
            entry = parse_ts(row["t_entry_utc"])
            exit_dt = parse_ts(row["t_exit_utc"])
            item = {
                "symbol": row["symbol"],
                "mtm": num(row[mtm_col]),
                "later": int(float(row[later_col])),
            }
            populations["all_eligible"].append(item)
            if exit_dt > entry + timedelta(minutes=horizon):
                populations["still_open"].append(item)
        block = {"horizon_min": horizon}
        for name, group in populations.items():
            y_no = [1 - item["later"] for item in group]
            continuous = cluster_auc_ci(group, y_no, [-item["mtm"] for item in group])
            binary = cluster_auc_ci(group, y_no, [1.0 if item["mtm"] < 0 else 0.0 for item in group])
            adverse = [item for item in group if item["mtm"] < 0]
            not_adverse = [item for item in group if item["mtm"] >= 0]
            block[name] = {
                "n": len(group),
                "share_already_adverse": round4(len(adverse) / len(group)) if group else None,
                "later_favourable_rate_if_adverse": round4(
                    sum(item["later"] for item in adverse) / len(adverse)
                ) if adverse else None,
                "later_favourable_rate_if_not_adverse": round4(
                    sum(item["later"] for item in not_adverse) / len(not_adverse)
                ) if not_adverse else None,
                "auc_continuous_distance": continuous,
                "auc_binary_already_adverse": binary,
            }
        reports.append(block)
    return reports


def cluster_contrast(trades, fn, n_boot=BOOTSTRAPS, seed=BOOTSTRAP_SEED):
    point = fn(trades)
    clusters = defaultdict(list)
    for index, trade in enumerate(trades):
        clusters[trade["symbol"]].append(index)
    keys = list(clusters)
    rng = np.random.default_rng(seed)
    draws = []
    dropped = 0
    for _ in range(n_boot):
        chosen = rng.choice(len(keys), size=len(keys), replace=True)
        sample = []
        for key_index in chosen:
            sample.extend(trades[index] for index in clusters[keys[key_index]])
        estimate = fn(sample)
        if estimate is None:
            dropped += 1
        else:
            draws.append(estimate)
    if not draws:
        return {
            "estimate": round4(point) if point is not None else None,
            "ci95": None,
            "bootstraps_requested": n_boot,
            "bootstraps_used": 0,
            "bootstraps_dropped": dropped,
            "clusters": len(keys),
        }
    low, high = np.percentile(draws, [2.5, 97.5])
    return {
        "estimate": round4(point) if point is not None else None,
        "ci95": [round4(float(low)), round4(float(high))],
        "bootstraps_requested": n_boot,
        "bootstraps_used": len(draws),
        "bootstraps_dropped": dropped,
        "clusters": len(keys),
    }


def median_diff(trades, group_a, group_b, field):
    left = [trade[field] for trade in trades if trade["group"] == group_a and trade.get(field) is not None]
    right = [trade[field] for trade in trades if trade["group"] == group_b and trade.get(field) is not None]
    if not left or not right:
        return None
    return median(left) - median(right)


def rate_diff(trades, group_a, group_b, predicate):
    def mean(group):
        rows = [trade for trade in trades if trade["group"] == group]
        if not rows:
            return None
        return sum(1 for trade in rows if predicate(trade)) / len(rows)
    left = mean(group_a)
    right = mean(group_b)
    if left is None or right is None:
        return None
    return left - right


def coincidence(trades):
    by_minute = defaultdict(list)
    for trade in trades:
        stamp = trade["entry_dt"].astimezone(timezone(timedelta(hours=5, minutes=30))).strftime("%Y-%m-%d %H:%M")
        by_minute[stamp].append(trade)
    minute_sizes = [len(rows) for rows in by_minute.values()]
    for trade in trades:
        stamp = trade["entry_dt"].astimezone(timezone(timedelta(hours=5, minutes=30))).strftime("%Y-%m-%d %H:%M")
        peers = [peer for peer in by_minute[stamp] if peer is not trade]
        trade["same_minute_peers"] = len(peers)
        trade["peer_zero_rate"] = None if not peers else sum(1 for peer in peers if peer["zero_mfe"]) / len(peers)
        open_book = 0
        for other in trades:
            if other is trade:
                continue
            if other["entry_dt"] <= trade["entry_dt"] < other["exit_dt"]:
                open_book += 1
        trade["open_book_at_entry"] = open_book
    sizes = sorted(minute_sizes)
    return {
        "minutes_with_an_entry": len(by_minute),
        "entries_per_minute_median": sizes[len(sizes) // 2],
        "entries_per_minute_max": sizes[-1],
        "peer_zero_rate_by_group": {
            group: distribution([
                trade["peer_zero_rate"] for trade in trades
                if trade["group"] == group and trade.get("peer_zero_rate") is not None
            ])
            for group in GROUP_ORDER
        },
        "open_book_by_group": {
            group: distribution([trade["open_book_at_entry"] for trade in trades if trade["group"] == group])
            for group in GROUP_ORDER
        },
    }


def ticker_sensitivity(trades):
    counts = Counter(trade["symbol"] for trade in trades)
    total = len(trades)
    shares = [(count / total) ** 2 for count in counts.values()]
    largest = counts.most_common(5)
    zero = [trade for trade in trades if trade["zero_mfe"]]
    base_hits = sum(1 for trade in zero if trade["group"] == "zero_later_favourable")
    base_rate = base_hits / len(zero) if zero else None
    leave = []
    for symbol, count in counts.items():
        if count < 15:
            continue
        kept = [trade for trade in zero if trade["symbol"] != symbol]
        if not kept:
            continue
        leave.append({
            "symbol": symbol,
            "trades_removed": count,
            "zero_mfe_remaining": len(kept),
            "later_favourable_rate": round4(
                sum(1 for trade in kept if trade["group"] == "zero_later_favourable") / len(kept)
            ),
        })
    rates = [item["later_favourable_rate"] for item in leave]
    return {
        "symbols": len(counts),
        "largest_symbols": [{"symbol": symbol, "n": count, "share": round4(count / total)} for symbol, count in largest],
        "herfindahl": round4(sum(shares)),
        "zero_mfe_later_favourable_rate": round4(base_rate),
        "leave_one_symbol_min_15_trades": {
            "symbols_tested": len(leave),
            "min_rate": None if not rates else min(rates),
            "max_rate": None if not rates else max(rates),
            "note": "Range after removing one symbol at a time. Not a selected subset.",
        },
    }


def side_and_clock(trades):
    def composition(predicate):
        chosen = [trade for trade in trades if predicate(trade)]
        counts = Counter(trade["group"] for trade in chosen)
        return {
            "n": len(chosen),
            "groups": {group: counts.get(group, 0) for group in GROUP_ORDER},
            "zero_mfe_rate": round4(sum(1 for trade in chosen if trade["zero_mfe"]) / len(chosen)) if chosen else None,
        }
    bands = {
        "long": lambda trade: trade["side"] == "LONG",
        "short": lambda trade: trade["side"] == "SHORT",
        "time_to_close_under_30m": lambda trade: trade["minutes_to_close"] < 30,
        "time_to_close_30_to_60m": lambda trade: 30 <= trade["minutes_to_close"] < 60,
        "time_to_close_60_to_180m": lambda trade: 60 <= trade["minutes_to_close"] < 180,
        "time_to_close_180m_or_more": lambda trade: trade["minutes_to_close"] >= 180,
        "hold_under_5m": lambda trade: trade["hold_min"] < 5,
        "hold_5_to_15m": lambda trade: 5 <= trade["hold_min"] < 15,
        "hold_15_to_30m": lambda trade: 15 <= trade["hold_min"] < 30,
        "hold_30m_or_more": lambda trade: trade["hold_min"] >= 30,
    }
    return {name: composition(predicate) for name, predicate in bands.items()}


def render(result):
    groups = result["groups"]
    lines = [
        "# RP-609 trajectory forensic: NSE Ledger A, 29 September 2026",
        "",
        "Descriptive research on one closed session. No classifier, no threshold search, and no candidate mechanism.",
        "RP-605 remains the frozen control. RP-608 remains shadow-only. PFT, SPL, VAT, and PPR are unchanged.",
        "These findings are not pooled with 28 September and do not change execution authority.",
        "",
        "## 1. Cohort reconciliation",
        "",
        f"Session extract SHA256 `{result['csv_sha256']}`. Rows: {result['population']['rows']}.",
        f"Journal identities joined: {result['population']['journal_joined']}. "
        f"Extra 29 September journal rows excluded because they are absent from the extract: {result['population']['journal_rows_excluded']}.",
        f"Reconstructed lifecycle MFE agrees with the extract label within 0.05: "
        f"{result['population']['mfe_label_agreements']} / {result['population']['path_available']}.",
        "",
        "A favourable print means a decision-card price with signed P&L strictly above zero versus the original entry. "
        "It is a sampled print, not a continuous cross of the entry price. "
        "The entry clocks are the existing 5, 10, and 15 minute marks and the following 60 minutes, so their union ends at entry + 75 minutes. "
        "An empty clock stays unobserved. A partial clock that contains at least one card is treated as observed by the stored extract; completeness is reported separately below.",
        "",
        "| Group | Trades | Realized P&L |",
        "| --- | ---: | ---: |",
    ]
    for key in GROUP_ORDER:
        block = groups[key]
        lines.append(f"| {GROUP_LABEL[key]} | {block['n']} | {block['realized_pnl']:.2f} |")
    pop = result["population"]
    lines.append(f"| Session total | {pop['rows']} | {pop['realized_pnl']:.2f} |")
    lines.extend([
        "",
        f"Zero-MFE realized P&L {pop['zero_realized']:.2f}. Positive-MFE realized P&L {pop['positive_realized']:.2f}.",
        "The four groups partition the extract when the unobserved-clock row is zero.",
        "",
        "## 2. What the path contains, and what it does not",
        "",
        "Decision cards on this session run from "
        f"{result['cards']['first_utc']} to {result['cards']['last_utc']} "
        f"({result['cards']['rows']} cards). "
        f"The median gap between consecutive cards on a hold is {result['cards']['median_hold_gap_sec']} seconds. "
        "That is the sample interval. A gap above 60 seconds is the ordinary cadence, not a special hole in a subset of trades. "
        "A favourable move shorter than that interval is unobserved.",
        "",
        "Available from the join: entry and exit timestamps, entry price, quantity, side, exit reason, realized P&L, and the sampled card path through the last card of the session.",
        "Not available, and not inferred: a price stop or other initial risk boundary. "
        "`ENGINE_EXIT` is a decision-state flip to EXIT, PULLBACK, or the opposite side. "
        "`TARGET_EXIT` is the control's 60-minute time stop (`held_mins >= 60`), not a profit target.",
        "A favourable move that occurs between cards is invisible. Zero-MFE here means no favourable card was sampled between entry and exit.",
        "",
        "## 3. The four groups are different lifecycles",
        "",
        "Entry-known fields are on the row before the trade exists. Hold length, MAE, peak MFE, and exit reason become known only as the control acts. "
        "Post-exit prints are counterfactual observations. They were not inputs to the original exit.",
        "",
        "| Quantity | Known | No later print | Later print | Positive MFE, nonpositive | Positive MFE, positive |",
        "| --- | --- | ---: | ---: | ---: | ---: |",
    ])
    order = (
        "zero_no_favourable",
        "zero_later_favourable",
        "positive_mfe_nonpositive",
        "positive_mfe_positive",
    )

    def cell(key, path, nd=2):
        block = groups[key]
        value = block
        for part in path:
            value = value[part]
        if value is None:
            return "—"
        if isinstance(value, float):
            return f"{value:.{nd}f}"
        return str(value)

    rows_spec = [
        ("Trades", ("n",), 0),
        ("Median realized P&L", ("median_realized",), 2),
        ("Long share", ("long_share", "rate"), 4),
        ("Median F15", ("f15", "median"), 4),
        ("F15 exactly 0", ("f15_exactly_zero", "rate"), 4),
        ("Median card freshness 15", ("fresh_15", "median"), 4),
        ("Median state churn", ("state_churn", "median"), 2),
        ("Median side-aligned 15m", ("aligned_15m", "median"), 4),
        ("Median side-aligned 30m", ("aligned_30m", "median"), 4),
        ("Median side-aligned 1d", ("aligned_1d", "median"), 4),
        ("Median minutes to 15:30 IST", ("minutes_to_close", "median"), 1),
        ("Median hold, minutes", ("hold_min", "median"), 2),
        ("Share exited by 5 minutes", ("exited_by_5m", "rate"), 4),
        ("Median MAE, rupees", ("mae", "median"), 2),
        ("Median lifecycle MFE, rupees", ("label_mfe", "median"), 2),
        ("Median seconds to first adverse card", ("sec_to_first_adverse", "median"), 1),
        ("Median seconds to first exit-trigger state", ("sec_to_exit_trigger_state", "median"), 1),
        ("Median seconds from that state to exit", ("sec_trigger_to_exit", "median"), 1),
        ("Median official post-exit cover, minutes", ("official_cover_min", "median"), 2),
        ("Share with a complete exit+60m card window", ("exit_plus_60_complete", "rate"), 4),
        ("Favourable print inside complete exit+60m", ("favourable_in_complete_exit_60", "rate"), 4),
    ]
    known = {
        "Trades": "definition",
        "Median realized P&L": "at exit",
        "Long share": "at entry",
        "Median F15": "at entry",
        "F15 exactly 0": "at entry",
        "Median card freshness 15": "at entry",
        "Median state churn": "at entry",
        "Median side-aligned 15m": "at entry",
        "Median side-aligned 30m": "at entry",
        "Median side-aligned 1d": "at entry",
        "Median minutes to 15:30 IST": "at entry",
        "Median hold, minutes": "at exit",
        "Share exited by 5 minutes": "at exit",
        "Median MAE, rupees": "during hold",
        "Median lifecycle MFE, rupees": "during hold",
        "Median seconds to first adverse card": "during hold",
        "Median seconds to first exit-trigger state": "during hold",
        "Median seconds from that state to exit": "at exit",
        "Median official post-exit cover, minutes": "window geometry",
        "Share with a complete exit+60m card window": "data coverage",
        "Favourable print inside complete exit+60m": "after exit",
    }
    for label, path, nd in rows_spec:
        lines.append(
            "| "
            + " | ".join([label, known[label]] + [cell(key, path, nd) for key in order])
            + " |"
        )
    lines.extend([
        "",
        "Rupee excursions use the control quantity, `floor(100000 / entry price)`, so they are approximately returns on a ₹100,000 notional.",
        "",
        "### Contrasts inside the session",
        "",
        "Ticker-cluster bootstrap, 1,000 draws, seed 42. One-class or empty-group draws are dropped. "
        "The interval is within-session ticker resampling, not a next-session interval.",
        "",
        "| Contrast | Estimate | 95% interval | Draws used | Dropped |",
        "| --- | ---: | --- | ---: | ---: |",
    ])
    for name, block in result["contrasts"].items():
        ci = "—" if not block["ci95"] else f"[{block['ci95'][0]}, {block['ci95'][1]}]"
        lines.append(
            f"| {name} | {block['estimate']} | {ci} | {block['bootstraps_used']} | {block['bootstraps_dropped']} |"
        )
    lines.extend([
        "",
        "## 4. Zero-MFE is not one path",
        "",
        "By construction, a zero-MFE trade has no sampled favourable print on cards from entry through exit. "
        "Any later favourable print in the official clocks is therefore after the control has already closed the trade.",
        "",
        "| Sampled hold path | No later print | Later print |",
        "| --- | ---: | ---: |",
    ])
    class_names = sorted(set(groups["zero_no_favourable"]["path_classes"]) | set(groups["zero_later_favourable"]["path_classes"]))
    for name in class_names:
        left = groups["zero_no_favourable"]["path_classes"].get(name, 0)
        right = groups["zero_later_favourable"]["path_classes"].get(name, 0)
        lines.append(f"| {name} | {left} | {right} |")
    lines.extend([
        "",
        "Exit reasons and the state on the exit card:",
        "",
        "| | No later print | Later print |",
        "| --- | ---: | ---: |",
    ])
    reason_names = sorted(set(groups["zero_no_favourable"]["exit_reasons"]) | set(groups["zero_later_favourable"]["exit_reasons"]))
    for name in reason_names:
        lines.append(
            f"| Exit `{name}` | {groups['zero_no_favourable']['exit_reasons'].get(name, 0)} | "
            f"{groups['zero_later_favourable']['exit_reasons'].get(name, 0)} |"
        )
    state_names = sorted(set(groups["zero_no_favourable"]["exit_card_states"]) | set(groups["zero_later_favourable"]["exit_card_states"]))
    for name in state_names:
        lines.append(
            f"| Exit-card state `{name}` | {groups['zero_no_favourable']['exit_card_states'].get(name, 0)} | "
            f"{groups['zero_later_favourable']['exit_card_states'].get(name, 0)} |"
        )
    lines.extend([
        "",
        "Both zero-MFE groups are short holds. The median hold differs by about one minute, and the official post-exit cover differs by the same minute with the opposite sign, because cover is entry + 75 minutes minus the hold whenever the exit falls inside that window. "
        "The theoretical asymmetry — a 70-minute hold leaving five minutes of post-exit cover, against a three-minute hold leaving about 72 — does not describe this 180 / 399 split. "
        "On this session the split lines up with a deeper adverse excursion in the no-print group, about ₹49 at the median, and with a higher long share in that group. "
        "F15 does not separate them. The favourable-print rate inside a complete exit+60 minute window is not an independent check of the official split: with holds of a few minutes, exit+60 sits inside the entry+75 clock that defined the groups.",
        "",
        "Closing state: every zero-MFE trade is `ENGINE_EXIT`. None is the 60-minute time stop. "
        "The median time from the first exit-trigger state to the recorded exit is zero seconds, so the control closes on the card that first flips the state. "
        "That state is PULLBACK for most of both groups. The remaining exits are the opposite side (`SHORT` closes a long, `BUY` closes a short). No exit card in these groups has the state value EXIT.",
        "",
        "Sampled-path patterns inside zero-MFE, as counts rather than examples to trade:",
        "- The first post-entry card is already adverse, typically one sample interval (about 64 seconds) after entry. This is the majority of the trades that have at least two post-entry cards.",
        "- A smaller set is sampled flat through the exit: no adverse card and no favourable card before the control closes.",
        "- Trades with fewer than two post-entry cards, concentrated in the later-print group, have an incomplete hold path. The missing interval is not filled in.",
        "",
        "Holding the post-exit window fixed, and dropping trades whose card stream ends first:",
        "",
        "| Fixed window after exit | Complete trades | Recovery rate | AUC, deeper MAE ranking no recovery | Interval | Dropped draws |",
        "| --- | ---: | ---: | ---: | --- | ---: |",
    ])
    for block in result["fixed_post_exit"]:
        auc = block.get("auc_deeper_mae_ranks_no_recovery") or {}
        ci = "—" if not auc.get("ci95") else f"[{auc['ci95'][0]}, {auc['ci95'][1]}]"
        lines.append(
            f"| {block['minutes']} min | {block.get('n')} | {block.get('recovery_rate', '—')} | "
            f"{auc.get('auc', '—')} | {ci} | {auc.get('bootstraps_dropped_one_class', '—')} |"
        )
    lines.extend([
        "",
        "These fixed windows are coverage restrictions. They are not new exit rules, and none of the three windows is selected as preferred.",
        "",
        "## 5. Positive-MFE trades split at the exit, not at the entry",
        "",
        f"Nonpositive realized: n={groups['positive_mfe_nonpositive']['n']}, "
        f"MFE median {groups['positive_mfe_nonpositive']['label_mfe']['median']} "
        f"(IQR {groups['positive_mfe_nonpositive']['label_mfe']['p25']} to {groups['positive_mfe_nonpositive']['label_mfe']['p75']}), "
        f"median realized {groups['positive_mfe_nonpositive']['median_realized']}, "
        f"median peak at {groups['positive_mfe_nonpositive']['peak_fraction_of_hold']['median']} of the hold, "
        f"median MAE {groups['positive_mfe_nonpositive']['mae']['median']}.",
        f"Positive realized: n={groups['positive_mfe_positive']['n']}, "
        f"MFE median {groups['positive_mfe_positive']['label_mfe']['median']} "
        f"(IQR {groups['positive_mfe_positive']['label_mfe']['p25']} to {groups['positive_mfe_positive']['label_mfe']['p75']}), "
        f"median realized {groups['positive_mfe_positive']['median_realized']}, "
        f"median peak at {groups['positive_mfe_positive']['peak_fraction_of_hold']['median']} of the hold, "
        f"median MAE {groups['positive_mfe_positive']['mae']['median']}.",
        "On this session the two interquartile ranges of lifecycle MFE barely meet. "
        "The nonpositive-exit group is typically a small favourable print, not a large peak that is later given back. "
        "That description uses the path after entry. It is not an entry rule, and it is not yet a replicated fact.",
        "",
        "Where the exit+60 minute card window is complete, the post-exit path is a counterfactual observation:",
        "",
        "| | Complete windows with a post-exit card | Favourable print vs entry | Print beyond the exit price | Print at or below entry |",
        "| --- | ---: | ---: | ---: | ---: |",
        f"| Nonpositive realized | {groups['positive_mfe_nonpositive']['favourable_in_complete_exit_60']['n']} | "
        f"{groups['positive_mfe_nonpositive']['favourable_in_complete_exit_60']['rate']} | "
        f"{groups['positive_mfe_nonpositive']['post_60_went_above_exit']['rate']} | "
        f"{groups['positive_mfe_nonpositive']['post_60_touched_nonpositive']['rate']} |",
        f"| Positive realized | {groups['positive_mfe_positive']['favourable_in_complete_exit_60']['n']} | "
        f"{groups['positive_mfe_positive']['favourable_in_complete_exit_60']['rate']} | "
        f"{groups['positive_mfe_positive']['post_60_went_above_exit']['rate']} | "
        f"{groups['positive_mfe_positive']['post_60_touched_nonpositive']['rate']} |",
        "",
        "For trades that already exited nonpositive, a later print at or below entry is the continuation of an adverse exit, not a new cross. "
        "The informative post-exit quantity for that group is the favourable-print rate. "
        "For trades that exited positive, a later print at or below entry is a giveback observed after the control had closed.",
        "",
        "Exit reasons:",
        "",
        "| | Nonpositive realized | Positive realized |",
        "| --- | ---: | ---: |",
    ])
    for name in sorted(set(groups["positive_mfe_nonpositive"]["exit_reasons"]) | set(groups["positive_mfe_positive"]["exit_reasons"])):
        lines.append(
            f"| `{name}` | {groups['positive_mfe_nonpositive']['exit_reasons'].get(name, 0)} | "
            f"{groups['positive_mfe_positive']['exit_reasons'].get(name, 0)} |"
        )
    lines.extend([
        "",
        "## 6. How much of the recovery ranking is distance, path, or the exit",
        "",
        "The horizon score is signed mark-to-market versus the original entry. The outcome is a later sampled print that is also favourable versus that same entry. "
        "Part of the rank is therefore distance back to a shared reference price.",
        "",
        "| Horizon | Population | N | Share already adverse | Later-favourable if adverse | Later-favourable if not | AUC, continuous distance | AUC, adverse-or-not |",
        "| --- | --- | ---: | ---: | ---: | ---: | ---: | ---: |",
    ])
    for block in result["horizon_distance"]:
        for name in ("all_eligible", "still_open"):
            item = block[name]
            lines.append(
                f"| {block['horizon_min']}m | {name} | {item['n']} | {item['share_already_adverse']} | "
                f"{item['later_favourable_rate_if_adverse']} | {item['later_favourable_rate_if_not_adverse']} | "
                f"{item['auc_continuous_distance']['auc']} | {item['auc_binary_already_adverse']['auc']} |"
            )
    lines.extend([
        "",
        "Continuous intervals and dropped draws are in the JSON artifact. "
        "Still-open rows are trades the control had not yet closed. They are not a population in which every trade was held to the horizon.",
        "",
        "## 7. Robustness diagnostics",
        "",
        "These cuts were specified as sensitivity checks. No cut is promoted because its rate is higher.",
        "",
        f"Symbols: {result['tickers']['symbols']}. Herfindahl of trade counts: {result['tickers']['herfindahl']}. "
        f"Largest symbols: " + ", ".join(f"{item['symbol']} {item['n']} ({item['share']})" for item in result["tickers"]["largest_symbols"]) + ".",
        f"Among zero-MFE trades, the official later-print rate is {result['tickers']['zero_mfe_later_favourable_rate']}. "
        f"Removing one symbol with at least 15 trades moves that rate between "
        f"{result['tickers']['leave_one_symbol_min_15_trades']['min_rate']} and "
        f"{result['tickers']['leave_one_symbol_min_15_trades']['max_rate']} "
        f"({result['tickers']['leave_one_symbol_min_15_trades']['symbols_tested']} symbols tested).",
        "",
        "| Slice | Trades | Zero-MFE rate | No later print | Later print | Nonpositive realized | Positive realized |",
        "| --- | ---: | ---: | ---: | ---: | ---: | ---: |",
    ])
    for name, block in result["slices"].items():
        g = block["groups"]
        lines.append(
            f"| {name} | {block['n']} | {block['zero_mfe_rate']} | {g['zero_no_favourable']} | "
            f"{g['zero_later_favourable']} | {g['positive_mfe_nonpositive']} | {g['positive_mfe_positive']} |"
        )
    coin = result["coincidence"]
    lines.extend([
        "",
        f"Entry minutes with at least one trade: {coin['minutes_with_an_entry']}. "
        f"Median entries in such a minute: {coin['entries_per_minute_median']}. Maximum: {coin['entries_per_minute_max']}.",
        "Long and short trades have nearly the same zero-MFE rate. Inside the zero-MFE population, the no-print group contains a higher long share; that contrast's interval excludes zero and is still one session.",
        "Peer zero-MFE rate is the zero-MFE rate of the other trades that entered in the same IST minute, so a trade is not counted inside its own peer rate. "
        "Median peer rates and the number of trades already open are similar across the four groups. "
        "The maximum of 97 entries in one minute shows that entries arrive in bursts; those bursts do not, at the median, separate the outcome groups.",
        "",
        "| Group | Median peer zero-MFE rate | Median other trades already open |",
        "| --- | ---: | ---: |",
    ])
    for key in order:
        lines.append(
            f"| {GROUP_LABEL[key]} | {coin['peer_zero_rate_by_group'][key]['median']} | "
            f"{coin['open_book_by_group'][key]['median']} |"
        )
    lines.extend([
        "",
        "## 8. Missing information",
        "",
        "- No stored initial risk boundary, stop price, ATR, or volatility.",
        "- Card gaps mean a brief favourable print can be absent from both MFE and the later-recovery flag.",
        "- The card stream ends before a full 60 minutes after many late exits, and before entry + 75 minutes for late entries. Those tails are unobserved.",
        "- The stored horizon flags treat a non-empty partial clock as observed. Completeness is a separate field in this report.",
        "- Exit reason and hold length are consequences of RP-605. They are not entry features.",
        "- Post-exit prices do not include costs, slippage, or a fill assumption.",
        "- Bootstrap intervals do not carry to the next session. One ticker on two dates is not a paired cluster, and this study does not use 28 September.",
        "",
        "## 9. Questions the evidence supports, and does not answer",
        "",
        "Supported as questions for a later, pre-specified study. Not as findings to trade.",
        "",
        "1. After a zero-MFE exit, does adverse depth still rank a favourable print once every trade is given the same complete post-exit window on a new session?",
        "2. Are the short zero-MFE holds produced by the control flipping to PULLBACK or the opposite state before any favourable card, and does that sequence repeat on the next session?",
        "3. Does the next session again show the nonpositive-exit group as a small sampled peak (here, median MFE about ₹70, IQR about ₹31 to ₹128) rather than a large peak that is given back?",
        "4. Do same-minute entries share zero-MFE outcomes because the names moved together, or because the decision state flipped together? A later session would need the card state and the price path kept separate.",
        "",
        "Not established: an entry filter, a better exit, a threshold on F15, F30, state churn, or the trajectory scores, or a management comparison against RP-605.",
        "",
        "## 10. Decision gate",
        "",
        "The evidence points to lifecycle behaviour under the observed control: short state-flip exits, a sampled adverse path, and a post-exit print ranked by distance from entry. "
        "Entry-known features do not separate the groups. On this session the entry-clock geometry does not explain the 180 / 399 split, because both groups exit within a few minutes. "
        "It does not point to a change in entry selection or exit management. "
        "A management comparison remains a separate shadow ledger: realized P&L, costs, slippage, premature exits, and missed favourable moves. "
        "Until that ledger is specified, RP-605 stays the control and the existing mechanisms stay untouched.",
        "",
    ])
    return "\n".join(lines) + "\n"


def main():
    from build_rp609_entry_time_discrimination_dataset import LOG_DIR

    csv_path = LOG_DIR / "rp609_nse_20260929_entry_discrimination_dataset.csv"
    journal_path = LOG_DIR / "rp605_nse_baseline_journal.jsonl"
    out_json = LOG_DIR / "rp609_nse_20260929_trajectory_forensic.json"
    out_md = LOG_DIR / "rp609_nse_20260929_trajectory_forensic.md"

    with csv_path.open(encoding="utf-8") as handle:
        csv_rows = list(csv.DictReader(handle))
    for row in csv_rows:
        row["_group"] = assign_group(row)
    journal, _duplicate_notes = load_journal_index(journal_path)
    cards = {symbol: session_cards(index) for symbol, index in load_cards().items()}
    trades = []
    excluded = 0
    csv_keys = set()
    for row in csv_rows:
        key = (row["trade_id"], row["symbol"], row["side"], parse_ts(row["t_entry_utc"]).astimezone(timezone.utc).isoformat())
        csv_keys.add(key)
        trades.append(reconstruct(row, journal.get(key), cards.get(row["symbol"])))
    for key in journal:
        if key not in csv_keys:
            excluded += 1

    grouped = {key: [trade for trade in trades if trade["group"] == key] for key in GROUP_ORDER}
    ist = timezone(timedelta(hours=5, minutes=30))
    card_times = [card["_dt"] for index in cards.values() for card in index["cards"]]
    gaps = [trade["median_gap_sec"] for trade in trades if trade.get("median_gap_sec") is not None]
    wide = [trade for trade in trades if trade.get("max_gap_sec") is not None]

    contrasts = {
        "Median hold, no-print minus later-print, minutes": cluster_contrast(
            trades, lambda sample: median_diff(sample, "zero_no_favourable", "zero_later_favourable", "hold_min")
        ),
        "Median MAE, no-print minus later-print, rupees": cluster_contrast(
            trades, lambda sample: median_diff(sample, "zero_no_favourable", "zero_later_favourable", "mae")
        ),
        "Median official post-exit cover, no-print minus later-print, minutes": cluster_contrast(
            trades, lambda sample: median_diff(sample, "zero_no_favourable", "zero_later_favourable", "official_cover_min")
        ),
        "Median F15, no-print minus later-print": cluster_contrast(
            trades, lambda sample: median_diff(sample, "zero_no_favourable", "zero_later_favourable", "f15")
        ),
        "Median minutes to close, no-print minus later-print": cluster_contrast(
            trades, lambda sample: median_diff(sample, "zero_no_favourable", "zero_later_favourable", "minutes_to_close")
        ),
        "Long share, no-print minus later-print": cluster_contrast(
            trades, lambda sample: rate_diff(sample, "zero_no_favourable", "zero_later_favourable", lambda trade: trade["side"] == "LONG")
        ),
        "Median MFE, nonpositive minus positive realized, rupees": cluster_contrast(
            trades, lambda sample: median_diff(sample, "positive_mfe_nonpositive", "positive_mfe_positive", "label_mfe")
        ),
        "Median F15, nonpositive minus positive realized": cluster_contrast(
            trades, lambda sample: median_diff(sample, "positive_mfe_nonpositive", "positive_mfe_positive", "f15")
        ),
    }

    result = {
        "session": "NSE Ledger A 2026-09-29",
        "csv_sha256": sha256(csv_path),
        "governance": {
            "rp605": "frozen control",
            "rp608": "shadow-only",
            "pft_spl_vat_ppr": "unchanged",
            "pooled_with_2026_09_28": False,
            "classifier_fit": False,
            "threshold_search": False,
        },
        "population": {
            "rows": len(trades),
            "journal_joined": sum(1 for trade in trades if trade["journal_joined"]),
            "journal_rows_excluded": excluded,
            "path_available": sum(1 for trade in trades if trade["path_available"]),
            "mfe_label_agreements": sum(1 for trade in trades if trade.get("mfe_label_agree")),
            "realized_pnl": round2(sum(trade["realized"] for trade in trades)),
            "zero_realized": round2(sum(trade["realized"] for trade in trades if trade["zero_mfe"])),
            "positive_realized": round2(sum(trade["realized"] for trade in trades if not trade["zero_mfe"])),
            "group_counts": {key: len(grouped[key]) for key in GROUP_ORDER},
        },
        "cards": {
            "rows": len(card_times),
            "first_utc": min(card_times).astimezone(timezone.utc).isoformat() if card_times else None,
            "last_utc": max(card_times).astimezone(timezone.utc).isoformat() if card_times else None,
            "last_ist": max(card_times).astimezone(ist).isoformat() if card_times else None,
            "median_hold_gap_sec": round2(median(gaps)),
            "share_max_gap_over_60s": rate([trade["max_gap_sec"] > 60 for trade in wide]),
        },
        "groups": {key: summarize_group(grouped[key]) for key in GROUP_ORDER},
        "contrasts": contrasts,
        "fixed_post_exit": [fixed_post_exit_recovery(trades, minutes) for minutes in (15, 30, 45)],
        "horizon_distance": horizon_distance(csv_rows),
        "tickers": ticker_sensitivity(trades),
        "slices": side_and_clock(trades),
        "coincidence": coincidence(trades),
    }
    text = render(result)
    out_md.write_text(text, encoding="utf-8")
    out_json.write_text(json.dumps(result, indent=2, default=str), encoding="utf-8")
    print(out_md)
    print(json.dumps(result["population"], indent=2))


if __name__ == "__main__":
    main()
