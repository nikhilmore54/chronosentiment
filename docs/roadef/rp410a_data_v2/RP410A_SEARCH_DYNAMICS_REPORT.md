# RP-410A — Evolutionary Search Dynamics Characterisation

## Status

**Generated automatically by `scripts/rp410a_analysis.py`.**

---

## A. Zone Distribution — All Accepted Moves

| Zone | Count | % |
|------|------:|--:|
| Peak | 1 | 0.9% |
| Shoulder | 26 | 23.2% |
| Transition | 23 | 20.5% |
| Tail | 28 | 25.0% |
| Mixed | 32 | 28.6% |
| Neutral | 2 | 1.8% |
| **Total** | **112** | 100% |

---

## B. Collapsed Basin vs Shape Competition

| Metric | Collapsed Basin | Shape Competition |
|--------|----------------:|------------------:|
| Peak % | 0.0% | 1.2% |
| Shoulder % | 20.0% | 24.4% |
| Transition % | 13.3% | 23.2% |
| Tail % | 30.0% | 23.2% |
| Mixed % | 36.7% | 25.6% |
| Neutral % | 0.0% | 2.4% |
| Avg SDI | 0.799448 | 2.035887 |
| Avg MLU | 0.533641 | 0.750547 |
| Avg Diversity | 22.11 | 20.14 |
| Avg Stagnation | 4.88 | 3.31 |

---

## C. Operator Fingerprints

| Operator | Total | Peak % | Shoulder % | Transition % | Tail % | Mixed % | Neutral % |
|----------|------:|-------:|-----------:|-------------:|-------:|--------:|----------:|
| crossover | 61 | 0.0% | 24.6% | 21.3% | 26.2% | 27.9% | 0.0% |
| crossover+mutation | 38 | 2.6% | 15.8% | 18.4% | 31.6% | 31.6% | 0.0% |
| mutation | 13 | 0.0% | 38.5% | 23.1% | 0.0% | 23.1% | 15.4% |

---

## D. Generation Summary

Total generation records: 342

| Instance | Class | Gens | Final SDI | Final MLU | Total Moves | Peak | Shoulder | Transition | Tail |
|----------|-------|-----:|----------:|----------:|------------:|-----:|---------:|-----------:|-----:|
| setA-01 | shape_competition | 91 | 1.9950 | 0.7187 | 20 | 0 | 10 | 7 | 0 |
| setA-02 | collapsed_basin | 21 | 0.0000 | null | 0 | 0 | 0 | 0 | 0 |
| setA-03 | shape_competition | 81 | 2.2578 | 0.7732 | 16 | 0 | 4 | 6 | 2 |
| setA-04 | collapsed_basin | 14 | 1.6698 | 0.5920 | 10 | 0 | 2 | 1 | 3 |
| setA-05 | collapsed_basin | 11 | 0.4331 | 0.1687 | 7 | 0 | 0 | 2 | 3 |
| setA-06 | collapsed_basin | 13 | 1.7822 | 0.5704 | 8 | 0 | 2 | 1 | 1 |
| setA-07 | collapsed_basin | 21 | 0.0000 | null | 0 | 0 | 0 | 0 | 0 |
| setA-08 | collapsed_basin | 9 | 1.4100 | 0.5576 | 5 | 0 | 2 | 0 | 2 |
| setA-09 | shape_competition | 11 | 2.2722 | 0.7303 | 7 | 0 | 0 | 1 | 3 |
| setA-10 | shape_competition | 11 | 1.9316 | 0.6511 | 8 | 0 | 1 | 2 | 2 |
| setA-11 | shape_competition | 10 | 1.8086 | 0.7186 | 7 | 0 | 0 | 2 | 4 |
| setA-12 | shape_competition | 11 | 1.3644 | 0.7600 | 7 | 0 | 3 | 0 | 1 |
| setA-13 | shape_competition | 6 | 2.4094 | 0.7949 | 4 | 1 | 1 | 0 | 0 |
| setA-14 | shape_competition | 8 | 1.7055 | 0.6001 | 4 | 0 | 1 | 0 | 2 |
| setA-15 | shape_competition | 8 | 2.6419 | 0.8620 | 5 | 0 | 0 | 0 | 3 |
| setA-16 | shape_competition | 5 | 0.0000 | null | 0 | 0 | 0 | 0 | 0 |
| setA-17 | shape_competition | 2 | 1.2432 | 0.4405 | 1 | 0 | 0 | 0 | 0 |
| setA-18 | shape_competition | 4 | 2.2419 | 0.8167 | 3 | 0 | 0 | 1 | 2 |
| setA-19 | shape_competition | 3 | 0.0000 | null | 0 | 0 | 0 | 0 | 0 |
| setA-20 | shape_competition | 2 | 0.0000 | null | 0 | 0 | 0 | 0 | 0 |

---

## E. Hypothesis Assessment

### H1 — Transition/Tail dominance

**Prediction:** ≥ 80% of accepted moves improve Transition or Tail zones.

**Observed:** Transition = 23 (20.5%), Tail = 28 (25.0%)

**Status:** TBD — compare against 80% threshold.

### H2 — Shoulder improvements rare after generation 50

**Prediction:** Shoulder move frequency drops sharply after generation 50.

**Status:** TBD — inspect `rp410a_zone_evolution.csv` for shoulder trend.

### H3 — Collapsed-basin instances never generate Peak improvements

**Observed:** Collapsed Basin Peak % = 0.0%

**Status:** TBD — compare against Shape Competition Peak %.

### H4 — Different operators produce different zone fingerprints

**Status:** TBD — inspect operator fingerprint table above.

---

## F. Data Files

| File | Contents |
|------|----------|
| `rp410a_zone_distribution.csv` | Move counts by zone, overall and per instance |
| `rp410a_zone_evolution.csv` | Per-generation zone histogram for every run |
| `rp410a_basin_comparison.csv` | Collapsed Basin vs Shape Competition aggregates |
| `rp410a_operator_fingerprints.csv` | Zone distribution per operator |

---

*End of RP-410A report.*
