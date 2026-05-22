#!/usr/bin/env python3
"""Recommendation readiness certification report.

Deterministic gatekeeper for ChronoSentiment paper logs:
- hard PASS/WARN/FAIL gates
- explicit FINAL READY / NOT READY decision
"""

from __future__ import annotations

import argparse
import csv
import math
import re
from dataclasses import dataclass
from pathlib import Path
from statistics import mean, pstdev
from typing import Iterable


TRADE_RE = re.compile(
    r"\[TRADE_PATH\] rec_id=(?P<rec_id>\d+) sym=(?P<sym>[^ ]+) "
    r"mfe=(?P<mfe>[-\d.]+) mae=(?P<mae>[-\d.]+) pnl=(?P<pnl>[-\d.]+) "
    r"ret_at_exit=(?P<ret_at_exit>[-\d.]+) (?:edge_bps=(?P<edge_bps>[-\d.]+) )?"
    r"(?:rank=(?P<rank>[-\d.]+) )?"
    r"(?:rec_score=(?P<rec_score>[-\d.]+) )?"
    r"(?:rec_feas=(?P<rec_feas>[-\d.]+) )?"
    r"(?:rec_conf=(?P<rec_conf>[-\d.]+) )?"
    r"(?:rec_voters=(?P<rec_voters>\d+) )?"
    r"(?:intent_age=(?P<intent_age>\d+) )?"
    r"(?:momentum_3=(?P<momentum_3>[-\d.]+) )?"
    r"(?:vol_5=(?P<vol_5>[-\d.]+) )?"
    r"(?:score_std_5=(?P<score_std_5>[-\d.]+) )?"
    r"(?:vol_bps=(?P<vol_bps>[-\d.]+) )?"
    r"(?:partial_realized_pnl=(?P<partial_realized_pnl>[-\d.]+) )?"
    r"(?:remainder_exit_type=(?P<remainder_exit_type>[^ ]+) )?"
    r"(?:trail_active=(?P<trail_active>\d+) )?"
    r"(?:trail_stop_at_exit=(?P<trail_stop_at_exit>[-\d.]+) )?"
    r"dur=(?P<dur>\d+) armed=(?P<armed>\d+) "
    r"state=(?P<state>[^ ]+) exit_type=(?P<exit_type>\S+)"
)


@dataclass
class Trade:
    rec_id: int
    sym: str
    mfe: float
    mae: float
    pnl: float
    ret_at_exit: float
    edge_bps: float
    rank: float
    rec_score: float
    rec_feas: float
    rec_conf: float
    rec_voters: int
    momentum_3: float
    vol_5: float
    score_std_5: float
    vol_bps: float
    partial_realized_pnl: float
    remainder_exit_type: str
    trail_active: int
    trail_stop_at_exit: float
    dur: int
    armed: int
    state: str
    exit_type: str
    intent_age: int = 0


def read_trades(path: Path) -> list[Trade]:
    trades: list[Trade] = []
    for line in path.read_text(errors="replace").splitlines():
        m = TRADE_RE.search(line)
        if not m:
            continue
        trades.append(
            Trade(
                rec_id=int(m.group("rec_id")),
                sym=m.group("sym"),
                mfe=float(m.group("mfe")),
                mae=float(m.group("mae")),
                pnl=float(m.group("pnl")),
                ret_at_exit=float(m.group("ret_at_exit")),
                edge_bps=float(m.group("edge_bps") or 0.0),
                rank=float(m.group("rank") or 0.0),
                rec_score=float(m.group("rec_score") or 0.0),
                rec_feas=float(m.group("rec_feas") or 0.0),
                rec_conf=float(m.group("rec_conf") or 0.0),
                rec_voters=int(m.group("rec_voters") or 0),
                momentum_3=float(m.group("momentum_3") or 0.0),
                vol_5=float(m.group("vol_5") or 0.0),
                score_std_5=float(m.group("score_std_5") or 0.0),
                vol_bps=float(m.group("vol_bps") or 0.0),
                partial_realized_pnl=float(m.group("partial_realized_pnl") or 0.0),
                remainder_exit_type=(m.group("remainder_exit_type") or ""),
                trail_active=int(m.group("trail_active") or 0),
                trail_stop_at_exit=float(m.group("trail_stop_at_exit") or 0.0),
                dur=int(m.group("dur")),
                armed=int(m.group("armed")),
                state=m.group("state"),
                exit_type=m.group("exit_type"),
                intent_age=int(m.group("intent_age") or 0),
            )
        )
    return trades


def expectancy(ts: Iterable[Trade]) -> float:
    vals = list(ts)
    if not vals:
        return 0.0
    return sum(t.pnl for t in vals) / len(vals)


def chunk5(seq: list[Trade]) -> list[list[Trade]]:
    if not seq:
        return [[], [], [], [], []]
    step = math.ceil(len(seq) / 5)
    out = [seq[i : i + step] for i in range(0, len(seq), step)]
    while len(out) < 5:
        out.append([])
    return out[:5]


def gate_label(pass_cond: bool, warn_cond: bool) -> str:
    if pass_cond:
        return "PASS"
    if warn_cond:
        return "WARN"
    return "FAIL"


def fmt(x: float) -> str:
    return f"{x:.6f}"


def quantile_buckets(trades: list[Trade], bucket_count: int, proxy: str) -> list[list[Trade]]:
    if not trades:
        return [[] for _ in range(bucket_count)]
    ordered = sorted(trades, key=lambda t: trade_feature(t, proxy))
    step = math.ceil(len(ordered) / bucket_count)
    out = [ordered[i : i + step] for i in range(0, len(ordered), step)]
    while len(out) < bucket_count:
        out.append([])
    return out[:bucket_count]


def monotonic_score(vals: list[float]) -> float:
    if len(vals) < 2:
        return 0.0
    idx = list(range(len(vals)))
    mx = mean(idx)
    my = mean(vals)
    num = sum((i - mx) * (v - my) for i, v in zip(idx, vals))
    denx = math.sqrt(sum((i - mx) ** 2 for i in idx))
    deny = math.sqrt(sum((v - my) ** 2 for v in vals))
    if denx <= 1e-12 or deny <= 1e-12:
        return 0.0
    return num / (denx * deny)


def select_top_percent(trades: list[Trade], proxy: str, pct: int) -> list[Trade]:
    if not trades:
        return []
    if pct >= 100:
        return list(trades)
    k = max(1, math.ceil((pct / 100.0) * len(trades)))
    return sorted(trades, key=lambda t: trade_feature(t, proxy), reverse=True)[:k]


def trade_feature(t: Trade, proxy: str) -> float:
    if proxy == "edge_bps_v2":
        # v2 candidate: blend expected edge with execution-aware rank.
        return t.edge_bps * t.rank
    if proxy == "good_trade_classifier":
        # Deterministic entry-time classifier proxy (no post-trade leakage).
        # Blend confidence/feasibility/rank and penalize friction.
        edge_norm = max(-2.0, min(2.0, t.edge_bps / 50.0))
        voters_norm = min(1.0, t.rec_voters / 5.0)
        vol_pen = min(1.0, max(0.0, t.vol_bps / 100.0))
        return (
            0.9 * t.rank
            + 0.7 * t.rec_feas
            + 0.6 * t.rec_conf
            + 0.4 * voters_norm
            + 0.3 * edge_norm
            - 0.5 * vol_pen
        )
    if proxy == "rec_voters_vol":
        # Minimal orthogonal blend: consensus strength minus volatility friction.
        voters_norm = min(1.0, t.rec_voters / 5.0)
        vol_pen = min(1.0, max(0.0, t.vol_bps / 100.0))
        return voters_norm - 0.4 * vol_pen
    return getattr(t, proxy)


def evaluate(path: Path, proxy: str, top_pct: int, prefilter_bottom_pct: int, rank_tol: float) -> dict[str, object]:
    all_trades = read_trades(path)
    # Phase-2 stabilization: remove weakest half (or configured cutoff) before top-K selection.
    prefilter_keep = max(0, min(100, 100 - prefilter_bottom_pct))
    prefiltered = select_top_percent(all_trades, proxy, prefilter_keep) if prefilter_keep > 0 else []
    trades = select_top_percent(prefiltered, proxy, top_pct)
    n = len(trades)
    total_pnl = sum(t.pnl for t in trades)
    mean_exp = expectancy(trades)

    # Gate 1: Integrity
    violations = sum(1 for t in trades if t.pnl > t.mfe + 1e-9)
    g_integrity = "PASS" if violations == 0 else "FAIL"

    # Gate 2: Slice stability
    sl = chunk5(trades)
    slice_exps = [expectancy(s) for s in sl]
    pos = sum(1 for e in slice_exps if e > 0.0)
    g_slice = gate_label(pos >= 4, pos == 3)

    # Gate 3: Contribution concentration
    if n == 0 or total_pnl <= 1e-12:
        top10_share = float("inf")
        top10_positive_share = float("inf")
        g_contrib = "FAIL"
    else:
        k = max(1, math.ceil(0.10 * n))
        top = sorted((t.pnl for t in trades), reverse=True)[:k]
        top10_share = sum(top) / total_pnl
        pos_sum = sum(t.pnl for t in trades if t.pnl > 0.0)
        top10_positive_share = (sum(top) / pos_sum) if pos_sum > 1e-12 else float("inf")
        g_contrib = gate_label(top10_share < 0.50, 0.50 <= top10_share <= 0.75)

    # Gate 4: Ranking monotonicity (3 buckets + tolerance to reduce small-N brittleness).
    b = quantile_buckets(trades, 3, proxy)
    b_exps = [expectancy(x) for x in b]
    strict_inc = all((b_exps[i + 1] - b_exps[i]) > rank_tol for i in range(2))
    non_dec = all((b_exps[i + 1] - b_exps[i]) >= -rank_tol for i in range(2))
    score = monotonic_score(b_exps)
    g_rank = gate_label(strict_inc, non_dec and b_exps[-1] > b_exps[0] and score > 0.5)

    # Gate 5: Filter uplift
    if n == 0:
        filtered: list[Trade] = []
    else:
        k20 = max(1, math.ceil(0.20 * n))
        filtered = sorted(trades, key=lambda t: trade_feature(t, proxy), reverse=True)[:k20]
    base_exp = expectancy(trades)
    filt_exp = expectancy(filtered)
    g_filter = "PASS" if (filt_exp > base_exp and len(filtered) < len(trades)) else "FAIL"

    # Gate 6: Expectancy stability
    sig = pstdev(slice_exps) if slice_exps else 0.0
    g_stab = "PASS" if sig < abs(mean_exp) else "FAIL"

    gates = {
        "Integrity": g_integrity,
        "Slice Stability": g_slice,
        "Contribution": g_contrib,
        "Ranking": g_rank,
        "Filter Uplift": g_filter,
        "Stability": g_stab,
    }
    final_ready = all(v == "PASS" for v in gates.values())

    return {
        "path": str(path),
        "proxy": proxy,
        "top_pct": top_pct,
        "prefilter_bottom_pct": prefilter_bottom_pct,
        "prefilter_kept": len(prefiltered),
        "total_trades_in_log": len(all_trades),
        "trades": n,
        "total_pnl": total_pnl,
        "mean_exp": mean_exp,
        "violations": violations,
        "slice_exps": slice_exps,
        "slice_pos": pos,
        "slice_min": min(slice_exps) if slice_exps else 0.0,
        "slice_max": max(slice_exps) if slice_exps else 0.0,
        "slice_std": sig,
        "top10_share": top10_share,
        "top10_positive_share": top10_positive_share,
        "bucket_exps": b_exps,
        "rank_score": score,
        "base_exp": base_exp,
        "filt_exp": filt_exp,
        "uplift": filt_exp - base_exp,
        "filt_count": len(filtered),
        "gates": gates,
        "final_ready": final_ready,
    }


def percentile_threshold(vals: list[float], pct: int) -> float:
    if not vals:
        return float("inf")
    s = sorted(vals)
    keep = max(1, math.ceil((pct / 100.0) * len(s)))
    idx = max(0, len(s) - keep)
    return s[idx]


def write_labeled_csv(path: Path, trades: list[Trade], proxy: str, pct: int, out_csv: Path) -> tuple[int, float]:
    thresh = percentile_threshold([trade_feature(t, proxy) for t in trades], pct)
    out_csv.parent.mkdir(parents=True, exist_ok=True)
    good_count = 0
    with out_csv.open("w", newline="") as f:
        w = csv.writer(f)
        w.writerow(
            [
                "rec_id",
                "sym",
                "edge_bps",
                "mfe",
                "mae",
                "pnl",
                "ret_at_exit",
                "rank",
                "rec_score",
                "rec_feas",
                "rec_conf",
                "rec_voters",
                "vol_bps",
                "dur",
                "armed",
                "label_base_top_pct_proxy",
                "label_dur_ge",
                "label_ret_pos",
                f"good_trade_top_{pct}_{proxy}",
            ]
        )
        for t in trades:
            base = 1 if trade_feature(t, proxy) >= thresh else 0
            dur_gate = getattr(write_labeled_csv, "_dur_gate", 0)
            ret_gate = getattr(write_labeled_csv, "_ret_gate", False)
            pass_dur = (t.dur >= dur_gate) if dur_gate > 0 else True
            pass_ret = (t.ret_at_exit > 0.0) if ret_gate else True
            good = 1 if (base and pass_dur and pass_ret) else 0
            good_count += good
            w.writerow(
                [
                    t.rec_id,
                    t.sym,
                    t.edge_bps,
                    t.mfe,
                    t.mae,
                    t.pnl,
                    t.ret_at_exit,
                    t.rank,
                    t.rec_score,
                    t.rec_feas,
                    t.rec_conf,
                    t.rec_voters,
                    t.vol_bps,
                    t.dur,
                    t.armed,
                    base,
                    dur_gate,
                    int(ret_gate),
                    good,
                ]
            )
    return good_count, thresh


def print_feature_separation(trades: list[Trade], proxy: str, pct: int) -> None:
    thresh = percentile_threshold([trade_feature(t, proxy) for t in trades], pct)
    dur_gate = getattr(print_feature_separation, "_dur_gate", 0)
    ret_gate = getattr(print_feature_separation, "_ret_gate", False)
    good = []
    rest = []
    for t in trades:
        base = trade_feature(t, proxy) >= thresh
        pass_dur = (t.dur >= dur_gate) if dur_gate > 0 else True
        pass_ret = (t.ret_at_exit > 0.0) if ret_gate else True
        if base and pass_dur and pass_ret:
            good.append(t)
        else:
            rest.append(t)
    print("\n=== FEATURE SEPARATION (good vs rest) ===")
    if not good or not rest:
        print("Insufficient split for good vs rest.")
        return
    label_desc = f"top_{pct}% by {proxy}"
    if dur_gate > 0:
        label_desc += f" AND dur>={dur_gate}"
    if ret_gate:
        label_desc += " AND ret_at_exit>0"
    print(f"label: {label_desc} | good={len(good)} rest={len(rest)}")
    print("feature\tgood_mean\trest_mean\tdelta(good-rest)")
    features = [
        "edge_bps",
        "rank",
        "rec_score",
        "rec_feas",
        "rec_conf",
        "rec_voters",
        "momentum_3",
        "vol_5",
        "score_std_5",
        "vol_bps",
        "mfe",
        "mae",
        "pnl",
        "ret_at_exit",
        "dur",
        "armed",
    ]
    for feat in features:
        g = sum(getattr(t, feat) for t in good) / len(good)
        r = sum(getattr(t, feat) for t in rest) / len(rest)
        print(f"{feat}\t{g:.6f}\t{r:.6f}\t{(g-r):.6f}")


def print_survival_diagnostics(
    trades: list[Trade],
    features: list[str],
    q: int = 5,
    survive_dur_ge: int = 10,
    survive_requires_ret_positive: bool = False,
) -> None:
    if not trades:
        print("\n=== SURVIVAL DIAGNOSTICS ===")
        print("No trades.")
        return
    print("\n=== SURVIVAL DIAGNOSTICS ===")
    survive_desc = f"dur>={survive_dur_ge}"
    if survive_requires_ret_positive:
        survive_desc += " AND ret_at_exit>0"
    print(f"target: survive = {survive_desc}")
    print("feature\tcorr_survive\tcorr_pnl\tmonotonic_survival\tq1_survive\tq5_survive\tdelta_q5_q1")
    survive = [
        1.0
        if (t.dur >= survive_dur_ge and (t.ret_at_exit > 0.0 or not survive_requires_ret_positive))
        else 0.0
        for t in trades
    ]
    pnl = [t.pnl for t in trades]

    def corr(a: list[float], b: list[float]) -> float:
        ma = sum(a) / len(a)
        mb = sum(b) / len(b)
        num = sum((x - ma) * (y - mb) for x, y in zip(a, b))
        da = math.sqrt(sum((x - ma) ** 2 for x in a))
        db = math.sqrt(sum((y - mb) ** 2 for y in b))
        if da <= 1e-12 or db <= 1e-12:
            return 0.0
        return num / (da * db)

    for feat in features:
        vals = [float(getattr(t, feat)) for t in trades]
        c_surv = corr(vals, survive)
        c_pnl = corr(vals, pnl)
        ordered = sorted(trades, key=lambda t: float(getattr(t, feat)))
        step = math.ceil(len(ordered) / q)
        buckets = [ordered[i : i + step] for i in range(0, len(ordered), step)]
        while len(buckets) < q:
            buckets.append([])
        buckets = buckets[:q]
        surv_rates = []
        for b in buckets:
            if not b:
                surv_rates.append(0.0)
            else:
                surv_rates.append(
                    sum(
                        1
                        for t in b
                        if t.dur >= survive_dur_ge
                        and (t.ret_at_exit > 0.0 or not survive_requires_ret_positive)
                    )
                    / len(b)
                )
        mono = all(surv_rates[i] <= surv_rates[i + 1] + 1e-12 for i in range(len(surv_rates) - 1))
        q1 = surv_rates[0]
        q5 = surv_rates[-1]
        print(
            f"{feat}\t{c_surv:.3f}\t{c_pnl:.3f}\t{'YES' if mono else 'NO'}\t"
            f"{q1:.3f}\t{q5:.3f}\t{(q5-q1):.3f}"
        )


def print_report(r: dict[str, object], primary: bool) -> None:
    kind = "PRIMARY" if primary else "REFERENCE"
    print(f"\n=== RECOMMENDATION READINESS ({kind}) ===")
    print(f"log: {r['path']}")
    print(f"proxy: {r['proxy']} (provisional realized-path proxy)")
    print(
        f"filter: drop_bottom_{r['prefilter_bottom_pct']}% then top_{r['top_pct']}% "
        f"({r['trades']}/{r['prefilter_kept']}/{r['total_trades_in_log']} trades)"
    )
    gates = r["gates"]
    for k in ("Integrity", "Slice Stability", "Contribution", "Ranking", "Filter Uplift", "Stability"):
        print(f"{k}: {gates[k]}")
    final = "READY" if r["final_ready"] else "NOT READY"
    icon = "✅" if r["final_ready"] else "❌"
    print(f"\nFINAL: {icon} {final}")
    print("\n-- Metrics --")
    print(f"trades={r['trades']} expectancy={fmt(r['mean_exp'])} total_pnl={fmt(r['total_pnl'])} violations={r['violations']}")
    print(f"slice_pos={r['slice_pos']}/5 slice_min={fmt(r['slice_min'])} slice_max={fmt(r['slice_max'])} slice_stddev={fmt(r['slice_std'])}")
    t10 = r["top10_share"]
    t10s = "inf" if t10 == float("inf") else f"{t10:.3f}"
    t10p = r["top10_positive_share"]
    t10ps = "inf" if t10p == float("inf") else f"{t10p:.3f}"
    print(f"top10_share={t10s} top10_positive_share={t10ps}")
    print(f"bucket_expectancies={[round(x, 6) for x in r['bucket_exps']]} monotonic_score={r['rank_score']:.3f}")
    print(f"filter_baseline={fmt(r['base_exp'])} filter_top20={fmt(r['filt_exp'])} uplift={fmt(r['uplift'])} filtered_count={r['filt_count']}")


def print_matrix(results: list[dict[str, object]]) -> None:
    print("\n=== READINESS MATRIX ===")
    print(
        "Log\tProxy\tTop%\tFinal\tIntegrity\tSlice\tContrib\tRank\tFilter\tStability\tTrades\tExpectancy\tTop10Share"
    )
    for r in results:
        gates = r["gates"]
        final = "READY" if r["final_ready"] else "NOT_READY"
        t10 = r["top10_share"]
        t10s = "inf" if t10 == float("inf") else f"{t10:.3f}"
        print(
            f"{Path(r['path']).name}\t{r['proxy']}\t{r['top_pct']}\t{final}\t"
            f"{gates['Integrity']}\t{gates['Slice Stability']}\t{gates['Contribution']}\t"
            f"{gates['Ranking']}\t{gates['Filter Uplift']}\t{gates['Stability']}\t"
            f"{r['trades']}\t{fmt(r['mean_exp'])}\t{t10s}"
        )


def main() -> int:
    ap = argparse.ArgumentParser(description="Recommendation readiness gate report")
    ap.add_argument("logs", nargs="+", help="One or more paper log files")
    ap.add_argument("--primary-log", default="", help="Primary log path for go/no-go")
    proxy_choices = [
        "edge_bps",
        "edge_bps_v2",
        "good_trade_classifier",
        "rec_voters",
        "rec_voters_vol",
        "mfe",
        "ret_at_exit",
        "dur",
        "mae",
        "rank",
        "vol_bps",
    ]
    ap.add_argument("--proxy", default="edge_bps", choices=proxy_choices)
    ap.add_argument(
        "--proxies",
        nargs="+",
        choices=proxy_choices,
        default=[],
        help="Optional list of proxies to evaluate in one run (overrides --proxy)",
    )
    ap.add_argument(
        "--filter-percentiles",
        nargs="+",
        type=int,
        default=[100],
        help="Top-percent filters to certify (e.g. 100 30 20 10)",
    )
    ap.add_argument(
        "--prefilter-bottom-percent",
        type=int,
        default=50,
        help="Drop this bottom percent by proxy before top-K selection",
    )
    ap.add_argument(
        "--ranking-tolerance",
        type=float,
        default=0.0001,
        help="Tolerance for 3-bucket ranking monotonicity checks",
    )
    ap.add_argument(
        "--matrix",
        action="store_true",
        help="Print compact matrix view (proxy x percentile)",
    )
    ap.add_argument(
        "--label-top-decile-by",
        default="",
        choices=proxy_choices,
        help="Create good_trade labels from top decile by this proxy",
    )
    ap.add_argument(
        "--label-percentile",
        type=int,
        default=10,
        help="Percentile used for good_trade labeling (default: 10)",
    )
    ap.add_argument(
        "--export-csv",
        default="",
        help="Output CSV path for labeled trades (single-log mode recommended)",
    )
    ap.add_argument(
        "--feature-separation",
        action="store_true",
        help="Print good-vs-rest feature means using labeling settings",
    )
    ap.add_argument(
        "--label-requires-dur-ge",
        type=int,
        default=0,
        help="Optional tightened label gate: require dur >= N",
    )
    ap.add_argument(
        "--label-requires-ret-positive",
        action="store_true",
        help="Optional tightened label gate: require ret_at_exit > 0",
    )
    ap.add_argument(
        "--survival-diagnostics",
        action="store_true",
        help="Print feature survival diagnostics (quantiles + correlations)",
    )
    ap.add_argument(
        "--survival-dur-ge",
        type=int,
        default=10,
        help="Survival target threshold: dur >= N (default: 10)",
    )
    ap.add_argument(
        "--survival-requires-ret-positive",
        action="store_true",
        help="Survival target additionally requires ret_at_exit > 0",
    )
    args = ap.parse_args()

    primary = str(Path(args.primary_log).resolve()) if args.primary_log else ""
    code = 0
    pcts = []
    for p in args.filter_percentiles:
        if p <= 0 or p > 100:
            print(f"ERROR: invalid percentile {p}; must be 1..100")
            return 2
        pcts.append(p)
    if args.prefilter_bottom_percent < 0 or args.prefilter_bottom_percent >= 100:
        print("ERROR: --prefilter-bottom-percent must be in [0, 99]")
        return 2
    if args.label_percentile <= 0 or args.label_percentile > 100:
        print("ERROR: --label-percentile must be 1..100")
        return 2
    if args.label_requires_dur_ge < 0:
        print("ERROR: --label-requires-dur-ge must be >= 0")
        return 2
    if args.survival_dur_ge < 0:
        print("ERROR: --survival-dur-ge must be >= 0")
        return 2

    all_results: list[dict[str, object]] = []
    proxy_list = args.proxies if args.proxies else [args.proxy]

    for raw in args.logs:
        p = Path(raw)
        if not p.exists():
            print(f"\nERROR: missing log {p}")
            code = 2
            continue
        raw_trades = read_trades(p)
        is_primary = str(p.resolve()) == primary if primary else False
        for proxy in proxy_list:
            for pct in pcts:
                res = evaluate(
                    p,
                    proxy,
                    pct,
                    args.prefilter_bottom_percent,
                    args.ranking_tolerance,
                )
                all_results.append(res)
                if not args.matrix:
                    print_report(res, is_primary)
                if is_primary and not res["final_ready"]:
                    code = 1
        if args.label_top_decile_by:
            write_labeled_csv._dur_gate = args.label_requires_dur_ge
            write_labeled_csv._ret_gate = args.label_requires_ret_positive
            out_csv = (
                Path(args.export_csv)
                if args.export_csv
                else p.with_name(f"{p.stem}_labeled.csv")
            )
            good_n, thresh = write_labeled_csv(
                p,
                raw_trades,
                args.label_top_decile_by,
                args.label_percentile,
                out_csv,
            )
            print(
                f"\nLabeled CSV: {out_csv} | label=top_{args.label_percentile}% by {args.label_top_decile_by} "
                f"| threshold={thresh:.6f} | good={good_n}/{len(raw_trades)}"
            )
            if args.feature_separation:
                print_feature_separation._dur_gate = args.label_requires_dur_ge
                print_feature_separation._ret_gate = args.label_requires_ret_positive
                print_feature_separation(
                    raw_trades, args.label_top_decile_by, args.label_percentile
                )
        if args.survival_diagnostics:
            print_survival_diagnostics(
                raw_trades,
                [
                    "rec_feas",
                    "rec_conf",
                    "rec_score",
                    "rec_voters",
                    "momentum_3",
                    "vol_5",
                    "score_std_5",
                    "vol_bps",
                    "rank",
                    "edge_bps",
                ],
                survive_dur_ge=args.survival_dur_ge,
                survive_requires_ret_positive=args.survival_requires_ret_positive,
            )
    if args.matrix and all_results:
        print_matrix(all_results)
    return code


if __name__ == "__main__":
    raise SystemExit(main())
