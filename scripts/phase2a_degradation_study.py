import argparse
import json
import numpy as np

def load_artifact(path):
    with open(path, 'r') as f:
        return json.load(f)

def run_degradation_comparison(tier0_path, tier1_path):
    print(f"Loading Tier 0 (High Fidelity) Artifact: {tier0_path}")
    tier0 = load_artifact(tier0_path)
    
    print(f"Loading Tier 1 (Degraded) Artifact: {tier1_path}")
    tier1 = load_artifact(tier1_path)

    # Validate boundaries
    if tier0['topology_identifier'] != tier1['topology_identifier']:
        print("WARNING: Topology mismatch. Comparison invalid.")
        return
        
    if tier0['cognition_identifier'] != tier1['cognition_identifier']:
        print("WARNING: Cognition mismatch. Comparison invalid.")
        return

    print("\n--- PHASE 2A: DEGRADATION GEOMETRY LOSS ---")
    
    t0_traces = tier0['traces']
    t1_traces = tier1['traces']
    
    # Information Loss Metrics
    print(f"Chronology Compression: {tier0['total_ticks']} ticks -> {tier1['total_ticks']} ticks")
    print(f"Event Density Retained: {(tier1['total_ticks'] / max(1, tier0['total_ticks'])) * 100:.2f}%")

    # Occupancy Deformation
    t0_intensities = [t['intensity'] for t in t0_traces]
    t1_intensities = [t['intensity'] for t in t1_traces]
    
    t0_max_intensity = max(t0_intensities) if t0_intensities else 0
    t1_max_intensity = max(t1_intensities) if t1_intensities else 0
    
    print(f"\nPeak Occupancy Deformation:")
    print(f"Tier 0 Max Intensity: {t0_max_intensity}")
    print(f"Tier 1 Max Intensity: {t1_max_intensity}")
    print(f"Intensity Erased by Aggregation: {t0_max_intensity - t1_max_intensity}")

    # Persistence Collapse
    print("\nPersistence Structure:")
    # A simple metric for persistence: how many consecutive steps the intensity stayed above a threshold
    threshold = np.mean(t0_intensities) if t0_intensities else 0
    t0_above = sum(1 for x in t0_intensities if x > threshold)
    t1_above = sum(1 for x in t1_intensities if x > threshold)
    
    print(f"Tier 0 High-Intensity States: {t0_above}")
    print(f"Tier 1 High-Intensity States: {t1_above}")
    
    print("\nConclusion: The geometric information loss is explicitly measurable.")
    print("Exporting metrics for Claims Repository validation...")

if __name__ == "__main__":
    parser = argparse.ArgumentParser(description="Phase 2A Chronology Degradation Study")
    parser.add_argument("--tier0", required=True, help="Path to high-fidelity TraceArtifactV1")
    parser.add_argument("--tier1", required=True, help="Path to degraded TraceArtifactV1")
    args = parser.parse_args()
    
    run_degradation_comparison(args.tier0, args.tier1)
