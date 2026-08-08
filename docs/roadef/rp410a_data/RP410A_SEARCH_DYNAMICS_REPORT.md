# RP-410A — Evolutionary Search Dynamics Characterisation

## Status

**Generated automatically by `scripts/rp410a_analysis.py`.**

---

## A. Zone Distribution — All Accepted Moves

| Zone | Count | % |
|------|------:|--:|
| Peak | 0 | 0.0% |
| Shoulder | 31 | 23.5% |
| Transition | 29 | 22.0% |
| Tail | 38 | 28.8% |
| Mixed | 33 | 25.0% |
| Neutral | 1 | 0.8% |
| **Total** | **132** | 100% |

---

## B. Collapsed Basin vs Shape Competition

| Metric | Collapsed Basin | Shape Competition |
|--------|----------------:|------------------:|
| Peak % | 0.0% | 0.0% |
| Shoulder % | 17.1% | 26.4% |
| Transition % | 14.6% | 25.3% |
| Tail % | 24.4% | 30.8% |
| Mixed % | 43.9% | 16.5% |
| Neutral % | 0.0% | 1.1% |
| Avg SDI | 0.837461 | 1.892829 |
| Avg MLU | 0.531647 | 0.734482 |
| Avg Diversity | 23.82 | 21.04 |
| Avg Stagnation | 4.46 | 3.83 |

---

## C. Operator Fingerprints

| Operator | Total | Peak % | Shoulder % | Transition % | Tail % | Mixed % | Neutral % |
|----------|------:|-------:|-----------:|-------------:|-------:|--------:|----------:|
| initial | 132 | 0.0% | 23.5% | 22.0% | 28.8% | 25.0% | 0.8% |

---

## D. Generation Summary

Total generation records: 356

| Instance | Class | Gens | Final SDI | Final MLU | Total Moves | Peak | Shoulder | Transition | Tail |
|----------|-------|-----:|----------:|----------:|------------:|-----:|---------:|-----------:|-----:|
| setA-01 | shape_competition | 62 | 1.9443 | 0.7105 | 20 | 0 | 9 | 5 | 3 |
| setA-02 | collapsed_basin | 21 | 0.0000 | null | 0 | 0 | 0 | 0 | 0 |
| setA-03 | shape_competition | 92 | 2.2387 | 0.6793 | 13 | 0 | 3 | 6 | 2 |
| setA-04 | collapsed_basin | 16 | 1.6771 | 0.6025 | 9 | 0 | 0 | 3 | 2 |
| setA-05 | collapsed_basin | 13 | 0.4484 | 0.1759 | 11 | 0 | 3 | 2 | 3 |
| setA-06 | collapsed_basin | 15 | 1.4845 | 0.4859 | 14 | 0 | 4 | 1 | 3 |
| setA-07 | collapsed_basin | 21 | 0.0000 | null | 0 | 0 | 0 | 0 | 0 |
| setA-08 | collapsed_basin | 11 | 1.2089 | 0.3797 | 7 | 0 | 0 | 0 | 2 |
| setA-09 | shape_competition | 12 | 2.3833 | 0.7304 | 10 | 0 | 1 | 3 | 5 |
| setA-10 | shape_competition | 12 | 1.8645 | 0.5913 | 9 | 0 | 4 | 2 | 2 |
| setA-11 | shape_competition | 11 | 1.8058 | 0.7206 | 9 | 0 | 0 | 1 | 6 |
| setA-12 | shape_competition | 11 | 1.4348 | 0.7600 | 7 | 0 | 2 | 1 | 1 |
| setA-13 | shape_competition | 17 | 0.0000 | null | 0 | 0 | 0 | 0 | 0 |
| setA-14 | shape_competition | 9 | 1.5829 | 0.5595 | 7 | 0 | 2 | 2 | 2 |
| setA-15 | shape_competition | 10 | 2.6330 | 0.8669 | 9 | 0 | 2 | 2 | 5 |
| setA-16 | shape_competition | 6 | 0.0000 | null | 0 | 0 | 0 | 0 | 0 |
| setA-17 | shape_competition | 4 | 1.2641 | 0.4470 | 3 | 0 | 1 | 0 | 0 |
| setA-18 | shape_competition | 6 | 2.2172 | 0.8319 | 4 | 0 | 0 | 1 | 2 |
| setA-19 | shape_competition | 4 | 2.8235 | 0.9543 | 0 | 0 | 0 | 0 | 0 |
| setA-20 | shape_competition | 3 | 0.0000 | null | 0 | 0 | 0 | 0 | 0 |

---

## E. Hypothesis Assessment

### H1 — Transition/Tail dominance

**Prediction:** ≥ 80% of accepted moves improve Transition or Tail zones.

**Observed:** Transition = 29 (22.0%), Tail = 38 (28.8%)

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
