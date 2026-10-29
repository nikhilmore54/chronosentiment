#!/usr/bin/env python3
"""
RP-609 Stage 5 Replication: Build 2026-09-28 Dataset.

Extracts the 28 September 2026 NSE Ledger A baseline trades and decision cards,
reconstructing in-hold and fixed-horizon post-entry observation clocks.

Governance:
- RP-605 stays the frozen control. RP-608 stays shadow-only.
- PFT, SPL, VAT, and PPR are unchanged.
- Pure historical replication: frozen definitions applied to 28 September.
"""

from __future__ import annotations

import csv
import hashlib
import json
from collections import Counter, defaultdict
from datetime import datetime, timedelta, timezone
from pathlib import Path

WORKSPACE = Path(__file__).resolve().parent.parent
LOG_DIR = WORKSPACE / "live_capture" / "shadow_validation"
JOURNAL_PATH = LOG_DIR / "rp605_nse_baseline_journal.jsonl"
CARDS_PATH = LOG_DIR / "rp605_nse_decision_cards_log.jsonl"
OUT_CSV = LOG_DIR / "rp609_nse_20260928_replication_dataset.csv"
OUT_AUDIT_JSON = LOG_DIR / "rp609_nse_20260928_replication_manifest.json"

TARGET_DATE = "2026-09-28"
HORIZONS_MIN = (5, 10, 15)
POST_HORIZON_MIN = 60


def parse_ts(val: str) -> datetime:
    val = val.strip().replace("Z", "+00:00")
    return datetime.fromisoformat(val)


def signed_pnl(side: str, entry_px: float, current_px: float, qty: float) -> float:
    if side == "LONG":
        return (current_px - entry_px) * qty
    else:
        return (entry_px - current_px) * qty


def sha256_file(path: Path) -> str:
    digest = hashlib.sha256()
    with path.open("rb") as f:
        for chunk in iter(lambda: f.read(1024 * 1024), b""):
            digest.update(chunk)
    return digest.hexdigest()


def main():
    print("Building RP-609 2026-09-28 Replication Dataset...")

    # Load 28 September journal trades
    trades_raw = []
    with JOURNAL_PATH.open(encoding="utf-8") as f:
        for line in f:
            if not line.strip():
                continue
            row = json.loads(line)
            ts = row.get("entry_timestamp") or row.get("entry_time") or ""
            if ts.startswith(TARGET_DATE):
                trades_raw.append(row)

    print(f"Extracted {len(trades_raw)} trades for {TARGET_DATE} from baseline journal.")

    # Load 28 September decision cards
    cards_by_sym = defaultdict(list)
    card_count = 0
    with CARDS_PATH.open(encoding="utf-8") as f:
        for line in f:
            if not line.strip():
                continue
            c = json.loads(line)
            ts = c.get("timestamp_utc") or ""
            if ts.startswith(TARGET_DATE):
                dt = parse_ts(ts)
                cards_by_sym[c["instrument"]].append({
                    "_dt": dt,
                    "price": float(c["current_price"]),
                    "decision_state": c.get("decision_state"),
                })
                card_count += 1

    for sym in cards_by_sym:
        cards_by_sym[sym].sort(key=lambda x: x["_dt"])

    print(f"Loaded {card_count} decision cards across {len(cards_by_sym)} symbols for {TARGET_DATE}.")

    dataset_rows = []
    zero_mfe_count = 0
    pos_mfe_count = 0

    for t in trades_raw:
        trade_id = t["trade_id"]
        sym = t["symbol"]
        side = t["side"]
        t_entry = parse_ts(t["entry_timestamp"])
        t_exit = parse_ts(t["exit_timestamp"])
        entry_px = float(t["entry_price"])
        exit_px = float(t["exit_price"])
        qty = float(t["quantity"])
        realized_pnl = float(t.get("pnl") if t.get("pnl") is not None else (signed_pnl(side, entry_px, exit_px, qty)))

        sym_cards = cards_by_sym.get(sym, [])

        # In-hold cards
        in_hold = [c for c in sym_cards if t_entry <= c["_dt"] <= t_exit]

        # Calculate in-hold MFE
        mfe_val = 0.0
        for c in in_hold:
            pnl = signed_pnl(side, entry_px, c["price"], qty)
            if pnl > mfe_val:
                mfe_val = pnl

        is_zero_mfe = 1 if mfe_val <= 0.05 else 0
        if is_zero_mfe:
            zero_mfe_count += 1
        else:
            pos_mfe_count += 1

        row = {
            "trade_id": trade_id,
            "symbol": sym,
            "side": side,
            "t_entry_utc": t_entry.isoformat(),
            "t_exit_utc": t_exit.isoformat(),
            "feature_entry_price": entry_px,
            "feature_quantity": qty,
            "feature_f15_at_entry": t.get("f15_at_entry", 0.0),
            "feature_is_fresh": t.get("is_fresh", True),
            "label_realized_pnl": round(realized_pnl, 4),
            "label_mfe_pnl": round(mfe_val, 4),
            "label_zero_mfe": is_zero_mfe,
        }

        # Observation and outcome clocks
        for h in HORIZONS_MIN:
            t_horiz = t_entry + timedelta(minutes=h)
            t_post_end = t_horiz + timedelta(minutes=POST_HORIZON_MIN)

            w_cards = [c for c in sym_cards if t_entry <= c["_dt"] <= t_horiz]
            later_cards = [c for c in sym_cards if t_horiz < c["_dt"] <= t_post_end]

            w_mfe = max([signed_pnl(side, entry_px, c["price"], qty) for c in w_cards], default=None)
            later_mfe = max([signed_pnl(side, entry_px, c["price"], qty) for c in later_cards], default=None)

            row[f"obs_{h}m_mfe_pnl"] = round(w_mfe, 4) if w_mfe is not None else ""
            row[f"obs_{h}m_favourable_seen"] = int(w_mfe > 0.05) if w_mfe is not None else ""
            row[f"outcome_{h}m_mfe_next_60m"] = round(later_mfe, 4) if later_mfe is not None else ""
            row[f"outcome_{h}m_later_favourable"] = int(later_mfe > 0.05) if later_mfe is not None else ""

        dataset_rows.append(row)

    # Sort deterministically
    dataset_rows.sort(key=lambda r: (r["t_entry_utc"], r["trade_id"]))

    fieldnames = list(dataset_rows[0].keys())
    with OUT_CSV.open("w", newline="", encoding="utf-8") as f:
        writer = csv.DictWriter(f, fieldnames=fieldnames)
        writer.writeheader()
        writer.writerows(dataset_rows)

    sha256 = sha256_file(OUT_CSV)
    print(f"Generated {OUT_CSV} with {len(dataset_rows)} rows. SHA256: {sha256}")
    print(f"  Zero-MFE count: {zero_mfe_count} ({zero_mfe_count/len(dataset_rows)*100:.2f}%)")
    print(f"  Positive-MFE count: {pos_mfe_count} ({pos_mfe_count/len(dataset_rows)*100:.2f}%)")

    manifest = {
        "session_date": TARGET_DATE,
        "csv_path": str(OUT_CSV),
        "csv_sha256": sha256,
        "rows": len(dataset_rows),
        "symbols": len({r["symbol"] for r in dataset_rows}),
        "zero_mfe_count": zero_mfe_count,
        "positive_mfe_count": pos_mfe_count,
        "zero_mfe_rate": round(zero_mfe_count / len(dataset_rows), 4),
        "sum_realized_pnl": round(sum(r["label_realized_pnl"] for r in dataset_rows), 2),
    }

    with OUT_AUDIT_JSON.open("w", encoding="utf-8") as f:
        json.dump(manifest, f, indent=2)
    print(f"Wrote manifest to {OUT_AUDIT_JSON}")


if __name__ == "__main__":
    main()
