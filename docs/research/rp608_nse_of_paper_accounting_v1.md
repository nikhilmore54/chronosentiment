# NSE OF Paper Accounting Standard V1

**Standard ID:** `NSE_OF_TOUCH_PLUS_STATUTORY_V1`  
**Protocol:** `RP-608-NSE-OF-PAPER-TRADE-001`  
**Locked:** 2026-11-02  
**Authority:** 0.0% · OF ±0.30 / reverse-OF **unchanged**

## Cost treatment

| Component | Basis |
| --- | --- |
| Brokerage | 0.03% per executed order, cap ₹20/order |
| STT | 0.025% sell turnover |
| Exchange | 0.00307% buy+sell turnover |
| SEBI | 0.0001% buy+sell turnover |
| GST | 18% × (brokerage + exchange + SEBI) — **not** turnover / STT / stamp |
| Stamp | 0.003% buy turnover |
| Bid-ask spread | In ask-in / bid-out fills only — **not** recharged |
| Slippage | Validated incremental only in all-in; 2 bps/leg = **sensitivity** |

## Mandatory views

1. Touch gross  
2. Statutory net = touch gross − statutory overhead  
3. All-in net = statutory net − validated incremental slippage (default 0)

## Artifacts

| Path | Role |
| --- | --- |
| `of_trade_outcomes.jsonl` | **Immutable** historical journal (legacy costs preserved) |
| `of_trade_outcomes_recalc_statutory_v1.jsonl` | Full historical recalculation under V1 |
| `of_paper_accounting_v1_summary.json` | Aggregate V1 vs legacy comparison |
| Future closes | Written with V1 fields on the live journal |

Implementation: `scripts/rp608_nse_equity_costs.py` · ledger uses `calc_nse_of_paper_accounting`.

## Legs API (required for correctness)

Prefer actual executed orders:

```python
executed_orders = [
    {"side": "BUY", "turnover": 100000.00},
    {"side": "SELL", "turnover": 110000.00},
]
```

| Direction | Entry leg | Exit leg | STT | Stamp |
| --- | --- | --- | --- | --- |
| LONG | BUY @ entry | SELL @ exit | sell (exit) | buy (entry) |
| SHORT | SELL @ entry | BUY @ exit | sell (entry) | buy (exit) |

Brokerage caps apply **per order turnover**, never via equal-split of combined turnover.

Reconstruction helper: `executed_orders_from_ledger_trade()` — prefers `executed_orders` / `fills` / entry+exit fills; otherwise one touch entry + one touch exit from price×qty + direction. Never divides combined turnover by order count.

## Acceptance disposition (2026-11-02)

| Item | Status |
| --- | --- |
| Legs API design | **Accepted** (subject to production recon) |
| LONG / SHORT STT & stamp | Validated |
| Unequal-order brokerage | Validated |
| Touch-price spread treatment | Correct (not recharged) |
| Statutory rates | **Locked** — do not reopen; no alternate model |
| One-trade production recon | `scripts/research/rp608_nse_of_paper_production_recon_one_trade.py` + fixture `tests/fixtures/nse_of_paper_tatasteel_production_trade.json` |
| Historical V1 bulk totals | **PROVISIONAL** |
| Historical journal | Unchanged |
| Future NSE OF accounting | Direction-aware legs API |
| OF ±0.30 / reverse-OF / authority | Unchanged (0%) |

Remaining gate for finalizing historical overhead: production recon PASS on real executed-order costs (independent broker schedule, documented rounding).
