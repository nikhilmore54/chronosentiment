#!/usr/bin/env python3
"""
scripts/build_rp615_predictive_dataset.py

RP-615-003: Builds the pre-trade causal feature and multi-target outcome dataset
for all 7,264 canonical baseline opportunities across 99 symbols and 4 sessions.

Causal Boundary:
  - All pre-trade features strictly evaluated at t <= t_entry - 15s.
  - Zero post-entry data leakage.

Outputs:
  - live_capture/shadow_validation/rp615_003_predictive_dataset.csv
"""

from __future__ import annotations

import csv
import hashlib
import json
import math
import sys
import time
from collections import defaultdict
from datetime import datetime, timezone, timedelta
from pathlib import Path
from typing import Any, Dict, List

import numpy as np

WORKSPACE = Path(__file__).resolve().parent.parent
sys.path.insert(0, str(WORKSPACE))

LOG_DIR = WORKSPACE / "live_capture" / "shadow_validation"
OPP_DATASET = LOG_DIR / "rp613_opportunity_forensic_dataset.csv"
FEATURE_DATASET = LOG_DIR / "rp615_001_feature_dataset.csv"
LEDGER_CSV = LOG_DIR / "rp615_001_counterfactual_ledger.csv"
OUT_CSV = LOG_DIR / "rp615_003_predictive_dataset.csv"

IST = timezone(timedelta(hours=5, minutes=30))


def sha256_file(path: Path) -> str:
    digest = hashlib.sha256()
    with path.open("rb") as f:
        for chunk in iter(lambda: f.read(1024 * 1024), b""):
            digest.update(chunk)
    return digest.hexdigest()


def parse_ts(ts_str: str) -> datetime:
    ts_str = ts_str.strip()
    if ts_str.endswith("Z"):
        ts_str = ts_str[:-1] + "+00:00"
    return datetime.fromisoformat(ts_str)


def main():
    t_start = time.time()
    print("=" * 80)
    print("RP-615-003: BUILDING PREDICTIVE FEATURE & OUTCOME DATASET")
    print("Governance: Research-Only (0% Live Authority) | RP-605 Control (100% Live)")
    print("=" * 80)

    # 1. Load Baseline Opportunities
    print("\n[Step 1] Loading 7,264 canonical baseline opportunities...")
    opps = {}
    with OPP_DATASET.open(encoding="utf-8") as f:
        reader = csv.DictReader(f)
        for r in reader:
            t_id = r["trade_id"]
            s_date = r["session_date"]
            key = (s_date, t_id)
            opps[key] = {
                "trade_id": t_id,
                "session_date": s_date,
                "symbol": r["symbol"],
                "side": r["direction"],
                "t_entry_utc": r["t_entry_utc"],
                "t_exit_utc": r["t_exit_utc"],
                "entry_price": float(r["entry_price"]),
                "exit_price": float(r["exit_price"]),
                "quantity": float(r["quantity"]),
                "holding_sec": float(r.get("holding_duration_sec", 64.0)),
                "mfe_pct": float(r.get("mfe_pct", 0.0)),
                "mae_pct": float(r.get("mae_pct", 0.0)),
                "is_zero_mfe": int(r.get("is_zero_mfe", 0)),
                "exit_reason": r.get("exit_reason", "ENGINE_EXIT"),
                "feat_vol_accel": float(r.get("feat_vol_accel", 1.0)) if r.get("feat_vol_accel") else 1.0,
                "feat_ret_accel": float(r.get("feat_ret_accel", 0.0)) if r.get("feat_ret_accel") else 0.0,
                "feat_dist_extreme_pct": float(r.get("feat_dist_extreme_pct", 0.0)) if r.get("feat_dist_extreme_pct") else 0.0,
            }

    # 2. Merge Point-in-Time Features from RP-615-001
    print("\n[Step 2] Merging point-in-time causal features (F01 - F09)...")
    with FEATURE_DATASET.open(encoding="utf-8") as f:
        reader = csv.DictReader(f)
        for r in reader:
            key = (r["session_date"], r["trade_id"])
            if key in opps:
                opps[key]["status"] = r["status"]
                opps[key]["ineligibility_reason"] = r.get("ineligibility_reason", "")
                opps[key]["F01_ret_15m"] = float(r["F01_ret_15m"]) if r["F01_ret_15m"] else 0.0
                opps[key]["F02_ema_align"] = float(r["F02_ema_align"]) if r["F02_ema_align"] else 0.0
                opps[key]["F03_ker_15m"] = float(r["F03_ker_15m"]) if r["F03_ker_15m"] else 0.0
                opps[key]["F04_rvol_15m"] = float(r["F04_rvol_15m"]) if r["F04_rvol_15m"] else 1.0
                opps[key]["F05_atr_norm"] = float(r["F05_atr_norm"]) if r["F05_atr_norm"] else 0.0
                opps[key]["F06_sigma_60"] = float(r["F06_sigma_60"]) if r["F06_sigma_60"] else 0.0
                opps[key]["F07_cs_raw"] = float(r["F07_cs_raw"]) if r["F07_cs_raw"] else 0.0
                opps[key]["F07_cs_aligned"] = float(r["F07_cs_aligned"]) if r["F07_cs_aligned"] else 0.0
                opps[key]["F08_wick_asym"] = float(r["F08_wick_asym"]) if r["F08_wick_asym"] else 0.0
                opps[key]["F09_n_trig"] = int(r["F09_n_trig"]) if r["F09_n_trig"] else 0

    # 3. Merge Certified Financial Outcomes from Ledger
    print("\n[Step 3] Merging certified Formula A financial accounting...")
    with LEDGER_CSV.open(encoding="utf-8") as f:
        reader = csv.DictReader(f)
        for r in reader:
            key = (r["session_date"], r["trade_id"])
            if key in opps:
                opps[key]["gross_pnl"] = float(r["gross_pnl"])
                opps[key]["friction"] = float(r["friction"])
                opps[key]["net_pnl"] = float(r["net_pnl"])
                opps[key]["is_win"] = int(r["is_win"]) if "is_win" in r else (1 if float(r["net_pnl"]) > 0 else 0)

    # 4. Compute Inter-Trade Cooldown and Intraday Time Features
    print("\n[Step 4] Computing instrument cooldown and temporal features...")
    trade_list = list(opps.values())
    sym_session = defaultdict(list)
    for t in trade_list:
        sym_session[(t["session_date"], t["symbol"])].append(t)

    for (s_date, sym), s_list in sym_session.items():
        s_list_sorted = sorted(s_list, key=lambda x: parse_ts(x["t_entry_utc"]))
        last_exit_time = None
        for t in s_list_sorted:
            entry_dt = parse_ts(t["t_entry_utc"])
            if last_exit_time is None:
                delta_mins = 999.0
            else:
                delta_mins = (entry_dt - last_exit_time).total_seconds() / 60.0
            t["cooldown_mins"] = delta_mins

            entry_ist = entry_dt.astimezone(IST)
            m_since_open = (entry_ist.hour - 9) * 60 + entry_ist.minute - 15
            t["time_of_day_pct"] = max(0.0, min(1.0, m_since_open / 375.0))

            exit_dt = parse_ts(t["t_exit_utc"])
            if last_exit_time is None or exit_dt > last_exit_time:
                last_exit_time = exit_dt

    # 5. Build Final Dataset Rows
    print("\n[Step 5] Assembling final predictive dataset rows...")
    out_rows = []
    for t in trade_list:
        notional = t["entry_price"] * t["quantity"]
        net_ret_bps = (t["net_pnl"] / notional * 10000.0) if notional > 0 else 0.0
        gross_ret_bps = (t["gross_pnl"] / notional * 10000.0) if notional > 0 else 0.0

        out_rows.append({
            "trade_id": t["trade_id"],
            "session_date": t["session_date"],
            "symbol": t["symbol"],
            "side": t["side"],
            "side_num": 1 if t["side"] == "LONG" else -1,
            "t_entry_utc": t["t_entry_utc"],
            "t_exit_utc": t["t_exit_utc"],
            "status": t.get("status", "ELIGIBLE"),
            "ineligibility_reason": t.get("ineligibility_reason", ""),
            # Features (F01 - F15)
            "F01_ret_15m": f"{t.get('F01_ret_15m', 0.0):.6f}",
            "F02_ema_align": f"{t.get('F02_ema_align', 0.0):.6f}",
            "F03_ker_15m": f"{t.get('F03_ker_15m', 0.0):.6f}",
            "F04_rvol_15m": f"{t.get('F04_rvol_15m', 1.0):.6f}",
            "F05_atr_norm": f"{t.get('F05_atr_norm', 0.0):.6f}",
            "F06_sigma_60": f"{t.get('F06_sigma_60', 0.0):.6f}",
            "F07_cs_raw": f"{t.get('F07_cs_raw', 0.0):.6f}",
            "F07_cs_aligned": f"{t.get('F07_cs_aligned', 0.0):.6f}",
            "F08_wick_asym": f"{t.get('F08_wick_asym', 0.0):.6f}",
            "F09_n_trig": t.get("F09_n_trig", 0),
            "F10_cooldown_mins": f"{t['cooldown_mins']:.2f}",
            "F11_time_of_day_pct": f"{t['time_of_day_pct']:.4f}",
            "F12_vol_accel": f"{t['feat_vol_accel']:.4f}",
            "F13_ret_accel": f"{t['feat_ret_accel']:.4f}",
            "F14_dist_extreme_pct": f"{t['feat_dist_extreme_pct']:.4f}",
            # Targets
            "target_is_win": t["is_win"],
            "target_net_pnl": f"{t['net_pnl']:.2f}",
            "target_gross_pnl": f"{t['gross_pnl']:.2f}",
            "target_friction": f"{t['friction']:.2f}",
            "target_net_ret_bps": f"{net_ret_bps:.4f}",
            "target_gross_ret_bps": f"{gross_ret_bps:.4f}",
            "target_mfe_pct": f"{t['mfe_pct']:.6f}",
            "target_mae_pct": f"{t['mae_pct']:.6f}",
            "target_holding_sec": f"{t['holding_sec']:.1f}",
            "target_is_zero_mfe": t["is_zero_mfe"],
            "target_exit_reason": t["exit_reason"],
        })

    with OUT_CSV.open("w", encoding="utf-8", newline="") as f:
        writer = csv.DictWriter(f, fieldnames=list(out_rows[0].keys()))
        writer.writeheader()
        writer.writerows(out_rows)

    t_elapsed = time.time() - t_start
    print(f"\n[Completed] Predictive dataset written ({len(out_rows)} rows) in {t_elapsed:.2f}s.")
    print(f"  Artifact: {OUT_CSV}")
    print(f"  SHA-256:  {sha256_file(OUT_CSV)}")


if __name__ == "__main__":
    main()
