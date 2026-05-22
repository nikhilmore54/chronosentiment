import re
import pandas as pd
import numpy as np

# Configuration
LOG_FILE = "scripts/training_snapshots_rich.log"

def main():
    data = []
    flt = r"([-+]?\d*\.?\d+(?:[eE][-+]?\d+)?)"
    pattern = re.compile(
        fr"\[SNAPSHOT\] sym=([^\s]+) eos={flt} trap={flt} exh={flt} frag={flt} mom={flt} trap_d={flt} vol_r={flt} reg=([^\s]+) r1={flt} r5={flt} r10={flt} r20={flt}"
    )
    
    with open(LOG_FILE, "r") as f:
        for line in f:
            m = pattern.search(line)
            if m:
                data.append({
                    "trap_d": float(m.group(7)),
                    "mom": float(m.group(6)),
                    "vol_r": float(m.group(8))
                })
    df = pd.DataFrame(data)
    
    # Best Scorer: {'trap_exh': 0, 'trap_d': 2, 'mom_vol': -1}
    df['score'] = (df['trap_d'] * 2.0) - (df['mom'] * df['vol_r'])
    
    threshold = df['score'].quantile(0.99)
    print(f"99th Percentile Score Threshold: {threshold:.6f}")

if __name__ == "__main__":
    main()
