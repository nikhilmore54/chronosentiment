#!/usr/bin/env python3
"""
One-pass metrics from live_engine logs when MOMENTUM_VOTER_BOOTSTRAP is enabled.
Deterministic parsing only (.cursor/rules/chronosentiment-core.mdc).
"""
from __future__ import annotations

import argparse
import re
import statistics
from pathlib import Path

BOOT_RE = re.compile(
    r"\[MOMENTUM_VOTER_BOOTSTRAP\]\s+sym=([^\s]+)\s+floor=([-+]?\d*\.?\d+)\s+k=(\d+)\s+"
    r"mom_abs=([-+]?\d*\.?\d+)\s+voters_raw=(\d+)\s+synthetic_reco=(\d+)"
)
MCHECK_RE = re.compile(r"\[MOMENTUM_CHECK\]\s+sym=([^\s]+)")
EDGE_COMP_RE = re.compile(
    r"\[EDGE_COMPONENTS\]\s+sym=([^\s]+)\s+"
    r"raw_momentum=([-+]?\d*\.?\d+)\s+norm_momentum=([-+]?\d*\.?\d+)\s+momentum_weight=([-+]?\d*\.?\d+)\s+"
    r"momentum_contribution=([-+]?\d*\.?\d+)\s+composite_contribution=([-+]?\d*\.?\d+)\s+score_contribution=([-+]?\d*\.?\d+)\s+"
    r"pre_gate_edge=([-+]?\d*\.?\d+)\s+post_gate_edge=([-+]?\d*\.?\d+)"
)
REC_RE = re.compile(
    r"\[RECOMMENDATION\]\s+rec_id=\d+\s+sym=([^\s]+)\s+dir=([A-Za-z_]+)"
)


def iter_lines(paths: list[Path]):
    for p in sorted(paths, key=lambda x: (x.stat().st_mtime_ns, str(x))):
        with p.open("r", encoding="utf-8", errors="replace") as f:
            for line in f:
                yield line


def analyze(lines: list[str]) -> dict:
    n_batches = sum(1 for ln in lines if "[SYMBOL_TS]" in ln)
    n_mcheck = sum(1 for ln in lines if "[MOMENTUM_CHECK]" in ln)
    # Prefer batch×symbol updates when Symbol_TS present (one stdin line = one timestep across syms).
    symbols_per_batch = 3
    total_cycles = n_batches * symbols_per_batch if n_batches else max(1, n_mcheck)
    boot_lines: list[re.Match[str]] = []
    last_ec: dict[str, tuple[float, float, float]] = {}
    edges_boot_pre: list[float] = []
    edges_boot_post: list[float] = []
    mom_at_boot: list[float] = []
    reco_lines: list[tuple[str, str]] = []

    for ln in lines:
        m = EDGE_COMP_RE.search(ln)
        if m:
            sym = m.group(1)
            mom_c = float(m.group(5))
            pre = float(m.group(8))
            post = float(m.group(9))
            last_ec[sym] = (mom_c, pre, post)

        m = BOOT_RE.search(ln)
        if m:
            boot_lines.append(m)
            sym = m.group(1)
            mom_abs = float(m.group(4))
            mom_at_boot.append(mom_abs)
            tup = last_ec.get(sym)
            if tup:
                _, pre, post = tup
                edges_boot_pre.append(pre)
                edges_boot_post.append(post)

        m = REC_RE.search(ln)
        if m:
            reco_lines.append((m.group(1), m.group(2)))

    sym_order = [m.group(1) for m in boot_lines]
    max_run = 0
    cur_run = 0
    prev_s: str | None = None
    for s in sym_order:
        if s == prev_s:
            cur_run += 1
        else:
            cur_run = 1
            prev_s = s
        max_run = max(max_run, cur_run)

    n_boot = len(boot_lines)
    n_reco = len(reco_lines)
    denom = total_cycles if total_cycles > 0 else 1

    edge_nonzero = sum(1 for x in edges_boot_post if x > 1e-15)
    edge_rate = edge_nonzero / max(1, len(edges_boot_post))

    # Sign consistency: compare last EDGE_COMPONENTS mom sign before each bootstrap to reco side (best-effort).
    sign_ok = 0
    sign_checked = 0
    # Replay lightweight: store bootstrap events with mom sign from boot line (mom_abs only — recover sign from history)
    # Re-scan pairing bootstrap with following RECOMMENDATION same symbol (next occurrence).
    sym_boot_order = [m.group(1) for m in boot_lines]
    # Pair each bootstrap to next REC line for same sym (greedy)
    rec_idx = 0
    i_b = 0
    while i_b < len(boot_lines) and rec_idx < len(reco_lines):
        bs = boot_lines[i_b].group(1)
        while rec_idx < len(reco_lines) and reco_lines[rec_idx][0] != bs:
            rec_idx += 1
        if rec_idx >= len(reco_lines):
            break
        # Need momentum sign: from edges list index — approximate from EDGE snapshot before boot
        mom_full = last_ec.get(bs)
        if mom_full:
            mom_c, _, _ = mom_full
            side = reco_lines[rec_idx][1]
            want_buy = mom_c > 0
            is_buy = "BUY" in side.upper()
            sign_checked += 1
            if want_buy == is_buy:
                sign_ok += 1
        i_b += 1
        rec_idx += 1

    return {
        "input_batches": n_batches,
        "symbol_timesteps_est": total_cycles,
        "bootstrap_count": n_boot,
        "bootstrap_rate": n_boot / denom,
        "reco_count": n_reco,
        "reco_rate": n_reco / denom,
        "edge_nonzero_on_bootstrap_cycles": edge_nonzero,
        "edge_nonzero_rate_bootstrap_rows": edge_rate,
        "avg_edge_bootstrap_post": (
            statistics.mean(edges_boot_post) if edges_boot_post else 0.0
        ),
        "max_edge_bootstrap_post": max(edges_boot_post) if edges_boot_post else 0.0,
        "max_consecutive_bootstrap": max_run,
        "bootstrap_events_by_symbol": len(set(sym_order)) if sym_order else 0,
        "sign_ok": sign_ok,
        "sign_checked": sign_checked,
        "avg_mom_abs_bootstrap": statistics.mean(mom_at_boot) if mom_at_boot else 0.0,
    }


def main() -> None:
    ap = argparse.ArgumentParser()
    ap.add_argument(
        "paths",
        nargs="*",
        type=Path,
        help="Log files (merged shards). If empty, read stdin.",
    )
    args = ap.parse_args()
    if args.paths:
        lines = list(iter_lines(args.paths))
    else:
        lines = sys.stdin.readlines()

    stats = analyze(lines)
    print("### Momentum voter bootstrap — validation pack")
    for k, v in stats.items():
        if isinstance(v, float):
            print(f"  {k}: {v:.6f}")
        else:
            print(f"  {k}: {v}")

    # Pass/fail hints (objective thresholds from user note)
    br = stats["bootstrap_rate"]
    rr = stats["reco_rate"]
    print()
    print("### Quick verdict")
    if stats["bootstrap_count"] == 0:
        print("  Bootstrap: **none fired** — floor/K too strict or tape too flat.")
    elif br < 0.002:
        print(f"  Bootstrap rate {br*100:.3f}% — **quiet** (<0.2% band)")
    elif br > 0.10:
        print(f"  Bootstrap rate {br*100:.2f}% — **noisy** (>10%); raise floor or K")
    else:
        print(f"  Bootstrap rate {br*100:.2f}% — **in target band** (0.5–5% heuristic)")

    if stats["reco_count"] == 0 and stats["bootstrap_count"] > 0:
        print("  Recommendations: **none** while bootstrap fired — check gates / p90 / synthetic path.")
    elif 0.002 <= rr <= 0.03:
        print(f"  Reco rate {rr*100:.2f}% — within ~0.2–3% heuristic.")

    if stats["max_consecutive_bootstrap"] > 4:
        print(
            f"  Burst check: max consecutive bootstrap runs = {stats['max_consecutive_bootstrap']} (watch >3)"
        )


if __name__ == "__main__":
    main()
