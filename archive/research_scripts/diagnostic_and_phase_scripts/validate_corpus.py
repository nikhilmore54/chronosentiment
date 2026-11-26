# scripts/validate_corpus.py
"""Validate the integrity of a batch of Kite historical captures.
Checks performed per session (date/symbol):
- Expected candle count (approx 375 for a full NSE trading day)
- Duplicate timestamps
- SHA-256 lineage between raw, canonical, derived files and manifest entries
Outputs:
- per‑session JSON reports in the same directory (validation_report.json)
- an aggregated `validation_summary.json` at the root of the batch output.
"""
import json
import hashlib
from pathlib import Path
import csv
import sys
import argparse
import pandas as pd

EXPECTED_CANDLES = 375  # Approximate number of 1‑min candles for a normal session

def sha256_file(path: Path) -> str:
    return hashlib.sha256(path.read_bytes()).hexdigest()

def validate_session(session_path: Path, symbol: str) -> dict:
    report = {"symbol": symbol, "valid": True, "issues": []}
    # Paths
    raw_path = session_path / "raw" / f"{symbol}.json"
    canon_path = session_path / "canonical" / f"{symbol}_1m.csv"
    derived_path = session_path / "derived" / f"{symbol}_5m.csv"
    manifest_path = session_path / "capture_manifest.json"

    # Load manifest
    try:
        manifest = json.loads(manifest_path.read_text())
        manifest_entry = manifest["symbols"][symbol]
    except Exception as e:
        report["valid"] = False
        report["issues"].append(f"manifest load error: {e}")
        return report

    # ---- SHA‑256 checks ----
    def check_hash(file_path: Path, manifest_key: str):
        expected = manifest_entry.get(manifest_key)
        if not expected:
            return f"manifest missing {manifest_key}"
        try:
            actual = sha256_file(file_path)
        except Exception as e:
            return f"hash read error for {file_path.name}: {e}"
        if actual != expected:
            return f"hash mismatch for {file_path.name}: expected {expected[:8]}, got {actual[:8]}"
        return None

    for fp, key in [(raw_path, "raw_json_sha256"), (canon_path, "canonical_1m_sha256"), (derived_path, "derived_5m_sha256")]:
        err = check_hash(fp, key)
        if err:
            report["valid"] = False
            report["issues"].append(err)

    # ---- Candle count & duplicate timestamps ----
    try:
        df = pd.read_csv(canon_path)
        candle_count = len(df)
        if candle_count < EXPECTED_CANDLES * 0.9:  # allow small variance
            report["valid"] = False
            report["issues"].append(f"candle count low: {candle_count}")
        # duplicate timestamps
        dup = df.duplicated(subset=["timestamp"]).sum()
        if dup > 0:
            report["valid"] = False
            report["issues"].append(f"duplicate timestamps: {dup}")
        report["candle_count"] = candle_count
        report["duplicate_timestamps"] = int(dup)
    except Exception as e:
        report["valid"] = False
        report["issues"].append(f"CSV read error: {e}")

    return report

def main(batch_root: str):
    root = Path(batch_root)
    if not root.is_dir():
        sys.exit(f"[!] Batch root {batch_root} does not exist or is not a directory")

    all_reports = []
    for session_dir in sorted(root.iterdir()):
        if not session_dir.is_dir():
            continue
        date_str = session_dir.name
        for symbol_path in (session_dir / "canonical").glob("*_1m.csv"):
            symbol = symbol_path.stem.replace("_1m", "")
            rep = validate_session(session_dir, symbol)
            rep["date"] = date_str
            # write per‑session report
            report_path = session_dir / f"validation_report_{symbol}.json"
            report_path.write_text(json.dumps(rep, indent=2))
            all_reports.append(rep)

    # ---- Build summary ----
    total = len(all_reports)
    passed = sum(1 for r in all_reports if r["valid"])
    failed = total - passed
    invalid_details = []
    for r in all_reports:
        if not r["valid"]:
            invalid_details.append({"date": r["date"], "symbol": r["symbol"], "issues": r["issues"]})
    summary = {
        "total_sessions": total,
        "valid_sessions": passed,
        "invalid_sessions": failed,
        "invalid_details": invalid_details,
    }
    (root / "validation_summary.json").write_text(json.dumps(summary, indent=2))
    print(json.dumps(summary, indent=2))

if __name__ == "__main__":
    parser = argparse.ArgumentParser(description="Validate a batch of Kite historical captures")
    parser.add_argument("--input-dir", required=True, help="Root directory of batch capture (historical_capture/batch)")
    args = parser.parse_args()
    main(args.input_dir)
