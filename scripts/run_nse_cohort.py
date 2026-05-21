#!/usr/bin/env python3
"""
ChronoSentiment — NSE Cohort Ingestion Engine
======================================================
1. Loads 500 tickers alphabetically from a cohort batch file.
2. Parallel-downloads historical 5-minute bars for the past 5 days (million-tick corpus).
3. Reconstructs the exact synchronized chronological sequence of market state steps.
4. Pipes the sequence into the Rust-based live_observatory engine.
5. Computes coordinate projections, kinematics, Weibull hazard survival, and all
   5 Manifold Failure Precursors, persisting them to state_archive/raw/<symbol>/ structure.
"""

import os
import sys
import json
import math
import time
import re
import gzip
import shutil
import hashlib
import argparse
import subprocess
import concurrent.futures
from pathlib import Path
from datetime import datetime, timezone
import pandas as pd
import yfinance as yf

sys.path.insert(0, str(Path(__file__).resolve().parent))
from archive_dedupe import DedupeIndex, GzipWriterPool
from candle_substrate import load_frozen_cohort
from symbol_health import SymbolHealthRegistry, fetch_universe_parallel, health_paths

# --- Global Configurations ---
DEFAULT_ARCHIVE_ROOT = Path("state_archive")
STATE_NAMES = ["LIQUIDITY_EXHAUSTION", "NARRATIVE_PERSISTENCE", "NOISE_TRANSITIONAL"]
TEL_PATTERN = re.compile(
    r"\[TELEMETRY\]\s+(?P<ts>\d+)\s+sym=(?P<sym>\S+)\s+sig=(?P<sig>\S+)\s+margin=(?P<margin>[\d\.\-]+)\s+conv=(?P<conv>[\d\.\-]+)\s+eq=(?P<eq>[\d\.\-]+)\s+\|.*"
    r"atlas_eff=(?P<eff>[\d\.\-]+)\s+atlas_den=(?P<den>[\d\.\-]+)\s+atlas_res=(?P<res>[\d\.\-]+)\s+shadow_fert=(?P<fert>[\d\.\-]+)\s+atlas_age=(?P<age>\d+)\s*\|.*"
    r"genesis_comp=(?P<comp>[\d\.\-]+)\s+genesis_range=(?P<range>[\d\.\-]+)\s+genesis_bias=(?P<bias>[\d\.\-]+)"
)

class NSEIngestionEngine:
    def __init__(
        self,
        cohort_file: Path,
        archive_dir: Path,
        batch_id: int,
        run_label: str = "",
        interval: str = "5m",
        period: str = "5d",
        resume: bool = False,
        rebuild_dedupe: bool = False,
        from_frozen: bool = False,
        start_interval: int = 0,
        max_intervals: int | None = None,
    ):
        self.cohort_file = cohort_file
        self.archive_dir = archive_dir
        self.batch_id = batch_id
        self.run_label = run_label or f"run_{int(time.time())}"
        self.interval = interval
        self.period = period
        self.resume = resume
        self.rebuild_dedupe = rebuild_dedupe
        self.from_frozen = from_frozen
        self.start_interval = start_interval
        self.max_intervals = max_intervals
        self._frozen_manifest: dict = {}
        self.symbols = []
        self._cohort_set: set[str] = set()
        self.dedupe = DedupeIndex(archive_dir / "metadata" / "dedupe_index.json")
        self._gzip_pool = GzipWriterPool()
        self._progress_path = archive_dir / "metadata" / "ingestion_progress.json"
        h_state, h_events = health_paths(archive_dir)
        self.symbol_health = SymbolHealthRegistry(h_state, h_events)
        
        # Load PCA weights
        weights_path = Path("observatory/ecology_clustering_pca_weights.json")
        with open(weights_path) as f:
            w = json.load(f)
        self.mean = w["mean"]
        self.std = w["std"]
        self.pc1_vector = w["pc1_vector"]
        self.pc2_vector = w["pc2_vector"]
        self.centroids = w["centroids"]
        
        # State tracking caches
        self.history = {}
        self.state_trackers = {}
        # Sparse stable-state counters — persist every Nth stable tick
        self._stable_counters = {}
        _STABLE_SAMPLE_EVERY = 8  # 1-in-8 stable ticks written; events always written
        self.STABLE_SAMPLE_EVERY = _STABLE_SAMPLE_EVERY
        
        # Load symbols
        self.load_symbols()
        
    def load_symbols(self):
        if not self.cohort_file.exists():
            print(f"❌ Cohort file not found: {self.cohort_file}")
            sys.exit(1)
        with open(self.cohort_file) as f:
            self.symbols = [line.strip() for line in f if line.strip()]
        self._cohort_set = set(self.symbols)
        print(f"📋 Loaded {len(self.symbols)} symbols from cohort: {self.cohort_file.name}")

    def init_dedupe_index(self) -> None:
        if self.rebuild_dedupe or (self.resume and not self.dedupe.index_path.exists()):
            print("🔁 Rebuilding dedupe index from archive gzip streams...")
            n = self.dedupe.rebuild_from_archive(self.archive_dir, self._cohort_set)
            self.dedupe.save()
            print(f"   Indexed {n:,} (symbol, ts) keys")
        elif self.resume or self.dedupe.index_path.exists():
            n = self.dedupe.load()
            print(f"📇 Dedupe index loaded: {n:,} keys")
        if self.resume and self._progress_path.exists():
            with open(self._progress_path) as f:
                prog = json.load(f)
            print(
                f"⏩ Resume hint: interval {prog.get('last_interval_index', 0)}/"
                f"{prog.get('timeline_intervals', '?')} "
                f"(ticks persisted: {prog.get('processed_ticks', 0):,})"
            )

    def download_ticker_data(self, symbol: str) -> pd.DataFrame:
        """Download yfinance candles for a single ticker with robust retries."""
        for attempt in range(3):
            try:
                df = yf.download(
                    tickers=symbol,
                    period=self.period,
                    interval=self.interval,
                    auto_adjust=True,
                    progress=False,
                    threads=False
                )
                if df is not None and not df.empty:
                    if isinstance(df.columns, pd.MultiIndex):
                        df.columns = list(df.columns.get_level_values(0))
                    else:
                        df.columns = list(df.columns)
                    # Normalize index and format column names
                    df = df.dropna()
                    df = df[~df.index.duplicated(keep='first')]
                    df = df.sort_index()
                    return df
            except Exception as e:
                time.sleep(1.0)
        return pd.DataFrame()

    def fetch_all_cohort_data(self):
        """Parallel download with symbol health — quarantined symbols skip fetch."""
        print(f"📥 Initiating concurrent dataset acquisition for {len(self.symbols)} assets...")
        hs = self.symbol_health.summary()
        if any(hs.get(k, 0) for k in ("INVALID", "QUARANTINED", "DEGRADED")):
            print(f"   Symbol health pre-flight: {hs}")

        data_by_symbol, stats = fetch_universe_parallel(
            self.symbols,
            self.symbol_health,
            interval=self.interval,
            period=self.period,
            max_workers=15,
            quorum_ratio=0.50,
        )
        print(
            f"   Acquisition: success={stats['success']}/{stats['attempted']} "
            f"quarantined_skip={stats['skipped_quarantined']} quorum={stats['quorum_met']}"
        )
        return data_by_symbol

    def project_and_classify(self, features):
        norm_features = [(features[i] - self.mean[i]) / self.std[i] for i in range(5)]
        pc1 = sum(norm_features[i] * self.pc1_vector[i] for i in range(5))
        pc2 = sum(norm_features[i] * self.pc2_vector[i] for i in range(5))
        dists = [math.sqrt((pc1 - c[0])**2 + (pc2 - c[1])**2) for c in self.centroids]
        state_id = dists.index(min(dists))
        return pc1, pc2, state_id, min(dists)

    def process_telemetry_line(self, line: str):
        m = TEL_PATTERN.search(line)
        if not m:
            return None
            
        d = m.groupdict()
        symbol = d["sym"].replace("-", "_")
        
        features = [
            float(d["range"]), float(d["bias"]), float(d["eff"]),
            float(d["comp"]), float(d["res"])
        ]
        
        pc1, pc2, state_id, dist_to_centroid = self.project_and_classify(features)
        state_name = STATE_NAMES[state_id]
        
        ts = int(d["ts"])
        
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
        
        if state_name != prev_state:
            tracker["previous_state"] = prev_state
            tracker["current_state"] = state_name
            tracker["dwell_duration"] = 1
            tracker["active_corridor_id"] = f"C_{symbol}_{ts}"
        else:
            tracker["dwell_duration"] += 1
            
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
                
                acceleration = velocity - prev_velocity
                dot_prod = dx * prev_dx + dy * prev_dy
                if velocity > 1e-6 and prev_velocity > 1e-6:
                    cos_theta = max(-1.0, min(1.0, dot_prod / (velocity * prev_velocity)))
                    turn_angle = math.degrees(math.acos(cos_theta))
                    
        is_in_corridor = dist_to_centroid > 0.65
        
        bias_val = float(d["bias"])
        conv_val = float(d["conv"])
        eff_val = float(d["eff"])
        den_val = float(d["den"])
        
        queue_pressure = bias_val * (1.0 + conv_val)
        spread_elasticity = eff_val / (den_val + 1e-5)
        
        expected_dwell = 60.0 if state_name == "LIQUIDITY_EXHAUSTION" else 45.0 if state_name == "NARRATIVE_PERSISTENCE" else 15.0
        w_alpha = 1.2
        t_scaled = float(tracker["dwell_duration"]) / expected_dwell
        hazard_rate = (w_alpha / expected_dwell) * (t_scaled ** (w_alpha - 1.0))
        survival_probability = math.exp(-(t_scaled ** w_alpha))
        
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
            
        # --- Precursor Calculations ---
        local_entropy_expansion = 0.0
        attractor_leakage_rate = 0.0
        curvature_destabilization = 0.0
        manifold_density_thinning = 1.0 - conv_val
        
        if len(hist) >= 5:
            local_entropy_expansion = entropy_val - hist[-3]["entropy"]
            attractor_leakage_rate = dist_to_centroid - hist[-3].get("dist_to_centroid", dist_to_centroid)
            
            angles_window = [pt.get("turn_angle", 0.0) for pt in hist[-4:]] + [turn_angle]
            mean_angle = sum(angles_window) / len(angles_window)
            curvature_destabilization = math.sqrt(sum((a - mean_angle)**2 for a in angles_window) / len(angles_window))
            
        persistence_decay_velocity = survival_probability * hazard_rate
            
        record = {
            "ts": ts,
            "symbol": symbol,
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
            "queue_pressure": round(queue_pressure, 6),
            "spread_elasticity": round(spread_elasticity, 4),
            "instability_type": instability_type,
            "survival_probability": round(survival_probability, 4),
            "hazard_rate": round(hazard_rate, 6),
            "precursor_decay_velocity": round(persistence_decay_velocity, 6),
            "precursor_entropy_expansion": round(local_entropy_expansion, 4),
            "precursor_density_thinning": round(manifold_density_thinning, 4),
            "precursor_curvature_destabilization": round(curvature_destabilization, 4),
            "precursor_leakage_rate": round(attractor_leakage_rate, 4)
        }
        
        hist.append(record)
        if len(hist) > 100:
            hist.pop(0)
            
        self.persist_record(symbol, record)
        return record

    def persist_record(self, symbol: str, record: dict):
        ts = int(record["ts"])
        if not self.dedupe.check_and_add(symbol, ts):
            return

        record["schema_version"] = 1

        sym_dir = self.archive_dir / "raw" / symbol
        sym_dir.mkdir(parents=True, exist_ok=True)

        # ── Layer 1 — Canonical Barrier Archive (unconditional, barrier-native) ─
        # Written for every committed (symbol, ts) that passes dedupe.
        # No semantic gating. No sampling. Replay integrity depends only on
        # chronology boundary acceptance, not ecology classification.
        # One immutable file per (symbol, ts) — trivially hashable and certifiable.
        barriers_dir = sym_dir / "barriers"
        barriers_dir.mkdir(parents=True, exist_ok=True)
        barrier_path = barriers_dir / f"{ts}.json"
        if not barrier_path.exists():
            with open(barrier_path, "w") as f:
                json.dump(record, f, sort_keys=True)

        # ── Layer 1 pointer: latest.json uncompressed for fast UI reads ──────────
        with open(sym_dir / "latest.json", "w") as f:
            json.dump(record, f)

        # ── Layer 2 — Analytics Streams (event-gated, sampled, compressed) ──────
        # These are derived from Layer 1. Replay substrate integrity does NOT
        # depend on these writes. Storage-optimized for ecology extraction.
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
                return
        else:
            self._stable_counters[symbol] = 0

        # ── Layer 2a: daily telemetry stream (event-sampled gzip) ────────────────
        day_str = datetime.fromtimestamp(record["ts"], tz=timezone.utc).strftime("%Y_%m_%d")
        gz_path = sym_dir / f"telemetry_stream_{day_str}.jsonl.gz"
        line = json.dumps(record, sort_keys=True) + "\n"
        self._gzip_pool.writeln(gz_path, line)

        # ── Layer 2b: corridor event log (gzip) ──────────────────────────────────
        if record["corridor"]:
            corr_dir = self.archive_dir / "transitions" / "corridor_events"
            self._gzip_pool.writeln(corr_dir / f"{symbol}_events.jsonl.gz", line)

        # ── Layer 2c: collapse log (gzip, entropy > 0.95 in corridor) ────────────
        if record["corridor"] and record.get("entropy", 0.0) > 0.95:
            coll_dir = self.archive_dir / "transitions" / "collapse_events"
            self._gzip_pool.writeln(coll_dir / f"{symbol}_collapses.jsonl.gz", line)

    def save_progress(
        self,
        interval_idx: int,
        total_intervals: int,
        processed_ticks: int,
        corridors_detected: int,
        ts_fingerprint: str,
    ) -> None:
        self._progress_path.parent.mkdir(parents=True, exist_ok=True)
        with open(self._progress_path, "w") as f:
            json.dump(
                {
                    "batch_id": self.batch_id,
                    "last_interval_index": interval_idx,
                    "timeline_intervals": total_intervals,
                    "timeline_fingerprint": ts_fingerprint,
                    "processed_ticks": processed_ticks,
                    "corridors_classified": corridors_detected,
                    "dedupe_keys": len(self.dedupe.seen),
                    "updated_at_utc": datetime.now(timezone.utc).isoformat(),
                },
                f,
                indent=2,
            )
        if interval_idx % 50 == 0:
            self.dedupe.save()

    def run_ingestion(self):
        self.init_dedupe_index()
        try:
            self._run_ingestion_inner()
        finally:
            self._gzip_pool.flush_all()
            self._gzip_pool.close_all()

    def load_cohort_data(self) -> dict:
        if self.from_frozen:
            print(f"📦 Loading frozen OHLC substrate for batch {self.batch_id:03d}...")
            data, self._frozen_manifest = load_frozen_cohort(self.batch_id, self.symbols)
            print(
                f"   Frozen {len(data)} symbols | fingerprint={self._frozen_manifest.get('timeline_fingerprint')} "
                f"| hash={self._frozen_manifest.get('substrate_hash')}"
            )
            return data
        return self.fetch_all_cohort_data()

    def _run_ingestion_inner(self):
        data = self.load_cohort_data()
        if not data:
            print("❌ No historical candle data downloaded. Exiting.")
            return
            
        print("🔗 Aligning timeline across all loaded assets to preserve chronosynchrony...")
        # Get all unique timestamps across all symbols
        all_timestamps = set()
        for sym, df in data.items():
            for ts in df.index:
                all_timestamps.add(int(ts.timestamp()))
        
        sorted_timestamps = sorted(list(all_timestamps))
        start = self.start_interval
        end = (
            start + self.max_intervals
            if self.max_intervals is not None
            else len(sorted_timestamps)
        )
        sorted_timestamps = sorted_timestamps[start:end]
        self._sorted_timestamps = sorted_timestamps
        if self.start_interval or self.max_intervals is not None:
            print(
                f"   Interval window: [{start}:{end}) → {len(sorted_timestamps)} barriers"
            )
        ts_fingerprint = hashlib.sha256(
            ",".join(str(t) for t in sorted_timestamps).encode()
        ).hexdigest()[:16]
        print(f"⏱️ Chronological sequence spans {len(sorted_timestamps)} intervals. Starting pipeline projection...")
        print(f"   Timeline fingerprint: {ts_fingerprint}")
        
        # Initialize rust live_observatory process
        proc = subprocess.Popen(
            ["./target/release/examples/live_observatory"],
            stdin=subprocess.PIPE,
            stdout=subprocess.PIPE,
            stderr=subprocess.PIPE,
            text=True,
            bufsize=1
        )
        
        processed_ticks = 0
        corridors_detected = 0
        
        t0 = time.time()
        for idx, ts in enumerate(sorted_timestamps, 1):
            # Construct a synchronized batch of candles at this exact timestamp
            batch = []
            for sym, df in data.items():
                dt = pd.to_datetime(ts, unit='s', utc=True)
                if dt in df.index:
                    row = df.loc[dt]
                    if isinstance(row, pd.DataFrame):
                        row = row.iloc[0]
                    
                    def get_val(k):
                        raw_val = row.get(k, row.get(k.lower(), 0.0))
                        if hasattr(raw_val, "iloc"):
                            raw_val = raw_val.iloc[0]
                        return float(raw_val)
                        
                    batch.append({
                        "symbol": sym,
                        "timestamp": ts,
                        "open": get_val("Open"),
                        "high": get_val("High"),
                        "low": get_val("Low"),
                        "close": get_val("Close"),
                        "volume": get_val("Volume")
                    })
            
            if not batch:
                continue
                
            # Pipe batch to Rust observatory stdin
            proc.stdin.write(json.dumps(batch) + "\n")
            proc.stdin.flush()
            
            # Read telemetry stdout responses (one line per symbol in batch)
            for _ in range(len(batch)):
                while True:
                    tel_line = proc.stdout.readline()
                    if not tel_line:
                        break
                    if tel_line.startswith("[TELEMETRY]"):
                        rec = self.process_telemetry_line(tel_line)
                        if rec:
                            processed_ticks += 1
                            if rec.get("corridor"):
                                corridors_detected += 1
                        break
            
            if idx % 50 == 0 or idx == len(sorted_timestamps):
                rate = processed_ticks / (time.time() - t0)
                skipped = self.dedupe.skipped
                print(
                    f"   ⏱️ Interval {idx}/{len(sorted_timestamps)} | "
                    f"Processed {processed_ticks:,} states | Corridors: {corridors_detected:,} | "
                    f"Dedupe skipped: {skipped:,} | Velocity: {rate:.1f} states/sec"
                )
                self.save_progress(idx, len(sorted_timestamps), processed_ticks, corridors_detected, ts_fingerprint)

        self.dedupe.save()
        self._gzip_pool.flush_all()
        # Gracefully shutdown subprocess
        proc.stdin.close()
        proc.terminate()

        duration = time.time() - t0
        manifest = self.write_ingestion_manifest(
            symbols_downloaded=len(data),
            processed_ticks=processed_ticks,
            corridors_detected=corridors_detected,
            duration_sec=duration,
        )
        print("\n" + "=" * 60)
        print("🏆 BATCH INGESTION COMPLETE")
        print("=" * 60)
        print(f"  Processed Symbols   : {len(data)}")
        print(f"  Total State Ticks   : {processed_ticks:,}")
        print(f"  Corridors Classified: {corridors_detected:,}")
        print(f"  Execution Time      : {duration:.2f} seconds ({processed_ticks/duration:.1f} states/sec)")
        print(f"  Substrate Location  : {self.archive_dir}/raw/")
        print(f"  Manifest            : {manifest}")
        print("=" * 60)

    def write_ingestion_manifest(
        self,
        symbols_downloaded: int,
        processed_ticks: int,
        corridors_detected: int,
        duration_sec: float,
    ) -> Path:
        weights_path = Path("observatory/ecology_clustering_pca_weights.json")
        pca_hash = hashlib.sha256(weights_path.read_bytes()).hexdigest()[:16]
        ts_list = getattr(self, "_sorted_timestamps", [])
        ts_fingerprint = hashlib.sha256(
            ",".join(str(t) for t in ts_list).encode()
        ).hexdigest()[:16]
        manifest = {
            "batch_id": self.batch_id,
            "run_label": self.run_label,
            "cohort_file": str(self.cohort_file),
            "archive_dir": str(self.archive_dir),
            "interval": self.interval,
            "period": self.period,
            "symbols_cohort": len(self.symbols),
            "symbols_downloaded": symbols_downloaded,
            "timeline_intervals": len(ts_list),
            "timeline_fingerprint": ts_fingerprint,
            "timeline_first_ts": ts_list[0] if ts_list else None,
            "timeline_last_ts": ts_list[-1] if ts_list else None,
            "processed_ticks": processed_ticks,
            "corridors_classified": corridors_detected,
            "corridor_rate": round(corridors_detected / max(processed_ticks, 1), 6),
            "duration_sec": round(duration_sec, 3),
            "states_per_sec": round(processed_ticks / max(duration_sec, 1e-6), 2),
            "pca_weights_hash": pca_hash,
            "stable_sample_every": self.STABLE_SAMPLE_EVERY,
            "dedupe_keys": len(self.dedupe.seen),
            "dedupe_skipped": self.dedupe.skipped,
            "from_frozen": self.from_frozen,
            "frozen_substrate_hash": self._frozen_manifest.get("substrate_hash"),
            "frozen_timeline_fingerprint": self._frozen_manifest.get("timeline_fingerprint"),
            "completed_at_utc": datetime.now(timezone.utc).isoformat(),
        }
        manifest_dir = self.archive_dir / "manifests"
        manifest_dir.mkdir(parents=True, exist_ok=True)
        out = manifest_dir / f"ingestion_{self.run_label}.json"
        with open(out, "w") as f:
            json.dump(manifest, f, indent=2)
        latest = self.archive_dir / "ingestion_manifest.json"
        with open(latest, "w") as f:
            json.dump(manifest, f, indent=2)
        return out


def resolve_archive_dir(batch_id: int, shared_archive: bool, run_label: str = "") -> Path:
    if shared_archive:
        return DEFAULT_ARCHIVE_ROOT
    base = DEFAULT_ARCHIVE_ROOT / "batches" / f"batch_{batch_id:03d}"
    if run_label:
        return base / "runs" / run_label
    return base


def fresh_wipe_archive(archive_dir: Path) -> None:
    """Remove hot/warm telemetry layers; preserve manifests for replay comparison."""
    for sub in ("raw", "transitions", "trajectories", "topology", "metadata"):
        path = archive_dir / sub
        if path.exists():
            shutil.rmtree(path)


def cs_ingest_binary() -> Path:
    return Path(__file__).resolve().parents[1] / "target" / "release" / "cs-ingest"


def run_frozen_via_cs_ingest(
    *,
    batch_id: int,
    cohort_file: Path,
    archive_dir: Path,
    run_label: str,
    start_interval: int,
    max_intervals: int | None,
    fresh: bool,
    resume: bool,
    rebuild_dedupe: bool,
) -> None:
    """Canonical frozen replay path — validated replay-step in Rust."""
    binary = cs_ingest_binary()
    if not binary.exists():
        print(f"❌ cs-ingest not built: {binary}", file=sys.stderr)
        print("   Run: cargo build -p cs-ingest --release", file=sys.stderr)
        sys.exit(1)

    cmd = [
        str(binary),
        "replay-step",
        "--batch-id",
        str(batch_id),
        "--cohort",
        str(cohort_file),
        "--archive",
        str(archive_dir),
        "--start-interval",
        str(start_interval),
    ]
    if max_intervals is not None:
        cmd.extend(["--max-intervals", str(max_intervals)])
    if fresh:
        cmd.append("--fresh")
    if resume:
        cmd.append("--resume")
    if rebuild_dedupe:
        cmd.append("--rebuild-dedupe")

    print("=" * 60)
    print("CHRONOSENTIMENT — FROZEN REPLAY (cs-ingest)")
    print("=" * 60)
    print(f"  Backend            : cs-ingest replay-step")
    print(f"  Archive            : {archive_dir}")
    if fresh:
        print("  Certification      : --fresh (isolated archive)")
    print("=" * 60)

    t0 = time.time()
    proc = subprocess.run(
        cmd,
        cwd=Path(__file__).resolve().parents[1],
        check=True,
        text=True,
        capture_output=True,
    )
    duration = time.time() - t0
    out = (proc.stdout or "") + (proc.stderr or "")
    if proc.stdout:
        print(proc.stdout, end="")
    if proc.stderr:
        print(proc.stderr, end="", file=sys.stderr)

    def _grab(pattern: str, default: int = 0) -> int:
        m = re.search(pattern, out)
        return int(m.group(1)) if m else default

    processed_ticks = _grab(r"persisted ticks\s*:\s*(\d+)")
    corridors = _grab(r"corridors\s*:\s*(\d+)")
    dedupe_skip = _grab(r"dedupe skipped\s*:\s*(\d+)")

    if processed_ticks == 0 and dedupe_skip > 0:
        print(
            "❌ cs-ingest wrote 0 ticks (stale dedupe). Use --fresh for certification runs.",
            file=sys.stderr,
        )
        sys.exit(1)

    symbols = [line.strip() for line in cohort_file.read_text().splitlines() if line.strip()]
    data, frozen_manifest = load_frozen_cohort(batch_id, symbols)
    all_ts = sorted(
        {int(ts.timestamp()) for df in data.values() for ts in df.index}
    )
    start = start_interval
    end = start + max_intervals if max_intervals is not None else len(all_ts)
    sorted_ts = all_ts[start:end]

    engine = NSEIngestionEngine(
        cohort_file=cohort_file,
        archive_dir=archive_dir,
        batch_id=batch_id,
        run_label=run_label,
        from_frozen=True,
        start_interval=start_interval,
        max_intervals=max_intervals,
    )
    engine._sorted_timestamps = sorted_ts
    engine._frozen_manifest = frozen_manifest
    engine.init_dedupe_index()
    manifest_path = engine.write_ingestion_manifest(
        symbols_downloaded=len(data),
        processed_ticks=processed_ticks,
        corridors_detected=corridors,
        duration_sec=duration,
    )
    manifest = json.loads(manifest_path.read_text())
    manifest["ingest_backend"] = "cs-ingest"
    manifest_path.write_text(json.dumps(manifest, indent=2))
    latest = archive_dir / "ingestion_manifest.json"
    latest.write_text(json.dumps(manifest, indent=2))

    print("\n" + "=" * 60)
    print("🏆 FROZEN REPLAY COMPLETE (cs-ingest)")
    print("=" * 60)
    print(f"  Processed ticks     : {processed_ticks:,}")
    print(f"  Corridors           : {corridors:,}")
    print(f"  Dedupe skipped      : {dedupe_skip:,}")
    print(f"  Duration            : {duration:.2f}s")
    print(f"  Manifest            : {manifest_path}")
    print("=" * 60)


if __name__ == "__main__":
    parser = argparse.ArgumentParser(description="Ingest an alphabetical cohort batch of symbols.")
    parser.add_argument("--batch-id", type=int, default=1, help="Cohort batch ID to run (default: 1)")
    parser.add_argument(
        "--fresh",
        action="store_true",
        help="Wipe isolated batch archive before ingest (required for replay verification)",
    )
    parser.add_argument(
        "--shared-archive",
        action="store_true",
        help="Write to legacy state_archive/ root (not recommended during verification)",
    )
    parser.add_argument(
        "--run-label",
        default="",
        help="Label for isolated replay under batch_NNN/runs/LABEL (optional)",
    )
    parser.add_argument(
        "--resume",
        action="store_true",
        help="Load dedupe index; skip duplicate (symbol, ts) writes (full replay still runs)",
    )
    parser.add_argument(
        "--rebuild-dedupe",
        action="store_true",
        help="Rebuild dedupe index from existing gzip streams before ingest",
    )
    parser.add_argument(
        "--from-frozen",
        action="store_true",
        help="Load OHLC from state_archive/candles/batch_NNN/ (no yfinance)",
    )
    parser.add_argument(
        "--start-interval",
        type=int,
        default=0,
        help="First barrier index in aligned timeline (for bounded parity runs)",
    )
    parser.add_argument(
        "--max-intervals",
        type=int,
        default=None,
        help="Max barriers to process from start-interval",
    )
    parser.add_argument(
        "--python-ingest",
        action="store_true",
        help="Force legacy Python ingest (default for --from-frozen: cs-ingest when built)",
    )
    args = parser.parse_args()

    if args.fresh and args.resume:
        print("❌ Cannot use --fresh and --resume together", file=sys.stderr)
        sys.exit(1)

    cohort_file = Path(f"cohorts/batch_{args.batch_id:03d}.txt")
    run_label = args.run_label
    archive_dir = resolve_archive_dir(args.batch_id, args.shared_archive, run_label)
    manifest_label = run_label or f"run_{int(time.time())}"

    if args.fresh:
        if archive_dir.exists():
            print(f"🧹 --fresh: wiping telemetry layers in {archive_dir}")
            fresh_wipe_archive(archive_dir)
        else:
            print(f"🧹 --fresh: new archive {archive_dir}")

    archive_dir.mkdir(parents=True, exist_ok=True)

    use_cs_ingest = (
        args.from_frozen
        and not args.python_ingest
        and cs_ingest_binary().exists()
    )
    if use_cs_ingest:
        run_frozen_via_cs_ingest(
            batch_id=args.batch_id,
            cohort_file=cohort_file,
            archive_dir=archive_dir,
            run_label=manifest_label,
            start_interval=args.start_interval,
            max_intervals=args.max_intervals,
            fresh=args.fresh,
            resume=args.resume,
            rebuild_dedupe=args.rebuild_dedupe,
        )
        sys.exit(0)

    if args.from_frozen and not args.python_ingest:
        print(
            "⚠️  cs-ingest not found — falling back to Python ingest. "
            "Build with: cargo build -p cs-ingest --release",
            file=sys.stderr,
        )

    engine = NSEIngestionEngine(
        cohort_file=cohort_file,
        archive_dir=archive_dir,
        batch_id=args.batch_id,
        run_label=manifest_label,
        resume=args.resume,
        rebuild_dedupe=args.rebuild_dedupe,
        from_frozen=args.from_frozen,
        start_interval=args.start_interval,
        max_intervals=args.max_intervals,
    )
    engine.run_ingestion()
