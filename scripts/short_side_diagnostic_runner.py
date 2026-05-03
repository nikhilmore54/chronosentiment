#!/usr/bin/env python3
"""
Read-only runner for short-side diagnostic logs.

Aligned with .cursor/rules/chronosentiment-core.mdc:
- deterministic parsing only
- no strategy/gate mutation
- reproducible decision summary from emitted diagnostics
"""

from __future__ import annotations

import argparse
import json
import re
from pathlib import Path
from typing import Any

RAW_RE = re.compile(
    r"\[RAW_TENDENCY\]\s+bullish_events=(\d+)\s+bearish_events=(\d+)\s+wait_events=(\d+)"
)
SIDE_RE = re.compile(
    r"\[SIDE_DISTRIBUTION\].*?"
    r"candidates_buy=(\d+)\s+candidates_sell=(\d+)\s+"
    r"pass_buy=(\d+)\s+pass_sell=(\d+)\s+"
    r"final_buy=(\d+)\s+final_sell=(\d+)\s+"
    r"intents_created_buy=(\d+)\s+intents_created_sell=(\d+)\s+"
    r"intents_triggered_buy=(\d+)\s+intents_triggered_sell=(\d+)"
)
COMP_RE = re.compile(
    r"\[COMPONENT_DIAGNOSTIC\].*?"
    r"momentum_neg=(\d+)\s+composite_neg=(\d+)\s+score_neg=(\d+)\s+near_bearish=(\d+)"
)
DIAG_RE = re.compile(
    r"\[DIAG\].*?"
    r"edge=([-+]?\d*\.?\d+)\s+conf=([-+]?\d*\.?\d+).*?"
    r"voters=(\d+)\s+p90=([-+]?\d*\.?\d+).*?"
    r"rej:no_reco=(\d+)\s+low_edge=(\d+)\s+low_feas=(\d+)"
)
EDGE_PIPE_RE = re.compile(
    r"\[EDGE_PIPE\].*?"
    r"raw_edge=([-+]?\d*\.?\d+)\s+capture_prob=([-+]?\d*\.?\d+)\s+"
    r"expected_realized_edge=([-+]?\d*\.?\d+)\s+edge_gate=([-+]?\d*\.?\d+)\s+"
    r"edge_min=([-+]?\d*\.?\d+)"
)
SYMBOL_TS_RE = re.compile(r"\[SYMBOL_TS\]\s+(.+)$")
SYMBOL_PRICE_RE = re.compile(r"\[SYMBOL_PRICE\]\s+(.+)$")
REC_OUTCOME_RE = re.compile(r"\[REC_OUTCOME\].*?\ssym=([^\s]+).*?\spnl=([-+]?\d*\.?\d+)")


def parse_log(path: Path) -> dict[str, Any] | None:
    raw = side = comp = None
    diag_last: tuple[float, float, int, float, int, int, int] | None = None
    symbol_timestamps: dict[str, int] = {}
    symbol_prices: dict[str, float] = {}
    ticker_trade_summary: dict[str, dict[str, Any]] = {}
    symbol_ts_count = 0
    diag_lines = 0
    diag_nonzero = 0
    diag_low_edge_sum = 0
    edge_pipe_last: tuple[float, float, float, float, float] | None = None
    edge_pipe_lines = 0
    edge_pipe_nonzero = 0
    with path.open("r", encoding="utf-8") as f:
        for line in f:
            m = SYMBOL_PRICE_RE.search(line)
            if m:
                payload = m.group(1).strip()
                for part in payload.split(","):
                    part = part.strip()
                    if ":" not in part:
                        continue
                    sym, val_str = part.rsplit(":", 1)
                    sym = sym.strip()
                    if not sym:
                        continue
                    try:
                        symbol_prices[sym] = float(val_str.strip())
                    except ValueError:
                        continue
            m = SYMBOL_TS_RE.search(line)
            if m:
                symbol_ts_count += 1
                payload = m.group(1).strip()
                for part in payload.split(","):
                    part = part.strip()
                    if ":" not in part:
                        continue
                    sym, ts_str = part.rsplit(":", 1)
                    try:
                        ts_val = int(ts_str.strip())
                    except ValueError:
                        continue
                    sym = sym.strip()
                    if not sym:
                        continue
                    prev = symbol_timestamps.get(sym)
                    if prev is None or ts_val > prev:
                        symbol_timestamps[sym] = ts_val
            m = REC_OUTCOME_RE.search(line)
            if m:
                sym = m.group(1).strip()
                try:
                    pnl = float(m.group(2))
                except ValueError:
                    pnl = 0.0
                s = ticker_trade_summary.setdefault(
                    sym,
                    {
                        "closed_trades": 0,
                        "wins": 0,
                        "losses": 0,
                        "total_pnl": 0.0,
                        "last_pnl": 0.0,
                    },
                )
                s["closed_trades"] += 1
                s["total_pnl"] += pnl
                s["last_pnl"] = pnl
                if pnl >= 0.0:
                    s["wins"] += 1
                else:
                    s["losses"] += 1
            m = RAW_RE.search(line)
            if m:
                raw = tuple(map(int, m.groups()))
            m = SIDE_RE.search(line)
            if m:
                side = tuple(map(int, m.groups()))
            m = COMP_RE.search(line)
            if m:
                comp = tuple(map(int, m.groups()))
            m = DIAG_RE.search(line)
            if m:
                try:
                    edge = float(m.group(1))
                    conf = float(m.group(2))
                    voters = int(m.group(3))
                    p90 = float(m.group(4))
                    rej_no_reco = int(m.group(5))
                    low_edge = int(m.group(6))
                    low_feas = int(m.group(7))
                    diag_last = (
                        edge,
                        conf,
                        voters,
                        p90,
                        rej_no_reco,
                        low_edge,
                        low_feas,
                    )
                    diag_lines += 1
                    diag_low_edge_sum += low_edge
                    if edge > 0.0 or conf > 0.0 or voters > 0:
                        diag_nonzero += 1
                except ValueError:
                    pass
            m = EDGE_PIPE_RE.search(line)
            if m:
                try:
                    raw_edge = float(m.group(1))
                    capture_prob = float(m.group(2))
                    expected_realized_edge = float(m.group(3))
                    edge_gate = float(m.group(4))
                    edge_min = float(m.group(5))
                    edge_pipe_last = (
                        raw_edge,
                        capture_prob,
                        expected_realized_edge,
                        edge_gate,
                        edge_min,
                    )
                    edge_pipe_lines += 1
                    if raw_edge > 0.0 or expected_realized_edge > 0.0:
                        edge_pipe_nonzero += 1
                except ValueError:
                    pass
    has_ticker_level = bool(symbol_timestamps) or bool(ticker_trade_summary)
    if not (raw and side and comp):
        if not has_ticker_level:
            return None
        # Live stream may be mid-run; allow partial row for ticker-level observability.
        raw = (0, 0, 0)
        side = (0, 0, 0, 0, 0, 0, 0, 0, 0, 0)
        comp = (0, 0, 0, 0)

    bullish, bearish, wait = raw
    (
        cand_buy,
        cand_sell,
        pass_buy,
        pass_sell,
        final_buy,
        final_sell,
        created_buy,
        created_sell,
        trig_buy,
        trig_sell,
    ) = side
    momentum_neg, composite_neg, score_neg, near_bearish = comp
    return {
        "file": str(path),
        "file_name": path.name,
        "raw_tendency": {
            "bullish_events": bullish,
            "bearish_events": bearish,
            "wait_events": wait,
        },
        "side_distribution": {
            "candidates_buy": cand_buy,
            "candidates_sell": cand_sell,
            "pass_buy": pass_buy,
            "pass_sell": pass_sell,
            "final_buy": final_buy,
            "final_sell": final_sell,
            "intents_created_buy": created_buy,
            "intents_created_sell": created_sell,
            "intents_triggered_buy": trig_buy,
            "intents_triggered_sell": trig_sell,
        },
        "component_diagnostic": {
            "momentum_neg": momentum_neg,
            "composite_neg": composite_neg,
            "score_neg": score_neg,
            "near_bearish": near_bearish,
        },
        "diag_signal": {
            "edge": diag_last[0] if diag_last is not None else 0.0,
            "conf": diag_last[1] if diag_last is not None else 0.0,
            "voters": diag_last[2] if diag_last is not None else 0,
            "p90": diag_last[3] if diag_last is not None else 0.0,
            "rej_no_reco": diag_last[4] if diag_last is not None else 0,
            "low_edge": diag_last[5] if diag_last is not None else 0,
            "low_feas": diag_last[6] if diag_last is not None else 0,
            "diag_lines": diag_lines,
            "diag_nonzero": diag_nonzero,
            "diag_low_edge_sum": diag_low_edge_sum,
            "edge_pipe_lines": edge_pipe_lines,
            "edge_pipe_nonzero": edge_pipe_nonzero,
            "raw_edge": edge_pipe_last[0] if edge_pipe_last is not None else 0.0,
            "capture_prob": edge_pipe_last[1] if edge_pipe_last is not None else 0.0,
            "expected_realized_edge": edge_pipe_last[2] if edge_pipe_last is not None else 0.0,
            "edge_gate": edge_pipe_last[3] if edge_pipe_last is not None else 0.0,
            "edge_min": edge_pipe_last[4] if edge_pipe_last is not None else 0.0,
        },
        "symbol_timestamps": symbol_timestamps,
        "symbol_ts_count": symbol_ts_count,
        "symbol_prices": symbol_prices,
        "ticker_trade_summary": ticker_trade_summary,
        "near_bearish_ratio": near_bearish / (bullish + 1),
    }


def classify(parsed: dict[str, Any]) -> str:
    bearish = parsed["raw_tendency"]["bearish_events"]
    near = parsed["component_diagnostic"]["near_bearish"]
    sell_cand = parsed["side_distribution"]["candidates_sell"]
    final_sell = parsed["side_distribution"]["final_sell"]

    if final_sell > 0 or sell_cand > 0:
        return "Pattern A (Shorts present)"
    if bearish == 0 and near == 0:
        return "Pattern C (No bearish structure)"
    if bearish == 0 and near > 0:
        return "Pattern C (Sparse near-bearish)"
    if bearish > 0 and sell_cand == 0:
        return "Pattern B (Bearish seen, not mapped)"
    return "Unclassified"


def run_diagnostics(log_paths: list[Path | str]) -> dict[str, Any]:
    """
    Parse one or more diagnostic log files. Returns rows, summary, and parse errors.
    Safe to import from Streamlit (read-only, deterministic).
    """
    parsed_rows: list[dict[str, Any]] = []
    parse_errors: list[str] = []
    for log in log_paths:
        p = Path(log)
        if not p.exists():
            parse_errors.append(str(p))
            continue
        row = parse_log(p)
        if row is None:
            parse_errors.append(str(p))
            continue
        row["classification"] = classify(row)
        parsed_rows.append(row)

    parsed_rows.sort(key=lambda r: r["file_name"])
    summary = summarize(parsed_rows) if parsed_rows else {}
    return {
        "rows": parsed_rows,
        "summary": summary,
        "parse_errors": parse_errors,
    }


def collect_diagnostic_logs(
    directory: str,
    pattern: str = "diag_*.log",
) -> list[Path]:
    """List matching log files under a directory, sorted by name."""
    d = Path(directory)
    if not d.is_dir():
        return []
    return sorted(d.glob(pattern), key=lambda p: p.name)


def summarize(rows: list[dict[str, Any]]) -> dict[str, Any]:
    n = len(rows)
    total_slices = sum(int(r.get("symbol_ts_count", 0) or 0) for r in rows)
    near_offsets = sum(1 for r in rows if r["component_diagnostic"]["near_bearish"] > 0)
    bearish_offsets = sum(1 for r in rows if r["raw_tendency"]["bearish_events"] > 0)
    sell_cand_offsets = sum(1 for r in rows if r["side_distribution"]["candidates_sell"] > 0)
    sell_final_offsets = sum(1 for r in rows if r["side_distribution"]["final_sell"] > 0)

    if sell_final_offsets > 0 or sell_cand_offsets > 0:
        recommendation = "SHORTS_OBSERVED_REVIEW_GATES"
    elif bearish_offsets > 0:
        recommendation = "CHECK_DIRECTIONAL_MAPPING"
    elif near_offsets >= max(1, n // 2):
        recommendation = "CONSIDER_SHADOW_EPSILON_MAPPING"
    else:
        recommendation = "KEEP_LONG_ONLY"

    return {
        "files_parsed": n,
        "total_slices": total_slices,
        "offsets_with_near_bearish": near_offsets,
        "offsets_with_bearish_events": bearish_offsets,
        "offsets_with_sell_candidates": sell_cand_offsets,
        "offsets_with_final_sell": sell_final_offsets,
        "recommendation": recommendation,
    }


def main() -> int:
    ap = argparse.ArgumentParser(description=__doc__)
    ap.add_argument("logs", nargs="+", help="diagnostic log paths")
    ap.add_argument("--json", action="store_true", help="emit structured JSON output")
    args = ap.parse_args()

    result = run_diagnostics(list(args.logs))
    parsed_rows = result["rows"]
    summary = result["summary"]
    parse_errors = result["parse_errors"]

    if args.json:
        print(
            json.dumps(
                {"rows": parsed_rows, "summary": summary, "parse_errors": parse_errors},
                indent=2,
            )
        )
        return 0

    print(
        f"{'file':<42} | {'bull':>4} | {'bear':>4} | {'near':>4} | "
        f"{'BUY_c':>5} | {'SELL_c':>6} | {'SELL_f':>6} | class"
    )
    print("-" * 122)
    for r in parsed_rows:
        bull = r["raw_tendency"]["bullish_events"]
        bear = r["raw_tendency"]["bearish_events"]
        near = r["component_diagnostic"]["near_bearish"]
        buy_c = r["side_distribution"]["candidates_buy"]
        sell_c = r["side_distribution"]["candidates_sell"]
        sell_f = r["side_distribution"]["final_sell"]
        print(
            f"{r['file_name']:<42} | {bull:>4} | {bear:>4} | {near:>4} | "
            f"{buy_c:>5} | {sell_c:>6} | {sell_f:>6} | {r['classification']}"
        )

    if parse_errors:
        print("\nParse errors:")
        for e in parse_errors:
            print(f"- {e}")

    if summary:
        print("\nSummary:")
        print(
            f"offsets_with_near_bearish={summary['offsets_with_near_bearish']}/"
            f"{summary['files_parsed']}"
        )
        print(
            f"offsets_with_bearish_events={summary['offsets_with_bearish_events']}/"
            f"{summary['files_parsed']}"
        )
        print(
            f"offsets_with_sell_candidates={summary['offsets_with_sell_candidates']}/"
            f"{summary['files_parsed']}"
        )
        print(
            f"offsets_with_final_sell={summary['offsets_with_final_sell']}/"
            f"{summary['files_parsed']}"
        )
        print(f"FINAL_CLASSIFICATION: {summary['recommendation']}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
