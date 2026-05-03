#!/usr/bin/env python3
"""
Log-derived split view for [RECOMMENDATION] (src=) and optional [REC_OUTCOME] (pnl + src=).

Aligned with chronosentiment-core.mdc — reproducible aggregation from captured stdout.

Example:
  python3 scripts/reco_attribution_summary.py analysis/live_multi/live_BTC_USD.log
  grep -E '\\[RECOMMENDATION\\]|\\[REC_OUTCOME\\]' live.log | python3 scripts/reco_attribution_summary.py
"""
from __future__ import annotations

import argparse
import math
import re
import sys
from collections import defaultdict
from pathlib import Path

REC_LINE = re.compile(
    r"\[RECOMMENDATION\] rec_id=(?P<rec_id>\d+).*?\bedge=(?P<edge>[0-9.eE+-]+).*?\bsrc=(?P<src>strategy|momentum_bootstrap)\b"
)
OUTCOME_LINE = re.compile(
    r"\[REC_OUTCOME\] rec_id=(?P<rec_id>\d+).*?pnl=(?P<pnl>[0-9.eE+-]+)(?:\s+src=(?P<src>strategy|momentum_bootstrap))?"
)


def ordered_matches(lines: list[str]) -> list[tuple[str, float]]:
    out: list[tuple[str, float]] = []
    for line in lines:
        m = REC_LINE.search(line)
        if not m:
            continue
        edge = float(m.group("edge"))
        if not math.isfinite(edge):
            continue
        out.append((m.group("src"), edge))
    return out


def rec_id_to_src_map(lines: list[str]) -> dict[int, str]:
    m: dict[int, str] = {}
    for line in lines:
        r = REC_LINE.search(line)
        if r:
            m[int(r.group("rec_id"))] = r.group("src")
    return m


def cluster_counts_by_src(ordered: list[tuple[str, float]]) -> dict[str, int]:
    """Count maximal consecutive runs with length > 1 per src."""
    counts: dict[str, int] = defaultdict(int)
    if not ordered:
        return counts
    i = 0
    n = len(ordered)
    while i < n:
        src = ordered[i][0]
        j = i + 1
        while j < n and ordered[j][0] == src:
            j += 1
        run_len = j - i
        if run_len > 1:
            counts[src] += 1
        i = j
    return counts


def summarize(lines: list[str]) -> dict[str, dict[str, float]]:
    ordered = ordered_matches(lines)
    by_src: dict[str, list[float]] = defaultdict(list)
    for src, edge in ordered:
        by_src[src].append(edge)

    max_run_by_src: dict[str, int] = defaultdict(int)
    if ordered:
        cur_src, cur_n = ordered[0][0], 1
        max_run_by_src[cur_src] = 1
        for src, _ in ordered[1:]:
            if src == cur_src:
                cur_n += 1
            else:
                cur_src = src
                cur_n = 1
            max_run_by_src[src] = max(max_run_by_src[src], cur_n)

    clusters = cluster_counts_by_src(ordered)

    out: dict[str, dict[str, float]] = {}
    for src, edges in by_src.items():
        nrec = len(edges)
        avg = sum(edges) / nrec if nrec else 0.0
        out[src] = {
            "reco_count": float(nrec),
            "edge_avg": avg,
            "max_consecutive_src": float(max_run_by_src.get(src, 0)),
            "cluster_count": float(clusters.get(src, 0)),
        }
    return out


def parse_outcomes(
    lines: list[str], id_src: dict[int, str]
) -> dict[str, list[float]]:
    """PnL lists per src (line src= overrides; else join by rec_id)."""
    by_src: dict[str, list[float]] = defaultdict(list)
    for line in lines:
        m = OUTCOME_LINE.search(line)
        if not m:
            continue
        rid = int(m.group("rec_id"))
        pnl = float(m.group("pnl"))
        if not math.isfinite(pnl):
            continue
        src = m.group("src")
        if not src:
            src = id_src.get(rid)
        if not src:
            src = "unknown"
        by_src[src].append(pnl)
    return by_src


def main() -> None:
    ap = argparse.ArgumentParser(
        description="Aggregate RECOMMENDATION / REC_OUTCOME by src="
    )
    ap.add_argument(
        "path",
        nargs="?",
        help="Log file (default: stdin)",
    )
    args = ap.parse_args()
    if args.path:
        text = Path(args.path).read_text(encoding="utf-8", errors="replace")
        lines = text.splitlines()
    else:
        lines = sys.stdin.read().splitlines()

    stats = summarize(lines)
    id_src = rec_id_to_src_map(lines)
    outcome_pnls = parse_outcomes(lines, id_src)

    if not stats and not outcome_pnls:
        print(
            "No matching [RECOMMENDATION] or [REC_OUTCOME] lines.",
            file=sys.stderr,
        )
        sys.exit(1)

    if stats:
        total = sum(int(v["reco_count"]) for v in stats.values())
        print("### [RECOMMENDATION] by src")
        print(
            f"{'src':<22} {'reco_count':>12} {'reco_rate_%':>12} {'edge_avg':>14} "
            f"{'max_consec':>12} {'clusters':>10}"
        )
        for src in sorted(stats.keys()):
            v = stats[src]
            n = int(v["reco_count"])
            rate = 100.0 * n / total if total else 0.0
            print(
                f"{src:<22} {n:12d} {rate:12.4f} {v['edge_avg']:14.6f} "
                f"{int(v['max_consecutive_src']):12d} {int(v['cluster_count']):10d}"
            )
        print()

    if outcome_pnls:
        print("### [REC_OUTCOME] by src (hit = pnl>0)")
        print(
            f"{'src':<22} {'n':>8} {'hit_rate_%':>12} {'avg_pnl':>14} {'min_pnl':>12} {'max_pnl':>12}"
        )
        for src in sorted(outcome_pnls.keys()):
            pnls = outcome_pnls[src]
            k = len(pnls)
            hits = sum(1 for p in pnls if p > 0.0)
            hr = 100.0 * hits / k if k else 0.0
            avg = sum(pnls) / k if k else 0.0
            lo = min(pnls)
            hi = max(pnls)
            print(
                f"{src:<22} {k:8d} {hr:12.4f} {avg:14.6f} {lo:12.6f} {hi:12.6f}"
            )


if __name__ == "__main__":
    main()
