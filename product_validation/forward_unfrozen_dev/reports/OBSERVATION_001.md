# Forward Observation #1

**Not a performance result. Not G-GATE. Not a B4 replay.** Engine: `unfrozen-dev`.

| Field | Value |
|---|---|
| as_of | 2026-08-14T03:45:00Z (latest Yahoo NSE daily session ≤ now) |
| Source | YahooFinance delayed daily bars |
| Universe | RELIANCE.NS, TCS.NS, INFY.NS, HDFCBANK.NS, ICICIBANK.NS |
| New decisions | 5 |
| LONG / SHORT / NO_TRADE | 4 / 1 / 0 |
| 5D–60D outcomes | not yet elapsed |

| Ticker | Action | decision_id |
|---|---|---|
| RELIANCE.NS | LONG | `2d5d3243-839e-79a8-0a38-879ab3859ff5` |
| TCS.NS | LONG | `1e9887fb-7fae-3ea4-4201-d61a42765d32` |
| INFY.NS | LONG | `75a66092-0cd5-3f6c-77f8-b57f144b15d1` |
| HDFCBANK.NS | SHORT | `5ea9cc1b-20c8-a47d-55a9-52d31a14a674` |
| ICICIBANK.NS | LONG | `5f4d1de9-93f2-b25a-d8de-450154f443e8` |

Journal: `ledger.jsonl`. Next ticks: `./run_csp003_forward_tick.sh` (idempotent if as_of unchanged). Daily cadence: `./install_csp003_forward_schedule.sh`.
