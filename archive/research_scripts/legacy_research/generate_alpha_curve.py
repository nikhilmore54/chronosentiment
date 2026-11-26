import re
import json
import sys
import collections
import math

def bucket_intensity(val):
    if val < 2.0: return "i1.8-2.0"
    if val < 2.5: return "i2.0-2.5"
    return "i2.5+"

def bucket_stability(val):
    if val < 0.02: return "s0-0.02"
    if val < 0.05: return "s0.02-0.05"
    return "s0.05+"

def calculate_std(data, mean):
    if len(data) < 2: return 0.0
    variance = sum((x - mean) ** 2 for x in data) / (len(data) - 1)
    return math.sqrt(variance)

def main(log_path, output_path):
    # key -> list of pnls
    cells = collections.defaultdict(list)
    
    with open(log_path, 'r') as f:
        for line in f:
            # Parse executed trades
            if "[AUDIT_TRADE]" in line:
                try:
                    pnl = float(re.search(r"realized_pnl=([-\d\.]+)", line).group(1))
                    intensity = float(re.search(r"intensity=([-\d\.]+)", line).group(1))
                    stability = float(re.search(r"stability=([-\d\.]+)", line).group(1))
                    regime = re.search(r"regime=([A-Z_]+)", line).group(1)
                    mode = re.search(r"mode=([A-Z]+)", line).group(1)
                    
                    i_b = bucket_intensity(intensity)
                    s_b = bucket_stability(stability)
                    key = f"{mode}_{regime}_{i_b}_{s_b}"
                    cells[key].append(pnl)
                except Exception:
                    continue
            
            # Parse counterfactual kills
            elif "[STABILITY_KILL_AUDIT]" in line:
                try:
                    pnl = float(re.search(r"pnl_cf=([-\d\.]+)", line).group(1))
                    intensity = float(re.search(r"intensity=([-\d\.]+)", line).group(1))
                    stability = float(re.search(r"var=([-\d\.]+)", line).group(1))
                    regime = re.search(r"regime=([A-Z_]+)", line).group(1)
                    mode = re.search(r"mode=([A-Z]+)", line).group(1)
                    
                    i_b = bucket_intensity(intensity)
                    s_b = bucket_stability(stability)
                    key = f"{mode}_{regime}_{i_b}_{s_b}"
                    cells[key].append(pnl)
                except Exception:
                    continue

    alpha_curve = {}
    for key, pnls in cells.items():
        avg = sum(pnls) / len(pnls)
        alpha_curve[key] = {
            "avg_pnl": avg,
            "pnl_std": calculate_std(pnls, avg),
            "count": len(pnls)
        }
        
    with open(output_path, 'w') as f:
        json.dump(alpha_curve, f, indent=2)
    
    print(f"✅ Generated Alpha Curve with {len(alpha_curve)} cells to {output_path}")

if __name__ == "__main__":
    if len(sys.argv) < 3:
        print("Usage: python3 generate_alpha_curve.py <log_file> <output_json>")
    else:
        main(sys.argv[1], sys.argv[2])
