# Pre-open Data Availability Audit · 2026-11-02

**Protocol:** `RP-608-POLICY-D-PREOPEN-AVAIL-AUDIT-001`  
**Authority:** 0.0% · Observational only

## Executive finding

Pre-open infrastructure exists on the Pi, but **today’s session did not capture pre-open data**. Continuous L1 WS for the NSE daemon is a **session ticker cache**, not a dedicated pre-open tick archive. The scheduled pre-open path is **REST `/quote` snapshots**, and its timers were pinned to a **one-shot calendar date (2026-10-26)**, so they did not fire on 2026-11-02.

## What is installed

| Component | Role | Status 2026-11-02 |
| --- | --- | --- |
| `rp608-nse-preopen-snap{1,2,3}.service` | One-shot REST pre-open probes | Units present |
| Matching `.timer` units | Intended 09:07 / 09:11:30 / 09:14:30 IST | **Enabled but Trigger=n/a** (one-shot `OnCalendar=2026-10-26 …`) |
| `scripts/run_rp608_nse_live_probe.py --preopen` | Writes `preopen_discovery_ledger.jsonl` | REST `/quote`, not WS stream |
| `rp608-nse-shadow-daemon` L1 cache | Continuous full-mode WS for 119 tokens | Starts at session (~09:15); not a pre-open archive |
| Last ledger | `test_run/preopen_discovery_ledger.jsonl` | **2026-10-26 only** · 119/119 symbols |

## Field population (2026-10-26 ledger, n=119)

| Field | Population |
| --- | --- |
| `previous_close` | 119/119 |
| `indicative_price` | 119/119 — **all** `FALLBACK_LAST_PRICE` (no `INDICATIVE_AUCTION_CLEARING`) |
| `opening_gap_bps` | 118/119 non-zero |
| `raw_buy/sell_quantity`, imbalance | 119/119 `VALID_AUCTION_DEPTH` |
| `raw_total_imbalance_qty` | **0/119** always zero |
| Continuous WS probe (2026-10-27 evidence) | 119 ticks · **0 valid spreads · 119 zero depth** (not a pre-open certification) |

Interpretation: REST snapshots can cover the universe, but auction-specific fields (`indicative_close_price`, `total_imbalance_qty`) were **not** meaningfully populated on the only archived day. Gap construction therefore depended on last-price fallback — weaker than true indicative clearing.

## WebSocket vs REST (research implication)

| Question | Answer from this audit |
| --- | --- |
| Does current production continuously archive pre-open WS packets for 119 symbols? | **No** |
| Can REST snapshots be scheduled for pre-open? | **Yes**, once timers are daily (not one-shot dated) |
| Can we historically replay old pre-open ticks from Kite? | **No** (no tick-replay API) |
| What is required for Policy D forward tests? | Daily observational capture (REST minimum; optional WS continuity probe 09:00–09:15) |

## Remediation applied 2026-11-02 (observational only)

1. Pre-open / live-probe timers converted to **daily** `OnCalendar=*-*-* HH:MM:SS Asia/Kolkata` (next elapse **2026-11-03**).
2. New `rp608-nse-preopen-ws-continuity.{service,timer}` — 09:00–09:15 IST WS packet continuity probe (0% authority).
3. REST `--preopen` now also writes dated `preopen_discovery_snapshot_*.jsonl` files.

## Remaining next steps

1. On next session, verify WS continuity coverage for 119 symbols and compare vs REST indicative quality.
2. Only then build Policy D features on top of C.
3. Multi-session C economics remain queued (still required before any G3 change).

## Governance

- Live G3 unchanged.
- Policies A/B/C diagnostics unchanged.
- Policy D not deployed.
- No threshold or authority change.
