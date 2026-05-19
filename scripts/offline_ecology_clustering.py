"""
Phase B2: Offline Ecology Clustering & Transition Dynamics
Implements rolling PCA dimensionality reduction on telemetry windows,
uncovers stable attractors, computes the Markov Transition Matrix,
and measures Attractor Recurrence and Stability Metrics.
"""
import re
import json
import numpy as np

# Logs to reconstruct continuous trajectories
LOG_FILES = [
    ("Crypto 1m", "archive/replay_1m_gen11.log"),
    ("Crypto 5m OOS", "archive/replay_5m_oos1.log"),
    ("Equities 5m", "archive/replay_xasset_equities.log"),
    ("Commodities 5m", "archive/replay_xasset_commodities.log"),
]

tel_pattern = re.compile(
    r"margin=(?P<margin>[\d\.\-]+)\s+conv=(?P<conv>[\d\.\-]+)\s+eq=(?P<eq>[\d\.\-]+).*?"
    r"atlas_eff=(?P<eff>[\d\.\-]+)\s+atlas_den=(?P<den>[\d\.\-]+)\s+atlas_res=(?P<res>[\d\.\-]+).*?atlas_age=(?P<age>\d+).*?"
    r"genesis_comp=(?P<comp>[\d\.\-]+)\s+genesis_range=(?P<range>[\d\.\-]+)\s+genesis_bias=(?P<bias>[\d\.\-]+)"
)

def extract_continuous_telemetry(path):
    points = []
    try:
        with open(path) as f:
            for line in f:
                if "[TELEMETRY]" in line:
                    m = tel_pattern.search(line)
                    if m:
                        d = m.groupdict()
                        points.append([
                            float(d["range"]),  # Volatility bounds
                            float(d["bias"]),   # Prior bias
                            float(d["eff"]),    # Local efficiency
                            float(d["comp"]),   # Compression ratio
                            float(d["res"])     # Elastic capacity
                        ])
    except FileNotFoundError:
        pass
    return np.array(points)

# 1. Gather continuous state paths
all_series = {}
total_samples = 0
for name, path in LOG_FILES:
    pts = extract_continuous_telemetry(path)
    if len(pts) >= 50:
        all_series[name] = pts
        total_samples += len(pts)
        print(f"  Loaded continuous series: {name:<15} ({len(pts)} steps)")

if not all_series:
    print("❌ No valid continuous telemetry series found. Aborting.")
    exit(1)

# Combine all points for global PCA normalization
X_all = np.vstack(list(all_series.values()))

# Normalize (Z-score standard normalization)
mean_X = X_all.mean(axis=0)
std_X = X_all.std(axis=0)
std_X[std_X == 0] = 1.0
X_normalized = (X_all - mean_X) / std_X

# 2. Pure NumPy Principal Component Analysis (PCA)
cov_matrix = np.cov(X_normalized.T)
eigenvalues, eigenvectors = np.linalg.eigh(cov_matrix)
# Sort in descending order
idx = eigenvalues.argsort()[::-1]
eigenvalues = eigenvalues[idx]
eigenvectors = eigenvectors[:, idx]

# Project to top 2 PCs
PC1_vec = eigenvectors[:, 0]
PC2_vec = eigenvectors[:, 1]
print(f"\n  PCA Explained Variance: PC1 = {eigenvalues[0]/sum(eigenvalues)*100:.1f}%, PC2 = {eigenvalues[1]/sum(eigenvalues)*100:.1f}%")

# 3. Attractor Clustering Definition (Manifold Segmentation)
# We partition the 2D projected space into three logical attractors:
# - Attractor 0: LIQUIDITY_EXHAUSTION (High prior bias, low efficiency)
# - Attractor 1: NARRATIVE_PERSISTENCE (High compression, stable efficiency)
# - Attractor 2: NOISE_TRANSITIONAL (Low prior bias, high local variance)
projected_all = X_normalized.dot(np.column_stack((PC1_vec, PC2_vec)))

# Simple k-means equivalent mapping based on centroids of known ecology classes
centroids = np.zeros((3, 2))
# Seed centroids based on project topology findings
centroids[0] = [-1.2, -0.5]  # Liquidity Exhaustion
centroids[1] = [1.0, 1.2]   # Narrative Persistence
centroids[2] = [0.1, -0.8]  # Noise / Transitional

# 5 iterations of K-Means to stabilize global manifolds
for _ in range(5):
    dists = np.linalg.norm(projected_all[:, None, :] - centroids[None, :, :], axis=2)
    labels = dists.argmin(axis=1)
    for c in range(3):
        members = projected_all[labels == c]
        if len(members) > 0:
            centroids[c] = members.mean(axis=0)

attractor_names = {
    0: "LIQUIDITY_EXHAUSTION (A/C/D)",
    1: "NARRATIVE_PERSISTENCE (E)",
    2: "NOISE_TRANSITIONAL (Unstable)"
}

# 4. Markov Transition Dynamics Calculation
transitions = np.zeros((3, 3))
attractor_durations = {0: [], 1: [], 2: []}

for name, series in all_series.items():
    # Normalize series using global parameters
    norm_series = (series - mean_X) / std_X
    proj_series = norm_series.dot(np.column_stack((PC1_vec, PC2_vec)))
    
    # Label states
    dists = np.linalg.norm(proj_series[:, None, :] - centroids[None, :, :], axis=2)
    state_labels = dists.argmin(axis=1)
    
    # Calculate transitions
    for t in range(len(state_labels) - 1):
        s_curr = state_labels[t]
        s_next = state_labels[t+1]
        transitions[s_curr, s_next] += 1
        
    # Calculate half-life/stability durations
    curr_state = state_labels[0]
    run_length = 1
    for t in range(1, len(state_labels)):
        if state_labels[t] == curr_state:
            run_length += 1
        else:
            attractor_durations[curr_state].append(run_length)
            curr_state = state_labels[t]
            run_length = 1
    attractor_durations[curr_state].append(run_length)

# Normalize transition matrix to probabilities
transition_probs = np.zeros((3, 3))
for r in range(3):
    row_sum = transitions[r].sum()
    if row_sum > 0:
        transition_probs[r] = transitions[r] / row_sum
    else:
        transition_probs[r, r] = 1.0

# 5. Stability & Entropy Metrics
print("\n" + "=" * 80)
print("  PHASE B2: MARKOV TRANSITION PROBABILITY MATRIX")
print("=" * 80)
print(f"  {'Current State':<32} | {'To Attractor 0':^16} | {'To Attractor 1':^16} | {'To Attractor 2':^16}")
print("  " + "─" * 85)
for r in range(3):
    p0, p1, p2 = transition_probs[r]
    print(f"  {attractor_names[r]:<32} | {p0:^16.4f} | {p1:^16.4f} | {p2:^16.4f}")

print("\n" + "─" * 80)
print("  ATTRACTOR STABILITY & RECURRENCE METRICS")
print("─" * 80)
for r in range(3):
    durs = attractor_durations[r]
    mean_life = np.mean(durs) if durs else 0.0
    # Shannon Entropy of transition probabilities out of state
    probs = transition_probs[r]
    entropy = -sum(p * np.log2(p) for p in probs if p > 0)
    print(f"  {attractor_names[r]:<32}")
    print(f"    Attractor Half-Life: {mean_life:.1f} bars  |  Transition Entropy: {entropy:.4f} bits")

# Export offline clustering topology for UI visualization
export_data = {
    "centroids": centroids.tolist(),
    "transition_matrix": transition_probs.tolist(),
    "stability_metrics": {
        str(r): {
            "half_life": float(np.mean(attractor_durations[r])) if attractor_durations[r] else 0.0,
            "entropy": float(-sum(p * np.log2(p) for p in transition_probs[r] if p > 0))
        } for r in range(3)
    }
}

with open("observatory/ecology_clustering.json", "w") as f:
    json.dump(export_data, f, indent=4)
print(f"\n✅ Offline Ecology Clustering data exported to observatory/ecology_clustering.json")
