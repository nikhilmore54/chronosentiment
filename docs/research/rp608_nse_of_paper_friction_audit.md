# NSE OF Paper — Friction Calculation Audit

**Protocol:** `RP-608-NSE-OF-PAPER-TRADE-001`  
**Date:** 2026-11-02  
**Authority:** 0.0% · OF ±0.30 / reverse-OF **unchanged**  
**Artifact:** `of_paper_ledger_v1/of_paper_friction_audit.json`  
**Harness:** `scripts/research/rp608_nse_of_paper_friction_audit.py`

## Question

Does ledger arithmetic reconciliation (`gross − friction = net`) imply the **₹ friction magnitude is correct**?

## Answers

### 1. Journal ↔ function consistency — PASS

Recomputing `calc_indian_equity_friction(entry, exit, qty, half_spread_bps)` for every closed row matches journaled `friction_inr` (deltas ≈ 0). The ledger applies the function as written.

### 2. Component schedule (as coded)

| Component | Rule in code |
| --- | --- |
| Brokerage | `min(₹20, 0.03% × leg turnover)` per leg |
| STT | 0.025% sell leg only |
| Exchange | 0.00297% total turnover |
| SEBI | 0.0001% total turnover (`1e-6`) |
| GST | 18% on brokerage + exchange + SEBI |
| Stamp | 0.003% buy leg only |
| Spread | `half_spread_bps` × both leg turnovers |
| Slippage | **2.0 bps × both legs** (fixed model) |

### 3. Fill model vs friction — double-count risk — FAIL / CAUTION

Paper LONG fills:

- Entry = **ask** (`_quote_px` for_exit=False)
- Exit = **bid** (`_quote_px` for_exit=True)

So **gross PnL already embeds a full touch-spread crossing**.

Friction then **adds again**:

- half-spread cost on buy + sell turnovers  
- plus 2 bps slippage each leg  

Relative to touch fills, reported friction **overstates** execution cost. Statutory legs alone are the cleaner baseline for “did OF earn enough to pay costs?” diagnostics.

### 4. Friction sensitivity (positive gross → negative net)

Observed examples (journal): PERSISTENT, PIDILITIND, M&M — gross green, net red after full friction stack. TATASTEEL barely cleared (+₹0.47).

## Governance / remediation

Audit did **not** change OF ±0.30 or reverse-OF. Remediation locked as **`NSE_OF_TOUCH_PLUS_STATUTORY_V1`** — see [`rp608_nse_of_paper_accounting_v1.md`](rp608_nse_of_paper_accounting_v1.md):

- Future OF paper closes use touch + statutory (no double spread).  
- Historical rows preserved; parallel recalc artifact for all completed trades.  
- 2 bps/leg slippage = sensitivity only until validated.

Keep cohort separate from Estimand A and book-intelligence telemetry.
