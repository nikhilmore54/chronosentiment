import re
from collections import defaultdict
import numpy as np

log_file = "archive/replay_1m_gen11.log"

telemetry_buffer = None
trade_telemetry = {}

print("Mapping Expectancy Toxicity Atlas with Temporal Freshness...")

# Data structure for clustering
# We will cluster by Efficiency, Resilience, and Age to find toxicity.
# clusters: (eff_bin, res_bin, age_bin) -> list of pnl_bps
clusters = defaultdict(list)
exit_clusters = defaultdict(lambda: defaultdict(list))
chaos_cluster = [] # Efficiency < 0.25
elasticity_cluster = [] # Efficiency 0.25 - 0.40

with open(log_file, "r") as f:
    for line in f:
        if "[TELEMETRY]" in line:
            # [TELEMETRY] ... atlas_eff=0.4554 atlas_den=0.2632 atlas_res=0.2202 shadow_fert=1.1500 atlas_age=12
            m_metrics = re.search(r"atlas_eff=([-\d\.]+)\s+atlas_den=([-\d\.]+)\s+atlas_res=([-\d\.]+)\s+shadow_fert=([-\d\.]+)\s+atlas_age=(\d+)", line)
            if m_metrics:
                telemetry_buffer = {
                    "eff": float(m_metrics.group(1)),
                    "den": float(m_metrics.group(2)),
                    "res": float(m_metrics.group(3)),
                    "fert": float(m_metrics.group(4)),
                    "age": int(m_metrics.group(5))
                }
        elif "[REC_STATUS]" in line and "status=ACTIVE" in line:
            if telemetry_buffer:
                m = re.search(r"rec_id=(\d+)", line)
                if m:
                    trade_telemetry[m.group(1)] = telemetry_buffer
                telemetry_buffer = None
        elif "[AUDIT_TRADE]" in line:
            # [AUDIT_TRADE] rec_id=1 ... realized_pnl=-0.0035 ... exit_type=Mortality
            m = re.search(r"rec_id=(\d+).*realized_pnl=([-\d\.]+).*exit_type=(\w+)", line)
            if m:
                rec_id = m.group(1)
                pnl_bps = float(m.group(2)) * 10000
                exit_type = m.group(3)
                
                if rec_id in trade_telemetry:
                    t = trade_telemetry[rec_id]
                    eff = t["eff"]
                    res = t["res"]
                    age = t["age"]
                    
                    # Binning
                    eff_bin = round(eff / 0.1) * 0.1
                    res_bin = round(res / 0.1) * 0.1
                    age_bin = (age // 5) * 5 # Bin ages into 0-4, 5-9, 10-14, etc.
                    
                    clusters[(eff_bin, res_bin, age_bin)].append(pnl_bps)
                    exit_clusters[exit_type][(eff_bin, res_bin, age_bin)].append(pnl_bps)
                    
                    if eff < 0.25:
                        chaos_cluster.append(pnl_bps)
                    elif 0.25 <= eff <= 0.40:
                        elasticity_cluster.append(pnl_bps)

print("\n=== EXPECTANCY TOXICITY ATLAS ===")
print("Cluster defined by (Efficiency, Resilience, Elasticity Age)")

# Sort clusters by worst average PnL
cluster_stats = []
for k, v in clusters.items():
    if len(v) >= 3: # Min support
        avg_pnl = sum(v) / len(v)
        cluster_stats.append((k, avg_pnl, len(v), sum(v)))

cluster_stats.sort(key=lambda x: x[1]) # Sort by avg pnl ascending (most toxic first)

print("\n[TOXIC CLUSTERS] (Avg PnL < 0)")
for k, avg_pnl, count, total_pnl in cluster_stats:
    if avg_pnl < 0:
        print(f"Eff ~ {k[0]:.1f}, Res ~ {k[1]:.1f}, Age: {k[2]:2}-{k[2]+4:2} bars | Trades: {count:2} | Avg PnL: {avg_pnl:6.1f} bps | Total: {total_pnl:6.1f} bps")

print("\n[FERTILE CLUSTERS] (Avg PnL > 0)")
for k, avg_pnl, count, total_pnl in cluster_stats:
    if avg_pnl >= 0:
        print(f"Eff ~ {k[0]:.1f}, Res ~ {k[1]:.1f}, Age: {k[2]:2}-{k[2]+4:2} bars | Trades: {count:2} | Avg PnL: {avg_pnl:6.1f} bps | Total: {total_pnl:6.1f} bps")

print("\n=== MACRO REGION ANALYSIS ===")
chaos_avg = sum(chaos_cluster)/len(chaos_cluster) if chaos_cluster else 0
elastic_avg = sum(elasticity_cluster)/len(elasticity_cluster) if elasticity_cluster else 0
print(f"Over-Tolerated Chaos (Eff < 0.25): Trades: {len(chaos_cluster)}, Avg PnL: {chaos_avg:.2f} bps")
print(f"Elasticity Zone (Eff 0.25 - 0.40): Trades: {len(elasticity_cluster)}, Avg PnL: {elastic_avg:.2f} bps")

print("\n=== FALSE ELASTICITY DETECTION ===")
# False elasticity: Eff 0.25 - 0.40, but exited via Mortality
if "Mortality" in exit_clusters:
    mort_elastic = []
    mort_elastic_stale = []
    mort_elastic_fresh = []
    for k, v in exit_clusters["Mortality"].items():
        if 0.25 <= k[0] <= 0.40:
            mort_elastic.extend(v)
            if k[2] >= 10: # Stale elasticity (10+ bars since reload)
                mort_elastic_stale.extend(v)
            else:
                mort_elastic_fresh.extend(v)
    
    if mort_elastic:
        print(f"False Elasticity Total (Mortality in Elastic Zone): Trades: {len(mort_elastic)}, Avg PnL: {sum(mort_elastic)/len(mort_elastic):.2f} bps, Total Loss: {sum(mort_elastic):.2f} bps")
    if mort_elastic_stale:
        print(f"  - Stale Elasticity (>10 bars since reload): Trades: {len(mort_elastic_stale)}, Avg PnL: {sum(mort_elastic_stale)/len(mort_elastic_stale):.2f} bps, Total Loss: {sum(mort_elastic_stale):.2f} bps")
    if mort_elastic_fresh:
        print(f"  - Fresh Elasticity (<10 bars since reload): Trades: {len(mort_elastic_fresh)}, Avg PnL: {sum(mort_elastic_fresh)/len(mort_elastic_fresh):.2f} bps, Total Loss: {sum(mort_elastic_fresh):.2f} bps")

