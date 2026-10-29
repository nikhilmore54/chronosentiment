#!/usr/bin/env python3
"""
Deterministic (floor, K) grid on the same stdin tape as emit_validation_multisymbol_batches.py.

Floors = p92–p95 of |momentum_contribution| samples (live_engine semantics: delta_k30/price after warmup).

Selection (see user runbook): among rows satisfying band constraints, pick lowest bootstrap_rate,
then lowest floor value, then smallest K.

Aligned with .cursor/rules/chronosentiment-core.mdc — same tape → same percentiles → reproducible grid.
"""
from __future__ import annotations

import argparse
import os
import subprocess
import sys
from collections import deque
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent

# Import analyze without executing validate main
import importlib.util

_vb_path = ROOT / "scripts" / "validate_momentum_bootstrap_window.py"
_spec = importlib.util.spec_from_file_location("vb", _vb_path)
_vb = importlib.util.module_from_spec(_spec)
assert _spec.loader is not None
_spec.loader.exec_module(_vb)
analyze = _vb.analyze  # type: ignore[attr-defined]

SYMS = ("BTC-USD", "ETH-USD", "SOL-USD")
BASE = {"BTC-USD": 67240.0, "ETH-USD": 3500.0, "SOL-USD": 145.0}
T0 = 1714543200000


def percentile_nearest(values: list[float], p: float) -> float:
    if not values:
        return 0.0
    s = sorted(values)
    rank = ((max(0.0, min(100.0, p)) / 100.0) * (len(s) - 1))
    idx = int(round(rank))
    idx = max(0, min(idx, len(s) - 1))
    return s[idx]


def collect_abs_momentum_samples(steps: int) -> list[float]:
    """Match live_engine: per-symbol deque; after len>=301, mom = (close - close_31_back)/close."""
    hist: dict[str, list[float]] = {s: [] for s in SYMS}
    samples: list[float] = []
    for i in range(steps):
        for sym in SYMS:
            b = BASE[sym]
            drift = 1.0 + 1.2e-5 * (i - 100) + 3e-6 * ((i % 37) - 18)
            close = round(b * drift, 6)
            hist[sym].append(close)
            h = hist[sym]
            if len(h) > 30:
                price_now = h[-1]
                price_k30 = h[-31]
                delta_k30 = price_now - price_k30
                if abs(price_now) > 1e-12:
                    raw_m = delta_k30 / price_now
                    if len(h) >= 300:
                        samples.append(abs(raw_m))
    return samples


def find_engine() -> Path:
    for rel in (
        "target/release/examples/live_engine",
        "target/debug/examples/live_engine",
    ):
        p = ROOT / rel
        if p.is_file():
            return p
    raise SystemExit("Build live_engine first: cargo build --release --example live_engine")


def run_once(engine: Path, steps: int, floor: float, k: int) -> dict:
    env = os.environ.copy()
    env["VALIDATION_BATCH_STEPS"] = str(steps)
    env["MOMENTUM_VOTER_BOOTSTRAP"] = "1"
    env["MOMENTUM_BOOTSTRAP_FLOOR"] = f"{floor:.12g}"
    env["MOMENTUM_BOOTSTRAP_CONSISTENCY_K"] = str(k)

    emit = subprocess.Popen(
        [sys.executable, str(ROOT / "scripts" / "emit_validation_multisymbol_batches.py")],
        stdout=subprocess.PIPE,
        stderr=subprocess.DEVNULL,
        env=env,
    )
    proc = subprocess.run(
        [str(engine)],
        stdin=emit.stdout,
        stdout=subprocess.PIPE,
        stderr=subprocess.STDOUT,
        env=env,
        cwd=str(ROOT),
    )
    assert emit.stdout is not None
    emit.stdout.close()
    emit.wait()
    text = proc.stdout.decode("utf-8", errors="replace").splitlines()
    stats = analyze(text)
    stats["floor"] = floor
    stats["K"] = k
    stats["returncode"] = proc.returncode
    return stats


def main() -> None:
    ap = argparse.ArgumentParser()
    ap.add_argument("--steps", type=int, default=int(os.environ.get("VALIDATION_BATCH_STEPS", "420")))
    ap.add_argument("--dry-run", action="store_true", help="Print floors only; no engine runs.")
    ap.add_argument(
        "--max-run-cap",
        type=int,
        default=5,
        help="Burst guard for selection (default 5; use 3 for stricter runs).",
    )
    args = ap.parse_args()

    samples = collect_abs_momentum_samples(args.steps)
    if len(samples) < 10:
        raise SystemExit("Too few momentum samples; increase --steps")

    p92 = percentile_nearest(samples, 92)
    p93 = percentile_nearest(samples, 93)
    p94 = percentile_nearest(samples, 94)
    p95 = percentile_nearest(samples, 95)
    floors = [("p92", p92), ("p93", p93), ("p94", p94), ("p95", p95)]
    ks = [2, 3, 4]

    print("### Grid floors from tape (|momentum| after warmup)")
    print(f"  n_samples={len(samples)}  p92={p92:.9f}  p93={p93:.9f}  p94={p94:.9f}  p95={p95:.9f}")
    print(f"  K ∈ {ks}")
    print()

    if args.dry_run:
        return

    print("### Running engine grid (stderr progress)...")
    engine = find_engine()
    rows: list[dict] = []
    for name, fl in floors:
        for k in ks:
            sys.stderr.write(f"run floor={name} ({fl:.9f}) K={k} ...\n")
            sys.stderr.flush()
            st = run_once(engine, args.steps, fl, k)
            st["floor_label"] = name
            rows.append(st)

    # Table
    hdr = (
        "floor_lbl",
        "floor",
        "K",
        "bootstrap_rate",
        "reco_rate",
        "edge_nz_rate",
        "avg_edge_b",
        "max_edge_b",
        "max_run",
        "sign",
    )
    print("### All runs")
    print(
        "| floor | K | bootstrap_rate | reco_rate | edge_nz_rate | avg_edge | max_edge | max_run | sign_ok/check |"
    )
    print("|-------|---|----------------|-----------|--------------|----------|----------|---------|---------------|")
    for r in sorted(rows, key=lambda x: (x["floor_label"], x["K"])):
        sr = f"{r['sign_ok']}/{r['sign_checked']}"
        print(
            f"| {r['floor_label']} | {r['K']} | {r['bootstrap_rate']:.4f} | {r['reco_rate']:.4f} | "
            f"{r['edge_nonzero_rate_bootstrap_rows']:.4f} | {r['avg_edge_bootstrap_post']:.6f} | "
            f"{r['max_edge_bootstrap_post']:.6f} | {r['max_consecutive_bootstrap']} | {sr} |"
        )

    def satisfies(r: dict, max_run_cap: int) -> bool:
        br = r["bootstrap_rate"]
        rr = r["reco_rate"]
        if not (0.005 <= br <= 0.05):
            return False
        if not (0.002 <= rr <= 0.03):
            return False
        if r["max_consecutive_bootstrap"] > max_run_cap:
            return False
        if r["sign_checked"] != r["sign_ok"]:
            return False
        if r["sign_checked"] < 1:
            return False
        return True

    good = [r for r in rows if satisfies(r, args.max_run_cap)]
    print()
    print(
        f"### Selection (constraints: 0.5%≤bootstrap≤5%, 0.2%≤reco≤3%, max_run≤{args.max_run_cap}, sign_ok==sign_checked>0)"
    )
    if not good:
        print("  **No (floor, K) satisfied all constraints.**")
        print("  Try longer --steps, or adjust bands / tape drift.")
        return

    good.sort(key=lambda r: (r["bootstrap_rate"], r["floor"], r["K"]))
    best = good[0]
    print(f"  **Recommended:** floor={best['floor_label']} ({best['floor']:.9f}), K={best['K']}")
    print(
        f"    bootstrap_rate={best['bootstrap_rate']:.4f}  reco_rate={best['reco_rate']:.4f}  "
        f"max_run={best['max_consecutive_bootstrap']}"
    )


if __name__ == "__main__":
    main()
