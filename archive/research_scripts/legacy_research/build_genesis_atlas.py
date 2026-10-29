import re
from collections import defaultdict

log_file = "archive/replay_1m_gen11.log"

telemetry_buffer = None
trade_telemetry = {}

print("=== EDGE GENESIS OBSERVATORY ===")
print("Studying what existed BEFORE topology — where does edge originate?\n")

with open(log_file, "r") as f:
    for line in f:
        if "[TELEMETRY]" in line:
            m = re.search(
                r"atlas_eff=([-\d\.]+)\s+atlas_den=([-\d\.]+)\s+atlas_res=([-\d\.]+)\s+"
                r"shadow_fert=([-\d\.]+)\s+atlas_age=(\d+)\s+\|\s+"
                r"genesis_comp=([-\d\.]+)\s+genesis_range=([-\d\.]+)\s+genesis_bias=([-\d\.]+)",
                line
            )
            if m:
                telemetry_buffer = {
                    "eff": float(m.group(1)),
                    "res": float(m.group(3)),
                    "age": int(m.group(5)),
                    "comp": float(m.group(6)),
                    "range": float(m.group(7)),
                    "bias": float(m.group(8)),
                }
        elif "[REC_STATUS]" in line and "status=ACTIVE" in line:
            if telemetry_buffer:
                m = re.search(r"rec_id=(\d+)", line)
                if m:
                    trade_telemetry[m.group(1)] = telemetry_buffer
                telemetry_buffer = None
        elif "[AUDIT_TRADE]" in line:
            m = re.search(r"rec_id=(\d+).*realized_pnl=([-\d\.]+).*exit_type=(\w+)", line)
            if m:
                rec_id = m.group(1)
                pnl_bps = float(m.group(2)) * 10000
                exit_type = m.group(3)
                if rec_id in trade_telemetry:
                    trade_telemetry[rec_id]["pnl"] = pnl_bps
                    trade_telemetry[rec_id]["exit"] = exit_type

# Filter to trades with full data
trades = [t for t in trade_telemetry.values() if "pnl" in t]

if not trades:
    print("No trades with genesis data found!")
    exit()

winners = [t for t in trades if t["pnl"] > 0]
losers = [t for t in trades if t["pnl"] <= 0]

def avg(lst, key):
    vals = [t[key] for t in lst]
    return sum(vals) / len(vals) if vals else 0

print(f"Total trades with genesis data: {len(trades)}")
print(f"Winners: {len(winners)}, Losers: {len(losers)}\n")

# === 1. WINNERS vs LOSERS: Genesis Conditions ===
print("=" * 70)
print("WINNERS vs LOSERS: Pre-Entry Microstructure")
print("=" * 70)
print(f"{'Metric':<30} {'Winners':>12} {'Losers':>12} {'Delta':>12}")
print("-" * 70)

metrics = [
    ("Compression Ratio", "comp"),
    ("Pre-Entry Range", "range"),
    ("Pre-Entry Bias", "bias"),
    ("Directional Efficiency", "eff"),
    ("Resilience Score", "res"),
    ("Elasticity Age", "age"),
]

for name, key in metrics:
    w = avg(winners, key)
    l = avg(losers, key)
    d = w - l
    fmt = ".4f" if key != "age" else ".1f"
    print(f"{name:<30} {w:>12{fmt}} {l:>12{fmt}} {d:>+12{fmt}}")

# === 2. By Exit Type: Genesis Conditions ===
print("\n" + "=" * 70)
print("BY EXIT TYPE: Pre-Entry Genesis Conditions")
print("=" * 70)

exit_groups = defaultdict(list)
for t in trades:
    exit_groups[t["exit"]].append(t)

print(f"{'Exit':<15} {'N':>4} {'Avg PnL':>10} {'Comp':>8} {'Range':>10} {'Bias':>8} {'Age':>5}")
print("-" * 70)
for exit_type in ["TakeProfit", "TrailingStop", "Mortality", "StopLoss"]:
    if exit_type in exit_groups:
        g = exit_groups[exit_type]
        n = len(g)
        print(f"{exit_type:<15} {n:>4} {avg(g, 'pnl'):>+10.1f} {avg(g, 'comp'):>8.3f} {avg(g, 'range'):>10.6f} {avg(g, 'bias'):>+8.4f} {avg(g, 'age'):>5.1f}")

# === 3. Compression Release vs PnL ===
print("\n" + "=" * 70)
print("COMPRESSION RELEASE RATIO vs OUTCOME")
print("=" * 70)

comp_bins = defaultdict(list)
for t in trades:
    c = t["comp"]
    if c < 0.8:
        comp_bins["< 0.80 (contracting)"].append(t)
    elif c < 1.0:
        comp_bins["0.80 - 1.00 (stable)"].append(t)
    elif c < 1.3:
        comp_bins["1.00 - 1.30 (mild expansion)"].append(t)
    elif c < 1.8:
        comp_bins["1.30 - 1.80 (strong expansion)"].append(t)
    else:
        comp_bins["> 1.80 (explosive)"].append(t)

print(f"{'Compression Bin':<35} {'N':>4} {'Avg PnL':>10} {'Win%':>8}")
print("-" * 70)
for label in ["< 0.80 (contracting)", "0.80 - 1.00 (stable)", "1.00 - 1.30 (mild expansion)", "1.30 - 1.80 (strong expansion)", "> 1.80 (explosive)"]:
    if label in comp_bins:
        g = comp_bins[label]
        n = len(g)
        wr = len([t for t in g if t["pnl"] > 0]) / n * 100
        print(f"{label:<35} {n:>4} {avg(g, 'pnl'):>+10.1f} {wr:>7.1f}%")

# === 4. Pre-Bias Alignment ===
print("\n" + "=" * 70)
print("PRE-ENTRY BIAS ALIGNMENT (Was direction already established?)")
print("=" * 70)

# For BUY signals, positive bias = aligned. For SELL signals, negative = aligned.
# We'll just look at absolute bias magnitude for simplicity
bias_bins = defaultdict(list)
for t in trades:
    b = abs(t["bias"])
    if b < 0.15:
        bias_bins["< 0.15 (no prior trend)"].append(t)
    elif b < 0.35:
        bias_bins["0.15 - 0.35 (mild trend)"].append(t)
    elif b < 0.55:
        bias_bins["0.35 - 0.55 (moderate trend)"].append(t)
    else:
        bias_bins["> 0.55 (strong prior trend)"].append(t)

print(f"{'Bias Magnitude':<35} {'N':>4} {'Avg PnL':>10} {'Win%':>8}")
print("-" * 70)
for label in ["< 0.15 (no prior trend)", "0.15 - 0.35 (mild trend)", "0.35 - 0.55 (moderate trend)", "> 0.55 (strong prior trend)"]:
    if label in bias_bins:
        g = bias_bins[label]
        n = len(g)
        wr = len([t for t in g if t["pnl"] > 0]) / n * 100
        print(f"{label:<35} {n:>4} {avg(g, 'pnl'):>+10.1f} {wr:>7.1f}%")
