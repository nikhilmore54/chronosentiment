# NSE OF paper — TATASTEEL production reconciliation (acceptance gate)

**Trade ID:** `NSE-OF-PAPER-TATASTEEL.NS-15-1793604614090`  
**Source:** `live_capture/.../nse/of_paper_ledger_v1/of_trade_outcomes.jsonl` (snapshot fixture under `tests/fixtures/`)  
**Purpose:** Prove V1 legs API reproduces actual executed-order costs vs an independent Zerodha-schedule hand check.  
**Rates:** unchanged. **Historical journal:** not rewritten. **Bulk V1 totals:** remain **PROVISIONAL**.

## Executed orders (reconstructed — no equal-split)

Paper RT recorded one ask-in and one bid-out (no partial fills in journal). Legs are price × qty per execution:

| # | Side | Price | Qty | Order turnover | Brokerage (cap ₹20) |
| --- | --- | ---: | ---: | ---: | ---: |
| 1 | BUY | 175.63 | 14 | ₹2,458.82 | min(0.03%×2458.82, 20) = ₹0.7376 |
| 2 | SELL | 175.94 | 14 | ₹2,463.16 | min(0.03%×2463.16, 20) = ₹0.7389 |
| | | | | | **₹1.4766** |

Touch gross = (175.94 − 175.63) × 14 = **₹4.34**

## Independent statutory (literals; not imported from shared module)

| Charge | Basis | Amount (₹) |
| --- | --- | ---: |
| Brokerage | per-order as above | 1.4766 |
| STT | 0.025% × sell | 0.6158 |
| Exchange | 0.00307% × 4921.98 | 0.1511 |
| SEBI | ₹10/crore × 4921.98 | 0.0049 |
| Stamp | 0.003% × buy | 0.0738 |
| GST | 18% × (brok+exch+SEBI) | 0.2936 |
| **Statutory overhead** | round to 2 dp | **≈ 2.62** |

Statutory net ≈ 4.34 − 2.62 = **≈ ₹1.72**  
Spread crossing: **not charged** (touch fills).

Legacy journal `friction_inr=3.87` included spread+slip additives — not V1 truth.

## Multi-fill path (synthetic; proves no equal-split)

When `fills` lists unequal orders (₹1L + ₹10k buys, ₹1.1L sell), brokerage = **₹43** (20+3+20), not an equal-split of combined buy turnover.

## How to re-run

```bash
python3 scripts/research/rp608_nse_of_paper_production_recon_one_trade.py
python3 -m unittest tests.test_rp608_nse_equity_costs.TestNseEquityCosts.test_production_recon_tatasteel_from_ledger_fixture -v
```

Artifact: `artifacts/rp608_nse_of_paper/production_recon_tatasteel_v1.json`
