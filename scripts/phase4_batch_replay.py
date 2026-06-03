#!/usr/bin/env python3
import json
import logging
import os
import subprocess
from pathlib import Path
import sys
import shutil

sys.path.append(str(Path(__file__).parent.parent))

from scripts.csv_to_replay_substrate import process_csv
from scripts.session_metrics import compute_all

logging.basicConfig(level=logging.INFO, format="%(levelname)s: %(message)s")

PROJECT_ROOT = Path(__file__).parent.parent.resolve()
CORE_DIR = PROJECT_ROOT / "financial" / "strategies"
TRACE_REPLAY_BIN = PROJECT_ROOT / "target" / "release" / "financial_replay"

def ensure_trace_replay():
    if not TRACE_REPLAY_BIN.exists():
        logging.info("Building financial_replay...")
        subprocess.run(["cargo", "build", "--release", "--bin", "financial_replay"], cwd=CORE_DIR, check=True)

def get_all_csvs() -> list[Path]:
    csvs = []
    for batch in ["batch_q1", "batch_q2"]:
        base_dir = PROJECT_ROOT / "historical_capture" / batch
        if not base_dir.exists():
            continue
        for date_dir in sorted(base_dir.iterdir()):
            for symbol in ["NIFTY", "BANKNIFTY"]:
                csv_path = date_dir / "canonical" / f"{symbol}_1m.csv"
                if csv_path.exists():
                    csvs.append(csv_path)
    return csvs

def run_trace_replay(substrate_file: Path, namespace: str, cognition: str) -> dict:
    cmd = [
        str(TRACE_REPLAY_BIN),
        "--substrate", namespace,
        "--substrate-file", str(substrate_file),
        "--topology", "osc_50_1.0",
        "--cognition", cognition
    ]
    subprocess.run(cmd, cwd=PROJECT_ROOT, check=True, stdout=subprocess.DEVNULL, stderr=subprocess.DEVNULL)
    out_dir = PROJECT_ROOT / "artifacts" / namespace / "osc_50_1.0" / cognition
    summary_path = out_dir / "trace_summary.json"
    if not summary_path.exists():
        raise FileNotFoundError(f"Trace summary not found: {summary_path}")
    return json.loads(summary_path.read_text())

def main():
    ensure_trace_replay()
    
    csvs = get_all_csvs()
    logging.info(f"Found {len(csvs)} canonical CSVs to process.")
    
    substrate_dir = PROJECT_ROOT / "state_archive" / "phase4_substrates"
    if substrate_dir.exists():
        shutil.rmtree(substrate_dir)
    substrate_dir.mkdir(parents=True, exist_ok=True)
    
    catalog = []
    
    for csv_path in csvs:
        date_str = csv_path.parent.parent.name
        symbol = csv_path.name.split("_")[0]
        
        # 1. Compute original ecology metrics
        try:
            m = compute_all(csv_path, "dummy", None)
        except Exception as e:
            logging.error(f"Failed to compute metrics for {csv_path}: {e}")
            continue
            
        # 2. Generate Synthetic Substrate
        substrate_file = process_csv(csv_path, substrate_dir, symbol_override=symbol)
        namespace = f"phase4_{date_str}_{symbol}"
        
        # 3. Run Replays
        try:
            summary_rolling = run_trace_replay(substrate_file, namespace, "rolling_50")
            summary_event = run_trace_replay(substrate_file, namespace, "event_reset")
        except Exception as e:
            logging.error(f"Replay failed for {csv_path}: {e}")
            continue
            
        entry = {
            "date": date_str,
            "symbol": symbol,
            "open": m["open"],
            "high": m["high"],
            "low": m["low"],
            "close": m["close"],
            "session_range_pct": m["session_range_pct"],
            "net_return_pct": m["net_return_pct"],
            "trend_strength": m["trend_strength"],
            "realized_volatility": m["realized_volatility"],
            "persistence_rolling_50": summary_rolling.get("persistence"),
            "persistence_event_reset": summary_event.get("persistence"),
            "max_occupancy_rolling_50": summary_rolling.get("max"),
            "max_occupancy_event_reset": summary_event.get("max")
        }
        catalog.append(entry)
        logging.info(f"Processed {date_str} {symbol} | Trend: {m['trend_strength']:.2f} | Pers_R: {entry['persistence_rolling_50']} | Pers_E: {entry['persistence_event_reset']}")
        
    out_path = PROJECT_ROOT / "phase4_replay_catalog.json"
    with open(out_path, "w") as f:
        json.dump(catalog, f, indent=2)
        
    logging.info(f"Batch generation complete. Catalog saved to {out_path} ({len(catalog)} sessions).")

if __name__ == "__main__":
    main()
