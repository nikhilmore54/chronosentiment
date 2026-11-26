#!/usr/bin/env python3
"""
tests/test_rp615_stage3_specification.py

Unit test suite verifying the Stage 3 Layered Selection Specification.
Asserts that all 5 layers, decision dispositions, and governance boundaries are formally defined.
"""

import unittest
from pathlib import Path

WORKSPACE = Path(__file__).resolve().parent.parent
SPEC_MD = WORKSPACE / "live_capture" / "shadow_validation" / "rp615_stage3_layered_selection_specification.md"


class TestRP615Stage3Specification(unittest.TestCase):

    def test_specification_exists_and_contains_core_layers(self):
        self.assertTrue(SPEC_MD.exists(), f"Specification missing: {SPEC_MD}")
        content = SPEC_MD.read_text(encoding="utf-8")
        
        # Verify 5 layers
        self.assertIn("Layer 1: Point-in-Time Eligibility", content)
        self.assertIn("Layer 2: Predictive Evidence", content)
        self.assertIn("Layer 3: Economic Viability", content)
        self.assertIn("Layer 4: Context & Regime Stability", content)
        self.assertIn("Layer 5: Decision Disposition Engine", content)
        
        # Verify uncertainty-aware rule LCB
        self.assertIn("LCB", content)
        self.assertIn("Lower Confidence Bound", content)
        
        # Verify governance
        self.assertIn("RP-605 Baseline Control retains 100% Live Execution Authority", content)
        self.assertIn("RP-615 retains 0% Live Authority", content)
        self.assertIn("212532d3ac12f9c6", content)

    def test_stage3_1_clarification_addendum_integrity(self):
        clarif_md = WORKSPACE / "live_capture" / "shadow_validation" / "rp615_stage3_1_layered_selection_clarification.md"
        self.assertTrue(clarif_md.exists(), f"Clarification missing: {clarif_md}")
        content = clarif_md.read_text(encoding="utf-8")
        
        # Verify exposure tiering
        self.assertIn("TIER 1: CANDIDATE DEVELOPMENT PARTITION", content)
        self.assertIn("TIER 2: HISTORICAL DIAGNOSTIC COMPARATOR", content)
        self.assertIn("TIER 3: DIAGNOSTIC HOLDOUT COMPARATOR", content)
        self.assertIn("TIER 4: PROSPECTIVE SHADOW VALIDATION", content)
        
        # Verify signed breadth definition
        self.assertIn(r"CS_{\text{aligned}} = \text{side\_num} \times CS_{\text{raw}}", content)
        self.assertIn(r"CS_{\text{aligned}} \ge -0.1500", content)
        
        # Verify ablation isolation plan
        self.assertIn("Candidate 0 (Benchmark)", content)
        self.assertIn("Candidate 1A (LCB Only)", content)
        self.assertIn("Candidate 1B (LCB + Breadth)", content)
        self.assertIn("Candidate 1C (LCB + Cooldown)", content)
        self.assertIn("Candidate 2 (Full Composite)", content)

    def test_stage3_2_implementation_contract_integrity(self):
        contract_md = WORKSPACE / "live_capture" / "shadow_validation" / "rp615_stage3_2_implementation_contract.md"
        self.assertTrue(contract_md.exists(), f"Implementation contract missing: {contract_md}")
        content = contract_md.read_text(encoding="utf-8")
        
        # Verify PIT LPB vs retrospective evaluation
        self.assertIn("Lower Prediction Bound (LPB)", content)
        self.assertIn(r"LPB_{\text{PIT}}(x_i)", content)
        self.assertIn(r"\hat{\sigma}_{\text{res, OOF}}(x_i)", content)
        
        # Verify unconfounded ablation matrix
        self.assertIn("Cand 1-Point", content)
        self.assertIn("Cand 1A (LPB Only)", content)
        self.assertIn(r"Isolates marginal effect of $LPB$", content)
        
        # Verify 3-block forward CV protocol
        self.assertIn("Block 1", content)
        self.assertIn("Block 2", content)
        self.assertIn("Block 3", content)
        self.assertIn("Days 1–3", content)
        self.assertIn("Days 4–5", content)
        self.assertIn("Days 6–7", content)
        
        # Verify observable exit and concurrency
        self.assertIn("DISPOSITION_BLOCKED_CONCURRENCY", content)
        self.assertIn("DISPOSITION_BLOCKED_COOLDOWN", content)
        self.assertIn("Observable Symbol State Tracker", content)
        
        # Verify dimensionless MAE
        self.assertIn(r"\widehat{\text{MAE}}_{\text{pct}}", content)
        self.assertIn(r"\text{ATR}_{\text{norm}}", content)
        
        # Verify statistical uncertainty methods
        self.assertIn("Wilson Score 95% Confidence Interval", content)
        self.assertIn("Symbol-Session Clustered SE", content)


if __name__ == "__main__":
    unittest.main()
