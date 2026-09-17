# P4 Historical Intraday Opportunity Engine

**Status:** SPECIFICATION — implementation in [`scripts/p4_historical_intraday_replay.py`](../scripts/p4_historical_intraday_replay.py)  
**Date:** 2026-09-12  
**Purpose:** Expand the intraday discovery dataset from N=224 (Sep 3–11) to N=417
(Aug 20–Sep 7) using the full 5m Yahoo cache, without modifying any frozen P4 evidence.

---

## Motivation

The current P4 intraday discovery work is sample-limited:

| Finding | Current N | Minimum needed to act |
|---|---|---|
| LONG ENTER MFE@H60 ≥1.50% | 6 above threshold (1m) | N≥10 above threshold |
| SHORT cumret gap from 30m | 3 losers | N≥10 losers |
| H120 FAV protection (LONG) | 17 ENTER (1m) | N≥30 per direction |

The 5m Yahoo cache covers Jun 22 – Sep 11. The full ledger has 417 Watch COMPLETE
decisions from Aug 20 – Sep 7, all joinable to the 5m cache. This is 1.86× the
current Sep 3–11 sample and includes the Aug 20–21 cohorts (N=114) which are the
largest single cohorts in the dataset.

---

## Separation of Concerns

### Frozen P4 Evidence (DO NOT MODIFY)
- All conclusions from P4 Tracks A/B/C and Phase 4 retrospective
- [`docs/P4_LOSS_ANALYSIS.md`](P4_LOSS_ANALYSIS.md) — 1m vs 5m comparison matrix
- Prospective watch items (LONG MFE@H60 ≥1.50%, SHORT cumret gap from 30m)
- No retrospective rule changes

### Historical Product Development Dataset (this engine)
- Separate output: `datasets/p4_opportunity_dataset.json`
- Uses 5m bars (full Aug 20–Sep 7 coverage) as primary resolution
- Computes all path dimensions from the frozen P4 evidence plus new dimensions
- Safe to experiment with — does not contaminate P4 conclusions

---

## Data Sources

| Source | File/Dir | Coverage | N |
|---|---|---|---|
| Ledger entries | `live_capture/ledger/entries/` | Aug 20–Sep 7 | 956 total |
| COMPLETE observations | `time_machine/analysis/TIME009/observations/` | Aug 20–Sep 7 | 1,060 |
| 5m Yahoo cache | `intraday_capture/yahoo_cache_5m/` | Jun 22–Sep 11 | 102 tickers |

**Joinable Watch COMPLETE decisions: 417 (LONG=230, SHORT=187)**

---

## Output Schema

Each record in `datasets/p4_opportunity_dataset.json`:

```json
{
  "decision_id": "...",
  "ticker": "HCLTECH_NS",
  "direction": "SHORT",
  "cohort_date": "2026-08-21",
  "snap_unix": 1756080000,
  "target_rate": 0.0312,
  "rank_score": 0.73,
  "evidence_class": "A",
  "degradation_level": "LOW",
  "vol_regime": "NORMAL",
  "volume_regime": "HIGH",
  "daily_outcome": "WIN",
  "daily_return": 0.0287,

  "path_5m": {
    "bars_available": 60,
    "entry_price": 1842.50,
    "h15_ret": 0.0041,
    "h30_ret": 0.0089,
    "h60_ret": 0.0156,
    "h120_ret": 0.0234,
    "h180_ret": 0.0267,
    "h300_ret": 0.0287,
    "mfe_h60": 0.0178,
    "mae_h60": -0.0012,
    "mfe_h300": 0.0312,
    "mae_h300": -0.0023,
    "time_to_mfe_bars": 29,
    "time_to_mae_bars": 2,
    "first_adverse_bar": 2,
    "max_consec_adverse": 3,
    "peak_to_final_drawdown": -0.0008,
    "path_reversal": false,
    "h60_classification": "ENTER",
    "h120_classification": "ENTER-LATE",
    "intraday_winner": true
  },

  "opportunity_dimensions": {
    "early_momentum": 0.0041,
    "momentum_persistence": 0.87,
    "adverse_exposure": 0.0012,
    "mfe_h60_tier": "HIGH",
    "opportunity_quality_score": 82
  }
}
```

---

## Path Dimensions

| Dimension | Definition |
|---|---|
| `h15_ret` through `h300_ret` | Direction-adjusted cumulative return at each horizon |
| `mfe_h60`, `mfe_h300` | Max favourable excursion at H60 and H300 |
| `mae_h60`, `mae_h300` | Max adverse excursion at H60 and H300 |
| `time_to_mfe_bars` | Bar index of MFE within H300 window |
| `time_to_mae_bars` | Bar index of MAE within H300 window |
| `first_adverse_bar` | First bar where direction-adjusted return < −0.2% |
| `max_consec_adverse` | Longest run of consecutive adverse bars |
| `peak_to_final_drawdown` | (peak_cumret − final_cumret) / peak_cumret |
| `path_reversal` | True if H300 ret < 0 after being > target_rate/2 at any point |
| `h60_classification` | ENTER / AVOID / WAIT (frozen Phase 4 rules) |
| `h120_classification` | ENTER-LATE / AVOID-LATE / WAIT-LATE |
| `intraday_winner` | H300 ret > +0.2% |
| `early_momentum` | h15_ret |
| `momentum_persistence` | Fraction of H300 bars where cumret > 0 |
| `adverse_exposure` | abs(mae_h300) |
| `mfe_h60_tier` | LOW (<0.5%), MID (0.5–1.5%), HIGH (≥1.5%) |
| `opportunity_quality_score` | Composite 0–100 (see below) |

---

## Opportunity Quality Score

Composite score (0–100) computed from direction-adjusted path dimensions.
Each component is percentile-ranked within the full dataset and scaled 0–1.

```
Score = (
    30 × momentum_component      # early + sustained momentum
  + 25 × mfe_component           # magnitude of opportunity
  + 20 × adverse_component       # low adverse exposure
  + 15 × timing_component        # fast time-to-MFE
  + 10 × persistence_component   # fraction of bars favourable
)
```

This score is relative (ranks opportunities against each other), not absolute.
It is a product signal for the Decision Cockpit, not a trading rule.

---

## Implementation Plan

### Script: `scripts/p4_historical_intraday_replay.py`

**Phase 1 — Data loading**
- Load all ledger entries with COMPLETE observations
- Filter to Watch decisions only
- Join to 5m cache by ticker + snap_unix
- Report coverage (expect ~417 decisions, ~100% ticker match)

**Phase 2 — Path reconstruction**
- For each decision, find first bar after snap_unix
- Compute direction-adjusted returns at H15/H30/H60/H120/H180/H300
- Compute MFE/MAE at H60 and H300
- Compute all path dimensions listed above
- Apply frozen Phase 4 classification rules

**Phase 3 — Opportunity quality scoring**
- Compute all component scores
- Percentile-rank within dataset
- Compute composite score

**Phase 4 — Output**
- Write `datasets/p4_opportunity_dataset.json`
- Print summary statistics
- Print WIN/LOSS breakdown by H60 classification
- Print MFE@H60 distribution for LONG ENTER (extend the Sep 3–11 analysis)
- Print SHORT cumret gap analysis (extend the Sep 3–11 analysis)

---

## Expected Output

```
Historical Intraday Opportunity Engine
======================================
Ledger entries loaded:     956
COMPLETE Watch decisions:  417 (LONG=230, SHORT=187)
5m bars matched:           ~417 / 417 (100%)

H60 Classification breakdown:
  LONG:  ENTER=XX  AVOID=XX  WAIT=XX
  SHORT: ENTER=XX  AVOID=XX  WAIT=XX

LONG ENTER MFE@H60 distribution (extended, N=~XX):
  [bins]

SHORT ENTER cumret gap at 30m/60m/120m (extended, N=~XX losers):
  [checkpoint comparison]

Output written: datasets/p4_opportunity_dataset.json
```

---

## Governance

- This dataset is a product development artifact, not a research validation artifact
- Findings from this dataset do NOT modify frozen P4 conclusions
- Any threshold or rule discovered here requires a separate prospective validation
  gate before product use
- The Sep 15+ prospective run remains governed by the frozen P4 rules only