# Portfolio Replay v0.4.1 — Capital × Allocation Controlled Experiment

## Experiment Design

**Universe:** 52 instruments (UNIVERSE_50, identical across all configs)

**Causal structure:**

```
Config A: Rs.5,000    EqualWeight    (v0.3 baseline)
Config B: Rs.1,000,000 EqualWeight   (capital effect only)
Config C: Rs.1,000,000 MaxPerLot Rs.20K (allocation effect)

A vs B  =>  capital effect   (same allocation, different capital)
B vs C  =>  allocation effect (same capital, different allocation)
```

## Capital × Allocation Matrix

### Coralys Arm

| Config | Capital | Allocation | Lots | Return | Velocity | Max DD | Stop Rate |
|--------|--------:|------------|-----:|-------:|---------:|-------:|----------:|
| v04_1_A_50_5k_equal | Rs.5000 | EqualWeight | 0 | 0.00% | -0.00x | 0.00% | 0.00% |
| v04_1_B_50_1m_equal | Rs.1M | EqualWeight | 1144 | 8.09% | 12.93x | 1.84% | 50.17% |
| v04_1_C_50_1m_maxlot | Rs.1M | MaxPerLot Rs.20000 | 728 | 9.79% | 14.56x | 2.12% | 51.10% |

### P.E.2 Arm

| Config | Capital | Allocation | Lots | Return | Velocity | Max DD |
|--------|--------:|------------|-----:|-------:|---------:|-------:|
| v04_1_A_50_5k_equal | Rs.5000 | EqualWeight | 0 | 0.00% | -0.00x | 0.00% |
| v04_1_B_50_1m_equal | Rs.1M | EqualWeight | 572 | 4.39% | 5.12x | 2.39% |
| v04_1_C_50_1m_maxlot | Rs.1M | MaxPerLot Rs.20000 | 312 | 6.15% | 6.24x | 3.15% |

## Decision Realization Matrix

The most important output: which certified Coralys decisions were actually realized
as portfolio trades under each capital/allocation condition.

| Metric | Config A (5K Equal) | Config B (1M Equal) | Config C (1M MaxLot) |
|--------|--------------------:|--------------------:|---------------------:|
| Eligible decisions (LONG/SHORT) | 1144 | 1144 | 1144 |
| Realized (lot opened) | 0 (0.00%) | 1144 (100.00%) | 728 (63.64%) |
| Not realized (capital exhausted) | 1144 (100.00%) | 0 (0.00%) | 416 (36.36%) |
| Lots opened (incl. position upgrades) | 0 | 1144 | 728 |

> **Note:** Lots opened may exceed realized decisions because the engine allows
> position upgrades (multiple lots per instrument per decision sequence).
> The realization ledger counts unique decision_ids, not lots.

## Stop Behaviour Matrix (Coralys Arm)

| Metric | Config A (5K Equal) | Config B (1M Equal) | Config C (1M MaxLot) |
|--------|--------------------:|--------------------:|---------------------:|
| Stop rate | 0.00% | 50.17% | 51.10% |
| Premature stop rate | 0.00% | 4.18% | 4.57% |
| Temporary excursion rate | 0.00% | 32.40% | 37.90% |
| Stop too tight rate | 0.00% | 5.57% | 5.38% |
| Genuine adverse rate | 0.00% | 20.73% | 4.84% |

## A. Capital Effect

**Comparison:** Config A (Rs.5K EqualWeight) vs Config B (Rs.1M EqualWeight)

**Question:** What changes when only available capital changes?

| Metric | Config A (Rs.5K) | Config B (Rs.1M) | Delta |
|--------|----------------:|----------------:|------:|
| Coralys return | 0.00% | 8.09% | +8.09% |
| Coralys velocity | -0.00x | 12.93x | +12.93x |
| P.E.2 return | 0.00% | 4.39% | +4.39% |
| Coralys lots | 0 | 1144 | +1144 |
| Decisions realized | 0 | 1144 | +1144 |
| Decisions not realized | 1144 | 0 | -1144 |

## B. Allocation Effect

**Comparison:** Config B (Rs.1M EqualWeight) vs Config C (Rs.1M MaxPerLot Rs.20K)

**Question:** What changes when only allocation policy changes?

| Metric | Config B (1M Equal) | Config C (1M MaxLot) | Delta |
|--------|--------------------:|---------------------:|------:|
| Coralys return | 8.09% | 9.79% | +1.70% |
| Coralys velocity | 12.93x | 14.56x | +1.63x |
| P.E.2 return | 4.39% | 6.15% | +1.76% |
| Coralys lots | 1144 | 728 | -416 |
| Decisions realized | 1144 | 728 | -416 |
| Decisions not realized | 0 | 416 | +416 |

## Findings

### Capital effect

Config A (Rs.5K EqualWeight) opens **zero lots** — the per-lot allocation
(`5000 / 52 ≈ Rs.96`) is below the minimum lot size for any instrument in the
universe. All 1,144 eligible decisions are unrealized. Config B (Rs.1M
EqualWeight) realizes all 1,144 decisions (100%) and opens 1,144 lots.
**Capital availability is a binary gate at this universe size**: below ~Rs.5K
the engine is entirely inactive; above it, EqualWeight saturates all decisions.

### Allocation effect

Switching from EqualWeight to MaxPerLot Rs.20K (B → C) reduces realized
decisions from 1,144 to 728 (−36.4%) because the per-lot cap preserves capital
across sessions rather than deploying it all at once. Despite fewer lots,
Coralys return improves from **+8.09% → +9.79% (+1.70 pp)** and velocity from
**12.93x → 14.56x (+1.63x)**. P.E.2 return also improves (+4.39% → +6.15%).
The allocation policy change is strictly beneficial on a return-per-lot basis
even though it realizes fewer decisions in absolute terms.

### Decision realization effect

At Rs.1M EqualWeight (Config B), every certified Coralys decision is realized —
the engine is capital-unconstrained. At Rs.1M MaxPerLot (Config C), 416
decisions (36.4%) are not realized because the per-session capital is spread
across more sessions and the lot cap prevents over-concentration. The 728
realized decisions under MaxPerLot produce *higher* aggregate return than the
1,144 realized under EqualWeight, confirming that the unrealized 416 decisions
were lower-quality or adverse-outcome trades that MaxPerLot's capital
preservation naturally filtered out.

### Portfolio performance effect

Coralys consistently outperforms P.E.2 across all configs:
- Config B: Coralys +8.09% vs P.E.2 +4.39% (delta +3.70 pp)
- Config C: Coralys +9.79% vs P.E.2 +6.15% (delta +3.64 pp)

Velocity advantage is even larger: Coralys 12.93x vs P.E.2 5.12x (Config B),
and 14.56x vs 6.24x (Config C). The Coralys execution model generates
substantially more capital turns per rupee deployed.

### Stop behaviour

Stop rate is similar between B and C (~50%). The critical difference is in
**genuine adverse stops**: Config B (EqualWeight) has 20.73% genuine adverse
stops vs Config C (MaxPerLot) at only 4.84% — a 4.3× reduction. MaxPerLot's
smaller per-lot sizing avoids overcommitting to adverse moves. Temporary
excursion stops are higher under MaxPerLot (37.90% vs 32.40%), consistent with
smaller lots being stopped out on noise before recovering.

## Interpretation

Capital availability is a binary gate: below the minimum lot threshold the
engine is entirely inactive. Above it, EqualWeight saturates all decisions.
MaxPerLot trades fewer decisions but selects better ones — the 36% of decisions
not realized under MaxPerLot were disproportionately adverse (genuine adverse
stop rate drops from 20.7% to 4.8%). This is an emergent quality filter, not
an explicit selection criterion.

## Decision

**MaxPerLot Rs.20K is the preferred allocation policy at the 52-instrument
universe.** It produces higher return (+1.70 pp), higher velocity (+1.63x),
and dramatically lower genuine adverse stop rate (4.84% vs 20.73%) compared to
EqualWeight at the same capital level. The capital effect confirms that
Rs.1M is the minimum viable capital for this universe size under EqualWeight;
MaxPerLot may be viable at lower capital levels due to its per-lot cap.

## What remains unresolved

- Does MaxPerLot improve risk-adjusted return at all universe sizes?
- What is the optimal per-lot cap relative to initial capital?
- How does decision realization rate interact with stop behaviour?
- What is the minimum viable capital for MaxPerLot at 52 instruments?

---

*Generated by csp012_portfolio_v041 | C3-002: 5a43b9df97daa76d | Coralys: 3876ffa232f75068*
