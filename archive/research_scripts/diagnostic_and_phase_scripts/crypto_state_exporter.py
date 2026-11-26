import json
import sys
import os
from pathlib import Path

# Adjust path so we can import apps
sys.path.append(os.path.dirname(os.path.dirname(os.path.abspath(__file__))))

from apps.crypto_24h.observation_store import ObservationStore
from apps.crypto_24h.rolling_state_engine import RollingStateEngine

def main():
    workspace = Path("/Users/nikhil/ChronoSentiment_MEGA_FINAL")
    dataset = workspace / "datasets/e9/crypto/high_vol_BTCUSDT.json"
    output_path = workspace / "datasets/crypto_integration_trace_v0.jsonl"
    
    if not dataset.exists():
        print(f"Dataset not found: {dataset}")
        sys.exit(1)
        
    obs_store = ObservationStore()
    state_engine = RollingStateEngine()
    
    print(f"Loading {dataset}...")
    with open(dataset, 'r') as f:
        bars = json.load(f)
        
    print("Generating v0.2 state trace...")
    valid_states = 0
    
    with open(output_path, 'w') as out_f:
        for bar in bars:
            timestamp = bar["timestamp"] * 1000
            added = obs_store.add(
                timestamp, 
                bar["open"], 
                bar["high"], 
                bar["low"], 
                bar["close"], 
                bar["volume"]
            )
            if not added:
                continue
                
            state = state_engine.update(obs_store)
            if state is not None:
                # We need to extract the exact floating point representations
                record = {
                    "timestamp": timestamp,
                    "price": float(bar["close"]),
                    "volume_acceleration_60m": float(state.volume_acceleration_60m) if state.volume_acceleration_60m is not None else 1.0,
                    "persistence_240m": float(state.persistence_240m) if state.persistence_240m is not None else 0.0
                }
                
                # Using json.dumps ensures precise string float representation
                out_f.write(json.dumps(record) + "\n")
                valid_states += 1
                
    print(f"Successfully generated {valid_states} valid state records at {output_path}")

if __name__ == "__main__":
    main()
