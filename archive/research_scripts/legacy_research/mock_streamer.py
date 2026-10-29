import time
import json
import random
import sys
import math
from datetime import datetime

# --- Regime Settings ---
REGIMES = ["STABLE", "TREND", "CHOP", "ADVERSARIAL"]
SYMBOLS = ["BTC-USD", "ETH-USD", "SOL-USD", "IDEA.NS", "AXISBANK.NS", "BSE.NS", "RELIANCE.NS", "TCS.NS", "INFY.NS", "HDFCBANK.NS", "ICICIBANK.NS"]

def main():
    duration = 60
    if "--duration" in sys.argv:
        duration = int(sys.argv[sys.argv.index("--duration") + 1])
    
    brutal = "--brutal" in sys.argv
    adv_ratio = 0.4 # Default high adversity
    if "--adversarial-ratio" in sys.argv:
        adv_ratio = float(sys.argv[sys.argv.index("--adversarial-ratio") + 1])

    readers = []
    for i, sym in enumerate(SYMBOLS):
        readers.append({
            "symbol": sym,
            "price": 50000.0 if "BTC" in sym else (3000.0 if "ETH" in sym else 1000.0),
            "regime": "STABLE",
            "regime_timer": 0,
            "dir": 1 if i % 2 == 0 else -1,
            "ts": int(time.time()),
            "trap_active": False,
            "trap_phase": 0
        })

    print(f"📡 {'BRUTAL ' if brutal else ''}High-Fidelity Adversarial Streamer Active", file=sys.stderr)

    start_time = time.time()
    try:
        while (time.time() - start_time) < duration:
            batch = []
            for t in readers:
                # Regime Switching
                if t["regime_timer"] <= 0:
                    t["regime"] = random.choice(REGIMES)
                    # Brutal mode: much shorter regime timers -> instability
                    t["regime_timer"] = random.randint(5, 20) if brutal else random.randint(20, 100)
                    t["dir"] = random.choice([1, -1])
                    t["trap_active"] = False
                    t["trap_phase"] = 0
                
                t["regime_timer"] -= 1
                
                # Base Drift & Microstructure
                drift = 0.0
                imb_mod = 0.0 # Force imbalance modification
                
                # Brutal flip: flip trend mid-timer
                if brutal and random.random() < 0.1:
                    t["dir"] *= -1

                if t["regime"] == "TREND":
                    drift = t["dir"] * 0.0005 
                elif t["regime"] == "CHOP":
                    drift = t["dir"] * 0.0003
                    t["dir"] *= -1 
                elif t["regime"] == "ADVERSARIAL" or (brutal and random.random() < adv_ratio):
                    # MICROSTRUCTURE TRAP LOGIC
                    if t["trap_phase"] == 0:
                        # Phase 0: Bait (Positive drift, Positive Imbalance Accel)
                        drift = t["dir"] * 0.0002
                        imb_mod = t["dir"] * 0.05 # Strong pressure
                        if random.random() < (0.4 if brutal else 0.2): t["trap_phase"] = 1
                    elif t["trap_phase"] == 1:
                        # Phase 1: Exhaustion (Positive drift, NEGATIVE Imbalance Accel)
                        drift = t["dir"] * 0.0001 # Still moving up
                        imb_mod = -t["dir"] * 0.1 # Queue is vanishing!
                        if random.random() < (0.5 if brutal else 0.3): t["trap_phase"] = 2
                    elif t["trap_phase"] == 2:
                        # Phase 2: Reversal (Violent)
                        drift = -t["dir"] * 0.0020
                        imb_mod = -t["dir"] * 0.2
                        t["trap_phase"] = 0 # Reset
                
                # Random Noise (Brutal mode has higher jitter)
                noise_scale = 0.0005 if brutal else 0.0001
                noise = (random.random() - 0.5) * noise_scale
                
                prev_p = t["price"]
                t["price"] *= (1.0 + drift + noise)
                
                # We "hack" the volume/high/low to reflect imbalance
                # This isn't perfect but live_engine's tick_imbalance uses (close - prev_close)
                
                batch.append({
                    "symbol": t["symbol"],
                    "timestamp": t["ts"],
                    "open": prev_p,
                    "high": max(prev_p, t["price"]),
                    "low": min(prev_p, t["price"]),
                    "close": t["price"],
                    "volume": 1000.0 + (imb_mod * 10000.0) # Using volume as proxy for pressure if needed
                })
                t["ts"] += 60 
            
            print(json.dumps(batch), flush=True)
            time.sleep(0.01)
    except Exception as e:
        print(f"Error: {e}", file=sys.stderr)

if __name__ == "__main__":
    main()
