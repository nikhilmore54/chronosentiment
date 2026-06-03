#!/usr/bin/env python3
import json
import logging
import subprocess
from pathlib import Path
import random

logging.basicConfig(level=logging.INFO, format="%(levelname)s: %(message)s")

PROJECT_ROOT = Path(__file__).parent.parent.resolve()
CATALOG_FILE = PROJECT_ROOT / "phase4_replay_catalog.json"
BIN_PATH = PROJECT_ROOT / "target/release/trace_compiler"
SUBSTRATE_DIR = PROJECT_ROOT / "state_archive" / "phase6_substrates"
OUT_DIR = PROJECT_ROOT / "docs" / "certification" / "certified_artifacts"

def run_compiler(substrate_file: Path, strategy: str, latency_ms: int, miss_prob: float):
    cmd = [
        str(BIN_PATH),
        "--substrate-file", str(substrate_file),
        "--strategy", strategy,
        "--latency-ms", str(latency_ms),
        "--missed-fill-prob", str(miss_prob)
    ]
    res = subprocess.run(cmd, capture_output=True, text=True, check=True)
    return json.loads(res.stdout)

def main():
    if not CATALOG_FILE.exists():
        logging.error("Catalog not found.")
        return

    with open(CATALOG_FILE, "r") as f:
        data = json.load(f)

    valid_substrates = []
    for row in data:
        symbol = row.get("symbol")
        date_str = row.get("date")
        if symbol and date_str:
            sf = SUBSTRATE_DIR / f"{symbol}_{date_str}_synthetic.jsonl"
            if sf.exists():
                valid_substrates.append(sf)

    if not valid_substrates:
        logging.error("No valid substrates found.")
        return

    OUT_DIR.mkdir(parents=True, exist_ok=True)
    
    strategies = ["twap", "breakout", "momentum", "mean_reversion"]
    
    random.seed(42)
    selected = random.sample(valid_substrates, min(10, len(valid_substrates)))

    for i, sf in enumerate(selected):
        strat = random.choice(strategies)
        lat = random.choice([5, 50])
        miss = random.choice([0.0, 0.05])
        
        try:
            artifact = run_compiler(sf, strat, lat, miss)
            out_file = OUT_DIR / f"certified_artifact_{i+1:02d}_{strat}_{lat}ms.json"
            with open(out_file, "w") as f:
                json.dump(artifact, f, indent=2)
            logging.info(f"Generated {out_file.name}")
        except Exception as e:
            logging.error(f"Failed to generate artifact for {sf.name}: {e}")

if __name__ == "__main__":
    main()
