import argparse
import json
import random
import os

def generate_soak(count, out_file):
    with open(out_file, 'w') as f:
        ts = 1779197400000
        price = 100.0
        
        # Enforce deterministic generation for baseline reproducibility
        rng = random.Random(42)
        
        for i in range(count):
            # Jitter
            if rng.random() < 0.05:
                # Repeated timestamp (simulates concurrent fills or flush batches)
                pass
            elif rng.random() < 0.1:
                # Quiet period jump (simulates midday lulls or halts)
                ts += rng.randint(10000, 60000)
            elif rng.random() < 0.2:
                # Burst (simulates open/close density)
                ts += rng.randint(10, 50)
            else:
                ts += 1000
                
            if rng.random() < 0.1:
                # Repeated value (simulates illiquid prints)
                pass
            else:
                price += rng.uniform(-0.1, 0.1)
                
            f.write(json.dumps({"timestamp": ts, "price": max(1.0, price)}) + "\n")

if __name__ == "__main__":
    parser = argparse.ArgumentParser()
    parser.add_argument("count", type=int)
    parser.add_argument("out_file")
    args = parser.parse_args()
    
    os.makedirs(os.path.dirname(os.path.abspath(args.out_file)), exist_ok=True)
    generate_soak(args.count, args.out_file)
    print(f"Generated {args.count} events in {args.out_file}")
