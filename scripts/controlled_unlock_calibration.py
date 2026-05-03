#!/usr/bin/env python3
"""
Deterministic, replay-safe extraction for RECO edge_min calibration (ChronoSentiment core:
no invented data, same log bytes → same tables).

Phase 1–2: merge frozen shards (mtime asc, stable path tie-break), parse [EDGE_COMPONENTS]
and [DIAG], emit percentiles and counterfactual pre_gate_edge pass rates for edge_min A/B.
"""
from __future__ import annotations

import argparse
import re
import statistics
from collections import defaultdict
from dataclasses import dataclass, field
from pathlib import Path
from typing import Iterable

# Aligns with core/examples/live_engine.rs `percentile`
def _percentile_nearest(values: list[float], p: float) -> float:
    if not values:
        return 0.0
    s = sorted(values)
    rank = ((max(0.0, min(100.0, p)) / 100.0) * (len(s) - 1))
    idx = int(round(rank))
    idx = max(0, min(idx, len(s) - 1))
    return s[idx]


EDGE_COMP_RE = re.compile(
    r"\[EDGE_COMPONENTS\]\s+sym=([^\s]+)\s+"
    r"raw_momentum=([-+]?\d*\.?\d+)\s+"
    r"norm_momentum=([-+]?\d*\.?\d+)\s+"
    r"momentum_weight=([-+]?\d*\.?\d+)\s+"
    r"momentum_contribution=([-+]?\d*\.?\d+)\s+"
    r"composite_contribution=([-+]?\d*\.?\d+)\s+"
    r"score_contribution=([-+]?\d*\.?\d+)\s+"
    r"pre_gate_edge=([-+]?\d*\.?\d+)\s+"
    r"post_gate_edge=([-+]?\d*\.?\d+)\s+"
    r"voters=(\d+)"
)
DIAG_RE = re.compile(
    r"\[DIAG\]\s+sym=([^\s]+)\s+"
    r"edge=([-+]?\d*\.?\d+)\s+conf=([-+]?\d*\.?\d+)\s+"
    r".*?FINAL=(\d+)\s+"
    r"feas=([-+]?\d*\.?\d+)\s+voters=(\d+)\s+"
    r".*?low_edge=(\d+)"
)
MOMENTUM_RE = re.compile(r"\[MOMENTUM_CHECK\].*?sym=([^\s]+)\s+.*?condition_met=(\d+)")
SIDE_FINAL_RE = re.compile(
    r"\[SIDE_DISTRIBUTION\].*?final_buy=(\d+)\s+final_sell=(\d+)"
)


@dataclass
class SymbolSamples:
    pre_gate: list[float] = field(default_factory=list)
    mom_c: list[float] = field(default_factory=list)
    diag_edge: list[float] = field(default_factory=list)
    diag_voters: list[int] = field(default_factory=list)
    low_edge: list[int] = field(default_factory=list)
    momentum_cond: list[int] = field(default_factory=list)


def discover_shard_files(root: Path) -> list[Path]:
    """BTC/ETH/SOL: include *_A.log and *_B.log when present; deterministic order."""
    patterns = (
        "live_BTC_USD_*.log",
        "live_ETH_USD_*.log",
        "live_SOL_USD_*.log",
    )
    paths: list[Path] = []
    for pat in patterns:
        paths.extend(sorted(root.glob(pat)))
    # mtime asc, then path string (frozen window reproducibility)
    paths.sort(key=lambda p: (p.stat().st_mtime, str(p)))
    return paths


def iter_merged_lines(paths: Iterable[Path]) -> Iterable[str]:
    for p in paths:
        with p.open("r", encoding="utf-8", errors="replace") as f:
            for line in f:
                yield line


def parse_streams(
    lines: Iterable[str],
) -> tuple[dict[str, SymbolSamples], SymbolSamples, int, int]:
    """Per-symbol samples + pooled aggregates."""
    by_sym: dict[str, SymbolSamples] = defaultdict(SymbolSamples)
    pooled = SymbolSamples()

    last_side_final_buy = 0
    last_side_final_sell = 0

    for line in lines:
        m = SIDE_FINAL_RE.search(line)
        if m:
            last_side_final_buy = int(m.group(1))
            last_side_final_sell = int(m.group(2))

        m = EDGE_COMP_RE.search(line)
        if m:
            sym = m.group(1).strip()
            mom = float(m.group(5))
            pre = float(m.group(8))
            by_sym[sym].pre_gate.append(pre)
            by_sym[sym].mom_c.append(mom)
            pooled.pre_gate.append(pre)
            pooled.mom_c.append(mom)
            continue

        m = DIAG_RE.search(line)
        if m:
            sym = m.group(1).strip()
            edge = float(m.group(2))
            voters = int(m.group(6))
            low_e = int(m.group(7))
            by_sym[sym].diag_edge.append(edge)
            by_sym[sym].diag_voters.append(voters)
            by_sym[sym].low_edge.append(low_e)
            pooled.diag_edge.append(edge)
            pooled.diag_voters.append(voters)
            pooled.low_edge.append(low_e)
            continue

        m = MOMENTUM_RE.search(line)
        if m:
            sym = m.group(1).strip()
            cond = int(m.group(2))
            by_sym[sym].momentum_cond.append(cond)
            pooled.momentum_cond.append(cond)

    return dict(by_sym), pooled, last_side_final_buy, last_side_final_sell


def nonzero_rate(vals: list[float]) -> float:
    if not vals:
        return 0.0
    return sum(1 for v in vals if v > 0.0) / len(vals)


def rate_gt(vals: list[float], threshold: float) -> float:
    if not vals:
        return 0.0
    return sum(1 for v in vals if v >= threshold) / len(vals)


def voters_nonzero_rate(voters: list[int]) -> float:
    if not voters:
        return 0.0
    return sum(1 for v in voters if v > 0) / len(voters)


def diag_edge_nonzero_rate(edges: list[float]) -> float:
    if not edges:
        return 0.0
    return sum(1 for e in edges if e > 1e-12) / len(edges)


def momentum_condition_rate(flags: list[int]) -> float:
    if not flags:
        return 0.0
    return sum(flags) / len(flags)


def clustered_triggers(flags: list[int]) -> tuple[int, int]:
    """Returns (max_consecutive_ones, count_of_clusters)."""
    max_run = 0
    cur = 0
    clusters = 0
    prev = 0
    for x in flags:
        if x:
            cur += 1
            if prev == 0:
                clusters += 1
            max_run = max(max_run, cur)
        else:
            cur = 0
        prev = x
    return max_run, clusters


def fmt_row(cells: list[str], widths: list[int]) -> str:
    parts = [c.ljust(w) for c, w in zip(cells, widths)]
    return " | ".join(parts)


def main() -> None:
    ap = argparse.ArgumentParser()
    ap.add_argument(
        "--root",
        type=Path,
        default=Path("analysis/live_multi"),
        help="Directory containing live_*_A/B.log shards",
    )
    ap.add_argument("--epsilon", type=float, default=5e-5)
    ap.add_argument("--edge-min-old", type=float, default=0.0012)
    args = ap.parse_args()

    paths = discover_shard_files(args.root)
    if not paths:
        raise SystemExit(f"No shard files under {args.root}")

    lines = list(iter_merged_lines(paths))
    by_sym, pooled, side_buy, side_sell = parse_streams(lines)

    edge_min_old = args.edge_min_old
    epsilon = args.epsilon

    # Pooled percentiles on pre_gate_edge
    pre_all = pooled.pre_gate
    p50 = _percentile_nearest(pre_all, 50)
    p75 = _percentile_nearest(pre_all, 75)
    p90 = _percentile_nearest(pre_all, 90)
    p95 = _percentile_nearest(pre_all, 95)

    edge_min_p90 = max(p90, epsilon)
    edge_min_p95 = max(p95, epsilon)

    low_vals = pooled.low_edge
    avg_low = statistics.mean(low_vals) if low_vals else 0.0
    p75_low = _percentile_nearest([float(x) for x in low_vals], 75) if low_vals else 0.0

    mom_all = pooled.mom_c
    m50 = _percentile_nearest(mom_all, 50)
    m75 = _percentile_nearest(mom_all, 75)
    m90 = _percentile_nearest(mom_all, 90)
    m95 = _percentile_nearest(mom_all, 95)

    print("### Controlled Unlock — decision pack (frozen window)")
    print()
    print("**Invariant:** shard files (mtime asc, then path):", len(paths), "files,", len(lines), "lines")
    for p in paths:
        print(f"  - {p}  mtime={p.stat().st_mtime_ns}")
    print()

    # Table: percentiles per symbol + pooled
    print("### Percentiles (pre_gate_edge / momentum_contribution)")
    w = [12, 10, 10, 10, 10, 12, 12, 12, 12, 12]
    hdr = ["symbol", "n_pre", "p50_pre", "p75_pre", "p90_pre", "p95_pre", "nz_rate_pre", "p50_mom", "p90_mom", "p95_mom"]
    print(fmt_row(hdr, w))
    print("-" * (sum(w) + 3 * len(w)))

    def one_row(label: str, s: SymbolSamples) -> None:
        pg = s.pre_gate
        mz = s.mom_c
        print(
            fmt_row(
                [
                    label,
                    str(len(pg)),
                    f"{_percentile_nearest(pg, 50):.6f}",
                    f"{_percentile_nearest(pg, 75):.6f}",
                    f"{_percentile_nearest(pg, 90):.6f}",
                    f"{_percentile_nearest(pg, 95):.6f}",
                    f"{nonzero_rate(pg):.4f}",
                    f"{_percentile_nearest(mz, 50):.6f}",
                    f"{_percentile_nearest(mz, 90):.6f}",
                    f"{_percentile_nearest(mz, 95):.6f}",
                ],
                w,
            )
        )

    syms_sorted = sorted(by_sym.keys())
    for sym in syms_sorted:
        one_row(sym, by_sym[sym])
    one_row("__pooled__", pooled)

    print()
    print("### Proposed thresholds")
    print(f"  edge_min_old                  = {edge_min_old:.6f}")
    print(f"  epsilon                       = {epsilon:.6f}")
    print(f"  p90(pre_gate_edge) pooled     = {p90:.6f}")
    print(f"  p95(pre_gate_edge) pooled     = {p95:.6f}")
    print(f"  edge_min_new (max p90, eps)   = {edge_min_p90:.6f}")
    print(f"  edge_min_new_conservative     = {edge_min_p95:.6f}")
    print()

    # Counterfactual: raw pre_gate pass vs thresholds (single-axis unlock proxy)
    pass_old = rate_gt(pre_all, edge_min_old)
    pass_p90 = rate_gt(pre_all, edge_min_p90)
    pass_p95 = rate_gt(pre_all, edge_min_p95)

    max_pre = max(pre_all) if pre_all else 0.0
    avg_pre = statistics.mean(pre_all) if pre_all else 0.0

    spikes_p90 = sum(1 for v in pre_all if v > 3 * edge_min_p90) if pre_all else 0
    spikes_p95 = sum(1 for v in pre_all if v > 3 * edge_min_p95) if pre_all else 0

    mx_run, clusters = clustered_triggers(pooled.momentum_cond)

    print("### Replay deltas (offline counterfactual on same samples)")
    print("  Note: `pre_gate_edge ≥ edge_min` is necessary but not sufficient for live reco;")
    print("  voters/feas/conf gates unchanged. Use RECO_EDGE_MIN for full engine A/B.")
    print()
    wd = [28, 14, 14, 14]
    print(fmt_row(["metric", "baseline", "new(p90)", "new(p95)"], wd))
    print("-" * 60)
    print(
        fmt_row(
            [
                "voters_nonzero_rate (DIAG)",
                f"{voters_nonzero_rate(pooled.diag_voters):.4f}",
                "(unchanged)",
                "(unchanged)",
            ],
            wd,
        )
    )
    print(
        fmt_row(
            [
                "diag edge_nonzero_rate",
                f"{diag_edge_nonzero_rate(pooled.diag_edge):.4f}",
                "(unchanged)",
                "(unchanged)",
            ],
            wd,
        )
    )
    print(
        fmt_row(
            [
                "pre_gate ≥ edge_min (proxy)",
                f"{pass_old:.4f}",
                f"{pass_p90:.4f}",
                f"{pass_p95:.4f}",
            ],
            wd,
        )
    )
    print(
        fmt_row(
            [
                "avg pre_gate_edge",
                f"{avg_pre:.6f}",
                f"{avg_pre:.6f}",
                f"{avg_pre:.6f}",
            ],
            wd,
        )
    )
    print(
        fmt_row(
            [
                "max pre_gate_edge",
                f"{max_pre:.6f}",
                f"{max_pre:.6f}",
                f"{max_pre:.6f}",
            ],
            wd,
        )
    )
    sell_rate = float(side_sell) / max(1, (side_buy + side_sell)) if (side_buy + side_sell) else 0.0
    print(
        fmt_row(
            [
                "final_sell_rate (SIDE last)",
                f"{sell_rate:.4f}",
                "n/a",
                "n/a",
            ],
            wd,
        )
    )

    print()
    print("### Guardrails (from logs; momentum unchanged by edge_min)")
    print(f"  condition_rate (MOMENTUM_CHECK)  = {momentum_condition_rate(pooled.momentum_cond):.4f}")
    print(f"  edge_spike_count (pre > 3×thr)     = baseline n/a | p90={spikes_p90} | p95={spikes_p95}")
    print(f"  clustered_events                 = max_run={mx_run} clusters={clusters}")
    print(f"  avg(low_edge) DIAG               = {avg_low:.2f}   p75(low_edge)={p75_low:.2f}")
    print()
    print("### Acceptance checklist (objective)")
    a2_p90 = 0.005 <= pass_p90 <= 0.05
    a2_p95 = 0.005 <= pass_p95 <= 0.05
    print(f"  A2 p90 in [0.5%, 5%]: {a2_p90}  (rate={pass_p90*100:.2f}%)")
    print(f"  A2 p95 in [0.5%, 5%]: {a2_p95}  (rate={pass_p95*100:.2f}%)")
    print(f"  A3 no explosion (<10%): p90={pass_p90*100:.2f}%  p95={pass_p95*100:.2f}%")


if __name__ == "__main__":
    main()
