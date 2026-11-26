#!/usr/bin/env python3
"""
ChronoSentiment Telemetry Archiver Daemon
Monitors per-symbol live logs, parses telemetry lines, projects coordinates
onto the global PCA state space, classifies attractors, calculates kinematics,
and records longitudinal memory to state_archive/ raw, trajectories, and transitions.
"""

import os
import re
import sys
import json
import time
import math
import gzip
from pathlib import Path
from datetime import datetime, timezone

# Set up root path
_ROOT = Path(__file__).resolve().parents[1]
sys.path.insert(0, str(_ROOT))

# Configuration
LOG_DIR = _ROOT / "analysis" / "live_multi"
ARCHIVE_DIR = _ROOT / "state_archive"
WEIGHTS_PATH = _ROOT / "observatory" / "ecology_clustering_pca_weights.json"

# State and taxonomy definitions
STATE_NAMES = {
    0: "LIQUIDITY_EXHAUSTION",
    1: "NARRATIVE_PERSISTENCE",
    2: "NOISE_TRANSITIONAL"
}

# Regex for parsing the exact TELEMETRY log lines printed by live observatory
tel_pattern = re.compile(
    r"\[TELEMETRY\]\s+(?P<time>[\d:]+)\s+sym=(?P<sym>[A-Z\-_]+)\s+sig=(?P<sig>[A-Z_]+)\s+margin=(?P<margin>[\d\.\-]+)\s+conv=(?P<conv>[\d\.\-]+)\s+eq=(?P<eq>[\d\.\-]+).*?"
    r"legacy_exp=(?P<legacy_exp>[\d\.\-]+)\s+micro_exp=(?P<micro_exp>[\d\.\-]+)\s+gross=(?P<gross>[\d\.\-]+)\s+noise=(?P<noise>[\d\.\-]+).*?"
    r"atlas_eff=(?P<eff>[\d\.\-]+)\s+atlas_den=(?P<den>[\d\.\-]+)\s+atlas_res=(?P<res>[\d\.\-]+).*?"
    r"shadow_fert=(?P<shadow_fert>[\d\.\-]+)\s+atlas_age=(?P<age>\d+).*?"
    r"genesis_comp=(?P<comp>[\d\.\-]+)\s+genesis_range=(?P<range>[\d\.\-]+)\s+genesis_bias=(?P<bias>[\d\.\-]+)"
)

class TelemetryArchiver:
    def __init__(self):
        self.load_pca_weights()
        self.ensure_directory_structure()
        self.history = {}        # {symbol: [trajectory_points]}
        self.state_trackers = {} # {symbol: {current_state, duration, previous_state}}
        self._stable_counters = {}  # sparse sampling: 1-in-8 stable ticks written
        self.STABLE_SAMPLE_EVERY = 8

    def load_pca_weights(self):
        print(f"📖 Loading PCA Weights from {WEIGHTS_PATH}...")
        with open(WEIGHTS_PATH) as f:
            w = json.load(f)
            self.mean = np_array(w["mean"])
            self.std = np_array(w["std"])
            self.pc1_vector = np_array(w["pc1_vector"])
            self.pc2_vector = np_array(w["pc2_vector"])
            self.centroids = [np_array(c) for c in w["centroids"]]
        print("✅ PCA Weights loaded successfully.")

    def ensure_directory_structure(self):
        # Create standard structure as recommended
        for folder in [
            "raw",
            "trajectories/daily",
            "trajectories/weekly",
            "trajectories/event_windows",
            "transitions/corridor_events",
            "transitions/collapse_events",
            "transitions/persistence_events",
            "topology/manifold_snapshots",
            "topology/centroid_drift",
            "topology/entropy_surfaces",
            "metadata/observatory_versions",
            "metadata/replay_parameters",
            "metadata/taxonomy_definitions"
        ]:
            path = ARCHIVE_DIR / folder
            path.mkdir(parents=True, exist_ok=True)
        
        # Write metadata taxonomies
        tax_path = ARCHIVE_DIR / "metadata" / "taxonomy_definitions" / "state_schema.json"
        with open(tax_path, "w") as f:
            json.dump({
                "states": STATE_NAMES,
                "centroids": self.centroids,
                "features": ["range", "bias", "eff", "comp", "res"],
                "version": "1.2.0"
            }, f, indent=4)
        print("📁 Directory structure verified and taxonomy metadata registered.")

    def project_and_classify(self, features):
        # 1. Normalize
        norm_features = []
        for i, val in enumerate(features):
            norm_features.append((val - self.mean[i]) / self.std[i])
        
        # 2. Project
        pc1 = sum(norm_features[i] * self.pc1_vector[i] for i in range(5))
        pc2 = sum(norm_features[i] * self.pc2_vector[i] for i in range(5))
        
        # 3. Classify Attractor
        dists = []
        for centroid in self.centroids:
            dists.append(math.sqrt((pc1 - centroid[0])**2 + (pc2 - centroid[1])**2))
        
        state_id = dists.index(min(dists))
        return pc1, pc2, state_id, min(dists)

    def process_telemetry_line(self, line):
        m = tel_pattern.search(line)
        if not m:
            return None
        
        d = m.groupdict()
        symbol = d["sym"].replace("-", "_")
        
        # Parse features
        features = [
            float(d["range"]),
            float(d["bias"]),
            float(d["eff"]),
            float(d["comp"]),
            float(d["res"])
        ]
        
        pc1, pc2, state_id, dist_to_centroid = self.project_and_classify(features)
        state_name = STATE_NAMES[state_id]
        
        # Timestamp matching
        ts = int(time.time())
        
        # Initialize tracking for symbol if not present
        if symbol not in self.history:
            self.history[symbol] = []
        if symbol not in self.state_trackers:
            self.state_trackers[symbol] = {
                "current_state": state_name,
                "dwell_duration": 0,
                "previous_state": "UNINITIALIZED",
                "active_corridor_id": None
            }
            
        tracker = self.state_trackers[symbol]
        prev_state = tracker["current_state"]
        
        # State transition tracking logic
        if state_name != prev_state:
            tracker["previous_state"] = prev_state
            tracker["current_state"] = state_name
            tracker["dwell_duration"] = 1
            # Mark corridor transition
            tracker["active_corridor_id"] = f"C_{symbol}_{ts}"
        else:
            tracker["dwell_duration"] += 1
            
        # Kinematics calculations
        velocity = 0.0
        acceleration = 0.0
        turn_angle = 0.0
        
        hist = self.history[symbol]
        if len(hist) >= 1:
            prev_pt = hist[-1]
            dx = pc1 - prev_pt["pc1"]
            dy = pc2 - prev_pt["pc2"]
            velocity = math.sqrt(dx*dx + dy*dy)
            
            if len(hist) >= 2:
                prev_prev_pt = hist[-2]
                prev_dx = prev_pt["pc1"] - prev_prev_pt["pc1"]
                prev_dy = prev_pt["pc2"] - prev_prev_pt["pc2"]
                prev_velocity = math.sqrt(prev_dx*prev_dx + prev_dy*prev_dy)
                
                # Acceleration
                acceleration = velocity - prev_velocity
                
                # Turn Angle
                dot_prod = dx * prev_dx + dy * prev_dy
                mag_curr = velocity
                mag_prev = prev_velocity
                if mag_curr > 1e-6 and mag_prev > 1e-6:
                    cos_theta = max(-1.0, min(1.0, dot_prod / (mag_curr * mag_prev)))
                    turn_angle = math.degrees(math.acos(cos_theta))
                    
        # Corridor detection (when far from any stable attractor core)
        is_in_corridor = dist_to_centroid > 0.65
        
        # --- Hidden-State Instrumentation & Corridor Genesis Taxonomy ---
        bias_val = float(d["bias"])
        conv_val = float(d["conv"])
        eff_val = float(d["eff"])
        den_val = float(d["den"])
        
        queue_pressure = bias_val * (1.0 + conv_val)
        spread_elasticity = eff_val / (den_val + 1e-5)
        
        # State decay expectation and survival calculation (Weibull / Exponential)
        # Expected dwell time constants: LIQUIDITY_EXHAUSTION = 60, NARRATIVE_PERSISTENCE = 45, NOISE_TRANSITIONAL = 15
        expected_dwell = 60.0 if state_name == "LIQUIDITY_EXHAUSTION" else 45.0 if state_name == "NARRATIVE_PERSISTENCE" else 15.0
        
        # Hazard Rate: Weibull shape alpha=1.2 (aging effect)
        w_alpha = 1.2
        t_scaled = float(tracker["dwell_duration"]) / expected_dwell
        hazard_rate = (w_alpha / expected_dwell) * (t_scaled ** (w_alpha - 1.0))
        survival_probability = math.exp(-(t_scaled ** w_alpha))
        
        # Corridor Genesis Taxonomy
        entropy_val = float(d["eq"])
        if is_in_corridor:
            if entropy_val > 0.85 and velocity > 0.05 and turn_angle > 90.0:
                instability_type = "HARD_INSTABILITY"
            elif entropy_val > 0.85 and velocity <= 0.05:
                instability_type = "TOPOLOGY_FRAGMENTATION"
            elif entropy_val <= 0.85 and velocity > 0.05:
                instability_type = "CORRIDOR_MIGRATION"
            else:
                instability_type = "ATTRACTOR_LEAKAGE"
        elif tracker["previous_state"] != "UNINITIALIZED" and tracker["previous_state"] != state_name:
            instability_type = "RECOVERY"
        else:
            instability_type = "STABLE"
            
        # --- Transition Precursor Signals (Manifold Failure Mechanics) ---
        local_entropy_expansion = 0.0
        attractor_leakage_rate = 0.0
        curvature_destabilization = 0.0
        manifold_density_thinning = 1.0 - conv_val
        
        if len(hist) >= 5:
            # 1. Local Entropy Expansion: 3-tick delta of boundary entropy
            local_entropy_expansion = entropy_val - hist[-3]["entropy"]
            
            # 2. Attractor Leakage Rate: 3-tick delta of centroid distance
            attractor_leakage_rate = dist_to_centroid - hist[-3].get("dist_to_centroid", dist_to_centroid)
            
            # 3. Curvature Destabilization: standard deviation of turn angles over last 5 ticks
            angles_window = [pt.get("turn_angle", 0.0) for pt in hist[-4:]] + [turn_angle]
            mean_angle = sum(angles_window) / len(angles_window)
            curvature_destabilization = math.sqrt(sum((a - mean_angle)**2 for a in angles_window) / len(angles_window))
            
        # 4. Persistence Decay Velocity: Weibull probability density function (PDF of exit)
        persistence_decay_velocity = survival_probability * hazard_rate
            
        record = {
            "ts": ts,
            "symbol": d["sym"],
            "pc1": round(pc1, 4),
            "pc2": round(pc2, 4),
            "dist_to_centroid": round(dist_to_centroid, 4),
            "state": state_name,
            "entropy": round(float(d["eq"]), 4),
            "velocity": round(velocity, 4),
            "acceleration": round(acceleration, 4),
            "turn_angle": round(turn_angle, 2),
            "transition_confidence": round(float(d["margin"]), 4),
            "local_density": round(float(d["conv"]), 4),
            "corridor": is_in_corridor,
            "previous_state": tracker["previous_state"],
            "next_state": "STABLE" if not is_in_corridor else "TRANSITIONAL",
            "dwell_duration": tracker["dwell_duration"],
            "corridor_id": tracker["active_corridor_id"] if is_in_corridor else None,
            
            # --- Latent & Hidden-State Instrumentation ---
            "queue_pressure": round(queue_pressure, 6),
            "spread_elasticity": round(spread_elasticity, 4),
            "instability_type": instability_type,
            "survival_probability": round(survival_probability, 4),
            "hazard_rate": round(hazard_rate, 6),
            
            # --- Transition Precursor Signals (Manifold Failure Mechanics) ---
            "precursor_decay_velocity": round(persistence_decay_velocity, 6),
            "precursor_entropy_expansion": round(local_entropy_expansion, 4),
            "precursor_density_thinning": round(manifold_density_thinning, 4),
            "precursor_curvature_destabilization": round(curvature_destabilization, 4),
            "precursor_leakage_rate": round(attractor_leakage_rate, 4)
        }
        
        # Keep 100 points of trailing history in memory
        hist.append(record)
        if len(hist) > 100:
            hist.pop(0)
            
        # 4. Save to files
        self.persist_record(symbol, record)
        return record

    def persist_record(self, symbol, record):
        # ── Tier-gate: only sparse-sample stable ticks; always write events ───
        is_event = (
            record["instability_type"] != "STABLE"
            or record["corridor"]
            or record.get("precursor_entropy_expansion", 0.0) > 0.05
            or record.get("precursor_curvature_destabilization", 0.0) > 15.0
        )
        if not is_event:
            cnt = self._stable_counters.get(symbol, 0) + 1
            self._stable_counters[symbol] = cnt
            if cnt % self.STABLE_SAMPLE_EVERY != 0:
                sym_dir = ARCHIVE_DIR / "raw" / symbol
                sym_dir.mkdir(parents=True, exist_ok=True)
                with open(sym_dir / "latest.json", "w") as f:
                    json.dump(record, f)
                return
        else:
            self._stable_counters[symbol] = 0

        sym_dir = ARCHIVE_DIR / "raw" / symbol
        sym_dir.mkdir(parents=True, exist_ok=True)

        # ── Tier A (Hot): daily gzip rotation ───────────────────────────────
        day_str = datetime.fromtimestamp(record["ts"], tz=timezone.utc).strftime("%Y_%m_%d")
        gz_path = sym_dir / f"telemetry_stream_{day_str}.jsonl.gz"
        line = json.dumps(record) + "\n"
        with gzip.open(gz_path, "at", encoding="utf-8") as gz:
            gz.write(line)

        # ── Tier A pointer: latest.json uncompressed for fast UI reads ───────
        with open(sym_dir / "latest.json", "w") as f:
            json.dump(record, f)

        # ── Tier B (Warm): corridor event log ───────────────────────────
        if record["corridor"]:
            corr_dir = ARCHIVE_DIR / "transitions" / "corridor_events"
            corr_dir.mkdir(parents=True, exist_ok=True)
            with open(corr_dir / f"{symbol}_events.jsonl", "a") as f:
                f.write(line)

        # ── Tier B (Warm): collapse log ─────────────────────────────────
        if record["corridor"] and record.get("entropy", 0.0) > 0.95:
            coll_dir = ARCHIVE_DIR / "transitions" / "collapse_events"
            coll_dir.mkdir(parents=True, exist_ok=True)
            with open(coll_dir / f"{symbol}_collapses.jsonl", "a") as f:
                f.write(line)

def np_array(lst):
    return lst # Simple mapping since NumPy is not strictly needed for scalar vector multiplication

def tail_files():
    archiver = TelemetryArchiver()
    print("📡 Telemetry Persistence Archiver Daemon started successfully.")
    print(f"👀 Monitoring per-symbol live logs under {LOG_DIR}...")
    
    file_positions = {} # {path: last_size}
    
    while True:
        # Scan for live logs under analysis/live_multi/
        # They alternate between A and B, so we can monitor all A/B and symlink files
        log_files = list(LOG_DIR.glob("live_*.log*"))
        if not log_files:
            time.sleep(0.5)
            continue
            
        for path in log_files:
            # Skip symlink base files if they lead to duplicate reads of rotated variants
            if not path.is_file() or path.is_symlink():
                continue
                
            p_str = str(path)
            current_size = path.stat().st_size
            
            if p_str not in file_positions:
                print(f"📖 Backfilling historical telemetry (last 1000 lines) from {path.name}...")
                try:
                    with open(path, "r", errors="ignore") as f:
                        lines = f.readlines()
                    for line in lines[-1000:]:
                        if "[TELEMETRY]" in line:
                            try:
                                archiver.process_telemetry_line(line)
                            except Exception:
                                pass
                except Exception as e:
                    print(f"⚠️ Failed to backfill {path.name}: {e}")
                file_positions[p_str] = current_size
                continue
                
            last_size = file_positions[p_str]
            if current_size < last_size:
                # File rotated! Reset position to 0
                file_positions[p_str] = 0
                last_size = 0
                
            if current_size > last_size:
                # Read new data
                with open(path, "r", errors="ignore") as f:
                    f.seek(last_size)
                    new_lines = f.readlines()
                    file_positions[p_str] = f.tell()
                    
                    for line in new_lines:
                        if "[TELEMETRY]" in line:
                            try:
                                rec = archiver.process_telemetry_line(line)
                                # Only log non-stable events to suppress per-tick clutter
                                if rec and (rec["corridor"] or rec["instability_type"] != "STABLE"):
                                    print(f"[EVENT] sym={rec['symbol']} {rec['instability_type']} state={rec['state']} corridor={rec['corridor']}")
                            except Exception as e:
                                print(f"⚠️ Parse error: {e}", file=sys.stderr)
                                
        time.sleep(0.2)

if __name__ == "__main__":
    try:
        tail_files()
    except KeyboardInterrupt:
        print("\n👋 Telemetry Archiver Daemon stopped cleanly.")
        sys.exit(0)
