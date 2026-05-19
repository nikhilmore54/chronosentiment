"""
Phase B1: Ecology Signature Extraction
Extracts observable micro-features from raw telemetry streams to build the Ecology Signature Atlas.
Allows us to map LIQUIDITY_FLOW vs EVENT_DRIVEN physics before trades occur.
"""
import re
import math
import json
import numpy as np

CONDITIONS = {
    "A: Crypto 1m (training)":   "archive/replay_1m_gen11.log",
    "B: Crypto 5m (same regime)": "archive/replay_training_5m.log",
    "C: Crypto 5m (OOS regime)": "archive/replay_5m_oos1.log",
    "D: Equities 5m":            "archive/replay_xasset_equities.log",
    "E: Commodities 5m":         "archive/replay_xasset_commodities.log",
}

# Regex to parse rich telemetry strings
# Pattern: [TELEMETRY] ... margin=3.02 conv=1.05 eq=0.7551 | legacy_exp=... | atlas_eff=0.2954 atlas_den=0.4737 atlas_res=0.5518 shadow_fert=0.9900 atlas_age=11 | genesis_comp=2.3686 genesis_range=0.011065 genesis_bias=-0.4111
tel_pattern = re.compile(
    r"margin=(?P<margin>[\d\.\-]+)\s+conv=(?P<conv>[\d\.\-]+)\s+eq=(?P<eq>[\d\.\-]+).*?"
    r"atlas_eff=(?P<eff>[\d\.\-]+)\s+atlas_den=(?P<den>[\d\.\-]+)\s+atlas_res=(?P<res>[\d\.\-]+).*?atlas_age=(?P<age>\d+).*?"
    r"genesis_comp=(?P<comp>[\d\.\-]+)\s+genesis_range=(?P<range>[\d\.\-]+)\s+genesis_bias=(?P<bias>[\d\.\-]+)"
)

def compute_autocorrelation(x, lag=1):
    if len(x) < lag + 5:
        return 0.0
    mean = np.mean(x)
    var = np.var(x)
    if var < 1e-9:
        return 0.0
    xp = x - mean
    return np.sum(xp[:-lag] * xp[lag:]) / ((len(xp) - lag) * var)

def analyze_signatures(log_path):
    telemetries = []
    
    try:
        with open(log_path) as f:
            for line in f:
                if "[TELEMETRY]" in line:
                    m = tel_pattern.search(line)
                    if m:
                        d = m.groupdict()
                        telemetries.append({k: float(v) for k, v in d.items()})
    except FileNotFoundError:
        # Check standard location if not found in archive
        alt_path = log_path.replace("archive/", "")
        try:
            with open(alt_path) as f:
                for line in f:
                    if "[TELEMETRY]" in line:
                        m = tel_pattern.search(line)
                        if m:
                            d = m.groupdict()
                            telemetries.append({k: float(v) for k, v in d.items()})
        except FileNotFoundError:
            return None

    if len(telemetries) < 20:
        return None

    # Extraction of core feature streams
    biases = np.array([t["bias"] for t in telemetries])
    effs   = np.array([t["eff"] for t in telemetries])
    comps  = np.array([t["comp"] for t in telemetries])
    ranges = np.array([t["range"] for t in telemetries])
    convs  = np.array([t["conv"] for t in telemetries])
    reses  = np.array([t["res"] for t in telemetries])
    
    # ── Calculate Signatures ───────────────────────────────────────────────
    
    # 1. Volatility Persistence (Auto-correlation of pre-range volatility bounds)
    vol_persistence = compute_autocorrelation(ranges, lag=1)
    
    # 2. Directional Bias Persistence (Auto-correlation of pre-bias directionality)
    # High persistence = Event-driven persistent propagation (momentum)
    # Low/Negative persistence = mean-reverting/fragmented liquidity flow
    bias_persistence = compute_autocorrelation(biases, lag=1)
    
    # 3. Smoothness Stability (Standard deviation of local excursion efficiency)
    # Low variance = smooth, stable propagation field
    # High variance = turbulent/noisy field
    smoothness_stability = np.std(effs)
    
    # 4. Asymmetry Release Intensity (Average compression-release ratio)
    mean_compression = np.mean(comps)
    
    # 5. Consensus Agreement Entropy
    # How distributed or concentrated are consensus votes?
    mean_conv = np.mean(convs)
    
    # 6. Resilience Decay Half-Life Proxy
    # Average elasticity resilience remaining during propagation
    mean_resilience = np.mean(reses)
    
    return {
        "n_samples": len(telemetries),
        "vol_persistence": float(vol_persistence),
        "bias_persistence": float(bias_persistence),
        "smoothness_stability": float(smoothness_stability),
        "mean_compression": float(mean_compression),
        "mean_conv": float(mean_conv),
        "mean_resilience": float(mean_resilience),
    }

print("=" * 80)
print("  PHASE B1: ECOLOGY SIGNATURE ATLAS")
print("  Extracting latent market physics signatures from raw telemetry streams")
print("=" * 80)

atlas = {}
for label, path in CONDITIONS.items():
    res = analyze_signatures(path)
    if res is None:
        print(f"  {label:<30} ❌ No telemetry data found")
        continue
    
    atlas[label] = res
    print(f"\n  {label}")
    print(f"    Samples: {res['n_samples']:<6} Vol Persistence: {res['vol_persistence']:+.4f} (lag 1)")
    print(f"    Bias Persistence: {res['bias_persistence']:+.4f} (trend continuation proxy)")
    print(f"    Smoothness Var:   {res['smoothness_stability']:.4f} (microstructure stability)")
    print(f"    Compression Ratio: {res['mean_compression']:.4f} (genesis energy)")
    print(f"    Avg Consensus:     {res['mean_conv']:.4f} (agreement depth)")
    print(f"    Avg Resilience:    {res['mean_resilience']:.4f} (elastic capacity)")

# Output results to the observatory so it is visually dynamic
with open("observatory/ecology_signatures.json", "w") as f:
    json.dump(atlas, f, indent=4)
print(f"\n✅ Ecology Signature Atlas exported to observatory/ecology_signatures.json")
