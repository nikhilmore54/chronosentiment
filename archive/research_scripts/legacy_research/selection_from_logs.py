#!/usr/bin/env python3
"""
Offline selection experiments from live_engine + paper logs (deterministic).

Parses [TRADE_PATH] lines for closed trades:
  mfe, mae, pnl, dur (hold at exit — timing proxy; logs do not include bars_to_mfe)

Lifecycle score v1 (default): mfe - 0.5*|mae| - 0.002*dur
Lifecycle score v2 (--score-v2): mfe - 0.5*|mae| - 0.1*(dur/max_dur_log) + 0.5*pnl
Lifecycle score v3 (--score-v3, optional offline): v2 base + 0.3 * (mfe / (dur+1)) early_efficiency

Parses [REC_OUTCOME] for entry score. With --slice/--top, if REC_OUTCOME rows exist,
prints entry slice + entry_capture_ratio = entry_pnl / lifecycle_ceiling_pnl (same slices).

With --compare-lifecycle-v2-v3 (requires --slice/--top): prints v2 vs v3 lifecycle ceiling
totals, per-slice ceiling PnL, Jaccard overlap of selected rec_ids, pstdev of slice pnls,
and entry_capture_ratio for each ceiling (entry rank unchanged; missing REC_OUTCOME falls
back to lifecycle v2 for a fixed numerator).

See .cursor/rules + docs for deterministic replay discipline.
"""

from __future__ import annotations

import argparse
import re
import statistics
import sys
from collections.abc import Callable
from dataclasses import dataclass

TPATH = re.compile(
    r"\[TRADE_PATH\]\s+rec_id=(?P<rid>\d+)\s+sym=(?P<sym>\S+)\s+mfe=(?P<mfe>[-0-9.eE+]+)\s+"
    r"mae=(?P<mae>[-0-9.eE+]+)\s+pnl=(?P<pnl>[-0-9.eE+]+)\s+ret_at_exit=(?P<ret>[-0-9.eE+]+)\s+dur=(?P<dur>\d+)"
)
REC_OUT = re.compile(
    r"\[REC_OUTCOME\]\s+rec_id=(?P<rid>\d+)\s+sym=\S+\s+score=(?P<score>[-0-9.eE+]+)\s+"
    r"edge=(?P<edge>[-0-9.eE+]+)\s+feas=(?P<feas>[-0-9.eE+]+)\s+conf=(?P<conf>[-0-9.eE+]+)\s+"
    r"voters=(?P<voters>\d+)\s+S(?P<S>\d+)\s+pnl=(?P<pnl>[-0-9.eE+]+)"
)


@dataclass
class TradeClose:
    rec_id: int
    sym: str
    mfe: float
    mae: float
    pnl: float
    dur: int
    lifecycle_score: float
    lifecycle_score_v2: float
    lifecycle_score_v3: float


def lifecycle_score_v1(mfe: float, mae: float, dur: int) -> float:
    return mfe - 0.5 * abs(mae) - 0.002 * float(dur)


def lifecycle_score_v2(mfe: float, mae: float, dur: int, pnl: float, max_dur: int) -> float:
    md = max(max_dur, 1)
    return mfe - 0.5 * abs(mae) - 0.1 * (float(dur) / float(md)) + 0.5 * pnl


def lifecycle_score_v3(mfe: float, mae: float, dur: int, pnl: float, max_dur: int) -> float:
    md = max(max_dur, 1)
    norm_dur = float(dur) / float(md)
    early_efficiency = mfe / (float(dur) + 1.0)
    return (
        mfe - 0.5 * abs(mae) + 0.5 * pnl - 0.1 * norm_dur + 0.3 * early_efficiency
    )


def parse_trades(text: str) -> list[TradeClose]:
    raw: list[tuple[int, str, float, float, float, int]] = []
    for m in TPATH.finditer(text):
        rid = int(m.group("rid"))
        mfe = float(m.group("mfe"))
        mae = float(m.group("mae"))
        pnl = float(m.group("pnl"))
        dur = int(m.group("dur"))
        raw.append((rid, m.group("sym"), mfe, mae, pnl, dur))
    max_dur = max((r[5] for r in raw), default=1)
    out: list[TradeClose] = []
    for rid, sym, mfe, mae, pnl, dur in raw:
        out.append(
            TradeClose(
                rec_id=rid,
                sym=sym,
                mfe=mfe,
                mae=mae,
                pnl=pnl,
                dur=dur,
                lifecycle_score=lifecycle_score_v1(mfe, mae, dur),
                lifecycle_score_v2=lifecycle_score_v2(mfe, mae, dur, pnl, max_dur),
                lifecycle_score_v3=lifecycle_score_v3(mfe, mae, dur, pnl, max_dur),
            )
        )
    return out


def parse_entry_scores(text: str) -> dict[int, float]:
    d: dict[int, float] = {}
    for m in REC_OUT.finditer(text):
        d[int(m.group("rid"))] = float(m.group("score"))
    return d


def baseline(trades: list[TradeClose]) -> tuple[float, int]:
    if not trades:
        return 0.0, 0
    s = sum(t.pnl for t in trades)
    return s, len(trades)


def sim_keep_top_pct(
    trades: list[TradeClose], pct: float, score_fn: Callable[[TradeClose], float]
) -> tuple[float, int]:
    if not trades or pct <= 0:
        return 0.0, 0
    n = max(1, int(len(trades) * pct / 100.0))
    ranked = sorted(trades, key=score_fn, reverse=True)
    kept = ranked[:n]
    return sum(t.pnl for t in kept), len(kept)


def sim_slice_topk(
    trades: list[TradeClose],
    slice_n: int,
    top_k: int,
    score_fn: Callable[[TradeClose], float],
) -> tuple[float, int]:
    if slice_n < 1 or top_k < 1 or not trades:
        return 0.0, 0
    rows = slice_topk_per_slice(trades, slice_n, top_k, score_fn)
    total = sum(p for _, p in rows)
    counted = sum(len(s) for s, _ in rows)
    return total, counted


def slice_topk_per_slice(
    trades: list[TradeClose],
    slice_n: int,
    top_k: int,
    score_fn: Callable[[TradeClose], float],
) -> list[tuple[frozenset[int], float]]:
    """Per log-order chunk: frozenset(rec_id) of top-K by score, and sum pnl of those rows."""
    if slice_n < 1 or top_k < 1 or not trades:
        return []
    out: list[tuple[frozenset[int], float]] = []
    for i in range(0, len(trades), slice_n):
        chunk = trades[i : i + slice_n]
        ranked = sorted(chunk, key=score_fn, reverse=True)
        take = ranked[: min(top_k, len(ranked))]
        out.append((frozenset(t.rec_id for t in take), sum(t.pnl for t in take)))
    return out


def print_compare_lifecycle_v2_v3(
    trades: list[TradeClose],
    entry_scores: dict[int, float],
    slice_n: int,
    top_k: int,
) -> None:
    life_v2 = lambda t: t.lifecycle_score_v2
    life_v3 = lambda t: t.lifecycle_score_v3
    rows_v2 = slice_topk_per_slice(trades, slice_n, top_k, life_v2)
    rows_v3 = slice_topk_per_slice(trades, slice_n, top_k, life_v3)
    if not rows_v2:
        print("\n--compare-lifecycle-v2-v3: no slices.")
        return

    total_v2 = sum(p for _, p in rows_v2)
    total_v3 = sum(p for _, p in rows_v3)
    n_slots = sum(len(s) for s, _ in rows_v2)
    pnls_2 = [p for _, p in rows_v2]
    pnls_3 = [p for _, p in rows_v3]

    print(
        f"\n=== --compare-lifecycle-v2-v3 --slice {slice_n} --top {top_k} ==="
    )
    print(
        f"lifecycle_ceiling total_pnl: v2={total_v2:.6f}  v3={total_v3:.6f}  "
        f"(delta v3-v2: {total_v3 - total_v2:+.6f})"
    )
    print(
        f"lifecycle_ceiling avg_pnl/slot: v2={total_v2 / max(1, n_slots):.6f}  "
        f"v3={total_v3 / max(1, n_slots):.6f}"
    )
    if len(pnls_2) > 1:
        print(
            "per-slice ceiling pnl pstdev (lower => flatter slices): "
            f"v2={statistics.pstdev(pnls_2):.6f}  v3={statistics.pstdev(pnls_3):.6f}"
        )

    print(
        "\nper_slice  idx  pnl_v2    pnl_v3    overlap  k  jaccard  "
        "(overlap = |topK_v2 ∩ topK_v3| in this slice)"
    )
    jaccards: list[float] = []
    sum_overlap = 0
    for idx, ((s2, p2), (s3, p3)) in enumerate(zip(rows_v2, rows_v3)):
        inter = s2 & s3
        union = s2 | s3
        ov = len(inter)
        k = len(s2)
        jac = (ov / len(union)) if union else 1.0
        jaccards.append(jac)
        sum_overlap += ov
        print(
            f"  slice {idx:4d}  {p2:9.6f}  {p3:9.6f}  {ov:7d}  {k:2d}  {jac:7.4f}"
        )

    n_slices = len(rows_v2)
    print(
        f"\noverlap_summary: slices={n_slices}  sum_overlap={sum_overlap}  "
        f"slots={n_slots}  overlap/slot={sum_overlap / max(1, n_slots):.4f}"
    )
    print(
        f"jaccard_macro_mean (mean of per-slice Jaccard): "
        f"{sum(jaccards) / max(1, len(jaccards)):.4f}"
    )

    if entry_scores:
        entry_fb = life_v2
        pnl_e, ne = sim_slice_topk_entry_score(
            trades, entry_scores, slice_n, top_k, entry_fb
        )
        print(
            f"\nentry slice (rank [REC_OUTCOME]; missing rec_id fallback=lifecycle_v2): "
            f"slots={ne}  total_pnl={pnl_e:.6f}  avg={pnl_e / max(1, ne):.6f}"
        )
        r2 = pnl_e / total_v2 if abs(total_v2) > 1e-18 else float("nan")
        r3 = pnl_e / total_v3 if abs(total_v3) > 1e-18 else float("nan")
        print(
            f"entry_capture_ratio: v2={r2:.4f}  v3={r3:.4f}  "
            f"(delta v3-v2: {r3 - r2:+.4f})"
        )
        print("  interpret: higher ratio with high overlap => refinement, not reshuffle")
    else:
        print("\nentry slice / ratios: skipped (no [REC_OUTCOME] rows)")


def sim_slice_topk_entry_score(
    trades: list[TradeClose],
    entry: dict[int, float],
    slice_n: int,
    top_k: int,
    fallback_fn: Callable[[TradeClose], float],
) -> tuple[float, int]:
    if slice_n < 1 or top_k < 1 or not trades:
        return 0.0, 0

    def rank_key(t: TradeClose) -> float:
        return entry.get(t.rec_id, fallback_fn(t))

    total = 0.0
    counted = 0
    for i in range(0, len(trades), slice_n):
        chunk = trades[i : i + slice_n]
        ranked = sorted(chunk, key=rank_key, reverse=True)
        take = ranked[: min(top_k, len(ranked))]
        total += sum(t.pnl for t in take)
        counted += len(take)
    return total, counted


def main() -> None:
    ap = argparse.ArgumentParser(description="Selection simulations from paper TRADE_PATH logs.")
    ap.add_argument("log_path", help="Path to live_engine log")
    ap.add_argument(
        "--keep-top-pct",
        type=float,
        metavar="PCT",
        help="Ex-post: sum pnl if only top PCT%% by lifecycle score (global).",
    )
    ap.add_argument("--slice", type=int, metavar="N", help="Slice size (closes in log order).")
    ap.add_argument("--top", type=int, metavar="K", help="Keep top K per slice.")
    ap.add_argument(
        "--slice-by-entry-score",
        action="store_true",
        help="Also print entry-score slice (when used alone, prints entry + lifecycle + ratio).",
    )
    ap.add_argument(
        "--score-v2",
        action="store_true",
        help="Use v2 lifecycle score for lifecycle-based rankings (ignored if --score-v3).",
    )
    ap.add_argument(
        "--score-v3",
        action="store_true",
        help="Use v3 lifecycle score (v2-style + early_efficiency mfe/(dur+1)); overrides --score-v2.",
    )
    ap.add_argument(
        "--compare-lifecycle-v2-v3",
        action="store_true",
        help="With --slice and --top: print v2 vs v3 ceiling, per-slice pnl, overlap/Jaccard, entry ratios.",
    )
    args = ap.parse_args()

    if args.compare_lifecycle_v2_v3 and (
        args.slice is None or args.top is None
    ):
        print(
            "error: --compare-lifecycle-v2-v3 requires --slice N and --top K",
            file=sys.stderr,
        )
        sys.exit(2)

    with open(args.log_path, encoding="utf-8", errors="replace") as f:
        text = f.read()

    trades = parse_trades(text)
    entry_scores = parse_entry_scores(text)
    if args.score_v3:
        life_fn = lambda t: t.lifecycle_score_v3
        score_label = "lifecycle_v3"
    elif args.score_v2:
        life_fn = lambda t: t.lifecycle_score_v2
        score_label = "lifecycle_v2"
    else:
        life_fn = lambda t: t.lifecycle_score
        score_label = "lifecycle_v1"

    total_pnl, n = baseline(trades)
    print(f"=== {args.log_path} ===")
    print(f"parsed_closes: {n}")
    if n == 0:
        print("No [TRADE_PATH] matches; nothing to score.")
        sys.exit(0)

    print(f"total_pnl: {total_pnl:.6f}")
    print(f"avg_pnl:   {total_pnl / n:.6f}")
    s1 = [t.lifecycle_score for t in trades]
    s2 = [t.lifecycle_score_v2 for t in trades]
    s3 = [t.lifecycle_score_v3 for t in trades]
    print(
        f"lifecycle_v1: min={min(s1):.6f} max={max(s1):.6f} mean={sum(s1)/n:.6f}"
    )
    print(
        f"lifecycle_v2: min={min(s2):.6f} max={max(s2):.6f} mean={sum(s2)/n:.6f}"
    )
    print(
        f"lifecycle_v3: min={min(s3):.6f} max={max(s3):.6f} mean={sum(s3)/n:.6f}"
    )
    print(f"[REC_OUTCOME] rows for join: {len(entry_scores)}")
    if args.compare_lifecycle_v2_v3:
        print("ranking_mode: compare lifecycle_v2 vs lifecycle_v3 (slice sims)")
    else:
        print(f"ranking_mode: {score_label} (lifecycle sims)")

    if args.keep_top_pct is not None:
        pnl_k, nk = sim_keep_top_pct(trades, args.keep_top_pct, life_fn)
        print(f"\n--keep-top-pct {args.keep_top_pct} (ex-post global, {score_label}):")
        print(f"  kept_trades: {nk}  total_pnl: {pnl_k:.6f}  avg_pnl: {pnl_k/max(1,nk):.6f}")

    if args.slice is not None and args.top is not None:
        if args.compare_lifecycle_v2_v3:
            print_compare_lifecycle_v2_v3(
                trades, entry_scores, args.slice, args.top
            )
        else:
            pnl_l, nl = sim_slice_topk(trades, args.slice, args.top, life_fn)
            print(
                f"\n--slice {args.slice} --top {args.top} "
                f"(lifecycle ceiling, {score_label}):"
            )
            print(
                f"  counted_trade_rows: {nl}  total_pnl: {pnl_l:.6f}  "
                f"avg_pnl: {pnl_l/max(1,nl):.6f}"
            )

            want_entry = args.slice_by_entry_score or bool(entry_scores)
            if want_entry and entry_scores:
                pnl_e, ne = sim_slice_topk_entry_score(
                    trades, entry_scores, args.slice, args.top, life_fn
                )
                label = (
                    "--slice-by-entry-score"
                    if args.slice_by_entry_score
                    else "auto (REC_OUTCOME present)"
                )
                print(
                    f"\n  entry slice ({label}, rank by [REC_OUTCOME] score):"
                )
                print(
                    f"  counted_trade_rows: {ne}  total_pnl: {pnl_e:.6f}  "
                    f"avg_pnl: {pnl_e/max(1,ne):.6f}"
                )
                if abs(pnl_l) > 1e-18:
                    ratio = pnl_e / pnl_l
                    print(
                        f"\n  entry_capture_ratio "
                        f"(entry_pnl / lifecycle_ceiling_pnl): {ratio:.4f}"
                    )
                    print(
                        "    >0.8 strong  |  0.5–0.8 good  |  "
                        "<0.5 entry score misaligned"
                    )
                else:
                    print(
                        "\n  entry_capture_ratio: n/a "
                        "(lifecycle ceiling pnl ~ 0)"
                    )
            elif args.slice_by_entry_score and not entry_scores:
                print("\n  entry slice: skipped (no [REC_OUTCOME] rows)")

    print(
        "\nNote: dur is hold at exit; v2/v3 normalize dur by max(dur) in this log. v3 adds mfe/(dur+1)."
    )


if __name__ == "__main__":
    main()
