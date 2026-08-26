# ROADEF 2026 — Evidence Report v1.0

**Campaign:** campaign_v1.0_verify  
**Timestamp:** 2026-08-25T05:12:46.760719+00:00  
**Solver version:** 0.1.0  
**Total runtime:** 5590.5s  

## Summary

| Metric | Value |
|--------|-------|
| Total instances | 20 |
| Valid solutions | 15 |
| Invalid solutions | 5 |

## Quality Distribution

| Class | Count |
|-------|-------|
| Good | 2 |
| Competitive | 3 |
| Weak | 5 |
| Poor | 5 |
| Invalid | 5 |

## Per-Instance Results

| # | Instance | Demands | Nodes | Links | Slots | Obj | Avg MLU | Valid | Class | Runtime (ms) | Gens |
|---|----------|---------|-------|-------|-------|-----|---------|-------|-------|-------------|------|
| 1 | setA-01 | 40 | 20 | 80 | 2 | 47.8875 | 0.7105 | ✓ | Competitive | 24004 | 108 |
| 2 | setA-02 | 45 | 30 | 150 | 2 | ∞ | ∞ | ✗ | Invalid | 12780 | 21 |
| 3 | setA-03 | 20 | 50 | 250 | 2 | 60.4592 | 0.6793 | ✓ | Weak | 26553 | 66 |
| 4 | setA-04 | 200 | 50 | 250 | 2 | 64.5612 | 0.6900 | ✓ | Weak | 30518 | 9 |
| 5 | setA-05 | 100 | 100 | 396 | 2 | 14.2151 | 0.1859 | ✓ | Good | 30794 | 9 |
| 6 | setA-06 | 500 | 100 | 500 | 2 | 57.0447 | 0.6056 | ✓ | Competitive | 135486 | 7 |
| 7 | setA-07 | 800 | 100 | 500 | 2 | 286.6887 | 0.9308 | ✓ | Poor | 217383 | 5 |
| 8 | setA-08 | 200 | 150 | 654 | 2 | 52.3681 | 0.4808 | ✓ | Competitive | 71711 | 4 |
| 9 | setA-09 | 200 | 150 | 750 | 2 | 161.4954 | 0.7304 | ✓ | Poor | 79912 | 5 |
| 10 | setA-10 | 1000 | 150 | 966 | 2 | 93.6594 | 0.7087 | ✓ | Weak | 313490 | 4 |
| 11 | setA-11 | 400 | 200 | 1000 | 2 | 116.4583 | 0.7837 | ✓ | Poor | 211681 | 6 |
| 12 | setA-12 | 400 | 200 | 898 | 2 | 20.0424 | 0.8000 | ✓ | Good | 199902 | 4 |
| 13 | setA-13 | 2000 | 200 | 1000 | 2 | ∞ | ∞ | ✗ | Invalid | 411676 | 2 |
| 14 | setA-14 | 600 | 250 | 1108 | 2 | 93.1935 | 0.5821 | ✓ | Weak | 344003 | 4 |
| 15 | setA-15 | 600 | 250 | 1250 | 2 | 256.6357 | 0.8620 | ✓ | Poor | 322906 | 4 |
| 16 | setA-16 | 4800 | 250 | 1452 | 2 | ∞ | ∞ | ✗ | Invalid | 562845 | 1 |
| 17 | setA-17 | 2000 | 300 | 1270 | 2 | 63.1032 | 0.4111 | ✓ | Weak | 341033 | 1 |
| 18 | setA-18 | 2000 | 300 | 1500 | 2 | 799263.4335 | 0.9172 | ✓ | Poor | 339134 | 1 |
| 19 | setA-19 | 6000 | 300 | 1998 | 2 | ∞ | ∞ | ✗ | Invalid | 796646 | 1 |
| 20 | setA-20 | 6000 | 400 | 2000 | 2 | ∞ | ∞ | ✗ | Invalid | 1117891 | 1 |

## M19 Acceptance Criteria

| Criterion | Status |
|-----------|--------|
| All instances load successfully | ✓ PASS |
| MOGA optimizer runs end-to-end | ✓ PASS |
| Valid solutions produced | ✓ PASS |
| Zero modifications to Qualification Subsystem v1.0 | ✓ PASS |

## Notes

- M19 baseline: uniform waypoints across all time slots (per-time-slot optimization is Phase IV)
- Quality classes are ROADEF-specific (Excellent/Good/Competitive/Weak/Poor based on objective value)
- No published BKS available for ROADEF 2026 setA; quality class is absolute, not gap-based
- M20 will add Qualification Subsystem integration (FCF/FCS/FUC-001/ExecutionCertificate)
