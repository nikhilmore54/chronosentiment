# RP-608-INV-001: Inverse Strategy Evaluation Report
**Phase 1: Controlled Hypothesis Test of Trade Direction Reversal**

**Document Identifier**: `RP-608-INV-001-REPORT`  
**Protocol Series**: `RP-608 (Execution Reversals & Directional Attribution)`  
**Date**: 2026-10-22  
**Status**: COMPLETED CONTROLLED EXPERIMENT & FALSIFICATION REPORT  
**Governance Authority**: Strictly **0.0% Live Order Authority** (Shadow Evaluation Only)  
**Adapter State**: `adapters/chronosentiment` **100% FROZEN** (13/13 discipline invariants green)  
**Audited Dataset**: 2,029 historical trades across 99 NSE symbols (Sep 28 – Oct 01, 2026)

---

## 1. Executive Summary & Core Research Answers

Under research mandate `RP-608-INV-001`, a controlled, shadow-only evaluation of trade direction reversal was executed across all 2,029 historical trades. The experiment decoupled direction reversal from retroactive P&L negation by simulating four distinct variants, including forward path-dependent execution on 1-minute intraday bars.

```
+──────────────────────────────────────────────────────────────────────────────────────────────────────+
|                                    CORE RESEARCH QUESTIONS & ANSWERS                                 |
+──────────────────────────────────────────────────────────────────────────────────────+───────────────+
| Question                                                                             | Finding       |
+──────────────────────────────────────────────────────────────────────────────────────+───────────────+
| 1. Does reversing direction improve gross expectancy?                                | NO (-0.52 bps)|
| 2. Does that improvement survive realistic transaction costs?                        | NO (-9.52 bps)|
| 3. Does the result persist across sessions rather than being driven by one day?     | NO (Failed)   |
+──────────────────────────────────────────────────────────────────────────────────────+───────────────+
```

### Key Analytical Takeaways:
1. **Gross Expectancy Drops Under Exact Inversion**: The baseline strategy exhibited a small positive gross drift (+₹9,993.52 / +0.52 bps) driven by short positions. Exact reversal (Variant B) inverts this drift to **−₹9,993.52** (**−0.52 bps gross**). Gross expectancy deteriorates by **1.04 bps**.
2. **The Skewness Illusion**: Reversing direction produces an apparent surge in gross win rate from 33.0% to **67.0%**. However, this is an artifact of return asymmetry: the baseline had frequent small losses and infrequent large winners. Inversion turns frequent small losses into small wins, but converts large winners into devastating tail losses.
3. **Transaction Fee Annihilation**: With Indian statutory transaction costs of 9.00 bps (₹181,029.04), the apparent 67.0% gross win rate collapses to **32.7% net win rate**, generating a cumulative net loss of **−₹191,022.56** (**−9.52 bps net expectancy**).
4. **Session Instability**: Variant B's performance is driven by session-level trend regimes. While it gained +₹13,370 gross on Sep 28 (a choppy range-bound session), it suffered a catastrophic gross loss of **−₹19,875.61 on October 01 alone** (−4.23 bps gross, −13.23 bps net), proving extreme vulnerability to sustained directional trends.

---

## 2. Four-Variant Strategy Comparison

Four variants were evaluated across the identical trade ledger:

- **Variant A (Frozen Baseline)**: Original long/short positions, entry/exit prices, and execution rules.
- **Variant B (Exact Direction Reversal)**: Exact flip of side at entry (`LONG` $\to$ `SHORT`, `SHORT` $\to$ `LONG`) with identical entry and exit timestamps/prices.
- **Variant C (Inverse with Mirrored Symmetric Exits)**: Path-dependent replay on 1-minute OHLC bars. For a reversed position, a symmetric stop-loss is placed at distance $\Delta P = |P_{\text{entry}} - P_{\text{stop}}|$; if intrabar wicks touch the stop, the trade is terminated immediately.
- **Variant D (Range-Conditioned Reversal)**: Reverses direction ONLY for trades entering in the Upper Range ($Z_{\text{Upper}} \ge 0.80$, near 20-day resistance), while leaving other trades in baseline direction.

### Table 1: Comprehensive Four-Variant Performance Comparison
All variants evaluated over identical 2,029 completed trades with statutory friction reconciled to the Rupee:

| Variant | Trades | Gross P&L (₹) | Statutory Friction (₹) | Net Realized P&L (₹) | Gross Return (bps) | Net Expectancy (bps) | Gross Win Rate | Net Win Rate | Max Drawdown (₹) |
| :--- | :---: | :---: | :---: | :---: | :---: | :---: | :---: | :---: | :---: |
| **Variant A: Frozen Baseline** | 2,029 | +₹9,993.52 | ₹181,029.04 | −₹171,035.52 | **+0.52 bps** | **−8.48 bps** | 33.0% | 20.4% | ₹171,035.52 |
| **Variant B: Exact Direction Reversal** | 2,029 | −₹9,993.52 | ₹181,029.04 | −₹191,022.56 | **−0.52 bps** | **−9.52 bps** | 67.0% | 32.7% | ₹204,393.45 |
| **Variant C: Mirrored Symmetric Exits** | 2,029 | +₹12,205.67 | ₹181,029.04 | −₹168,823.37 | **+0.59 bps** | **−8.41 bps** | 64.6% | 31.5% | ₹180,408.68 |
| **Variant D: Range-Conditioned Reversal** | 2,029 | +₹17,531.01 | ₹181,029.04 | −₹163,498.03 | **+0.89 bps** | **−8.10 bps** | 34.0% | 21.0% | ₹163,498.03 |

---

## 3. Session-by-Session Attribution & Concentration Audit

### Table 2: Variant B (Exact Reversal) by Session
| Session Date | Trades | Gross P&L (₹) | Statutory Friction (₹) | Net P&L (₹) | Mean Gross Return | Net Expectancy | Gross Win Rate | Net Win Rate |
| :---: | :---: | :---: | :---: | :---: | :---: | :---: | :---: | :---: |
| **2026-09-28** | 460 | +₹13,370.89 | ₹41,084.82 | −₹27,713.92 | **+2.92 bps** | −6.08 bps | 74.6% | 30.7% |
| **2026-09-29** | 547 | −₹1,773.21 | ₹48,850.75 | −₹50,623.96 | **−0.36 bps** | −9.36 bps | 67.6% | 34.7% |
| **2026-09-30** | 547 | −₹1,715.60 | ₹48,792.00 | −₹50,507.60 | **−0.35 bps** | −9.35 bps | 64.9% | 32.0% |
| **2026-10-01** | 475 | **−₹19,875.61** | ₹42,301.48 | −₹62,177.08 | **−4.23 bps** | **−13.23 bps** | 61.3% | 33.1% |
| **Total** | **2,029** | **−₹9,993.52** | **₹181,029.04** | **−₹191,022.56** | **−0.52 bps** | **−9.52 bps** | **67.0%** | **32.7%** |

### Table 3: Variant C (Path-Dependent Replay with Mirrored Stops)
| Session Date | Trades | Stopped Out Count | Stopped Out % | Gross P&L (₹) | Net P&L (₹) | Mean Gross Return | Net Expectancy | Net Win Rate |
| :---: | :---: | :---: | :---: | :---: | :---: | :---: | :---: | :---: |
| **2026-09-28** | 460 | 69 | 15.0% | +₹11,585.31 | −₹29,499.50 | +2.52 bps | −6.48 bps | 29.8% |
| **2026-09-29** | 547 | 92 | 16.8% | +₹8,956.70 | −₹39,894.05 | +1.63 bps | −7.37 bps | 33.6% |
| **2026-09-30** | 547 | 107 | 19.6% | +₹2,054.91 | −₹46,737.09 | +0.34 bps | −8.66 bps | 31.3% |
| **2026-10-01** | 475 | 130 | 27.4% | **−₹10,391.25** | −₹52,692.72 | **−2.20 bps** | −11.20 bps | 31.2% |
| **Total** | **2,029** | **398** | **19.6%** | **+₹12,205.67** | **−₹168,823.37** | **+0.59 bps** | **−8.41 bps** | **31.5%** |

---

## 4. Execution Physics & Short-Sale Friction Mechanics

1. **Borrowing & Intraday Liquidity Constraints**:
   - In Indian equities (NSE cash market), short selling is permitted strictly for intraday square-off (MIS). Positions must be liquidated before 15:15 IST.
   - All 2,029 evaluated trades entered between 09:40 and 14:55 IST and closed before 15:15 IST. No overnight stock borrow or Securities Lending and Borrowing (SLBM) fees were incurred.
2. **Bid-Ask Spread Asymmetry**:
   - Long entries buy at the Ask and sell at the Bid. Reversing a position to a Short requires selling at the Bid and buying at the Ask.
   - At an average quoted spread of 1.8 bps, half-spread crossing cost is identical in both directions.
3. **Statutory Fee Symmetry**:
   - Securities Transaction Tax (STT) on NSE intraday equities is levied on the sell leg (0.025%). Whether the trade is Long (sell on exit) or Short (sell on entry), the STT fee of ~2.50 bps is identical.
   - Brokerage (flat ₹20 per executed order = ₹40 per round-trip), exchange turnover charges, SEBI turnover fees, and GST remain constant.
   - Total friction is invariant to trade direction: exactly **8.996 bps** (₹89.22 per ₹100,000 notional).

---

## 5. Formal Hypothesis Falsification Verdict

- **Hypothesis**: *The ChronoSentiment baseline loses money because its entry direction is systematically wrong; reversing the trade direction will convert the negative baseline into a profitable strategy.*
- **Empirical Status**: **FALSIFIED**.
  - Reversing direction produces a **negative gross return (−0.52 bps)** and a **worse net loss (−₹191,022.56)**.
  - Adding symmetric stop-loss protection (Variant C) caps runaway tail losses but leaves gross return at **+0.59 bps**, which covers less than 7% of statutory friction.
  - Range conditioning (Variant D) improves gross return to **+0.89 bps**, leaving net expectancy at **−8.10 bps**.
- **Conclusion**: The baseline's lack of profitability is **not caused by inverted direction**. It is caused by **near-zero gross predictability** coupled with **statutory transaction fee drag**. Inverting a zero-drift signal simply produces another zero-drift signal that is equally destroyed by friction.

---

## 6. Governance Actions

1. **`RP-608-INV-001` Formally Concluded**: Simple trade inversion is rejected as a standalone edge.
2. **No Strategy Mutation**: Neither Variant B nor Variant C will be deployed or promoted to live execution.
3. **Invariants Maintained**: `adapters/chronosentiment` remains **100% frozen**; live order authority remains strictly **0.0%**.
