#!/usr/bin/env python3
"""
Read-only window audit for live_engine logs (chronosentiment-core.mdc: deterministic).

``[SYMBOL_TS]`` epochs are **Unix UTC**. Use ``--tz UTC`` (default) so ``--start``/``--end``
match logs directly; use ``--tz Asia/Kolkata`` for IST wall times.

Tape semantics: non-``SYMBOL_TS`` lines are counted only when attributed to a ``SYMBOL_TS``
epoch inside the window.

**Price delta:** For each ``SYMBOL_TS``, the **first** following ``[SYMBOL_PRICE]`` for that
symbol is the sample close; consecutive closes (globally by epoch order across matched files)
yield ``delta = |p_i - p_{i-1}|``. Count transitions whose **newer** epoch falls in the window.
``delta_nonzero_ratio = delta_nonzero_count / max(1, SYMBOL_TS in window)``.

Examples::

  python3 scripts/audit_window.py \\
    --log-dir analysis/live_multi --symbol BTC-USD \\
    --start "2026-05-02 14:28" --end "2026-05-02 16:56"

  python3 scripts/audit_window.py \\
    --log-dir analysis/live_multi --symbol BTC-USD --tz Asia/Kolkata \\
    --start "2026-05-02 19:58" --end "2026-05-02 22:26"
"""

from __future__ import annotations

import argparse
import glob
import re
import sys
from collections import defaultdict
from datetime import datetime, timezone

try:
    from zoneinfo import ZoneInfo
except ImportError:  # pragma: no cover
    ZoneInfo = None  # type: ignore[misc, assignment]

TS_RE = re.compile(r"\[SYMBOL_TS\]\s+([^\s:]+):(\d+)")
PRICE_RE = re.compile(r"\[SYMBOL_PRICE\]\s+([^\s:]+):\s*([0-9.eE+-]+)")
MOM_RE = re.compile(r"\[MOMENTUM_CHECK\]")
EDGE_RE = re.compile(r"\[EDGE_COMPONENTS\]")
DIAG_EDGE_ONLY_RE = re.compile(r"\[DIAG\]\s+sym=\S+\s+edge=([0-9.eE+-]+)")
DIAG_GATES_RE = re.compile(
    r"pass_edge=(\d+)\s+pass_conf=(\d+)\s+pass_reco=(\d+)\s+FINAL=(\d+)"
)
DIAG_FEAS_VOTERS_RE = re.compile(r"feas=([0-9.eE+-]+)\s+voters=(\d+)")
EDGE_PIPE_RE = re.compile(
    r"\[EDGE_PIPE\]\s+sym=\S+\s+raw_edge=([0-9.eE+-]+).*edge_gate=([0-9.eE+-]+)\s+edge_min=([0-9.eE+-]+)"
)
RECO_RE = re.compile(r"\[RECOMMENDATION\]")

# Minimum absolute price move (USD) to count as "nonzero" delta (~flat input vs movement).
DEFAULT_DELTA_EPS = 0.01


def normalize_symbol(sym: str) -> str:
    return sym.strip().upper().replace("_", "-")


def normalize_epoch(ts_raw: int) -> int:
    if ts_raw > 1_000_000_000_000:
        return int(ts_raw // 1000)
    return int(ts_raw)


def parse_wall_clock(s: str, tz: ZoneInfo) -> datetime:
    s = s.strip()
    for fmt in ("%Y-%m-%d %H:%M", "%Y-%m-%d %H:%M:%S"):
        try:
            dt = datetime.strptime(s, fmt)
            return dt.replace(tzinfo=tz)
        except ValueError:
            continue
    raise SystemExit(f"Could not parse datetime: {s!r} (expected YYYY-MM-DD HH:MM)")


def parse_args() -> argparse.Namespace:
    ap = argparse.ArgumentParser(description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter)
    ap.add_argument("--log-dir", required=True, help="Directory with live_<SYM>_*.log*")
    ap.add_argument("--symbol", default="BTC-USD")
    ap.add_argument("--start", required=True, help="YYYY-MM-DD HH:MM (interpreted in --tz)")
    ap.add_argument("--end", required=True, help="YYYY-MM-DD HH:MM (interpreted in --tz)")
    ap.add_argument(
        "--tz",
        default="UTC",
        help="IANA zone for --start/--end (default UTC — matches SYMBOL_TS epoch domain)",
    )
    ap.add_argument(
        "--delta-eps",
        type=float,
        default=DEFAULT_DELTA_EPS,
        help=f"Abs price delta above this counts as nonzero (default {DEFAULT_DELTA_EPS} USD)",
    )
    ap.add_argument(
        "--ref-edge-min",
        type=float,
        default=None,
        help=(
            "Optional floor (e.g. RECO_EDGE_MIN default 0.0012 in Coverage mode). "
            "Prints fraction of [DIAG] edge= values strictly below this — calibration signal."
        ),
    )
    return ap.parse_args()


def pct(x: float, y: float) -> float:
    return (100.0 * x / y) if y else 0.0


def percentile(sorted_vals: list[float], p: float) -> float:
    """Linear interpolation percentile (p in [0,100])."""
    if not sorted_vals:
        return float("nan")
    x = sorted(sorted_vals)
    n = len(x)
    if n == 1:
        return x[0]
    k = (n - 1) * (p / 100.0)
    lo = int(k)
    hi = min(lo + 1, n - 1)
    return x[lo] + (k - lo) * (x[hi] - x[lo])


class DiagCollector:
    """Aggregates [DIAG] / [EDGE_PIPE] lines (caller ensures attribution / symbol filters)."""

    __slots__ = (
        "diag_edges",
        "pass_edge",
        "pass_conf",
        "pass_reco",
        "final_meta",
        "feas_vals",
        "voters_vals",
        "raw_edges_pipe",
        "edge_gate_pipe",
        "edge_min_pipe",
    )

    def __init__(self) -> None:
        self.diag_edges: list[float] = []
        self.pass_edge: defaultdict[int, int] = defaultdict(int)
        self.pass_conf: defaultdict[int, int] = defaultdict(int)
        self.pass_reco: defaultdict[int, int] = defaultdict(int)
        self.final_meta: defaultdict[int, int] = defaultdict(int)
        self.feas_vals: list[float] = []
        self.voters_vals: list[int] = []
        self.raw_edges_pipe: list[float] = []
        self.edge_gate_pipe: list[float] = []
        self.edge_min_pipe: list[float] = []

    def ingest_diag_line(self, line: str) -> None:
        me = DIAG_EDGE_ONLY_RE.search(line)
        if me:
            try:
                self.diag_edges.append(float(me.group(1)))
            except ValueError:
                pass
        mg = DIAG_GATES_RE.search(line)
        if mg:
            self.pass_edge[int(mg.group(1))] += 1
            self.pass_conf[int(mg.group(2))] += 1
            self.pass_reco[int(mg.group(3))] += 1
            self.final_meta[int(mg.group(4))] += 1
        mf = DIAG_FEAS_VOTERS_RE.search(line)
        if mf:
            try:
                self.feas_vals.append(float(mf.group(1)))
                self.voters_vals.append(int(mf.group(2)))
            except ValueError:
                pass


    def ingest_edge_pipe_line(self, line: str) -> None:
        mp = EDGE_PIPE_RE.search(line)
        if mp:
            try:
                self.raw_edges_pipe.append(float(mp.group(1)))
                self.edge_gate_pipe.append(float(mp.group(2)))
                self.edge_min_pipe.append(float(mp.group(3)))
            except ValueError:
                pass


def extract_ts_first_price_pairs(fp: str, sym_target: str) -> list[tuple[int, float]]:
    """Per file: (SYMBOL_TS epoch -> first SYMBOL_PRICE after it). Pending TS overwritten if no price yet."""
    out: list[tuple[int, float]] = []
    pending_ts: int | None = None
    with open(fp, "r", errors="ignore") as f:
        for line in f:
            m = TS_RE.search(line)
            if m:
                sym = normalize_symbol(m.group(1))
                if sym != sym_target:
                    continue
                pending_ts = normalize_epoch(int(m.group(2)))
                continue
            if pending_ts is None:
                continue
            pm = PRICE_RE.search(line)
            if pm and normalize_symbol(pm.group(1)) == sym_target:
                try:
                    price = float(pm.group(2))
                except ValueError:
                    continue
                out.append((pending_ts, price))
                pending_ts = None
    return out


def merge_close_series(all_pairs: list[tuple[int, float]]) -> list[tuple[int, float]]:
    """Sort by epoch; same-epoch: keep last price (defensive)."""
    by_ts: dict[int, float] = {}
    for ts, p in all_pairs:
        by_ts[ts] = p
    return sorted(by_ts.items(), key=lambda x: x[0])


def delta_transition_stats(
    series: list[tuple[int, float]],
    t0: int,
    t1: int,
    symbol_ts_in_window: int,
    eps: float,
) -> tuple[int, int, float | None, int]:
    """
    Count consecutive-close transitions whose **newer** epoch is in [t0, t1].
    Ratio is nonzero/SYMBOL_TS in window, or None if SYMBOL_TS count is 0.
    """
    nz = z = 0
    trans_in_win = 0
    for i in range(1, len(series)):
        ts_prev, p_prev = series[i - 1]
        ts_cur, p_cur = series[i]
        if not (t0 <= ts_cur <= t1):
            continue
        trans_in_win += 1
        d = abs(p_cur - p_prev)
        if d > eps:
            nz += 1
        else:
            z += 1
    ratio: float | None = (
        (nz / symbol_ts_in_window) if symbol_ts_in_window > 0 else None
    )
    return nz, z, ratio, trans_in_win


def audit_file(
    fp: str,
    sym_target: str,
    t0: int,
    t1: int,
    diag: DiagCollector | None = None,
) -> tuple[dict[str, int], list[int], list[float]]:
    """Return counts for one file, TS list in window, gaps between consecutive TS in window."""
    counts: defaultdict[str, int] = defaultdict(int)
    ts_list: list[int] = []
    gaps: list[float] = []
    last_epoch: int | None = None
    last_ts_in_win: int | None = None

    with open(fp, "r", errors="ignore") as f:
        for line in f:
            m = TS_RE.search(line)
            if m:
                sym = normalize_symbol(m.group(1))
                if sym != sym_target:
                    continue
                ts = normalize_epoch(int(m.group(2)))
                last_epoch = ts
                if t0 <= ts <= t1:
                    counts["SYMBOL_TS"] += 1
                    ts_list.append(ts)
                    if last_ts_in_win is not None:
                        gaps.append(float(ts - last_ts_in_win))
                    last_ts_in_win = ts
                continue

            if last_epoch is None or not (t0 <= last_epoch <= t1):
                continue

            pm = PRICE_RE.search(line)
            if pm and normalize_symbol(pm.group(1)) == sym_target:
                counts["SYMBOL_PRICE"] += 1

            if MOM_RE.search(line):
                counts["MOMENTUM_CHECK"] += 1
                if "condition_met=1" in line:
                    counts["MOMENTUM_COND_1"] += 1

            if EDGE_RE.search(line):
                counts["EDGE_COMPONENTS"] += 1

            if "[DIAG]" in line:
                sm_diag = re.search(r"sym=([^ \t]+)", line)
                if sm_diag and normalize_symbol(sm_diag.group(1)) != sym_target:
                    pass
                else:
                    me = DIAG_EDGE_ONLY_RE.search(line)
                    if me:
                        counts["DIAG_LINES"] += 1
                        try:
                            edge_val = float(me.group(1))
                            if abs(edge_val) > 1e-15:
                                counts["DIAG_EDGE_NONZERO"] += 1
                        except ValueError:
                            pass
                        if diag is not None:
                            diag.ingest_diag_line(line)

            if diag is not None and "[EDGE_PIPE]" in line:
                sm_ep = re.search(r"sym=([^ \t]+)", line)
                if sm_ep and normalize_symbol(sm_ep.group(1)) == sym_target:
                    diag.ingest_edge_pipe_line(line)

            if RECO_RE.search(line):
                counts["RECOMMENDATION"] += 1

    return dict(counts), ts_list, gaps


def main() -> int:
    if ZoneInfo is None:
        print("Python 3.9+ with zoneinfo required.", file=sys.stderr)
        return 1

    a = parse_args()
    try:
        tz = ZoneInfo(a.tz)
    except Exception as e:
        print(f"Invalid --tz {a.tz!r}: {e}", file=sys.stderr)
        return 1

    sym_u = normalize_symbol(a.symbol)
    start_dt = parse_wall_clock(a.start, tz)
    end_dt = parse_wall_clock(a.end, tz)
    if end_dt < start_dt:
        print("--end must be >= --start", file=sys.stderr)
        return 1

    t0 = int(start_dt.timestamp())
    t1 = int(end_dt.timestamp())

    stem = sym_u.replace("-", "_")
    pattern = f"{a.log_dir.rstrip('/')}/live_{stem}*.log*"
    files = sorted(glob.glob(pattern))
    if not files:
        print(f"No files matched {pattern!r}", file=sys.stderr)
        return 1

    agg: defaultdict[str, int] = defaultdict(int)
    all_ts: list[int] = []
    all_gaps: list[float] = []
    diag_g = DiagCollector()

    for fp in files:
        c, ts_l, g = audit_file(fp, sym_u, t0, t1, diag_g)
        all_ts.extend(ts_l)
        all_gaps.extend(g)
        for k, v in c.items():
            agg[k] += int(v)

    # Global close series for delta stats (all files).
    raw_pairs: list[tuple[int, float]] = []
    for fp in files:
        raw_pairs.extend(extract_ts_first_price_pairs(fp, sym_u))
    series = merge_close_series(raw_pairs)
    ts_n = int(agg["SYMBOL_TS"])
    d_nz, d_z, d_ratio_opt, trans_win = delta_transition_stats(series, t0, t1, ts_n, float(a.delta_eps))

    print(f"--start/--end interpreted in timezone: {a.tz}")
    print(f"  Wall range:     {a.start} → {a.end}  ({sym_u})")
    print(
        "  Same instant → UTC: "
        f"{datetime.fromtimestamp(t0, tz=timezone.utc).strftime('%Y-%m-%d %H:%M:%S')}Z → "
        f"{datetime.fromtimestamp(t1, tz=timezone.utc).strftime('%Y-%m-%d %H:%M:%S')}Z"
    )
    print(f"Epoch range [t0,t1]: {t0} .. {t1}")
    if series:
        lo, hi = series[0][0], series[-1][0]
        print(
            "Merged-log SYMBOL_TS coverage (all files): "
            f"{lo} .. {hi}  →  "
            f"{datetime.fromtimestamp(lo, tz=timezone.utc).strftime('%Y-%m-%d %H:%M:%SZ')} .. "
            f"{datetime.fromtimestamp(hi, tz=timezone.utc).strftime('%Y-%m-%d %H:%M:%SZ')}"
        )
        if lo > t1 or hi < t0:
            print(
                "  ⚠ Query window does NOT overlap merged-log timestamps — SYMBOL_TS in window will be 0.",
                file=sys.stderr,
            )
    print(f"Files:            {len(files)}")
    for fp in files:
        print(f"  - {fp}")

    print("\nIngestion:")
    print(f"  SYMBOL_TS: {agg['SYMBOL_TS']}")
    if all_ts:
        print(f"  First TS: {min(all_ts)}  Last TS: {max(all_ts)}")
        if all_gaps:
            print(f"  Max gap (s): {max(all_gaps):.0f}  Avg gap (s): {sum(all_gaps)/len(all_gaps):.2f}")

    print("\nPrice delta (consecutive candle closes; newer epoch in window):")
    print(f"  delta_eps (USD):        {a.delta_eps}")
    print(f"  transitions_in_window:  {trans_win}  (pairs ending in [t0,t1])")
    print(f"  delta_nonzero_count:    {d_nz}")
    print(f"  delta_zero_count:       {d_z}")
    if d_ratio_opt is not None:
        print(f"  delta_nonzero_ratio:    {d_ratio_opt:.4f}  (= nonzero / SYMBOL_TS in window)")
    else:
        print("  delta_nonzero_ratio:    N/A  (no SYMBOL_TS lines in window)")
    if series:
        print(f"  global_close_series:    {len(series)} unique SYMBOL_TS closes (merged files)")

    print("\nSignal (lines attributed to SYMBOL_TS epochs inside window):")
    mc = agg["MOMENTUM_CHECK"]
    print(f"  MOMENTUM_CHECK: {mc}")
    print(
        f"  condition_met=1: {agg['MOMENTUM_COND_1']} "
        f"({pct(float(agg['MOMENTUM_COND_1']), float(mc)):.2f}%)"
    )
    print(f"  EDGE_COMPONENTS lines: {agg['EDGE_COMPONENTS']}")
    dl = agg["DIAG_LINES"]
    dz = agg["DIAG_EDGE_NONZERO"]
    print(f"  DIAG lines: {dl}")
    print(f"  DIAG edge_nonzero: {dz} ({pct(float(dz), float(dl)):.2f}%)")

    if diag_g.diag_edges:
        de = diag_g.diag_edges
        print("\n[DIAG] depth (attributed window — calibration vs gating):")
        print(
            "  Note: DIAG `edge=` may be 0 when selected_edge_gate≈0 (then falls back to raw in logs); "
            "pass_* / FINAL are meta gates — emission still requires voters/rec_score path."
        )
        print(f"  DIAG edge= samples:    {len(de)}")
        print(
            f"  edge min/mean/max:     {min(de):.6f} / {sum(de)/len(de):.6f} / {max(de):.6f}"
        )
        print(
            f"  edge p50/p90/p95:      {percentile(de, 50):.6f} / "
            f"{percentile(de, 90):.6f} / {percentile(de, 95):.6f}"
        )
        pe1 = int(diag_g.pass_edge[1])
        pe0 = int(diag_g.pass_edge[0])
        print(f"  pass_edge=1 / =0:      {pe1} / {pe0}  (stability gate)")
        print(
            f"  pass_conf=1 / =0:      {int(diag_g.pass_conf[1])} / {int(diag_g.pass_conf[0])}"
        )
        print(
            f"  pass_reco=1 / =0:      {int(diag_g.pass_reco[1])} / {int(diag_g.pass_reco[0])}"
        )
        print(
            f"  FINAL=1 / =0:         {int(diag_g.final_meta[1])} / {int(diag_g.final_meta[0])}  "
            "(meta AND of stability ∧ conf ∧ reco structure)"
        )
        if diag_g.feas_vals:
            fv = diag_g.feas_vals
            print(
                f"  feas min/mean/max:     {min(fv):.4f} / {sum(fv)/len(fv):.4f} / {max(fv):.4f}"
            )
        if diag_g.voters_vals:
            vv = diag_g.voters_vals
            vz = sum(1 for v in vv if v == 0)
            print(f"  voters=0 lines:       {vz}/{len(vv)} ({pct(float(vz), float(len(vv))):.1f}%)")
        if diag_g.raw_edges_pipe:
            re_list = diag_g.raw_edges_pipe
            print(
                f"  EDGE_PIPE raw_edge:    min/mean/max {min(re_list):.6f} / "
                f"{sum(re_list)/len(re_list):.6f} / {max(re_list):.6f}  "
                "(pre-floor signal — compare to edge_min)"
            )
            print(
                f"  EDGE_PIPE raw p50/p90: {percentile(re_list, 50):.6f} / "
                f"{percentile(re_list, 90):.6f}"
            )
        if diag_g.edge_min_pipe:
            umn = sorted({round(x, 9) for x in diag_g.edge_min_pipe})
            print(f"  EDGE_PIPE edge_min:    {umn[0] if len(umn)==1 else f'{umn} (unique values)'}")
        if diag_g.edge_gate_pipe:
            eg = diag_g.edge_gate_pipe
            print(
                f"  EDGE_PIPE edge_gate:   min/mean/max {min(eg):.6f} / "
                f"{sum(eg)/len(eg):.6f} / {max(eg):.6f}"
            )
        if a.ref_edge_min is not None:
            ref = float(a.ref_edge_min)
            below = sum(1 for x in de if x < ref)
            print(
                f"  DIAG edge < --ref-edge-min ({ref}):  "
                f"{below}/{len(de)} ({pct(float(below), float(len(de))):.1f}%)  "
                "(DIAG field may be 0 while raw_edge carries scale — see EDGE_PIPE above)"
            )
            if diag_g.raw_edges_pipe:
                re_list = diag_g.raw_edges_pipe
                rb = sum(1 for x in re_list if x < ref)
                print(
                    f"  raw_edge < --ref-edge-min ({ref}): {rb}/{len(re_list)} "
                    f"({pct(float(rb), float(len(re_list))):.1f}%)"
                )
    elif dl > 0:
        print("\n[DIAG] depth: (could not parse gate fields — check log format)")

    print("\nOutput:")
    print(f"  RECOMMENDATION: {agg['RECOMMENDATION']}")
    print(f"  SYMBOL_PRICE (matched symbol): {agg['SYMBOL_PRICE']}")

    print("\nHints:")
    ref = a.ref_edge_min
    below_frac: float | None = None
    if diag_g.diag_edges and ref is not None:
        below_frac = sum(1 for x in diag_g.diag_edges if x < float(ref)) / len(diag_g.diag_edges)

    re_pipe = diag_g.raw_edges_pipe
    raw_flat = bool(re_pipe) and max(abs(x) for x in re_pipe) < 1e-12
    voters_all_zero = (
        dl > 0
        and agg["RECOMMENDATION"] == 0
        and diag_g.voters_vals
        and len(diag_g.voters_vals) == dl
        and sum(diag_g.voters_vals) == 0
    )

    if agg["SYMBOL_TS"] == 0:
        print("  → Case A: No SYMBOL_TS in window — TZ/window mismatch, ingestion gap, or no ticks.")
    elif trans_win > 0 and d_nz == 0:
        print("  → Case A′: SYMBOL_TS + prices but ~flat deltas — stale snapshots / coarse sampling / flat tape.")
    elif voters_all_zero and raw_flat:
        print(
            "  → Case B+C: **EDGE_PIPE `raw_edge`≈0** on every sample **and** **voters=0** on every DIAG — "
            "no pipeline edge magnitude **and** reco_min_voters blocks emission. "
            "Inspect edge formation / calibration **and** voter bootstrap "
            "(e.g. MOMENTUM_VOTER_BOOTSTRAP, reco_min_voters, voter pool)."
        )
    elif voters_all_zero and not re_pipe:
        print(
            "  → Case C (voters): **voters=0 on every DIAG** — reco_min_voters / voter pool blocks emission "
            "(try MOMENTUM_VOTER_BOOTSTRAP=1 or relax voter rules). "
            "No `[EDGE_PIPE]` raw_edge samples in this window — cannot compare DIAG vs pipeline scale."
        )
    elif voters_all_zero:
        # Implies re_pipe non-empty and not raw_flat — pipeline has some |raw_edge| ≥ 1e-12.
        print(
            "  → Case C (voters): **voters=0 on every DIAG** — reco_min_voters / voter pool blocks emission "
            "(try MOMENTUM_VOTER_BOOTSTRAP=1 or relax voter rules). "
            "EDGE_PIPE `raw_edge` is non-negligible here — DIAG `edge=` may still read 0; compare columns above."
        )
    elif d_nz > 0 and agg["MOMENTUM_COND_1"] > 0 and dz == 0:
        print("  → Case B: Movement + momentum hints but DIAG printed edge≈0 — check EDGE_PIPE raw_edge vs edge_min.")
    elif (
        dz > 0
        and agg["RECOMMENDATION"] == 0
        and below_frac is not None
        and below_frac > 0.7
    ):
        print(
            "  → Case B→C boundary: most DIAG edge < --ref-edge-min — **calibration** (floor vs tape scale) "
            "before concluding pure voter/conf gating."
        )
    elif dz > 0 and agg["RECOMMENDATION"] == 0:
        print("  → Case C: DIAG activity but no reco — meta gates (FINAL/pass_reco/voters/feas) or emit path.")
    elif agg["RECOMMENDATION"] > 0:
        print("  → Case D: Recos in window — check UI/tail if behavior still looks wrong.")
    elif dl == 0 and mc == 0:
        print("  → TS present but no MOMENTUM/DIAG in attributed slice — engine quiet upstream.")
    else:
        print("  → Inspect DIAG depth, [--ref-edge-min], and FINAL/pass_* counts above.")

    return 0


if __name__ == "__main__":
    raise SystemExit(main())
