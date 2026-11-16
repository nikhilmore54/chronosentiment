# RP-608-OFX — Stage 0C Kite Full-Mode Live Tick Feasibility

**Protocol:** `RP-608-OFX`  
**Phase:** `STAGE_0C_KITE_FULLMODE_LIVE_TICK_FEASIBILITY`  
**Disposition:** **EXECUTED · VERDICT LOCKED**  
**Date:** 2026-11-04  
**Authority:** 0.0%  
**Inspection class:** Agent isolated WebSocket probe + source/REST inspection — **not** independent third-party verification  

**Parent:** [`rp608_ofx_order_flow_response_scoped.md`](rp608_ofx_order_flow_response_scoped.md)  
**Prior:** Stage 0 / 0B  
**Artifact:** `artifacts/rp608_ofx/stage0c_kite_fullmode_live_tick_feasibility.json`

---

## Purpose

Distinguish what **Kite can provide on the live full-mode stream** from what we must **capture ourselves**, without treating historical candles as trade-level aggression.

Question answered:

> Can Kite WebSocket full mode support a sufficiently reliable **inferred-aggressor** observational path (with uncertainty), distinct from a true exchange aggressor tape?

---

## Governance (unchanged)

| Item | State |
| --- | --- |
| Four-book baseline | Untouched |
| OF ±0.30 / reverse-OF / execution | Frozen — probe was research-only |
| Durable Stage 0D journal | **Not started** by this note (requires separate approval) |
| Stage 1 observer | **Blocked** until Stage 0D quality gate |
| Crypto | Separate (`aggTrade`) — not pooled with NSE |

---

## Field matrix

| Field | Kite full mode (wire / docs) | Production `rp608_kite_ticker_cache` | Stage 0C probe |
| --- | --- | --- | --- |
| LTP | Yes | Retained | Yes |
| LTQ (last traded quantity) | Yes (bytes 8–12) | **Skipped** | Decoded |
| Last trade time | Yes (bytes 44–48) | **Skipped** | Decoded |
| Exchange timestamp | Yes | Retained | Yes |
| Cumulative volume | Yes | Retained | Yes |
| L1–L5 bid/ask | Yes | Retained (book-intel) | Yes |
| Aggressor side | **Not supplied** | N/A | **Inferred only** |
| Historical trade tape | Not via candle API (0B) | N/A | N/A |

REST `/quote` also exposes `last_quantity` + `last_trade_time` + 5-level `depth` (snapshot confirmation; not a durable tick tape).

---

## Live probe design

| Parameter | Value |
| --- | --- |
| Duration | 20 seconds |
| Symbols | 10 mapped NSE names (RELIANCE, TCS, INFY, …) |
| Mode | `full` (184-byte packets) |
| Isolation | Separate research WS (may briefly displace daemon ticker; reconnect expected) |
| Post-probe health | `connected=true`, 119/119 cached, `last_packet_age_sec=0` |

**Trade-like vs quote-repeat heuristic (provisional):**

- **Trade-like:** LTP/LTQ/LTT tuple change and/or cumulative **volume increase** vs prior packet for that token.  
- **Quote-only repeat:** same LTP/LTQ/LTT **and** same volume (last-trade fields carried on a non-execution update).

**Inferred aggressor (same-packet LTP vs bid/ask — provisional, not Lee-Ready prior-quote):**

- LTP ≥ ask → `BUY_INITIATED_CANDIDATE`  
- LTP ≤ bid → `SELL_INITIATED_CANDIDATE`  
- Inside spread → `AMBIGUOUS`  
- Missing/crossed → `UNAVAILABLE`

---

## Live probe results (summary)

| Metric | Value |
| --- | ---: |
| Full packets (10 symbols) | 248 |
| Trade-field changes | 201 (~81% of packets) |
| Volume increases | 121 |
| Quote-only repeats (same LT fields + volume) | 30 (~12%) |
| Infer BUY / SELL / AMBIGUOUS (on trade-like) | 128 / 72 / 18 |
| Ambiguous share | ~8.3% |
| Unavailable share | 0% (this short sample) |

**Interpretation:** Not every WebSocket tick is a new execution (~12% quote-only repeats in this sample). Volume increase and LT-field change give a usable **trade-like filter**. Direct aggressor is absent; same-packet inference yields mostly buy/sell candidates with a non-trivial ambiguous bucket.

---

## Verdict (locked)

| Claim | Result |
| --- | --- |
| Wire fields enough to **start** inferred-aggressor research capture | **PASS (feasibility)** |
| Direct exchange aggressor flag | **FAIL / absent** |
| “True trade tape” established | **No** — label as **inferred-aggressor capture** |
| Every tick = new execution | **False** — must filter |
| Production decoder ready for OFX | **No** — still drops LTQ/LTT |
| Stage 0D durable isolated journal | **Next** — if separately approved; not auto-started |
| Stage 1 aggression→response observer | **Still blocked** until 0D quality gate |

**Overall label:** `INFERRED_AGGRESSOR_CAPTURE_FEASIBLE_NOT_TRUE_TRADE_TAPE`

---

## Dataset separation (preserve)

| Dataset | Role |
| --- | --- |
| Stage 0B Kite historical candles | Retrospective **price-response** only |
| Stage 0A depth snapshots (~5s) | Displayed liquidity evolution |
| Stage 0C/0D live full-mode ticks | Prospective quote context + **inferred** aggression |
| Crypto `aggTrade` (future) | Separate market — do not pool |

Do **not** retro-label candle volume as aggressor flow.

---

## Caveats

1. Same-packet LTP vs bid/ask is **not** a prior-quote Lee-Ready implementation; confidence must stay provisional.  
2. 20s / 10-symbol sample is feasibility evidence, **not** a coverage certification.  
3. A second WS may briefly displace the daemon ticker; health recovered in this run. Stage 0D should prefer a **single-connection** research tee or decoder extension that does not open a competing socket.  
4. This is project-reported execution, not human independent verification of Pi journals.

---

## Next boundary

| Stage | Status |
| --- | --- |
| 0A depth | PASS (narrow) |
| 0B historical candles | PASS (price-response) |
| **0C live full-mode** | **PASS (inferred-aggressor feasibility)** |
| **0D** durable capture + quality / sync / coverage gate | **Not started** — separate approval |
| 1 observer | After 0D PASS only |

No execution changes. No threshold changes. Authority 0%. Step 3+ held. Gate R unreached.
