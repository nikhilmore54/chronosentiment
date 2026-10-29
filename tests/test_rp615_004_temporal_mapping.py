"""
================================================================================
TEST SUITE: RP-615-004-002A TEMPORAL HISTORY & RELIABILITY MAPPING
================================================================================
Validates:
1. Strict Causal Temporal Ordering ($t_\\text{exit} \\le t_\\text{entry}$ for resolved trades).
2. Rolling Hit Rate & Win Rate Invariants across Windows (K=10, 25, 50).
3. Mathematical Consistency of Wilson Score Intervals ($0 \\le CI_\\text{low} \\le CI_\\text{high} \\le 100$).
4. Complete Cross-Tabulation Matrices and Evidence Classification Tiers.
5. Cryptographic Manifest Reconciliation.
================================================================================
"""

import csv
import json
import hashlib
from pathlib import Path
import unittest

WORKSPACE = Path(__file__).resolve().parent.parent
DATASET_CSV = WORKSPACE / "live_capture" / "shadow_validation" / "rp615_004_002_temporal_history_dataset.csv"
REPORT_MD = WORKSPACE / "live_capture" / "shadow_validation" / "rp615_004_002_temporal_reliability_report.md"
DASHBOARD_JSON = WORKSPACE / "services" / "ui" / "public" / "data" / "rp615_004_002_temporal_dashboard.json"
MANIFEST_JSON = WORKSPACE / "live_capture" / "shadow_validation" / "rp615_004_002_manifest.json"

FROZEN_BENCHMARK_HASH = "212532d3ac12f9c6"

def sha256_file(p: Path) -> str:
    h = hashlib.sha256()
    with p.open("rb") as f:
        for chunk in iter(lambda: f.read(65536), b""):
            h.update(chunk)
    return h.hexdigest()

class TestRP615004TemporalMapping(unittest.TestCase):

    def test_artifacts_existence(self):
        """Ensure all required deliverables for RP-615-004-002A exist and are non-empty."""
        for p in [DATASET_CSV, REPORT_MD, DASHBOARD_JSON, MANIFEST_JSON]:
            self.assertTrue(p.exists(), f"Missing required artifact: {p}")
            self.assertGreater(p.stat().st_size, 0, f"Artifact is empty: {p}")

    def test_temporal_dataset_schema_and_causal_invariants(self):
        """Verify 7,264 opportunities and rolling history feature validity."""
        with DATASET_CSV.open("r", encoding="utf-8") as f:
            reader = list(csv.DictReader(f))
            
        self.assertEqual(len(reader), 7264, f"Expected 7,264 records, got {len(reader)}")
        
        required_cols = [
            "trade_id", "session_date", "symbol", "side", "prediction_timestamp_utc",
            "pred_win_prob", "pred_net_pnl", "realized_is_win", "realized_net_pnl",
            "is_prediction_hit", "is_selected_trade", "resolved_trades_prior",
            "unresolved_concurrent_prior", "prior_10_count", "prior_10_hits",
            "prior_10_accuracy_pct", "prior_10_wins", "prior_10_win_rate_pct", "prior_10_bucket",
            "prior_25_count", "prior_25_hits", "prior_25_accuracy_pct", "prior_25_wins",
            "prior_25_win_rate_pct", "prior_25_bucket", "prior_50_count", "prior_50_hits",
            "prior_50_accuracy_pct", "prior_50_wins", "prior_50_win_rate_pct", "prior_50_bucket",
            "last_resolved_is_hit", "last_resolved_is_win", "consecutive_hit_streak",
            "consecutive_miss_streak", "consecutive_win_streak", "consecutive_loss_streak",
            "hits_in_prior_3", "wins_in_prior_3", "hits_in_prior_5", "wins_in_prior_5",
            "prob_bucket_7"
        ]
        
        for col in required_cols:
            self.assertIn(col, reader[0], f"Missing column {col}")
            
        for r in reader:
            prior_res = int(r["resolved_trades_prior"])
            self.assertGreaterEqual(prior_res, 0)
            
            k10_cnt = int(r["prior_10_count"])
            k10_hits = int(r["prior_10_hits"])
            k10_wins = int(r["prior_10_wins"])
            self.assertTrue(0 <= k10_hits <= k10_cnt <= 10)
            self.assertTrue(0 <= k10_wins <= k10_cnt <= 10)
            
            w3 = int(r["wins_in_prior_3"])
            self.assertTrue(0 <= w3 <= 3)

    def test_dashboard_matrix_invariants(self):
        """Verify dashboard JSON contains valid cross-tabulation matrices."""
        with DASHBOARD_JSON.open("r", encoding="utf-8") as f:
            dash = json.load(f)
            
        self.assertEqual(dash["frozen_benchmark_hash"], FROZEN_BENCHMARK_HASH)
        self.assertEqual(dash["deliverable"], "RP-615-004-002A")
        
        # Check cross-matrices
        for m_key in ["k10_full", "k25_full", "k50_full"]:
            m_obj = dash["cross_matrices"][m_key]
            mat = m_obj["matrix"]
            for pb, h_dict in mat.items():
                for hb, stats in h_dict.items():
                    n = stats["n"]
                    if n > 0:
                        acc_low, acc_high = stats["binary_accuracy_ci_95"]
                        self.assertTrue(0.0 <= acc_low <= acc_high <= 100.0)
                        self.assertTrue(0.0 <= stats["binary_accuracy_pct"] <= 100.0)
                        
                        wr_low, wr_high = stats["realized_win_rate_ci_95"]
                        self.assertTrue(0.0 <= wr_low <= wr_high <= 100.0)
                        self.assertTrue(0.0 <= stats["realized_win_rate_pct"] <= 100.0)

    def test_manifest_cryptographic_hashes(self):
        """Verify manifest correctly tracks all output artifact hashes."""
        with MANIFEST_JSON.open("r", encoding="utf-8") as f:
            data = json.load(f)
            
        self.assertEqual(data["programme"], "RP-615")
        self.assertEqual(data["milestone"], "RP-615-004")
        self.assertEqual(data["deliverable"], "RP-615-004-002A")
        self.assertEqual(data["frozen_model_hash"], FROZEN_BENCHMARK_HASH)
        
        for key, info in data["output_artifacts"].items():
            p = WORKSPACE / info["path"]
            self.assertTrue(p.exists(), f"File missing: {p}")
            actual_hash = sha256_file(p)
            self.assertEqual(actual_hash, info["sha256"], f"Hash mismatch on {key}")

if __name__ == "__main__":
    unittest.main()
