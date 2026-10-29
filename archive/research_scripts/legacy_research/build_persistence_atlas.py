import re
from collections import defaultdict

log_file = "archive/replay_1m_gen11.log"

telemetry_buffer = None

# Store telemetry by rec_id
# rec_id -> {"sym": str, "eff": float, "den": float, "res": float, "fert": float}
trade_telemetry = {}

# Group telemetry by exit reason AND asset
# sym -> reason -> list of dicts
exit_groups = defaultdict(lambda: defaultdict(list))

print("Mapping Persistence Atlas (Cross-Asset)...")

with open(log_file, "r") as f:
    for line in f:
        if "[TELEMETRY]" in line:
            # [TELEMETRY] 22:29:40 sym=SOL-USD sig=BUY ... atlas_eff=0.4554 atlas_den=0.2632 atlas_res=0.2202 shadow_fert=1.1500
            m_sym = re.search(r"sym=([A-Z\-]+)", line)
            m_metrics = re.search(r"atlas_eff=([-\d\.]+)\s+atlas_den=([-\d\.]+)\s+atlas_res=([-\d\.]+)\s+shadow_fert=([-\d\.]+)", line)
            if m_sym and m_metrics:
                telemetry_buffer = {
                    "sym": m_sym.group(1),
                    "eff": float(m_metrics.group(1)),
                    "den": float(m_metrics.group(2)),
                    "res": float(m_metrics.group(3)),
                    "fert": float(m_metrics.group(4))
                }
        elif "[REC_STATUS]" in line and "status=ACTIVE" in line:
            if telemetry_buffer:
                m = re.search(r"rec_id=(\d+)", line)
                if m:
                    rec_id = m.group(1)
                    trade_telemetry[rec_id] = telemetry_buffer
                telemetry_buffer = None # Consume
        elif "[REC_STATUS]" in line and "status=CLOSED" in line:
            m = re.search(r"rec_id=(\d+).*reason=(\w+)", line)
            if m:
                rec_id = m.group(1)
                reason = m.group(2)
                if rec_id in trade_telemetry:
                    sym = trade_telemetry[rec_id]["sym"]
                    exit_groups[sym][reason].append(trade_telemetry[rec_id])

print("\n--- 🗺️ THE PERSISTENCE ATLAS (CROSS-ASSET) ---")
for sym, reasons in exit_groups.items():
    print(f"\n================ ASSET: {sym} ================")
    for reason, metrics in reasons.items():
        count = len(metrics)
        avg_eff = sum(m["eff"] for m in metrics) / count if count > 0 else 0
        avg_den = sum(m["den"] for m in metrics) / count if count > 0 else 0
        avg_res = sum(m["res"] for m in metrics) / count if count > 0 else 0
        avg_fert = sum(m["fert"] for m in metrics) / count if count > 0 else 0
        
        print(f"\nExit Topology: {reason} (Count: {count})")
        print(f"  Directional Efficiency : {avg_eff:.4f}")
        print(f"  Continuation Density   : {avg_den:.4f}")
        print(f"  Resilience Score       : {avg_res:.4f}")
        print(f"  Shadow Fertility Mult  : {avg_fert:.4f}")
