#!/usr/bin/env python3
"""
ChronoSentiment — Offline History Replayer
Reads the synchronized JSON Lines archive and pipes it directly into the live_observatory
at maximum speed, effectively simulating days of market physics in seconds.
"""

import argparse
import sys
import subprocess
import time
from pathlib import Path

_ROOT = Path(__file__).resolve().parents[1]

def main():
    parser = argparse.ArgumentParser(description="Offline History Replayer")
    parser.add_argument("--file", type=str, default="archive/30_day_history.jsonl", help="Path to JSONL history file relative to root")
    parser.add_argument("--gen", type=str, default="1", help="Replay generation tag for output serialization")
    args = parser.parse_args()

    input_file = _ROOT / args.file

    if not input_file.exists():
        print(f"❌ Error: {input_file} not found.")
        print("Please run 'python3 scripts/download_30d_history.py' first.")
        sys.exit(1)

    print(f"🚀 Starting ultra-fast offline replay from {input_file.name} (Gen {args.gen})")
    
    # 1. Setup Engine Process
    engine_cmd = ["cargo", "run", "--release", "--example", "live_observatory"]
    
    import os
    env = os.environ.copy()
    env["SOURCE_TYPE"] = "REPLAY"
    env["REPLAY_GENERATION"] = args.gen
    
    # Start the Rust engine
    process = subprocess.Popen(
        engine_cmd,
        stdin=subprocess.PIPE,
        stdout=sys.stdout, # Let the engine output directly to terminal so you can see progress
        stderr=sys.stderr,
        text=True,
        bufsize=1,
        cwd=str(_ROOT),
        env=env
    )
    
    start_time = time.time()
    lines_sent = 0
    
    print("🏗️  Engine started. Dumping history into the pipe...")
    
    try:
        # 2. Read and Pipe Data
        with open(input_file, 'r') as f:
            for line in f:
                if line.strip():
                    process.stdin.write(line)
                    lines_sent += 1
                    
        # Close stdin to signal to the engine that the stream is over
        process.stdin.close()
        
        print(f"🏁 Sent {lines_sent} synchronized timesteps. Waiting for engine to finish processing...")
        process.wait()
        
        elapsed = time.time() - start_time
        print(f"✅ Replay complete in {elapsed:.2f} seconds!")
        
    except KeyboardInterrupt:
        print("\n🛑 Replay interrupted by user.")
        process.kill()
    except Exception as e:
        print(f"❌ Error during replay: {e}")
        process.kill()

if __name__ == "__main__":
    main()
