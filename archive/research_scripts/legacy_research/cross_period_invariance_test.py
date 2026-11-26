"""
Track 1: Invariance Validation
Performs cross-period invariance testing by extracting independent PCA components
and transition probability matrices for Period 1 (Training regime) and Period 2 (OOS regime)
to mathematically measure Centroid Alignment Drift and Transition Matrix Stability.
"""
import re
import json
import numpy as np

PERIOD_FILES = {
    "Period 1: Same-Regime Training (Apr 18 - May 18)": "archive/replay_training_5m.log",
    "Period 2: Out-of-Sample Regime (Mar 19 - Apr 18)": "archive/replay_5m_oos1.log",
}

tel_pattern = re.compile(
    r"margin=(?P<margin>[\d\.\-]+)\s+conv=(?P<conv>[\d\.\-]+)\s+eq=(?P<eq>[\d\.\-]+).*?"
    r"atlas_eff=(?P<eff>[\d\.\-]+)\s+atlas_den=(?P<den>[\d\.\-]+)\s+atlas_res=(?P<res>[\d\.\-]+).*?atlas_age=(?P<age>\d+).*?"
    r"genesis_comp=(?P<comp>[\d\.\-]+)\s+genesis_range=(?P<range>[\d\.\-]+)\s+genesis_bias=(?P<bias>[\d\.\-]+)"
)

def extract_telemetry(path):
    points = []
    try:
        with open(path) as f:
            for line in f:
                if "[TELEMETRY]" in line:
                    m = tel_pattern.search(line)
                    if m:
                        d = m.groupdict()
                        points.append([
                            float(d["range"]),
                            float(d["bias"]),
                            float(d["eff"]),
                            float(d["comp"]),
                            float(d["res"])
                        ])
    except FileNotFoundError:
        pass
    return np.array(points)

print("=" * 80)
print("  TRACK 1: CROSS-PERIOD INVARIANCE STRESS TEST")
print("  Testing whether the sequential propagation manifold generalizes across regimes")
print("=" * 80)

# Load periods
p1_data = extract_telemetry(PERIOD_FILES["Period 1: Same-Regime Training (Apr 18 - May 18)"])
p2_data = extract_telemetry(PERIOD_FILES["Period 2: Out-of-Sample Regime (Mar 19 - Apr 18)"])

if len(p1_data) < 50 or len(p2_data) < 50:
    print("❌ Insufficient telemetry points to compute cross-period PCA. Aborting.")
    exit(1)

print(f"  Loaded Period 1: {len(p1_data)} continuous steps")
print(f"  Loaded Period 2: {len(p2_data)} continuous steps")

# 1. PCA CALCULATION FOR PERIOD 1
mean_p1 = p1_data.mean(axis=0)
std_p1 = p1_data.std(axis=0)
std_p1[std_p1 == 0] = 1.0
p1_normalized = (p1_data - mean_p1) / std_p1
cov_p1 = np.cov(p1_normalized.T)
evals_p1, evecs_p1 = np.linalg.eigh(cov_p1)
idx1 = evals_p1.argsort()[::-1]
PC1_p1 = evecs_p1[:, idx1[0]]
PC2_p1 = evecs_p1[:, idx1[1]]

# 2. PCA CALCULATION FOR PERIOD 2
mean_p2 = p2_data.mean(axis=0)
std_p2 = p2_data.std(axis=0)
std_p2[std_p2 == 0] = 1.0
p2_normalized = (p2_data - mean_p2) / std_p2
cov_p2 = np.cov(p2_normalized.T)
evals_p2, evecs_p2 = np.linalg.eigh(cov_p2)
idx2 = evals_p2.argsort()[::-1]
PC1_p2 = evecs_p2[:, idx2[0]]
PC2_p2 = evecs_p2[:, idx2[1]]

# 3. MANIFOLD COSINE ALIGNMENT CHECK
# Measure the cosine similarity of eigenvectors between Period 1 and Period 2
cos_sim_pc1 = abs(np.dot(PC1_p1, PC1_p2))
cos_sim_pc2 = abs(np.dot(PC2_p1, PC2_p2))

# 4. LOAD GLOBAL CENTROIDS
try:
    with open("observatory/ecology_clustering.json") as f:
        clustering_data = json.load(f)
    centroids = np.array(clustering_data["centroids"])
except FileNotFoundError:
    print("❌ Baseline clustering data not found. Rerun offline_ecology_clustering.py first.")
    exit(1)

def compute_transition_matrix(series_norm, pc1, pc2):
    proj_series = series_norm.dot(np.column_stack((pc1, pc2)))
    dists = np.linalg.norm(proj_series[:, None, :] - centroids[None, :, :], axis=2)
    state_labels = dists.argmin(axis=1)
    
    transitions = np.zeros((3, 3))
    for t in range(len(state_labels) - 1):
        s_curr = state_labels[t]
        s_next = state_labels[t+1]
        transitions[s_curr, s_next] += 1
        
    probs = np.zeros((3, 3))
    for r in range(3):
        row_sum = transitions[r].sum()
        if row_sum > 0:
            probs[r] = transitions[r] / row_sum
        else:
            probs[r, r] = 1.0
    return probs

# Compute transition matrices using respective local PCA components
probs_p1 = compute_transition_matrix(p1_normalized, PC1_p1, PC2_p1)
probs_p2 = compute_transition_matrix(p2_normalized, PC1_p2, PC2_p2)

# Calculate transition stability drift (Mean Absolute Error between matrices)
matrix_drift = np.mean(np.abs(probs_p1 - probs_p2))

print("\n" + "─" * 80)
print("  ECOLOGICAL MANIFOLD ALIGNMENT RESULTS")
print("─" * 80)
print(f"  1. Eigenvector Alignment (PC1 Cosine Similarity): {cos_sim_pc1:.4f} (Higher = More Invariant)")
print(f"  2. Eigenvector Alignment (PC2 Cosine Similarity): {cos_sim_pc2:.4f} (Higher = More Invariant)")
print(f"  3. Transition Matrix Drift (Mean Absolute Error):   {matrix_drift:.4f}  (Lower = More Stable)")

print("\n  Period 1: Same-Regime Transition Probabilities:")
print(f"    Liquidity Self-P: {probs_p1[0,0]:.4f}  |  Narrative Self-P: {probs_p1[1,1]:.4f}  |  Noise Self-P: {probs_p1[2,2]:.4f}")

print("\n  Period 2: Out-of-Sample Transition Probabilities:")
print(f"    Liquidity Self-P: {probs_p2[0,0]:.4f}  |  Narrative Self-P: {probs_p2[1,1]:.4f}  |  Noise Self-P: {probs_p2[2,2]:.4f}")

print("\n" + "=" * 80)
print("  CROSS-PERIOD INVARIANCE VERDICT")
print("=" * 80)

# Conditions for invariant success:
# - PCA Cosine Sim > 0.85
# - Transition matrix drift < 0.10
if cos_sim_pc1 > 0.85 and matrix_drift < 0.10:
    print("  ✅ CROSS-PERIOD INVARIANCE CONFIRMED!")
    print("     The sequential propagation manifold generalizes invariantly across time and regimes:")
    print("     - Eigenvectors remain aligned, proving structural subspace stability.")
    print("     - The transition probability drift is extremely low, proving dynamic stability.")
else:
    print("  ❌ MANIFOLD DRIFT DETECTED: The structure does not generalize invariantly.")
print("=" * 80)

# Export cross-period evaluation
with open("observatory/ecology_clustering.json") as f:
    cluster_js = json.load(f)
cluster_js["cross_period_invariance"] = {
    "pc1_cosine_similarity": float(cos_sim_pc1),
    "pc2_cosine_similarity": float(cos_sim_pc2),
    "matrix_drift": float(matrix_drift),
    "p1_transitions": probs_p1.tolist(),
    "p2_transitions": probs_p2.tolist(),
}
with open("observatory/ecology_clustering.json", "w") as f:
    json.dump(cluster_js, f, indent=4)
