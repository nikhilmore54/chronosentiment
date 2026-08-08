# ROADEF 2026 — Evidence Report v1.0

**Campaign:** campaign_v1.0_verify  
**Timestamp:** 2026-08-05T15:23:35.347061+00:00  
**Solver version:** 0.1.0  
**Total runtime:** 3879.4s  

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
| Weak | 6 |
| Poor | 4 |
| Invalid | 5 |

## Per-Instance Results

| # | Instance | Demands | Nodes | Links | Slots | Obj | Avg MLU | Valid | Class | Runtime (ms) | Gens |
|---|----------|---------|-------|-------|-------|-----|---------|-------|-------|-------------|------|
| 1 | setA-01 | 40 | 20 | 80 | 2 | 48.0436 | 0.7105 | ✓ | Competitive | 8585 | 51 |
| 2 | setA-02 | 45 | 30 | 150 | 2 | ∞ | ∞ | ✗ | Invalid | 1767 | 21 |
| 3 | setA-03 | 20 | 50 | 250 | 2 | 60.3802 | 0.7732 | ✓ | Weak | 16307 | 61 |
| 4 | setA-04 | 200 | 50 | 250 | 2 | 63.5893 | 0.6900 | ✓ | Weak | 30534 | 14 |
| 5 | setA-05 | 100 | 100 | 396 | 2 | 13.6289 | 0.1687 | ✓ | Good | 30785 | 11 |
| 6 | setA-06 | 500 | 100 | 500 | 2 | 54.4458 | 0.6408 | ✓ | Competitive | 135769 | 13 |
| 7 | setA-07 | 800 | 100 | 500 | 2 | ∞ | ∞ | ✗ | Invalid | 79481 | 21 |
| 8 | setA-08 | 200 | 150 | 654 | 2 | 53.7654 | 0.6313 | ✓ | Competitive | 72686 | 9 |
| 9 | setA-09 | 200 | 150 | 750 | 2 | 157.5633 | 0.7304 | ✓ | Poor | 77368 | 10 |
| 10 | setA-10 | 1000 | 150 | 966 | 2 | 86.1218 | 0.6283 | ✓ | Weak | 330622 | 10 |
| 11 | setA-11 | 400 | 200 | 1000 | 2 | 113.5486 | 0.7274 | ✓ | Poor | 204604 | 9 |
| 12 | setA-12 | 400 | 200 | 898 | 2 | 24.6694 | 0.8800 | ✓ | Good | 192790 | 10 |
| 13 | setA-13 | 2000 | 200 | 1000 | 2 | 73.8149 | 0.8504 | ✓ | Weak | 328788 | 7 |
| 14 | setA-14 | 600 | 250 | 1108 | 2 | 90.1268 | 0.6306 | ✓ | Weak | 340942 | 9 |
| 15 | setA-15 | 600 | 250 | 1250 | 2 | 248.0708 | 0.8620 | ✓ | Poor | 334876 | 9 |
| 16 | setA-16 | 4800 | 250 | 1452 | 2 | ∞ | ∞ | ✗ | Invalid | 305703 | 5 |
| 17 | setA-17 | 2000 | 300 | 1270 | 2 | 62.3806 | 0.4462 | ✓ | Weak | 311920 | 2 |
| 18 | setA-18 | 2000 | 300 | 1500 | 2 | 799251.4530 | 0.8602 | ✓ | Poor | 415784 | 8 |
| 19 | setA-19 | 6000 | 300 | 1998 | 2 | ∞ | ∞ | ✗ | Invalid | 314284 | 3 |
| 20 | setA-20 | 6000 | 400 | 2000 | 2 | ∞ | ∞ | ✗ | Invalid | 345705 | 2 |

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
