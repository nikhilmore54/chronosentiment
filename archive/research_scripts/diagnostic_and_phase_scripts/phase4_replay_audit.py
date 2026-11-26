#!/usr/bin/env python3
import json
import logging
import os
import shutil
import subprocess
from pathlib import Path
import sys
import hashlib

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

def hash_file(filepath: Path) -> str:
    h = hashlib.sha256()
    h.update(filepath.read_bytes())
    return h.hexdigest()

def get_q1_nifty_csvs() -> list[Path]:
    base_dir = PROJECT_ROOT / "historical_capture" / "batch_q1"
    csvs = []
    if not base_dir.exists():
        return csvs
    for date_dir in sorted(base_dir.iterdir()):
        csv_path = date_dir / "canonical" / "NIFTY_1m.csv"
        if csv_path.exists():
            csvs.append(csv_path)
    return csvs

def select_audit_sessions() -> tuple[list[Path], list[Path]]:
    csvs = get_q1_nifty_csvs()
    session_data = []
    for c in csvs:
        try:
            m = compute_all(c, "dummy", None)
            session_data.append((c, m["trend_strength"], m["session_range_pct"]))
        except Exception as e:
            logging.error(f"Error computing metrics for {c}: {e}")
            pass
            
    # Sort by trend strength
    session_data.sort(key=lambda x: x[1])
    
    # Bottom 5 = Ecology B proxy (Transient, low trend)
    # Top 5 = Ecology A proxy (Persistent, high trend)
    eco_b = [x[0] for x in session_data[:5]]
    eco_a = [x[0] for x in session_data[-5:]]
    return eco_a, eco_b

def run_trace_replay(substrate_file: Path, namespace: str) -> dict:
    cmd = [
        str(TRACE_REPLAY_BIN),
        "--substrate", namespace,
        "--substrate-file", str(substrate_file),
        "--topology", "osc_50_1.0",
        "--cognition", "rolling_50"
    ]
    # trace_replay runs in CORE_DIR? The demo script runs it from PROJECT_ROOT.
    subprocess.run(cmd, cwd=PROJECT_ROOT, check=True, stdout=subprocess.DEVNULL, stderr=subprocess.DEVNULL)
    
    # Expected output
    # By default trace_replay outputs to artifacts/<namespace>/<topology>/<cognition>/trace_summary.json
    out_dir = PROJECT_ROOT / "artifacts" / namespace / "osc_50_1.0" / "rolling_50"
    summary_path = out_dir / "trace_summary.json"
    
    if not summary_path.exists():
        raise FileNotFoundError(f"Trace summary not found: {summary_path}")
        
    return json.loads(summary_path.read_text()), summary_path

def main():
    ensure_trace_replay()
    
    logging.info("Selecting 5 Ecology A and 5 Ecology B sessions based on trend proxy...")
    eco_a, eco_b = select_audit_sessions()
    
    if len(eco_a) < 5 or len(eco_b) < 5:
        logging.error("Not enough sessions found.")
        sys.exit(1)
        
    audit_tmp = PROJECT_ROOT / "tmp_audit_substrates"
    audit_tmp.mkdir(exist_ok=True)
    
    results_a = []
    results_b = []
    
    def process_group(sessions, name):
        persistences = []
        for csv_path in sessions:
            date_str = csv_path.parent.parent.name
            logging.info(f"Processing {name} session: {date_str}")
            
            # 1. Generate Substrate
            substrate_file = process_csv(csv_path, audit_tmp, symbol_override="NIFTY")
            namespace = f"audit_{name}_{date_str}"
            
            # 2. Run Replay (Pass 1)
            try:
                summary1, summary_path = run_trace_replay(substrate_file, namespace)
                h1 = hash_file(summary_path)
            except Exception as e:
                logging.error(f"Replay failed for {date_str}: {e}")
                continue
                
            # 3. Run Replay (Pass 2) to verify determinism
            summary_path.unlink()
            summary2, summary_path = run_trace_replay(substrate_file, namespace)
            h2 = hash_file(summary_path)
            
            if h1 != h2:
                logging.error(f"DETERMINISM FAIL for {date_str}. Hashes: {h1} != {h2}")
            else:
                logging.info(f"  Determinism OK ({h1})")
                
            p = summary1.get("persistence")
            logging.info(f"  Persistence: {p}")
            persistences.append(p)
            
        return persistences

    logging.info("\n--- Evaluating Ecology A (High Trend / Persistent) ---")
    results_a = process_group(eco_a, "EcoA")
    
    logging.info("\n--- Evaluating Ecology B (Low Trend / Transient) ---")
    results_b = process_group(eco_b, "EcoB")
    
    logging.info("\n--- AUDIT RESULTS ---")
    if results_a and results_b:
        mean_a = sum(results_a) / len(results_a)
        mean_b = sum(results_b) / len(results_b)
        
        logging.info(f"Ecology A mean persistence: {mean_a:.4f}")
        logging.info(f"Ecology B mean persistence: {mean_b:.4f}")
        
        if abs(mean_a - mean_b) < 1e-6 and max(results_a) - min(results_a) < 1e-6:
            logging.error("❌ CRITICAL: Replay engine outputs ZERO variation across sessions.")
            logging.error("Phase 4 synthetic substrates do not trigger sensitive responses in the replay engine.")
            sys.exit(1)
        else:
            logging.info("✅ SUCCESS: Replay engine is sensitive to synthetic substrate variations.")
            sys.exit(0)

if __name__ == "__main__":
    main()
