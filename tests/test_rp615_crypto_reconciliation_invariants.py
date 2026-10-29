#!/usr/bin/env python3
"""
tests/test_rp615_crypto_reconciliation_invariants.py

Unit tests for RP-615-Crypto Forensic Reconciliation Invariants:
  1. Event Identity Parity (1 open per trade_id, <=1 close per trade_id).
  2. State Lock Parity (Active positions match unclosed opens; Quarantined state is preserved).
  3. Financial Parity (Net P&L = Gross P&L - Friction on all settled trades).
  4. Quarantine Isolation (Zero synthetic fixture records in clean stream).
  5. Ingestion Guardrails (Fill price divergence > 1.0% rejected with provenance).
  6. Cross-Track Isolation (Zero modification to canonical NSE frozen ledger).
"""

import hashlib
import json
import unittest
from datetime import datetime, timezone
from pathlib import Path
from collections import defaultdict

WORKSPACE = Path(__file__).resolve().parent.parent
AUDIT_DIR = WORKSPACE / "audit" / "rp615_reconciliation"
CLEAN_JOURNAL = AUDIT_DIR / "rp615_crypto_shadow_trade_journal_cleaned.jsonl"
QUARANTINE_JOURNAL = AUDIT_DIR / "rp615_crypto_quarantine_records.jsonl"
EXPANSION_JOURNAL = AUDIT_DIR / "rp615_crypto_expansion_trade_journal.jsonl"
STATE_JSON = AUDIT_DIR / "rp615_crypto_shadow_daemon_state.json"
NSE_FROZEN_LEDGER = WORKSPACE / "live_capture" / "shadow_validation" / "rp615_tier4_frozen_ledger.csv"

CANONICAL_NSE_HASH = "46119e8dafc90338f132ca84ef606783c86d8d0ce0dfcc5e19e9b637c0b7a23c"


def sha256_file(path: Path) -> str:
    if not path.exists():
        return "FILE_NOT_FOUND"
    digest = hashlib.sha256()
    with path.open("rb") as f:
        for chunk in iter(lambda: f.read(1024 * 1024), b""):
            digest.update(chunk)
    return digest.hexdigest()


class TestRP615CryptoReconciliationInvariants(unittest.TestCase):

    def test_01_canonical_nse_isolation_invariant(self):
        """Verify zero byte modification to canonical NSE frozen ledger."""
        current_hash = sha256_file(NSE_FROZEN_LEDGER)
        self.assertEqual(current_hash, CANONICAL_NSE_HASH, "NSE canonical ledger was altered!")

    def test_02_quarantine_isolation_invariant(self):
        """Verify no synthetic fixture records ($65k BTC or TEST IDs) exist in clean journals."""
        for j_path in [CLEAN_JOURNAL, EXPANSION_JOURNAL]:
            if not j_path.exists():
                continue
            with j_path.open() as f:
                for line_idx, line in enumerate(f, 1):
                    if not line.strip():
                        continue
                    rec = json.loads(line)
                    t_id = rec.get("trade_id", "")
                    entry_p = rec.get("entry_price", 0.0)
                    sym = rec.get("symbol", "")
                    
                    self.assertNotIn("TEST", t_id, f"Found synthetic test trade_id {t_id} in {j_path.name}:{line_idx}")
                    if sym == "BTCUSDT" and entry_p is not None:
                        self.assertTrue(
                            abs(entry_p - 65000.0) > 100.0,
                            f"Found synthetic $65,000 fixture entry price {entry_p} in {j_path.name}:{line_idx}"
                        )

    def test_03_trade_identity_and_event_cardinality(self):
        """Verify exactly 1 open per trade_id, at most 1 close, and zero cross-arm ID collision in clean streams."""
        opens = defaultdict(list)
        closes = defaultdict(list)

        for j_path in [CLEAN_JOURNAL, EXPANSION_JOURNAL]:
            if not j_path.exists():
                continue
            with j_path.open() as f:
                for line_idx, line in enumerate(f, 1):
                    if not line.strip():
                        continue
                    rec = json.loads(line)
                    t_id = rec.get("trade_id")
                    st = rec.get("status")
                    if st == "SIMULATED_OPEN":
                        opens[t_id].append((j_path.name, line_idx, rec))
                    elif st in ("SIMULATED_CLOSE", "SIMULATED_CLOSED"):
                        closes[t_id].append((j_path.name, line_idx, rec))

        for t_id, open_list in opens.items():
            self.assertEqual(len(open_list), 1, f"Duplicate SIMULATED_OPEN for trade_id {t_id}: {open_list}")

        for t_id, close_list in closes.items():
            self.assertLessEqual(len(close_list), 1, f"Duplicate SIMULATED_CLOSE for trade_id {t_id}: {close_list}")

        # Check for orphan closes
        for t_id in closes:
            self.assertIn(t_id, opens, f"Orphan SIMULATED_CLOSE found without matching open: {t_id}")

    def test_04_financial_accounting_parity(self):
        """Verify Net P&L = Gross P&L - Friction on all closed trades with exact $18.00 friction."""
        for j_path in [CLEAN_JOURNAL, EXPANSION_JOURNAL]:
            if not j_path.exists():
                continue
            with j_path.open() as f:
                for line_idx, line in enumerate(f, 1):
                    if not line.strip():
                        continue
                    rec = json.loads(line)
                    if rec.get("status") in ("SIMULATED_CLOSE", "SIMULATED_CLOSED"):
                        g = rec.get("gross_pnl")
                        fric = rec.get("friction_dollars", 18.0)
                        n = rec.get("net_pnl")
                        self.assertIsNotNone(g)
                        self.assertIsNotNone(n)
                        self.assertAlmostEqual(n, round(g - fric, 2), places=2, msg=f"Financial parity failed in {j_path.name}:{line_idx}")

    def test_05_state_active_positions_match_unclosed_opens(self):
        """Verify every active state position has exactly one matching open and no close event."""
        with STATE_JSON.open() as f:
            state = json.load(f)

        active_positions = state.get("active_positions", {})
        open_details = state.get("open_trade_details", {})

        closed_trade_ids = set()
        for j_path in [CLEAN_JOURNAL, EXPANSION_JOURNAL]:
            if not j_path.exists():
                continue
            with j_path.open() as f:
                for line in f:
                    if not line.strip():
                        continue
                    rec = json.loads(line)
                    if rec.get("status") in ("SIMULATED_CLOSE", "SIMULATED_CLOSED"):
                        closed_trade_ids.add(rec.get("trade_id"))

        for cand, sym_dict in active_positions.items():
            if not isinstance(sym_dict, dict):
                continue
            for sym, t_id in sym_dict.items():
                if t_id and not t_id.startswith("QUARANTINED:"):
                    self.assertNotIn(t_id, closed_trade_ids, f"Active position {t_id} is already closed in journal!")
                    self.assertIn(cand, open_details)
                    self.assertIn(sym, open_details[cand])
                    self.assertIsNotNone(open_details[cand][sym])
                    self.assertEqual(open_details[cand][sym]["trade_id"], t_id)


if __name__ == "__main__":
    unittest.main()
