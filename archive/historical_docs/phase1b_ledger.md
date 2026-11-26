# Phase 1B Ledger

This ledger records the chronological progression of Phase 1B (Ecological Validation).

---

## Entry 1 — Q1 Discovery (Phase 1A Complete)

**Date:** 2025 Q1  
**Status:** Discovery complete

Non-random ecological structure detected in Q1 session-metric space (NIFTY + BANKNIFTY, 120 sessions).

- Best k: 2 (silhouette 0.357, p = 0.032)
- Bootstrap ARI: 0.563 ± 0.234
- Perturbation ARI (σ=0.02, single trial): 0.455
- Cluster distribution: {0: 90, 1: 30}

**Conclusion:** Phenomenon provisionally detected. Independent-quarter replication required.

---

## Entry 2 — Phase 1B Provisional Complete

**Date:** 2026-06-02  
**Status:** Provisionally complete

- Pipeline frozen: `build_session_catalog.py`, `validate_ecologies.py`, `ecology_utils.py`
- Repository tagged: `phase1b_provisional_complete`
- Governance artifacts created: replication report template, this ledger
- Q2 data acquisition path confirmed: `scripts/kite_batch_historical.py`

**Conclusion:** Experimental apparatus certified. Awaiting Q2 data for replication.

---

## Entry 3 — Q2 Data Acquisition

**Date:** 2026-06-03  
**Status:** Complete

- Period: 2025-04-01 → 2025-06-30
- Trading days: 65
- Sessions validated: 122 (all valid)
- Catalog entries: 120 (after NaN drop on gap_pct for first-day sessions)
- Acquisition script: `scripts/kite_batch_historical.py` (frozen)
- Validation script: `scripts/validate_corpus.py` (frozen)

**Artifacts:**
- `historical_capture/batch_q2/` (65 day-folders)
- `phase1/analysis/coordinate_audit/session_catalog_q2.json` (120 entries)

---

## Entry 4 — Q2 Ecological Validation

**Date:** 2026-06-03  
**Status:** Complete

Ran `validate_ecologies.py` with Q2 catalog. Identical parameters to Q1 (k=2–10, Ward, 30 nulls, 30 bootstraps).

### Code changes to `validate_ecologies.py`

| Change | Affects science? |
|--------|-----------------|
| Added `--catalog`, `--output-dir`, `--k-range` CLI args | No — entry-point only |
| Added `OUTPUT_DIR.mkdir(parents=True, exist_ok=True)` | No — filesystem convenience |

No clustering, bootstrap, null-model, perturbation, or metric logic was altered.

### Results (auto-selected best-k)

| Metric | Q1 (k=2) | Q2 (k=3) |
|--------|-----------|-----------|
| Silhouette | 0.357 | 0.385 |
| Silhouette p | 0.032 | 0.032 |
| Bootstrap ARI | 0.563 ± 0.234 | 0.698 ± 0.191 |
| Perturbation ARI (single trial) | 0.455 | 0.967 |

**Artifacts:**
- `phase1/analysis/validation_q2/archive/datasets/ecology_certification.json`
- `phase1/analysis/validation_q2/archive/research_outputs/cluster_stability_report.md`
- `phase1/analysis/validation_q2/archive/datasets/null_model_comparison.json`
- `phase1/analysis/validation_q2/archive/research_outputs/cluster_stability_plot.png`

---

## Entry 5 — Perturbation ARI Audit

**Date:** 2026-06-03  
**Status:** Complete

The Q2 perturbation ARI of 0.967 was flagged as unusually high. A 10-trial audit was conducted.

### Findings

- No bugs in perturbation logic
- Identical procedure between Q1 and Q2
- No label reuse or path contamination
- The 0.967 was a legitimate high draw from a distribution with substantial variance

### Corrected values (10 trials, σ=0.02)

| Quarter | Single-trial | Multi-trial mean |
|---------|-------------|------------------|
| Q1 k=2 | 0.455 | 0.622 ± 0.194 |
| Q2 k=3 | 0.967 | 0.827 ± 0.152 |

### Recommendation

Perturbation procedure should report mean ± std across ≥10 trials per σ in all future runs.

---

## Entry 6 — Micro-cluster Investigation

**Date:** 2026-06-03  
**Status:** Complete

Q2 k=3 contains a 2-session micro-cluster (Cluster 2):

| Session | Date | Symbol | gap_pct (std) | Significance |
|---------|------|--------|---------------|-------------|
| 6 | 2025-04-04 | NIFTY | -5.6σ | Pre-crash session |
| 7 | 2025-04-04 | BANKNIFTY | -6.1σ | Pre-crash session |

2025-04-04 was the final session before the April 7 crash (Nifty -5%). These are genuine market-event outliers.

### Sensitivity check

| Without micro-cluster pair | k=2 silhouette | k=3 silhouette |
|---------------------------|----------------|----------------|
| 118 sessions | **0.379** | 0.361 |

**k=2 becomes preferred when the outlier pair is removed.** The k-shift is driven by 2 extreme sessions, not a genuine third ecology.

---

## Entry 7 — Phase 1B Closure

**Date:** 2026-06-03  
**Status:** CLOSED

### Formal conclusion

Independent-quarter replication supports the existence of non-random ecological structure in session-metric space.

### Evidence summary

| Criterion | Q1 | Q2 | Verdict |
|-----------|-----|-----|---------|
| Non-random structure | Yes (p=0.032) | Yes (p=0.032) | **Replicates** |
| Signal strength | Silhouette 0.357 | Silhouette 0.385 | **Improves** |
| Bootstrap stability | ARI 0.563 | ARI 0.698 | **Improves** |
| Perturbation robustness | ARI 0.622 (audited) | ARI 0.827 (audited) | **Improves** |
| Dominant partition | k=2 | k=2 (k=3 outlier-driven) | **Stable** |

### Governance notes

- Scientific instrument unchanged between Q1 and Q2
- Perturbation single-trial weakness documented; multi-trial reporting recommended
- k=3 preference in Q2 explained by 2 extreme outlier sessions (2025-04-04)
- Future characterization should use ecological coordinates, not fixed cluster labels
- Q3 replication becomes additional evidence, not a blocker

### Repository checkpoint

Tag: `phase1b_closed`
