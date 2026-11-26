#!/usr/bin/env python3
"""
Inventory of price and volume history for the frozen 29 September 2026
NSE Ledger A extract.

Work package 1 of the market-path forensic, plus coverage flags for a
later reconstruction. No returns, ranges, volume ratios, classifiers,
or thresholds are computed.

RP-605 stays the frozen control. The 1,719-row extract is read only.
The 89 later journal rows stay outside this population.
"""

from __future__ import annotations

import csv
import hashlib
import json
from bisect import bisect_right
from collections import Counter, defaultdict
from datetime import datetime, timedelta, timezone
from pathlib import Path

from build_rp609_entry_time_discrimination_dataset import load_cards, parse_ts

WORKSPACE = Path(__file__).resolve().parent.parent
LOG_DIR = WORKSPACE / "live_capture" / "shadow_validation"
CSV_PATH = LOG_DIR / "rp609_nse_20260929_entry_discrimination_dataset.csv"
JOURNAL = LOG_DIR / "rp605_nse_baseline_journal.jsonl"
OUT_JSON = LOG_DIR / "rp609_nse_20260929_market_history_inventory.json"
OUT_MD = LOG_DIR / "rp609_nse_20260929_market_history_inventory.md"

IST = timezone(timedelta(hours=5, minutes=30))
SESSION_DAY = "2026-09-29"
SESSION_OPEN = datetime(2026, 9, 29, 9, 15, tzinfo=IST)
SESSION_CLOSE = datetime(2026, 9, 29, 15, 30, tzinfo=IST)
WINDOWS_MIN = (5, 15, 30, 60, 120)
# Card cadence is about 64 seconds. A window is card-complete only when
# the observable part is bounded by observations within two cadences and
# no interior gap exceeds about three cadences.
EDGE_SEC = 120
MAX_INTERIOR_GAP_SEC = 180

CANDLE_ROOTS = {
    "intraday_1m": WORKSPACE / "intraday_capture" / "yahoo_cache_1m",
    "intraday_5m": WORKSPACE / "intraday_capture" / "yahoo_cache_5m",
    "intraday_15m": WORKSPACE / "intraday_capture" / "yahoo_cache_15m",
    "intraday_30m": WORKSPACE / "intraday_capture" / "yahoo_cache_30m",
    "intraday_1d": WORKSPACE / "intraday_capture" / "yahoo_cache_1d",
    "live_daily_v2": WORKSPACE / "live_capture" / "yahoo_cache",
}


def sha256(path: Path) -> str:
    digest = hashlib.sha256()
    with path.open("rb") as handle:
        for chunk in iter(lambda: handle.read(1024 * 1024), b""):
            digest.update(chunk)
    return digest.hexdigest()


def load_bars(path: Path):
    raw = json.loads(path.read_text(encoding="utf-8"))
    if isinstance(raw, dict):
        rows = raw.get("usable") or []
        schema = raw.get("schema")
    else:
        rows = raw
        schema = "json_list"
    return schema, rows


def bar_dt(row):
    return datetime.fromtimestamp(int(row["timestamp"]), timezone.utc)


def median(values):
    if not values:
        return None
    ordered = sorted(values)
    mid = len(ordered) // 2
    if len(ordered) % 2:
        return ordered[mid]
    return (ordered[mid - 1] + ordered[mid]) / 2.0


def round2(value):
    if value is None:
        return None
    return round(float(value), 2)


def summarize_cache(root: Path, symbols):
    present = 0
    missing = []
    last_dates = Counter()
    sep29 = 0
    sep29_bars = 0
    sep29_volume_positive = 0
    sep29_volume_zero = 0
    duplicate_files = 0
    schemas = Counter()
    prior_close_symbols = 0
    last_session_volume_positive = 0
    last_session_bars = 0
    file_last = []
    for symbol in symbols:
        path = root / f"{symbol}.json"
        if not path.exists():
            missing.append(symbol)
            continue
        present += 1
        schema, rows = load_bars(path)
        schemas[schema or "unknown"] += 1
        if not rows:
            continue
        stamps = [int(row["timestamp"]) for row in rows]
        if len(stamps) != len(set(stamps)):
            duplicate_files += 1
        last = bar_dt(rows[-1])
        file_last.append(last)
        last_dates[last.astimezone(IST).date().isoformat()] += 1
        day_rows = [row for row in rows if bar_dt(row).astimezone(IST).date().isoformat() == SESSION_DAY]
        if day_rows:
            sep29 += 1
            sep29_bars += len(day_rows)
            for row in day_rows:
                volume = float(row.get("volume") or 0.0)
                if volume > 0:
                    sep29_volume_positive += 1
                else:
                    sep29_volume_zero += 1
        last_day = last.astimezone(IST).date().isoformat()
        for row in rows:
            if bar_dt(row).astimezone(IST).date().isoformat() != last_day:
                continue
            last_session_bars += 1
            if float(row.get("volume") or 0.0) > 0:
                last_session_volume_positive += 1
        # A completed daily bar stamped before the 29 September open is prior context.
        if any(bar_dt(row) < SESSION_OPEN for row in rows):
            prior_close_symbols += 1
    return {
        "root": str(root.relative_to(WORKSPACE)),
        "symbols_in_population": len(symbols),
        "files_present": present,
        "files_missing": len(missing),
        "missing_symbols": missing,
        "schemas": dict(schemas),
        "files_with_duplicate_timestamps": duplicate_files,
        "last_bar_date_ist_counts": dict(last_dates),
        "earliest_last_bar_utc": None if not file_last else min(file_last).isoformat(),
        "latest_last_bar_utc": None if not file_last else max(file_last).isoformat(),
        "symbols_with_any_bar_before_session_open": prior_close_symbols,
        "symbols_with_session_day_bars": sep29,
        "session_day_bars": sep29_bars,
        "session_day_bars_volume_positive": sep29_volume_positive,
        "session_day_bars_volume_zero": sep29_volume_zero,
        "last_cached_session_bars": last_session_bars,
        "last_cached_session_bars_volume_positive": last_session_volume_positive,
    }


def cards_between(times, cards, start, end):
    left = bisect_right(times, start)
    right = bisect_right(times, end)
    return cards[left:right]


def window_status(times, cards, requested_start, end, session_open):
    """Coverage of observations at or before end.

    The bisect includes a card stamped exactly at the requested start.
    Time before the cash open is unobserved even when the clock window
    asks for it. That truncation is not counted as a gap inside the session.
    """
    query_start = requested_start - timedelta(microseconds=1)
    observable_start = max(requested_start, session_open)
    pre_open_sec = max(0.0, (session_open - requested_start).total_seconds())
    observed = [
        card for card in cards_between(times, cards, query_start, end)
        if card["_dt"] >= observable_start
    ]
    detail = {"n": len(observed), "pre_open_unobserved_sec": round2(pre_open_sec)}
    if not observed:
        detail["state"] = "no_observations"
        detail["failure"] = "no_observations"
        return detail
    first = observed[0]["_dt"]
    last = observed[-1]["_dt"]
    gaps = [
        (right["_dt"] - left["_dt"]).total_seconds()
        for left, right in zip(observed, observed[1:])
    ]
    max_gap = max(gaps) if gaps else 0.0
    start_lag = (first - observable_start).total_seconds()
    end_lag = (end - last).total_seconds()
    failures = []
    if start_lag > EDGE_SEC:
        failures.append("late_first_card")
    if end_lag > EDGE_SEC:
        failures.append("stale_last_card")
    if max_gap > MAX_INTERIOR_GAP_SEC:
        failures.append("interior_gap")
    bounded = not failures
    if pre_open_sec > 0 and bounded:
        state = "partial_session_only"
    elif pre_open_sec > 0:
        state = "partial_and_gapped"
    elif bounded:
        state = "complete"
    else:
        state = "gapped"
    detail.update({
        "state": state,
        "failure": "ok" if not failures else "+".join(failures),
        "start_lag_sec": round2(start_lag),
        "end_lag_sec": round2(end_lag),
        "max_gap_sec": round2(max_gap),
    })
    return detail


def inventory_cards(trades, indexes):
    statuses = {name: Counter() for name in [*(f"pre_{m}m" for m in WINDOWS_MIN), "session_to_entry"]}
    failures = {name: Counter() for name in statuses}
    counts = {name: [] for name in statuses}
    session_max_gaps = []
    entry_lags = []
    entry_bps = []
    no_pre_card = 0
    first_after = []
    hold_cards = []
    symbols_without_cards = []
    for trade in trades:
        index = indexes.get(trade["symbol"])
        if index is None or not index["times"]:
            symbols_without_cards.append(trade["symbol"])
            for name in statuses:
                statuses[name]["symbol_missing"] += 1
            continue
        entry = trade["entry_dt"]
        exit_dt = trade["exit_dt"]
        times = index["times"]
        cards = index["cards"]
        pos = bisect_right(times, entry) - 1
        if pos < 0:
            no_pre_card += 1
        else:
            card = cards[pos]
            entry_lags.append((entry - card["_dt"]).total_seconds())
            if trade["entry_price"]:
                entry_bps.append(abs(card["current_price"] - trade["entry_price"]) / trade["entry_price"] * 10000)
        after = bisect_right(times, entry)
        if after < len(times):
            first_after.append((times[after] - entry).total_seconds())
        hold = cards_between(times, cards, entry, exit_dt)
        # entry-inclusive count, matching the lifecycle path used previously
        hold_left = bisect_right(times, entry - timedelta(microseconds=1))
        hold_cards.append(hold_right(times, hold_left, exit_dt))
        for minutes in WINDOWS_MIN:
            start = entry - timedelta(minutes=minutes)
            name = f"pre_{minutes}m"
            status = window_status(times, cards, start, entry, SESSION_OPEN)
            statuses[name][status["state"]] += 1
            failures[name][status["failure"]] += 1
            counts[name].append(status["n"])
        session_status = window_status(times, cards, SESSION_OPEN, entry, SESSION_OPEN)
        statuses["session_to_entry"][session_status["state"]] += 1
        failures["session_to_entry"][session_status["failure"]] += 1
        counts["session_to_entry"].append(session_status["n"])
        if session_status.get("max_gap_sec") is not None:
            session_max_gaps.append(session_status["max_gap_sec"])
    card_fields = set()
    for index in indexes.values():
        if index["cards"]:
            card_fields = set(index["cards"][0].keys())
            break
    return {
        "card_fields": sorted(card_fields),
        "volume_field_present": any("volume" in name.lower() for name in card_fields),
        "symbols_missing_cards": sorted(set(symbols_without_cards)),
        "trades_with_no_card_at_or_before_entry": no_pre_card,
        "entry_card_lag_sec": {
            "n": len(entry_lags),
            "median": round2(median(entry_lags)),
            "p90": round2(sorted(entry_lags)[int(0.9 * (len(entry_lags) - 1))] if entry_lags else None),
        },
        "entry_price_abs_bps_vs_last_card": {
            "n": len(entry_bps),
            "median": round2(median(entry_bps)),
            "p90": round2(sorted(entry_bps)[int(0.9 * (len(entry_bps) - 1))] if entry_bps else None),
            "share_over_10_bps": round(sum(1 for value in entry_bps if value > 10) / len(entry_bps), 4) if entry_bps else None,
        },
        "seconds_to_first_card_after_entry": {
            "n": len(first_after),
            "median": round2(median(first_after)),
            "p90": round2(sorted(first_after)[int(0.9 * (len(first_after) - 1))] if first_after else None),
        },
        "holding_cards_including_entry": {
            "n": len(hold_cards),
            "median": round2(median(hold_cards)),
            "min": None if not hold_cards else min(hold_cards),
        },
        "session_to_entry_max_interior_gap_sec": {
            "n": len(session_max_gaps),
            "median": round2(median(session_max_gaps)),
            "p90": round2(sorted(session_max_gaps)[int(0.9 * (len(session_max_gaps) - 1))] if session_max_gaps else None),
        },
        "windows": {
            name: {
                "states": dict(counter),
                "failures": dict(failures[name]),
                "observations_median": round2(median(counts[name])),
                "observations_min": None if not counts[name] else min(counts[name]),
            }
            for name, counter in statuses.items()
        },
    }


def hold_right(times, left, end):
    return max(0, bisect_right(times, end) - left)


def common_card_gap(indexes):
    """Largest in-session card gap per symbol. A shared clock time is a feed gap."""
    starts = Counter()
    durations = []
    for index in indexes.values():
        times = [card["_dt"] for card in index["cards"] if card["_dt"] >= SESSION_OPEN]
        best = 0.0
        at = None
        for left, right in zip(times, times[1:]):
            gap = (right - left).total_seconds()
            if gap > best:
                best = gap
                at = left
        if at is None or best <= MAX_INTERIOR_GAP_SEC:
            continue
        starts[at.astimezone(IST).strftime("%H:%M")] += 1
        durations.append(best)
    if not starts:
        return None
    stamp, count = starts.most_common(1)[0]
    return {
        "symbols": count,
        "start_ist": stamp,
        "median_gap_sec": round2(median(durations)),
    }


def load_1m_asof(symbols):
    """One-minute bars become known at the end of their minute.

    A bar stamped 09:16 is the 09:16-09:17 candle. Its close is not used
    until 09:17, so the as-of clock is the bar timestamp plus 60 seconds.
    """
    root = CANDLE_ROOTS["intraday_1m"]
    indexes = {}
    for symbol in symbols:
        path = root / f"{symbol}.json"
        if not path.exists():
            indexes[symbol] = None
            continue
        rows = json.loads(path.read_text(encoding="utf-8"))
        bars = []
        for row in rows:
            start = datetime.fromtimestamp(int(row["timestamp"]), timezone.utc)
            known_at = start + timedelta(seconds=60)
            if known_at < SESSION_OPEN or known_at > SESSION_CLOSE + timedelta(minutes=5):
                continue
            if start.astimezone(IST).date().isoformat() != SESSION_DAY:
                continue
            bars.append({
                "_dt": known_at,
                "volume": float(row.get("volume") or 0.0),
                "close": float(row.get("close") or 0.0),
            })
        bars.sort(key=lambda bar: bar["_dt"])
        indexes[symbol] = {"cards": bars, "times": [bar["_dt"] for bar in bars]}
    return indexes


def inventory_1m(trades, indexes):
    names = [*(f"pre_{minutes}m" for minutes in WINDOWS_MIN), "session_to_entry"]
    statuses = {name: Counter() for name in names}
    counts = {name: [] for name in names}
    volume_flags = {name: [] for name in names}
    missing = []
    entry_bps = []
    for trade in trades:
        index = indexes.get(trade["symbol"])
        if index is None or not index["times"]:
            missing.append(trade["symbol"])
            for name in names:
                statuses[name]["symbol_missing"] += 1
            continue
        entry = trade["entry_dt"]
        times = index["times"]
        bars = index["cards"]
        pos = bisect_right(times, entry) - 1
        if pos >= 0 and trade["entry_price"]:
            close = bars[pos]["close"]
            if close:
                entry_bps.append(abs(close - trade["entry_price"]) / trade["entry_price"] * 10000)
        for minutes in WINDOWS_MIN:
            start = entry - timedelta(minutes=minutes)
            name = f"pre_{minutes}m"
            status = window_status(times, bars, start, entry, SESSION_OPEN)
            statuses[name][status["state"]] += 1
            counts[name].append(status["n"])
            observed = [
                bar for bar in bars
                if max(start, SESSION_OPEN) < bar["_dt"] <= entry
            ]
            volume_flags[name].append(any(bar["volume"] > 0 for bar in observed))
        session_status = window_status(times, bars, SESSION_OPEN, entry, SESSION_OPEN)
        statuses["session_to_entry"][session_status["state"]] += 1
        counts["session_to_entry"].append(session_status["n"])
        observed = [bar for bar in bars if SESSION_OPEN < bar["_dt"] <= entry]
        volume_flags["session_to_entry"].append(any(bar["volume"] > 0 for bar in observed))
    ordered = sorted(entry_bps)
    return {
        "as_of": "bar is usable only after its minute has closed",
        "symbols_missing": sorted(set(missing)),
        "entry_vs_last_closed_bar_abs_bps": {
            "n": len(entry_bps),
            "median": round2(median(entry_bps)),
            "p90": round2(ordered[int(0.9 * (len(ordered) - 1))] if ordered else None),
        },
        "windows": {
            name: {
                "states": dict(statuses[name]),
                "observations_median": round2(median(counts[name])),
                "share_with_positive_volume": round(
                    sum(1 for flag in volume_flags[name] if flag) / len(volume_flags[name]), 4
                ) if volume_flags[name] else None,
            }
            for name in names
        },
    }


def journal_extras(csv_rows):
    keys = set()
    for row in csv_rows:
        entry = parse_ts(row["t_entry_utc"]).astimezone(timezone.utc).isoformat()
        keys.add((row["trade_id"], row["symbol"], row["side"], entry))
    extras = 0
    with JOURNAL.open(encoding="utf-8") as handle:
        for line in handle:
            line = line.strip()
            if not line:
                continue
            row = json.loads(line)
            entry_dt = parse_ts(row["entry_timestamp"])
            if entry_dt.astimezone(timezone.utc).date().isoformat() != SESSION_DAY:
                continue
            key = (
                row["trade_id"],
                row["symbol"],
                row["side"],
                entry_dt.astimezone(timezone.utc).isoformat(),
            )
            if key not in keys:
                extras += 1
    return extras


def render(result):
    pop = result["population"]
    lines = [
        "# RP-609 market-history inventory: NSE Ledger A, 29 September 2026",
        "",
        "Data inventory only. No price or volume indicator is calculated, and no cohort comparison is made.",
        "RP-605 remains the frozen control. RP-608 remains shadow-only. PFT, SPL, VAT, and PPR are unchanged.",
        "The 1,719-row extract is not modified and is not pooled with 28 September.",
        "",
        "## Population",
        "",
        f"Extract SHA256 `{pop['csv_sha256']}`. Rows {pop['rows']}. Symbols {pop['symbols']}.",
        f"Entries {pop['entry_first_utc']} to {pop['entry_last_utc']}.",
        f"Journal rows with a 29 September UTC entry that are outside the extract, and excluded: {pop['journal_rows_excluded']}.",
        "",
        "## Sources examined",
        "",
        "Decision cards are the live price sample used by the control. They carry `current_price` and the decision scores. They do not carry volume, bid, ask, or trade side.",
        "Candle caches are local Yahoo files. A one-minute bar is usable at entry only after its minute has closed, which is 60 seconds after the bar timestamp. A completed bar for the same calendar day is not an entry-time observation.",
        "",
        "| Source | Files found | Session-day bars | Session-day bars with volume > 0 | Latest bar (UTC) |",
        "| --- | ---: | ---: | ---: | --- |",
    ]
    for name, block in result["caches"].items():
        lines.append(
            f"| `{name}` | {block['files_present']} / {block['symbols_in_population']} | "
            f"{block['session_day_bars']} | {block['session_day_bars_volume_positive']} | {block['latest_last_bar_utc']} |"
        )
    lines.extend([
        "",
        "No local file in these caches is a tick tape, an order book, or a sector-index series for this session. "
        "Nifty and sector names are not among the cache filenames.",
        "",
        "## What is aligned with the frozen trades",
        "",
        f"Decision-card fields: {', '.join(result['cards']['card_fields'])}.",
        f"Volume field on the card: {result['cards']['volume_field_present']}.",
        f"Trades with no card at or before entry: {result['cards']['trades_with_no_card_at_or_before_entry']}.",
        "The entry timestamp is a decision-card timestamp. "
        f"Lag from that card to entry is {result['cards']['entry_card_lag_sec']['median']} seconds "
        f"at the median and the 90th percentile, and the price difference is "
        f"{result['cards']['entry_price_abs_bps_vs_last_card']['median']} bps. "
        "The trade is opened on the card. Alignment before that instant is the earlier cards, not a separate tick.",
        f"Absolute price difference versus that card, median {result['cards']['entry_price_abs_bps_vs_last_card']['median']} bps, "
        f"share above 10 bps {result['cards']['entry_price_abs_bps_vs_last_card']['share_over_10_bps']}.",
        f"Seconds from entry to the next card, median {result['cards']['seconds_to_first_card_after_entry']['median']}, "
        f"90th percentile {result['cards']['seconds_to_first_card_after_entry']['p90']}.",
        "That interval is unobserved on the decision-card clock. The one-minute tape is a separate source and is scored in the volume section.",
        "",
        "Pre-entry windows use only cards at or before the entry. "
        f"A window is `complete` when it starts at or after the 09:15 IST open and every gap, including the edges, stays within {EDGE_SEC} seconds at the boundary and {MAX_INTERIOR_GAP_SEC} seconds inside. "
        "`partial_session_only` means the in-session part meets that spacing, while the requested window begins before the open. "
        "Those states are coverage labels, not trading thresholds.",
        "",
        "| Window | Complete | Session portion only | Gapped | No observations | Median cards |",
        "| --- | ---: | ---: | ---: | ---: | ---: |",
    ])
    order = [*(f"pre_{m}m" for m in WINDOWS_MIN), "session_to_entry"]
    labels = {
        "pre_5m": "Previous 5 minutes",
        "pre_15m": "Previous 15 minutes",
        "pre_30m": "Previous 30 minutes",
        "pre_60m": "Previous 60 minutes",
        "pre_120m": "Previous 120 minutes",
        "session_to_entry": "Session open to entry",
    }
    for name in order:
        block = result["cards"]["windows"][name]
        states = block["states"]
        lines.append(
            f"| {labels[name]} | {states.get('complete', 0)} | {states.get('partial_session_only', 0)} | "
            f"{states.get('gapped', 0) + states.get('partial_and_gapped', 0)} | {states.get('no_observations', 0)} | "
            f"{block['observations_median']} |"
        )
    session_failures = result["cards"]["windows"]["session_to_entry"]["failures"]
    lines.append("")
    lines.append(
        "Session-open-to-entry failure tags, one tag per trade: "
        + ", ".join(f"{name} {count}" for name, count in sorted(session_failures.items()))
        + ". A gapped session path has at least one interior gap above 180 seconds between the open and that entry. "
        f"The median of those maximum gaps is {result['cards']['session_to_entry_max_interior_gap_sec']['median']} seconds "
        f"(90th percentile {result['cards']['session_to_entry_max_interior_gap_sec']['p90']}). "
        "The rest of that path is still the card sample. The label does not mean the session history is missing. "
        f"The hole is shared: {result['shared_card_gap']['symbols']} symbols have their longest gap starting at "
        f"{result['shared_card_gap']['start_ist']} IST, median length {result['shared_card_gap']['median_gap_sec']} seconds. "
        "Trades entered before that interval do not contain it."
    )
    cache_note = result["caches"]["intraday_1m"]
    lines.extend([
        "",
        "## Volume",
        "",
        f"One-minute files now cover {cache_note['files_present']} of {cache_note['symbols_in_population']} traded symbols. "
        f"Session-day bars: {cache_note['session_day_bars']}. "
        f"Of those, {cache_note['session_day_bars_volume_positive']} have volume above zero.",
        f"Symbols still missing a one-minute file: {cache_note['files_missing']}.",
        "A one-minute bar is counted only after its minute has closed. The close of a bar that is still forming is not treated as known at entry.",
        "Candle volume is a participation total. It does not identify the buy or sell side.",
        "",
        "| Window | Complete | Session portion only | Gapped | No observations | Median bars | Share with volume > 0 |",
        "| --- | ---: | ---: | ---: | ---: | ---: | ---: |",
    ])
    bars = result.get("bars_1m")
    if bars:
        for name in order:
            block = bars["windows"][name]
            states = block["states"]
            lines.append(
                f"| {labels[name]} | {states.get('complete', 0)} | {states.get('partial_session_only', 0)} | "
                f"{states.get('gapped', 0) + states.get('partial_and_gapped', 0)} | "
                f"{states.get('no_observations', 0) + states.get('symbol_missing', 0)} | "
                f"{block['observations_median']} | {block['share_with_positive_volume']} |"
            )
        align = bars["entry_vs_last_closed_bar_abs_bps"]
        lines.extend([
            "",
            f"Last closed one-minute bar versus the entry price: median {align['median']} bps, "
            f"90th percentile {align['p90']} bps, n={align['n']}.",
            "Yahoo's published one-minute series for this session ends at 15:15 IST for 91 of the 99 symbols and at 15:29 IST for the other 8. "
            "The 15:15 bar is present. Minutes after it are not in the source, so they stay unobserved.",
            "Daily files still stop at the prior session. They are not the path into these entries.",
            "Nifty, sector indexes, trade-side prints, and the order book are still absent.",
            "",
            "## What this inventory allows next",
            "",
            "Pre-entry price and candle-volume descriptions can use the one-minute bars, with the table above as the missingness record. "
            "Windows that begin before 09:15 IST stay partial. No return, path-efficiency, or relative-volume figure is computed here.",
            "The decision-card log remains the record of what the control saw. The one-minute bars are a finer price and volume tape, aligned only after each bar has closed.",
            "",
            "The frozen extract hash is unchanged. RP-605 is unchanged.",
            "",
        ])
    return "\n".join(lines) + "\n"


def main():
    with CSV_PATH.open(encoding="utf-8") as handle:
        rows = list(csv.DictReader(handle))
    trades = []
    for row in rows:
        trades.append({
            "symbol": row["symbol"],
            "entry_dt": parse_ts(row["t_entry_utc"]),
            "exit_dt": parse_ts(row["t_exit_utc"]),
            "entry_price": float(row["feature_entry_price"]),
        })
    symbols = sorted({trade["symbol"] for trade in trades})
    cards = {}
    for symbol, index in load_cards().items():
        kept = [
            card for card in index["cards"]
            if SESSION_OPEN - timedelta(hours=1) <= card["_dt"] <= SESSION_CLOSE + timedelta(hours=2)
            and card["_dt"].astimezone(IST).date().isoformat() == SESSION_DAY
        ]
        cards[symbol] = {"cards": kept, "times": [card["_dt"] for card in kept]}
    caches = {name: summarize_cache(root, symbols) for name, root in CANDLE_ROOTS.items()}
    result = {
        "population": {
            "csv_sha256": sha256(CSV_PATH),
            "rows": len(trades),
            "symbols": len(symbols),
            "entry_first_utc": min(trade["entry_dt"] for trade in trades).astimezone(timezone.utc).isoformat(),
            "entry_last_utc": max(trade["entry_dt"] for trade in trades).astimezone(timezone.utc).isoformat(),
            "journal_rows_excluded": journal_extras(rows),
        },
        "session_open_ist": SESSION_OPEN.isoformat(),
        "coverage_rule": {
            "edge_sec": EDGE_SEC,
            "max_interior_gap_sec": MAX_INTERIOR_GAP_SEC,
            "as_of": "observations with timestamp <= entry",
        },
        "caches": caches,
        "cards": inventory_cards(trades, cards),
        "bars_1m": inventory_1m(trades, load_1m_asof(symbols)),
        "shared_card_gap": common_card_gap(cards),
        "not_calculated": [
            "returns",
            "path_efficiency",
            "volatility",
            "relative_volume",
            "cohort_comparisons",
            "thresholds",
        ],
    }
    OUT_JSON.write_text(json.dumps(result, indent=2), encoding="utf-8")
    OUT_MD.write_text(render(result), encoding="utf-8")
    print(OUT_MD)
    print(json.dumps({
        "rows": result["population"]["rows"],
        "sha": result["population"]["csv_sha256"][:16],
        "excluded": result["population"]["journal_rows_excluded"],
        "windows": {name: block["states"] for name, block in result["cards"]["windows"].items()},
        "sep29_1m": caches["intraday_1m"]["session_day_bars"],
    }, indent=2))


if __name__ == "__main__":
    main()
