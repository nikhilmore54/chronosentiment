#!/usr/bin/env python3
"""
Join [TRIGGER_METRICS] with [REC_OUTCOME] on rec_id; bucket realized PnL by latency.

Read-only analysis — no engine dependency. Aligns with deterministic replay logs
(.cursor/rules/chronosentiment-core.mdc): parse stderr/tee files, no randomness.

Usage:
  ./target/release/examples/live_engine < tape.jsonl 2>&1 | python3 scripts/trigger_latency_bucket_report.py
  python3 scripts/trigger_latency_bucket_report.py < combined.log

Two independent reports (do not mix dimensions in one bucket):
  A) PnL vs intent_age_updates (intent → trigger)
  B) PnL vs confirm_updates (candidate → confirm)

Buckets use p30 / p70 of the chosen dimension over *joined* rows only (defaults;
override with --p-low / --p-high).

Baseline (uplift / shadow): ``baseline_mode=no_filters`` — all *joined* triggered trades
(same rec_id join as elsewhere); no vol/mom rejection. CF1/CF2 are counterfactual filters
on top of that baseline only (analysis-only).
"""

from __future__ import annotations

import argparse
import json
import re
import sys
from dataclasses import dataclass
from typing import Any

TRIGGER = re.compile(
    r"\[TRIGGER_METRICS\]\s+sym=(?P<sym>\S+)\s+rec_id=(?P<rid>\d+)\s+"
    r"confirm_updates=(?P<confirm>\d+)\s+intent_age_updates=(?P<intent_age>\d+)\s+"
    r"intent_age_bars=(?P<intent_bars>\d+)"
)
TRIGGER_STATE = re.compile(
    r"\[TRIGGER_STATE\]\s+rec_id=(?P<rid>\d+)\s+confirm_updates=(?P<confirm>\d+)\s+"
    r"intent_age_updates=(?P<intent_age>\d+)\s+"
    r"trigger_momentum_3=(?P<mom>[-0-9.eE+]+)\s+trigger_vol_5=(?P<vol>[-0-9.eE+]+)"
)
REC_OUT = re.compile(
    r"\[REC_OUTCOME\]\s+rec_id=(?P<rid>\d+)\s+sym=\S+\s+score=[-0-9.eE+]+\s+"
    r"edge=[-0-9.eE+]+\s+feas=[-0-9.eE+]+\s+conf=[-0-9.eE+]+\s+"
    r"voters=\d+\s+S\d+\s+pnl=(?P<pnl>[-0-9.eE+]+)"
)
TRADE_PATH = re.compile(
    r"\[TRADE_PATH\]\s+rec_id=(?P<rid>\d+)\s+sym=\S+\s+mfe=[-0-9.eE+]+\s+"
    r"mae=[-0-9.eE+]+\s+pnl=(?P<pnl>[-0-9.eE+]+)\s+ret_at_exit=[-0-9.eE+]+"
)
SHADOW_DECISION = re.compile(
    r"\[SHADOW_DECISION\]\s+rec_id=(?P<rid>\d+)\s+would_block_vol=(?P<wv>[01])\s+"
    r"would_block_mom=(?P<wm>[01])\s+would_block_any=(?P<wa>[01])\s+reason=(?P<reason>\S+)\s+"
    r"vol_percentile=(?P<vp>[-0-9.]+)\s+mom_percentile=(?P<mp>[-0-9.]+)"
)

DEFAULT_HYPOTHESIS_ID_PATTERN = r"^[a-z0-9_]+$"


@dataclass(frozen=True)
class TriggerRow:
    sym: str
    confirm_updates: int
    intent_age_updates: int
    intent_age_bars: int


@dataclass(frozen=True)
class JoinedRow:
    rec_id: int
    confirm_updates: int
    intent_age_updates: int
    trigger_momentum_3: float
    trigger_vol_5: float
    pnl: float


@dataclass(frozen=True)
class ShadowRow:
    would_block_vol: bool
    would_block_mom: bool
    would_block_any: bool
    reason: str


def validate_hypothesis_id_pattern(
    hypothesis_id: str | None, hypothesis_id_pattern: str | None
) -> None:
    if not hypothesis_id:
        return
    pattern = hypothesis_id_pattern or DEFAULT_HYPOTHESIS_ID_PATTERN
    if not re.fullmatch(pattern, hypothesis_id):
        raise ValueError(
            f"hypothesis_id '{hypothesis_id}' does not match pattern '{pattern}'"
        )


def linear_percentile(sorted_vals: list[int], p: float) -> float:
    """p in [0, 100]. Linear interpolation between closest ranks."""
    n = len(sorted_vals)
    if n == 0:
        return 0.0
    if n == 1:
        return float(sorted_vals[0])
    idx = (n - 1) * (p / 100.0)
    lo = int(idx)
    hi = min(lo + 1, n - 1)
    w = idx - lo
    return sorted_vals[lo] * (1.0 - w) + sorted_vals[hi] * w


def linear_percentile_f64(sorted_vals: list[float], p: float) -> float:
    """p in [0, 100]. Linear interpolation between closest ranks."""
    n = len(sorted_vals)
    if n == 0:
        return 0.0
    if n == 1:
        return float(sorted_vals[0])
    idx = (n - 1) * (p / 100.0)
    lo = int(idx)
    hi = min(lo + 1, n - 1)
    w = idx - lo
    return sorted_vals[lo] * (1.0 - w) + sorted_vals[hi] * w


def median_f64(vals: list[float]) -> float:
    if not vals:
        return 0.0
    s = sorted(vals)
    n = len(s)
    if n % 2 == 1:
        return s[n // 2]
    return 0.5 * (s[n // 2 - 1] + s[n // 2])


def summarize_pnls(pnls: list[float]) -> dict[str, float]:
    n = len(pnls)
    if n == 0:
        return {
            "count": 0.0,
            "sum_pnl": 0.0,
            "avg_pnl": 0.0,
            "median_pnl": 0.0,
            "hit_rate": 0.0,
            "expectancy": 0.0,
        }
    avg = sum(pnls) / n
    return {
        "count": float(n),
        "sum_pnl": sum(pnls),
        "avg_pnl": avg,
        "median_pnl": median_f64(pnls),
        "hit_rate": sum(1 for p in pnls if p > 0.0) / n,
        "expectancy": avg,
    }


def drawdown_stats_from_rows(rows: list[JoinedRow]) -> tuple[float, float]:
    """Running cumulative pnl drawdown stats (absolute pnl units)."""
    if not rows:
        return 0.0, 0.0
    cum = 0.0
    peak = 0.0
    max_dd = 0.0
    dd_sum = 0.0
    for r in rows:
        cum += r.pnl
        if cum > peak:
            peak = cum
        dd = peak - cum
        if dd > max_dd:
            max_dd = dd
        dd_sum += dd
    avg_dd = dd_sum / len(rows)
    return max_dd, avg_dd


def bucket_label(v: int, x: float, y: float) -> str:
    if v <= x:
        return f"fast (<= {x:.1f})"
    if v <= y:
        return f"medium ({x:.1f} < v <= {y:.1f})"
    return f"slow (> {y:.1f})"


def report_dimension(
    name: str,
    pairs: list[tuple[int, float]],
    p_low: float,
    p_high: float,
) -> None:
    if not pairs:
        print(f"\n=== {name} ===\n(no joined rows)\n")
        return
    lats = sorted(v for v, _ in pairs)
    x = linear_percentile(lats, p_low)
    y = linear_percentile(lats, p_high)
    print(f"\n=== {name} ===")
    print(f"joined_n={len(pairs)}  p_low={p_low:g}% -> X={x:.4g}  p_high={p_high:g}% -> Y={y:.4g}")
    # bucket counts
    buckets: dict[str, list[float]] = {}
    for lat, pnl in pairs:
        lab = bucket_label(lat, x, y)
        buckets.setdefault(lab, []).append(pnl)
    order = [
        f"fast (<= {x:.1f})",
        f"medium ({x:.1f} < v <= {y:.1f})",
        f"slow (> {y:.1f})",
    ]
    print(f"{'bucket':<42} {'count':>8} {'avg_pnl':>12} {'hit_rate':>10}")
    for lab in order:
        pnls = buckets.get(lab, [])
        n = len(pnls)
        avg = sum(pnls) / n if n else 0.0
        hr = sum(1 for p in pnls if p > 0.0) / n if n else 0.0
        print(f"{lab:<42} {n:>8} {avg:>12.6f} {hr:>10.4f}")
    print()


def report_volatility_bands(joined_rows: list[JoinedRow]) -> None:
    print("\n=== C) trigger_vol_5 percentile bands ===")
    if not joined_rows:
        print("(no joined rows)\n")
        return
    vols = sorted(r.trigger_vol_5 for r in joined_rows)
    p30 = linear_percentile_f64(vols, 30.0)
    p50 = linear_percentile_f64(vols, 50.0)
    p70 = linear_percentile_f64(vols, 70.0)
    p85 = linear_percentile_f64(vols, 85.0)
    intent_med = median_f64([float(r.intent_age_updates) for r in joined_rows])
    print(
        f"vol_percentiles: p30={p30:.6f} p50={p50:.6f} p70={p70:.6f} p85={p85:.6f} | intent_age_median={intent_med:.3f}"
    )

    def band_name(vol: float) -> str:
        if vol <= p30:
            return "LOW (<= p30)"
        if vol <= p70:
            return "MID (p30..p70)"
        if vol <= p85:
            return "HIGH (p70..p85)"
        return "EXTREME (> p85)"

    groups: dict[str, list[JoinedRow]] = {
        "LOW (<= p30)": [],
        "MID (p30..p70)": [],
        "HIGH (p70..p85)": [],
        "EXTREME (> p85)": [],
    }
    for r in joined_rows:
        groups[band_name(r.trigger_vol_5)].append(r)

    print(
        f"{'band':<17} {'count':>8} {'avg_pnl':>12} {'hit_rate':>10} {'median_pnl':>12} {'avg_pnl_lat<=med':>17} {'avg_pnl_lat>med':>16}"
    )
    for band in ["LOW (<= p30)", "MID (p30..p70)", "HIGH (p70..p85)", "EXTREME (> p85)"]:
        rows = groups[band]
        pnls = [r.pnl for r in rows]
        n = len(pnls)
        avg = sum(pnls) / n if n else 0.0
        hr = sum(1 for p in pnls if p > 0.0) / n if n else 0.0
        med = median_f64(pnls)
        lo_lat = [r.pnl for r in rows if float(r.intent_age_updates) <= intent_med]
        hi_lat = [r.pnl for r in rows if float(r.intent_age_updates) > intent_med]
        avg_lo = sum(lo_lat) / len(lo_lat) if lo_lat else 0.0
        avg_hi = sum(hi_lat) / len(hi_lat) if hi_lat else 0.0
        print(
            f"{band:<17} {n:>8} {avg:>12.6f} {hr:>10.4f} {med:>12.6f} {avg_lo:>17.6f} {avg_hi:>16.6f}"
        )
    print()


def report_counterfactual_p85(joined_rows: list[JoinedRow]) -> dict[str, float]:
    print("\n=== Counterfactual (vol <= p85) ===")
    if not joined_rows:
        print("BASELINE: count=0 avg_pnl=0.000000 hit_rate=0.0000 median_pnl=0.000000 sum_pnl=0.000000")
        print("FILTERED: count=0 avg_pnl=0.000000 hit_rate=0.0000 median_pnl=0.000000 sum_pnl=0.000000")
        print("\nDELTA (FILTERED - BASELINE):")
        print("delta_count=0 retained_%=0.00")
        print("delta_avg_pnl=0.000000 delta_hit_rate=0.0000 delta_median_pnl=0.000000 delta_sum_pnl=0.000000")
        print("\nEXTREME (excluded):")
        print("count=0 avg_pnl=0.000000 sum_pnl=0.000000")
        return {
            "delta_avg_pnl": 0.0,
            "delta_hit_rate": 0.0,
            "retained_pct": 0.0,
            "baseline_count": 0.0,
        }
    vols = sorted(r.trigger_vol_5 for r in joined_rows)
    p85 = linear_percentile_f64(vols, 85.0)
    baseline_rows = joined_rows
    filtered_rows = [r for r in joined_rows if r.trigger_vol_5 <= p85]
    extreme_rows = [r for r in joined_rows if r.trigger_vol_5 > p85]

    b = summarize_pnls([r.pnl for r in baseline_rows])
    f = summarize_pnls([r.pnl for r in filtered_rows])
    e = summarize_pnls([r.pnl for r in extreme_rows])

    print(
        f"BASELINE: count={int(b['count'])} avg_pnl={b['avg_pnl']:.6f} hit_rate={b['hit_rate']:.4f} median_pnl={b['median_pnl']:.6f} sum_pnl={b['sum_pnl']:.6f}"
    )
    print(
        f"FILTERED: count={int(f['count'])} avg_pnl={f['avg_pnl']:.6f} hit_rate={f['hit_rate']:.4f} median_pnl={f['median_pnl']:.6f} sum_pnl={f['sum_pnl']:.6f}"
    )
    delta_count = int(f["count"] - b["count"])
    retained_pct = (100.0 * f["count"] / b["count"]) if b["count"] > 0.0 else 0.0
    print("\nDELTA (FILTERED - BASELINE):")
    print(f"delta_count={delta_count} retained_%={retained_pct:.2f}")
    print(
        f"delta_avg_pnl={f['avg_pnl'] - b['avg_pnl']:.6f} delta_hit_rate={f['hit_rate'] - b['hit_rate']:.4f} delta_median_pnl={f['median_pnl'] - b['median_pnl']:.6f} delta_sum_pnl={f['sum_pnl'] - b['sum_pnl']:.6f}"
    )
    print("\nEXTREME (excluded):")
    print(
        f"count={int(e['count'])} avg_pnl={e['avg_pnl']:.6f} sum_pnl={e['sum_pnl']:.6f}"
    )
    return {
        "delta_avg_pnl": f["avg_pnl"] - b["avg_pnl"],
        "delta_hit_rate": f["hit_rate"] - b["hit_rate"],
        "retained_pct": retained_pct,
        "baseline_count": b["count"],
    }


def print_set_metrics(prefix: str, rows: list[JoinedRow]) -> None:
    s = summarize_pnls([r.pnl for r in rows])
    print(
        f"{prefix}: count={int(s['count'])} avg_pnl={s['avg_pnl']:.6f} hit_rate={s['hit_rate']:.4f} median_pnl={s['median_pnl']:.6f} sum_pnl={s['sum_pnl']:.6f}"
    )


def print_delta(base: list[JoinedRow], filtered: list[JoinedRow]) -> None:
    b = summarize_pnls([r.pnl for r in base])
    f = summarize_pnls([r.pnl for r in filtered])
    retained_pct = (100.0 * f["count"] / b["count"]) if b["count"] > 0.0 else 0.0
    print(
        f"DELTA: delta_count={int(f['count']-b['count'])} retained_%={retained_pct:.2f} "
        f"delta_avg_pnl={f['avg_pnl']-b['avg_pnl']:.6f} delta_hit_rate={f['hit_rate']-b['hit_rate']:.4f} "
        f"delta_median_pnl={f['median_pnl']-b['median_pnl']:.6f} delta_sum_pnl={f['sum_pnl']-b['sum_pnl']:.6f}"
    )


def report_post_filter_analysis(joined_rows: list[JoinedRow]) -> None:
    print("\n=== Post-filter (vol <= p85) ===")
    if not joined_rows:
        print("(no joined rows)")
        return
    vols = sorted(r.trigger_vol_5 for r in joined_rows)
    p85_vol = linear_percentile_f64(vols, 85.0)
    base = [r for r in joined_rows if r.trigger_vol_5 <= p85_vol]
    print(f"scope_count={len(base)} (vol <= p85={p85_vol:.6f})")
    if not base:
        print("(no rows after vol filter)")
        return
    print_set_metrics("BASELINE", base)

    moms = sorted(r.trigger_momentum_3 for r in base)
    mom_p15 = linear_percentile_f64(moms, 15.0)
    mom_p20 = linear_percentile_f64(moms, 20.0)
    mom_p25 = linear_percentile_f64(moms, 25.0)
    mom_p30 = linear_percentile_f64(moms, 30.0)
    mom_p50 = linear_percentile_f64(moms, 50.0)
    mom_p70 = linear_percentile_f64(moms, 70.0)
    mom_p85 = linear_percentile_f64(moms, 85.0)
    lats = sorted(float(r.intent_age_updates) for r in base)
    lat_p30 = linear_percentile_f64(lats, 30.0)
    lat_p50 = linear_percentile_f64(lats, 50.0)
    lat_p70 = linear_percentile_f64(lats, 70.0)
    lat_p85 = linear_percentile_f64(lats, 85.0)
    print(
        f"momentum_pcts: p15={mom_p15:.6f} p20={mom_p20:.6f} p25={mom_p25:.6f} p30={mom_p30:.6f} p50={mom_p50:.6f} p70={mom_p70:.6f} p85={mom_p85:.6f}"
    )
    print(
        f"latency_pcts: p30={lat_p30:.3f} p50={lat_p50:.3f} p70={lat_p70:.3f} p85={lat_p85:.3f}"
    )

    print("\n-- Momentum tails --")
    low_tail = [r for r in base if r.trigger_momentum_3 <= mom_p15]
    kept = [r for r in base if r.trigger_momentum_3 > mom_p15]
    print("EXCLUDE mom <= p15:")
    print_set_metrics("  BASELINE", base)
    print_set_metrics("  FILTERED", kept)
    print_delta(base, kept)
    print_set_metrics("  EXCLUDED", low_tail)

    high_tail = [r for r in base if r.trigger_momentum_3 >= mom_p85]
    kept = [r for r in base if r.trigger_momentum_3 < mom_p85]
    print("\nEXCLUDE mom >= p85:")
    print_set_metrics("  BASELINE", base)
    print_set_metrics("  FILTERED", kept)
    print_delta(base, kept)
    print_set_metrics("  EXCLUDED", high_tail)

    print("\n-- Latency tails --")
    slow_tail = [r for r in base if float(r.intent_age_updates) >= lat_p70]
    kept = [r for r in base if float(r.intent_age_updates) < lat_p70]
    print("EXCLUDE lat >= p70:")
    print_set_metrics("  BASELINE", base)
    print_set_metrics("  FILTERED", kept)
    print_delta(base, kept)
    print_set_metrics("  EXCLUDED", slow_tail)

    fast_tail = [r for r in base if float(r.intent_age_updates) <= lat_p30]
    kept = [r for r in base if float(r.intent_age_updates) > lat_p30]
    print("\nEXCLUDE lat <= p30:")
    print_set_metrics("  BASELINE", base)
    print_set_metrics("  FILTERED", kept)
    print_delta(base, kept)
    print_set_metrics("  EXCLUDED", fast_tail)

    print("\n-- Optional interaction (if n >= 10) --")
    if len(base) >= 10:
        mid_mom = [r for r in base if r.trigger_momentum_3 > mom_p15 and r.trigger_momentum_3 < mom_p85]
        fast = [r for r in mid_mom if float(r.intent_age_updates) <= lat_p30]
        slow = [r for r in mid_mom if float(r.intent_age_updates) >= lat_p70]
        print_set_metrics("FAST in mom MID", fast)
        print_set_metrics("SLOW in mom MID", slow)
    else:
        print(f"skipped (n={len(base)} < 10)")


def format_set_line(name: str, rows: list[JoinedRow]) -> str:
    s = summarize_pnls([r.pnl for r in rows])
    return (
        f"{name}: count={int(s['count'])} avg_pnl={s['avg_pnl']:.6f} "
        f"hit_rate={s['hit_rate']:.4f} median_pnl={s['median_pnl']:.6f} sum_pnl={s['sum_pnl']:.6f}"
    )


def format_delta_line(
    label: str,
    ref_rows: list[JoinedRow],
    target_rows: list[JoinedRow],
    baseline_count: int,
) -> str:
    r = summarize_pnls([x.pnl for x in ref_rows])
    t = summarize_pnls([x.pnl for x in target_rows])
    retained_pct = (100.0 * t["count"] / baseline_count) if baseline_count > 0 else 0.0
    return (
        f"{label}: delta_count={int(t['count']-r['count'])} retained_%={retained_pct:.2f} "
        f"delta_avg_pnl={t['avg_pnl']-r['avg_pnl']:.6f} delta_hit_rate={t['hit_rate']-r['hit_rate']:.4f} "
        f"delta_median_pnl={t['median_pnl']-r['median_pnl']:.6f} delta_sum_pnl={t['sum_pnl']-r['sum_pnl']:.6f}"
    )


def report_combined_counterfactual(joined_rows: list[JoinedRow], mom_low_pct: float) -> dict[str, float]:
    print("\n=== Combined Counterfactual ===")
    if not joined_rows:
        print("BASELINE: count=0 avg_pnl=0.000000 hit_rate=0.0000 median_pnl=0.000000 sum_pnl=0.000000")
        print("CF1 (vol <= p85): count=0 avg_pnl=0.000000 hit_rate=0.0000 median_pnl=0.000000 sum_pnl=0.000000")
        print("DELTA vs BASELINE: delta_count=0 retained_%=0.00 delta_avg_pnl=0.000000 delta_hit_rate=0.0000 delta_median_pnl=0.000000 delta_sum_pnl=0.000000")
        print("EXCLUDED (vol>p85): excluded_count=0 excluded_avg_pnl=0.000000 excluded_sum_pnl=0.000000")
        print(f"CF2 (vol <= p85 & mom > p{mom_low_pct:g}): count=0 avg_pnl=0.000000 hit_rate=0.0000 median_pnl=0.000000 sum_pnl=0.000000")
        print("DELTA vs BASELINE: delta_count=0 retained_%=0.00 delta_avg_pnl=0.000000 delta_hit_rate=0.0000 delta_median_pnl=0.000000 delta_sum_pnl=0.000000")
        print("DELTA vs CF1: delta_count=0 retained_%=0.00 delta_avg_pnl=0.000000 delta_hit_rate=0.0000 delta_median_pnl=0.000000 delta_sum_pnl=0.000000")
        print(f"EXCLUDED (mom<=p{mom_low_pct:g}): excluded_count=0 excluded_avg_pnl=0.000000 excluded_sum_pnl=0.000000")
        print("CF3 (audit): kept mom>=p85 count=0 avg_pnl=0.000000")
        return {
            "usable": 0.0,
            "delta_avg_cf2_vs_cf1": 0.0,
            "delta_hit_cf2_vs_cf1": 0.0,
            "retained_cf2_vs_base": 0.0,
        }
    baseline = joined_rows
    base_count = len(baseline)
    vol_p85 = linear_percentile_f64(sorted(r.trigger_vol_5 for r in baseline), 85.0)
    cf1 = [r for r in baseline if r.trigger_vol_5 <= vol_p85]
    excluded_vol = [r for r in baseline if r.trigger_vol_5 > vol_p85]

    print(format_set_line("BASELINE", baseline))
    print(format_set_line("CF1 (vol <= p85)", cf1))
    print(format_delta_line("DELTA vs BASELINE", baseline, cf1, base_count))
    exv = summarize_pnls([r.pnl for r in excluded_vol])
    print(
        f"EXCLUDED (vol>p85): excluded_count={int(exv['count'])} excluded_avg_pnl={exv['avg_pnl']:.6f} excluded_sum_pnl={exv['sum_pnl']:.6f}"
    )

    if not cf1:
        print(f"CF2 (vol <= p85 & mom > p{mom_low_pct:g}): count=0 avg_pnl=0.000000 hit_rate=0.0000 median_pnl=0.000000 sum_pnl=0.000000")
        print("DELTA vs BASELINE: delta_count=0 retained_%=0.00 delta_avg_pnl=0.000000 delta_hit_rate=0.0000 delta_median_pnl=0.000000 delta_sum_pnl=0.000000")
        print("DELTA vs CF1: delta_count=0 retained_%=0.00 delta_avg_pnl=0.000000 delta_hit_rate=0.0000 delta_median_pnl=0.000000 delta_sum_pnl=0.000000")
        print(f"EXCLUDED (mom<=p{mom_low_pct:g}): excluded_count=0 excluded_avg_pnl=0.000000 excluded_sum_pnl=0.000000")
        print("CF3 (audit): kept mom>=p85 count=0 avg_pnl=0.000000")
        return {
            "usable": 0.0,
            "delta_avg_cf2_vs_cf1": 0.0,
            "delta_hit_cf2_vs_cf1": 0.0,
            "retained_cf2_vs_base": 0.0,
        }

    mom_p15 = linear_percentile_f64(sorted(r.trigger_momentum_3 for r in cf1), mom_low_pct)
    mom_p85 = linear_percentile_f64(sorted(r.trigger_momentum_3 for r in cf1), 85.0)
    cf2 = [r for r in cf1 if r.trigger_momentum_3 > mom_p15]
    excluded_mom_low = [r for r in cf1 if r.trigger_momentum_3 <= mom_p15]
    kept_mom_high = [r for r in cf2 if r.trigger_momentum_3 >= mom_p85]

    print(format_set_line(f"CF2 (vol <= p85 & mom > p{mom_low_pct:g})", cf2))
    print(format_delta_line("DELTA vs BASELINE", baseline, cf2, base_count))
    print(format_delta_line("DELTA vs CF1", cf1, cf2, base_count))
    exm = summarize_pnls([r.pnl for r in excluded_mom_low])
    print(
        f"EXCLUDED (mom<=p{mom_low_pct:g}): excluded_count={int(exm['count'])} excluded_avg_pnl={exm['avg_pnl']:.6f} excluded_sum_pnl={exm['sum_pnl']:.6f}"
    )
    kh = summarize_pnls([r.pnl for r in kept_mom_high])
    print(
        f"CF3 (audit): kept mom>=p85 count={int(kh['count'])} avg_pnl={kh['avg_pnl']:.6f}"
    )

    cf1s = summarize_pnls([r.pnl for r in cf1])
    cf2s = summarize_pnls([r.pnl for r in cf2])
    return {
        "usable": 1.0 if len(cf1) >= 2 else 0.0,
        "delta_avg_cf2_vs_cf1": cf2s["avg_pnl"] - cf1s["avg_pnl"],
        "delta_hit_cf2_vs_cf1": cf2s["hit_rate"] - cf1s["hit_rate"],
        "retained_cf2_vs_base": (100.0 * cf2s["count"] / base_count) if base_count > 0 else 0.0,
        "positive_slice": 1.0 if (cf2s["avg_pnl"] - cf1s["avg_pnl"] > 0 and cf2s["hit_rate"] - cf1s["hit_rate"] > 0) else 0.0,
    }


def portfolio_uplift_metrics(
    joined_rows: list[JoinedRow], mom_low_pct: float
) -> dict[str, Any]:
    """Shared CF1/CF2 vs baseline metrics for printing and auto-decision."""
    if not joined_rows:
        return {"ok": False}
    baseline = joined_rows
    p85_vol = linear_percentile_f64(sorted(r.trigger_vol_5 for r in baseline), 85.0)
    cf1 = [r for r in baseline if r.trigger_vol_5 <= p85_vol]
    if cf1:
        mom_cut = linear_percentile_f64(sorted(r.trigger_momentum_3 for r in cf1), mom_low_pct)
        cf2 = [r for r in cf1 if r.trigger_momentum_3 > mom_cut]
    else:
        cf2 = []

    def set_metrics(rows: list[JoinedRow]) -> dict[str, float]:
        s = summarize_pnls([r.pnl for r in rows])
        mdd, add = drawdown_stats_from_rows(rows)
        s["max_dd"] = mdd
        s["avg_dd"] = add
        return s

    b = set_metrics(baseline)
    c1 = set_metrics(cf1)
    c2 = set_metrics(cf2)
    bcount = b["count"] if b["count"] > 0 else 1.0
    return {
        "ok": True,
        "b": b,
        "c1": c1,
        "c2": c2,
        "bcount": bcount,
        "delta_sum_cf2_vs_base": c2["sum_pnl"] - b["sum_pnl"],
        "delta_avg_cf2_vs_base": c2["avg_pnl"] - b["avg_pnl"],
        "delta_hit_cf2_vs_base": c2["hit_rate"] - b["hit_rate"],
        "delta_maxdd_cf2_vs_base": c2["max_dd"] - b["max_dd"],
        "retained_cf2_vs_base": 100.0 * c2["count"] / bcount,
        "cf1_usable": len(cf1) >= 2,
        "positive_slice_cf2_vs_cf1": 1.0
        if (
            c2["avg_pnl"] - c1["avg_pnl"] > 0.0
            and c2["hit_rate"] - c1["hit_rate"] > 0.0
            and c2["max_dd"] - b["max_dd"] <= 1e-12
        )
        else 0.0,
    }


def report_portfolio_uplift(joined_rows: list[JoinedRow], mom_low_pct: float) -> dict[str, float]:
    print("\n=== Portfolio Uplift ===")
    print("baseline_mode=no_filters")
    m = portfolio_uplift_metrics(joined_rows, mom_low_pct)
    if not m["ok"]:
        print("BASELINE_HEALTH: trade_count=0 sum_pnl=0.000000 avg_pnl=0.000000 hit_rate=0.0000 max_dd=0.000000")
        print("BASELINE: count=0 sum_pnl=0.000000 avg_pnl=0.000000 median_pnl=0.000000 hit_rate=0.0000 max_dd=0.000000 avg_dd=0.000000")
        print("CF1 (vol filter): count=0 sum_pnl=0.000000 avg_pnl=0.000000 median_pnl=0.000000 hit_rate=0.0000 max_dd=0.000000 avg_dd=0.000000")
        print("DELTA vs BASELINE: delta_sum_pnl=0.000000 delta_avg_pnl=0.000000 delta_hit_rate=0.0000 delta_max_dd=0.000000 retained_%=0.00")
        print(f"CF2 (vol + momentum p{mom_low_pct:g}): count=0 sum_pnl=0.000000 avg_pnl=0.000000 median_pnl=0.000000 hit_rate=0.0000 max_dd=0.000000 avg_dd=0.000000")
        print("DELTA vs BASELINE: delta_sum_pnl=0.000000 delta_avg_pnl=0.000000 delta_hit_rate=0.0000 delta_max_dd=0.000000 retained_%=0.00")
        print("DELTA vs CF1: delta_sum_pnl=0.000000 delta_avg_pnl=0.000000 delta_hit_rate=0.0000")
        return {
            "usable": 0.0,
            "delta_sum_cf2_vs_base": 0.0,
            "delta_avg_cf2_vs_base": 0.0,
            "delta_hit_cf2_vs_base": 0.0,
            "delta_maxdd_cf2_vs_base": 0.0,
            "retained_cf2_vs_base": 0.0,
            "positive_slice": 0.0,
        }
    b, c1, c2 = m["b"], m["c1"], m["c2"]
    bcount = m["bcount"]

    print(
        f"BASELINE_HEALTH: trade_count={int(b['count'])} sum_pnl={b['sum_pnl']:.6f} avg_pnl={b['avg_pnl']:.6f} hit_rate={b['hit_rate']:.4f} max_dd={b['max_dd']:.6f}"
    )
    print(
        f"BASELINE: count={int(b['count'])} sum_pnl={b['sum_pnl']:.6f} avg_pnl={b['avg_pnl']:.6f} median_pnl={b['median_pnl']:.6f} hit_rate={b['hit_rate']:.4f} max_dd={b['max_dd']:.6f} avg_dd={b['avg_dd']:.6f}"
    )
    print(
        f"CF1 (vol filter): count={int(c1['count'])} sum_pnl={c1['sum_pnl']:.6f} avg_pnl={c1['avg_pnl']:.6f} median_pnl={c1['median_pnl']:.6f} hit_rate={c1['hit_rate']:.4f} max_dd={c1['max_dd']:.6f} avg_dd={c1['avg_dd']:.6f}"
    )
    print(
        f"DELTA vs BASELINE: delta_sum_pnl={c1['sum_pnl']-b['sum_pnl']:.6f} delta_avg_pnl={c1['avg_pnl']-b['avg_pnl']:.6f} delta_hit_rate={c1['hit_rate']-b['hit_rate']:.4f} delta_max_dd={c1['max_dd']-b['max_dd']:.6f} retained_%={(100.0*c1['count']/bcount):.2f}"
    )
    print(
        f"CF2 (vol + momentum p{mom_low_pct:g}): count={int(c2['count'])} sum_pnl={c2['sum_pnl']:.6f} avg_pnl={c2['avg_pnl']:.6f} median_pnl={c2['median_pnl']:.6f} hit_rate={c2['hit_rate']:.4f} max_dd={c2['max_dd']:.6f} avg_dd={c2['avg_dd']:.6f}"
    )
    print(
        f"DELTA vs BASELINE: delta_sum_pnl={c2['sum_pnl']-b['sum_pnl']:.6f} delta_avg_pnl={c2['avg_pnl']-b['avg_pnl']:.6f} delta_hit_rate={c2['hit_rate']-b['hit_rate']:.4f} delta_max_dd={c2['max_dd']-b['max_dd']:.6f} retained_%={(100.0*c2['count']/bcount):.2f}"
    )
    print(
        f"DELTA vs CF1: delta_sum_pnl={c2['sum_pnl']-c1['sum_pnl']:.6f} delta_avg_pnl={c2['avg_pnl']-c1['avg_pnl']:.6f} delta_hit_rate={c2['hit_rate']-c1['hit_rate']:.4f}"
    )
    if abs(b["avg_pnl"]) > 1e-12:
        ratio = c2["avg_pnl"] / b["avg_pnl"]
        print(f"IMPROVEMENT_CF2_vs_BASELINE: ratio_avg_pnl={ratio:.4f}")
    else:
        print("IMPROVEMENT_CF2_vs_BASELINE: ratio_avg_pnl=n/a (baseline avg near zero)")
    usable = 1.0 if m["cf1_usable"] else 0.0
    positive = m["positive_slice_cf2_vs_cf1"]
    return {
        "usable": usable,
        "delta_sum_cf2_vs_base": m["delta_sum_cf2_vs_base"],
        "delta_avg_cf2_vs_base": m["delta_avg_cf2_vs_base"],
        "delta_hit_cf2_vs_base": m["delta_hit_cf2_vs_base"],
        "delta_maxdd_cf2_vs_base": m["delta_maxdd_cf2_vs_base"],
        "retained_cf2_vs_base": m["retained_cf2_vs_base"],
        "positive_slice": positive,
    }


def evaluate_auto_decision(
    m: dict[str, Any],
    min_joined: int,
    min_retained_pct: float,
    max_retained_pct: float,
) -> dict[str, Any]:
    """Deterministic PROMOTE/HOLD/REJECT from CF2 vs baseline (single log / slice)."""
    if not m.get("ok"):
        return {
            "decision": "HOLD",
            "confidence": "LOW",
            "reasons": ["no joined rows"],
            "gates": {
                "delta_sum_pnl": 0.0,
                "delta_avg_pnl": 0.0,
                "delta_hit_rate": 0.0,
                "delta_max_dd": 0.0,
                "retained_pct": 0.0,
                "baseline_n": 0,
            },
        }
    b = m["b"]
    ds = m["delta_sum_cf2_vs_base"]
    da = m["delta_avg_cf2_vs_base"]
    dh = m["delta_hit_cf2_vs_base"]
    dd = m["delta_maxdd_cf2_vs_base"]
    ret = m["retained_cf2_vs_base"]
    n_base = int(b["count"])
    reasons: list[str] = []

    promote_core = (
        ds > 0.0
        and da > 0.0
        and dh > 0.0
        and dd <= 1e-12
        and min_retained_pct <= ret <= max_retained_pct
        and n_base >= min_joined
        and m["cf1_usable"]
    )
    if promote_core:
        reasons.append("delta_sum_pnl>0")
        reasons.append("delta_avg_pnl>0")
        reasons.append("delta_hit_rate>0")
        reasons.append("delta_max_dd<=0")
        reasons.append(f"retained_{ret:.1f}% in [{min_retained_pct},{max_retained_pct}]")
        reasons.append(f"baseline_joined_n>={min_joined}")
        reasons.append("cf1_usable (mom percentile defined)")
    reject = ds < 0.0 and da < 0.0
    if reject:
        reasons.append("CF2 worse than baseline on sum and avg")

    if promote_core:
        decision = "PROMOTE"
        conf = "HIGH" if n_base >= 5 and m["positive_slice_cf2_vs_cf1"] >= 0.5 else "MEDIUM"
    elif reject:
        decision = "REJECT"
        conf = "MEDIUM"
    else:
        decision = "HOLD"
        conf = "LOW"
        if not m["cf1_usable"]:
            reasons.append("cf1 count<2 (cannot define mom tail)")
        if n_base < min_joined:
            reasons.append(f"baseline_joined_n<{min_joined}")
        if not (min_retained_pct <= ret <= max_retained_pct):
            reasons.append("retention outside band")
        if not (ds > 0 and da > 0 and dh > 0 and dd <= 1e-12):
            reasons.append("CF2 vs baseline gates not all satisfied")

    return {
        "decision": decision,
        "confidence": conf,
        "reasons": reasons,
        "gates": {
            "delta_sum_pnl": ds,
            "delta_avg_pnl": da,
            "delta_hit_rate": dh,
            "delta_max_dd": dd,
            "retained_pct": ret,
            "baseline_n": n_base,
        },
    }


def print_auto_decision(
    m: dict[str, Any],
    min_joined: int,
    min_retained_pct: float,
    max_retained_pct: float,
) -> dict[str, Any]:
    decision = evaluate_auto_decision(
        m,
        min_joined=min_joined,
        min_retained_pct=min_retained_pct,
        max_retained_pct=max_retained_pct,
    )
    print("\n=== Auto Decision (CF2 vs baseline, read-only) ===")
    print(f"DECISION: {decision['decision']}")
    print(f"CONFIDENCE: {decision['confidence']}")
    reasons = decision["reasons"]
    print(f"REASON: {'; '.join(reasons) if reasons else '(none)'}")
    g = decision["gates"]
    print(
        f"GATES: delta_sum={g['delta_sum_pnl']:.6f} delta_avg={g['delta_avg_pnl']:.6f} "
        f"delta_hit={g['delta_hit_rate']:.4f} delta_max_dd={g['delta_max_dd']:.6f} "
        f"retained_%={g['retained_pct']:.2f} baseline_n={g['baseline_n']}"
    )
    return decision


def build_json_report(
    joined_rows: list[JoinedRow],
    mom_low_pct: float,
    min_joined: int,
    min_retained_pct: float,
    max_retained_pct: float,
    hypothesis_id: str | None,
) -> dict[str, Any]:
    m = portfolio_uplift_metrics(joined_rows, mom_low_pct)
    decision = evaluate_auto_decision(
        m,
        min_joined=min_joined,
        min_retained_pct=min_retained_pct,
        max_retained_pct=max_retained_pct,
    )
    if not m.get("ok"):
        return {
            "hypothesis_id": hypothesis_id,
            "decision": decision["decision"],
            "confidence": decision["confidence"],
            "reasons": decision["reasons"],
            "metrics": {
                "baseline": {
                    "count": 0,
                    "sum_pnl": 0.0,
                    "avg_pnl": 0.0,
                    "hit_rate": 0.0,
                    "max_drawdown": 0.0,
                },
                "cf1": {
                    "count": 0,
                    "retained_pct": 0.0,
                },
                "cf2": {
                    "count": 0,
                    "retained_pct": 0.0,
                },
                "cf2_vs_baseline": {
                    "delta_sum_pnl": 0.0,
                    "delta_avg_pnl": 0.0,
                    "delta_hit_rate": 0.0,
                    "delta_max_dd": 0.0,
                },
            },
            "gates": decision["gates"],
        }
    b, c1, c2 = m["b"], m["c1"], m["c2"]
    return {
        "hypothesis_id": hypothesis_id,
        "decision": decision["decision"],
        "confidence": decision["confidence"],
        "reasons": decision["reasons"],
        "metrics": {
            "baseline": {
                "count": int(b["count"]),
                "sum_pnl": b["sum_pnl"],
                "avg_pnl": b["avg_pnl"],
                "hit_rate": b["hit_rate"],
                "max_drawdown": b["max_dd"],
            },
            "cf1": {
                "count": int(c1["count"]),
                "retained_pct": (100.0 * c1["count"] / m["bcount"]) if m["bcount"] > 0 else 0.0,
            },
            "cf2": {
                "count": int(c2["count"]),
                "retained_pct": m["retained_cf2_vs_base"],
            },
            "cf2_vs_baseline": {
                "delta_sum_pnl": m["delta_sum_cf2_vs_base"],
                "delta_avg_pnl": m["delta_avg_cf2_vs_base"],
                "delta_hit_rate": m["delta_hit_cf2_vs_base"],
                "delta_max_dd": m["delta_maxdd_cf2_vs_base"],
            },
        },
        "gates": decision["gates"],
    }


def report_shadow_uplift(
    joined_rows: list[JoinedRow], shadow_map: dict[int, ShadowRow], mom_low_pct: float
) -> None:
    print("\n=== Shadow Uplift ===")
    print("baseline_mode=no_filters")
    if not joined_rows:
        print("(no joined rows)")
        return

    # Recompute "would block" using the validated per-slice method so shadow uplift
    # remains directly comparable to offline CF results.
    p85 = linear_percentile_f64(sorted(r.trigger_vol_5 for r in joined_rows), 85.0)
    cf1 = [r for r in joined_rows if r.trigger_vol_5 <= p85]
    mom_p15 = linear_percentile_f64(sorted(r.trigger_momentum_3 for r in cf1), mom_low_pct) if cf1 else 0.0
    calc_shadow_map: dict[int, ShadowRow] = {}
    for r in joined_rows:
        wbv = r.trigger_vol_5 > p85
        wbm = r.trigger_momentum_3 <= mom_p15
        wba = wbv or wbm
        reason = "both" if (wbv and wbm) else ("extreme_volatility" if wbv else ("extreme_negative_momentum" if wbm else "none"))
        calc_shadow_map[r.rec_id] = ShadowRow(wbv, wbm, wba, reason)

    def metrics(rows: list[JoinedRow]) -> dict[str, float]:
        s = summarize_pnls([x.pnl for x in rows])
        mdd, _ = drawdown_stats_from_rows(rows)
        s["max_dd"] = mdd
        return s

    baseline = joined_rows
    would_block = [r for r in joined_rows if calc_shadow_map.get(r.rec_id, ShadowRow(False, False, False, "none")).would_block_any]
    would_keep = [r for r in joined_rows if not calc_shadow_map.get(r.rec_id, ShadowRow(False, False, False, "none")).would_block_any]
    b = metrics(baseline)
    k = metrics(would_keep)
    retained = (100.0 * k["count"] / b["count"]) if b["count"] > 0 else 0.0
    print(
        f"BASELINE_HEALTH: trade_count={int(b['count'])} sum_pnl={b['sum_pnl']:.6f} avg_pnl={b['avg_pnl']:.6f} hit_rate={b['hit_rate']:.4f} max_dd={b['max_dd']:.6f}"
    )
    print(
        f"BASELINE: count={int(b['count'])} sum_pnl={b['sum_pnl']:.6f} avg_pnl={b['avg_pnl']:.6f} median_pnl={b['median_pnl']:.6f} hit_rate={b['hit_rate']:.4f} max_dd={b['max_dd']:.6f}"
    )
    print(
        f"WOULD_KEEP: count={int(k['count'])} sum_pnl={k['sum_pnl']:.6f} avg_pnl={k['avg_pnl']:.6f} median_pnl={k['median_pnl']:.6f} hit_rate={k['hit_rate']:.4f} max_dd={k['max_dd']:.6f}"
    )
    print(
        f"DELTA vs BASELINE: retained_%={retained:.2f} delta_sum_pnl={k['sum_pnl']-b['sum_pnl']:.6f} delta_avg_pnl={k['avg_pnl']-b['avg_pnl']:.6f} delta_hit_rate={k['hit_rate']-b['hit_rate']:.4f} delta_max_dd={k['max_dd']-b['max_dd']:.6f}"
    )

    vol_only = [r for r in would_block if calc_shadow_map.get(r.rec_id, ShadowRow(False, False, False, "none")).would_block_vol and not calc_shadow_map.get(r.rec_id, ShadowRow(False, False, False, "none")).would_block_mom]
    mom_only = [r for r in would_block if (not calc_shadow_map.get(r.rec_id, ShadowRow(False, False, False, "none")).would_block_vol) and calc_shadow_map.get(r.rec_id, ShadowRow(False, False, False, "none")).would_block_mom]
    both = [r for r in would_block if calc_shadow_map.get(r.rec_id, ShadowRow(False, False, False, "none")).would_block_vol and calc_shadow_map.get(r.rec_id, ShadowRow(False, False, False, "none")).would_block_mom]
    for label, rows in [("VOL_ONLY", vol_only), ("MOM_ONLY", mom_only), ("BOTH", both)]:
        s = summarize_pnls([x.pnl for x in rows])
        print(f"{label}: count={int(s['count'])} avg_pnl={s['avg_pnl']:.6f} sum_pnl={s['sum_pnl']:.6f}")


def main() -> int:
    ap = argparse.ArgumentParser(description=__doc__)
    ap.add_argument(
        "--p-low",
        type=float,
        default=30.0,
        help="lower percentile boundary (fast vs medium), default 30",
    )
    ap.add_argument(
        "--p-high",
        type=float,
        default=70.0,
        help="upper percentile boundary (medium vs slow), default 70",
    )
    ap.add_argument(
        "--dump-joined",
        action="store_true",
        help="print raw joined rows: rec_id, confirm_updates, intent_age_updates, trigger_momentum_3, trigger_vol_5, pnl",
    )
    ap.add_argument(
        "--post-filter-analysis",
        action="store_true",
        help="run momentum/latency tail counterfactuals inside vol<=p85 subset",
    )
    ap.add_argument(
        "--mom-low-pct",
        type=float,
        default=15.0,
        help="low momentum tail percentile used by CF2 exclusion (default 15)",
    )
    ap.add_argument(
        "--portfolio-uplift",
        action="store_true",
        help="print portfolio-style uplift block (sum/max_dd) for baseline/CF1/CF2",
    )
    ap.add_argument(
        "--auto-decision",
        action="store_true",
        help="print deterministic PROMOTE/HOLD/REJECT from CF2 vs baseline (read-only; does not imply --portfolio-uplift)",
    )
    ap.add_argument(
        "--decision-min-joined",
        type=int,
        default=3,
        help="minimum baseline joined trades for PROMOTE (default 3)",
    )
    ap.add_argument(
        "--decision-min-retained-pct",
        type=float,
        default=50.0,
        help="minimum retained %% for PROMOTE (default 50)",
    )
    ap.add_argument(
        "--decision-max-retained-pct",
        type=float,
        default=95.0,
        help="maximum retained %% for PROMOTE (default 95)",
    )
    ap.add_argument(
        "--shadow-uplift",
        action="store_true",
        help="print shadow decision uplift (would-block vs baseline) and attribution buckets",
    )
    ap.add_argument(
        "--json",
        action="store_true",
        help="emit only JSON report for auto-decision/batch orchestration",
    )
    ap.add_argument(
        "--hypothesis-id",
        default=None,
        help="optional hypothesis identifier for JSON tracking/registry",
    )
    ap.add_argument(
        "--require-hypothesis-id",
        action="store_true",
        help="hard-fail if hypothesis_id is missing (contract enforcement)",
    )
    ap.add_argument(
        "--hypothesis-id-pattern",
        default=None,
        help="optional regex that hypothesis_id must fully match",
    )
    args = ap.parse_args()
    text = sys.stdin.read()

    triggers: dict[int, TriggerRow] = {}
    for m in TRIGGER.finditer(text):
        rid = int(m.group("rid"))
        triggers[rid] = TriggerRow(
            sym=m.group("sym"),
            confirm_updates=int(m.group("confirm")),
            intent_age_updates=int(m.group("intent_age")),
            intent_age_bars=int(m.group("intent_bars")),
        )
    trigger_state_rows: dict[int, tuple[int, int, float, float]] = {}
    for m in TRIGGER_STATE.finditer(text):
        rid = int(m.group("rid"))
        trigger_state_rows[rid] = (
            int(m.group("confirm")),
            int(m.group("intent_age")),
            float(m.group("mom")),
            float(m.group("vol")),
        )
    shadow_rows: dict[int, ShadowRow] = {}
    for m in SHADOW_DECISION.finditer(text):
        rid = int(m.group("rid"))
        shadow_rows[rid] = ShadowRow(
            would_block_vol=(m.group("wv") == "1"),
            would_block_mom=(m.group("wm") == "1"),
            would_block_any=(m.group("wa") == "1"),
            reason=m.group("reason"),
        )

    outcomes: dict[int, float] = {}
    for m in REC_OUT.finditer(text):
        outcomes[int(m.group("rid"))] = float(m.group("pnl"))
    # Fallback: some runs close trades at finalize and may emit [TRADE_PATH]
    # without a corresponding [REC_OUTCOME]. Keep REC_OUTCOME as priority.
    for m in TRADE_PATH.finditer(text):
        rid = int(m.group("rid"))
        if rid not in outcomes:
            outcomes[rid] = float(m.group("pnl"))

    joined_a: list[tuple[int, float]] = []
    joined_b: list[tuple[int, float]] = []
    joined_rows: list[JoinedRow] = []
    trigger_join_source = trigger_state_rows if trigger_state_rows else {
        rid: (t.confirm_updates, t.intent_age_updates, 0.0, 0.0)
        for rid, t in triggers.items()
    }
    for rid, t in trigger_join_source.items():
        if rid not in outcomes:
            continue
        pnl = outcomes[rid]
        confirm_updates, intent_age_updates, trigger_momentum_3, trigger_vol_5 = t
        joined_a.append((intent_age_updates, pnl))
        joined_b.append((confirm_updates, pnl))
        joined_rows.append(
            JoinedRow(
                rec_id=rid,
                confirm_updates=confirm_updates,
                intent_age_updates=intent_age_updates,
                trigger_momentum_3=trigger_momentum_3,
                trigger_vol_5=trigger_vol_5,
                pnl=pnl,
            )
        )

    if args.json:
        if args.require_hypothesis_id and not args.hypothesis_id:
            raise ValueError("--require-hypothesis-id set but --hypothesis-id is missing/empty")
        validate_hypothesis_id_pattern(args.hypothesis_id, args.hypothesis_id_pattern)
        payload = build_json_report(
            joined_rows,
            args.mom_low_pct,
            min_joined=args.decision_min_joined,
            min_retained_pct=args.decision_min_retained_pct,
            max_retained_pct=args.decision_max_retained_pct,
            hypothesis_id=args.hypothesis_id,
        )
        print(json.dumps(payload, indent=2))
        return 0

    print("trigger_latency_bucket_report (read-only)")
    trigger_count = len(trigger_join_source)
    print(f"triggers={trigger_count} outcomes={len(outcomes)} joined={len(joined_a)}")
    trig_only = trigger_count - len(joined_a)
    oc_only = sum(1 for rid in outcomes if rid not in trigger_join_source)
    print(f"trigger_only={trig_only} outcome_only={oc_only}")
    if args.dump_joined:
        print(
            "\njoined_rows (rec_id, confirm_updates, intent_age_updates, trigger_momentum_3, trigger_vol_5, pnl)"
        )
        if not joined_rows:
            print("(none)")
        else:
            for row in sorted(joined_rows, key=lambda r: r.rec_id):
                print(
                    f"{row.rec_id},{row.confirm_updates},{row.intent_age_updates},{row.trigger_momentum_3:.6f},{row.trigger_vol_5:.6f},{row.pnl:.6f}"
                )

    report_dimension(
        "A) intent_age_updates (intent -> trigger)",
        joined_a,
        args.p_low,
        args.p_high,
    )
    report_dimension(
        "B) confirm_updates (candidate -> confirm)",
        joined_b,
        args.p_low,
        args.p_high,
    )
    report_volatility_bands(joined_rows)
    report_counterfactual_p85(joined_rows)
    if args.post_filter_analysis:
        report_post_filter_analysis(joined_rows)
    report_combined_counterfactual(joined_rows, args.mom_low_pct)
    if args.portfolio_uplift:
        report_portfolio_uplift(joined_rows, args.mom_low_pct)
    if args.auto_decision:
        print_auto_decision(
            portfolio_uplift_metrics(joined_rows, args.mom_low_pct),
            min_joined=args.decision_min_joined,
            min_retained_pct=args.decision_min_retained_pct,
            max_retained_pct=args.decision_max_retained_pct,
        )
    if args.shadow_uplift:
        report_shadow_uplift(joined_rows, shadow_rows, args.mom_low_pct)
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
