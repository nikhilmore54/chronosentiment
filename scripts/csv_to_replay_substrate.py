#!/usr/bin/env python3
import argparse
import csv
import json
import logging
from datetime import datetime, timezone
from pathlib import Path
from typing import Dict, Any, List

# Setup logging
logging.basicConfig(level=logging.INFO, format="%(levelname)s: %(message)s")

def process_csv(csv_path: Path, output_dir: Path, symbol_override: str = None) -> Path:
    """
    Process a single 1m OHLCV CSV file and output a Level 1 Synthetic Ecology Replay Substrate.
    """
    symbol = symbol_override or csv_path.stem.split("_")[0]
    output_filename = f"{symbol.lower()}_synthetic.jsonl"
    output_path = output_dir / output_filename
    
    ticks_generated = 0
    with open(csv_path, "r", encoding="utf-8") as fin, open(output_path, "w", encoding="utf-8") as fout:
        reader = csv.DictReader(fin)
        for row in reader:
            # Parse timestamp to ms
            dt = datetime.strptime(row["timestamp"], "%Y-%m-%dT%H:%M:%SZ").replace(tzinfo=timezone.utc)
            base_ts = int(dt.timestamp() * 1000)
            
            o = float(row["open"])
            h = float(row["high"])
            l = float(row["low"])
            c = float(row["close"])
            v = float(row["volume"])
            
            # Split volume over 4 ticks
            tick_v = v / 4.0
            
            # Path logic to preserve extremes
            if c >= o:
                # Up candle: Open -> Low -> High -> Close
                path = [o, l, h, c]
            else:
                # Down candle: Open -> High -> Low -> Close
                path = [o, h, l, c]
                
            for i, price in enumerate(path):
                tick = {
                    "symbol": symbol,
                    "timestamp": base_ts + (i * 15000),  # 0, 15s, 30s, 45s
                    "price": price,
                    "volume": tick_v,
                    "is_buyer_maker": False
                }
                fout.write(json.dumps(tick) + "\n")
                ticks_generated += 1
                
    logging.info(f"Processed {csv_path.name} -> {ticks_generated} ticks into {output_path.name}")
    return output_path

def create_manifest(output_dir: Path, source_dir: str):
    """
    Create Safeguard 1 Provenance Manifest.
    """
    manifest = {
        "generator": "synthetic_ohlc_expansion_v1",
        "source_dir": str(source_dir),
        "expansion_policy": "directional_ohlc_4tick",
        "certification_level": "L1_ECOLOGY_ONLY",
        "generated_at": datetime.now(timezone.utc).isoformat()
    }
    
    manifest_path = output_dir / "manifest.json"
    with open(manifest_path, "w", encoding="utf-8") as f:
        json.dump(manifest, f, indent=2)
    logging.info(f"Created provenance manifest at {manifest_path}")

def main():
    parser = argparse.ArgumentParser(description="Convert Canonical 1m CSVs into Synthetic Ecology Replay Substrates")
    parser.add_argument("--input-dir", required=True, help="Directory containing CSV files (e.g. historical_capture/batch_q1/2025-01-02/canonical)")
    parser.add_argument("--output-dir", required=True, help="Directory to save the .jsonl substrate files")
    parser.add_argument("--symbol", help="Override symbol name (default uses filename prefix)")
    
    args = parser.parse_args()
    
    input_dir = Path(args.input_dir)
    output_dir = Path(args.output_dir)
    
    if not input_dir.exists() or not input_dir.is_dir():
        logging.error(f"Input directory does not exist: {input_dir}")
        return
        
    output_dir.mkdir(parents=True, exist_ok=True)
    
    # Process all CSVs
    csv_files = list(input_dir.glob("*.csv"))
    if not csv_files:
        logging.warning(f"No CSV files found in {input_dir}")
        
    for csv_path in csv_files:
        process_csv(csv_path, output_dir, args.symbol)
        
    create_manifest(output_dir, str(input_dir))
    logging.info("Conversion complete.")

if __name__ == "__main__":
    main()
