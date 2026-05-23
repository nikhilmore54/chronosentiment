# Archive Ingestion Justification: 2026 Semiconductor Stability Retests (Phase 2E-M)

**Universe Targets:** 
- `2026_nvda_sync_failed_cont_shift_neg_10m_5m`, `2026_amd_sync_failed_cont_shift_neg_10m_5m`
- `2026_nvda_sync_failed_cont_shift_pos_10m_5m`, `2026_amd_sync_failed_cont_shift_pos_10m_5m`
- `2026_nvda_sync_failed_cont_shift_pos_20m_5m`, `2026_amd_sync_failed_cont_shift_pos_20m_5m`
**Date:** 2026-05-23

## Rule 12 Replay Metadata Snapshot
- **Session Type:** Bounded (Continuous intraday session)
- **Replay Class:** Replay Stability Retest (Perturbed Extraction Windows)
- **Authority Type:** Composite (Yahoo Presentation)
- **Chronology Density:** 5m Presentation

## 1. What pressure class?
`replay reproducibility pressure` / `window-local robustness`

## 2. What recurrence axis?
Stability Retest. Testing the lateral exhaustion divergence pair (`2026_semi_failed_cont_5m`) by slightly perturbing the temporal extraction window (-10m, +10m, +20m) while keeping the same macroscopic structure.

## 3. What existing assumption does it pressure?
Pressures the assumption that observed continuity geometry is robust and not just an artifact of a specific, overfit 60-tick window. If shifting the window by 2-4 ticks (10-20 minutes) completely changes the geometry for NVDA (from preservation to fracture) or AMD (from fracture to preservation), then the observation is brittle and lacks replay-local robustness. If the divergence geometry holds, it confirms the persistence mapping is stable within the local ecology.

## 4. What makes it phenomenologically distinct?
These replay surfaces are deliberately near-isomorphic to Claim 017 but temporally offset. This isolates the chronometry boundaries themselves as the independent variable.
