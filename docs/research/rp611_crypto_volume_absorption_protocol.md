# RP-611C — Crypto Volume Absorption Shadow Experiment

**Protocol Reference:** `RP-611-CRYPTO-VOLUME-ABSORPTION-001`  
**Short ID:** `RP-611C`  
**Status:** **REGISTERED** (2026-11-01)  
**Live Order Authority:** **0.0%**  
**Instrument:** BTC/USDT **spot** only  
**Candle interval:** 15 minutes  
**Volume source:** BTC/USDT **spot kline volume** (same series as ChronoSentiment 15m candles)  
**Order-flow input:** Spot L1 `bookTicker` (observational; does not mutate existing OF lifecycle)  
**Directions:** LONG and SHORT  
**Execution:** Shadow simulation only  

Aligned with `.cursor/rules/chronosentiment-core.mdc`.

**Not RP-609 / RP-610.** Multi-horizon programmes remain separate. Existing Policy B/C, trade-plan overlay, and OF latch/exit are **unchanged**.

---

## 1. Registration lock

Registered on explicit governance instruction **“approve and register”** (2026-11-01).

| Parameter | Locked value |
|-----------|--------------|
| RVOL threshold | ≥ 1.50 |
| Max body efficiency | ≤ 0.35 |
| Min relevant wick | ≥ 0.40 |
| Confirmation | Next completed 15m candle vs absorption midpoint |
| Simulated target | 1.5R |
| Max holding | 4 completed bars / 60 minutes |
| Ambiguous same-bar TP+SL | Stop-first |
| L1 LONG support | imbalance ≥ +0.30 |
| L1 SHORT support | imbalance ≤ −0.30 |
| Evaluation window | 30 calendar days (UTC day reporting + regime tags) |
| Perpetual / futures volume | **Excluded** |
| Threshold adaptation | **Forbidden** during the registered window |

**Experiment clock:** **STARTED** (see `artifacts/rp611_crypto_volume_absorption/registration.json`). Phase `1_data_integrity`. Full signal observer **not deployed** until separately authorized.

Any futures/perpetual volume experiment requires a **new** protocol ID, volume source, and ledger.

---

## 2. Two independent detection layers

### Layer 1 — Candle-volume absorption

Same frozen candle geometry as RP-611N, using **spot** OHLCV.

### Layer 2 — Live L1 order-flow observation

\\( I = (BidQty - AskQty) / (BidQty + AskQty) \\)

- Same-direction support / neutral / opposite-side confirm recorded continuously.
- Must **not** alter `RP-608` OF recommendation lifecycle or trade-plan exits.
- RP-611C may **read** bookTicker and write to its own ledger only.

---

## 3. Experimental arms

| Arm | Entry condition | Research question |
|-----|-----------------|-------------------|
| A | Candle-volume absorption only | Absorption alone? |
| B | L1 OF confirmation only | L1 alone? |
| C | Absorption + same-direction L1 | Combination lift? |
| D | Absorption + L1 + reversal tracking | Lifecycle vs trade outcome separation |

Arm D tracks OF recommendation telemetry separately from simulated absorption trade P&L.

---

## 4. Lifecycle separation (mandatory)

| Object | Owner |
|--------|--------|
| Absorption candidate / confirmation | RP-611C ledger |
| Simulated absorption trade (1.5R / stop / 60m) | RP-611C ledger |
| Existing OF recommendation | Existing overlay — **unchanged** |
| Existing Policy B/C plan | Existing overlay — **unchanged** |

Do not merge absorption trades into Policy B/C plans. Do not report OF reverse-flow as an absorption trade exit unless the RP-611C arm explicitly defines that coupling in a future **new** protocol version.

---

## 5. Central research question

Does candle-volume absorption predict subsequent price direction, and does L1 confirmation improve that prediction enough to justify its complexity?

Prepared outcomes: (1) absorption works, L1 adds little; (2) L1 works, absorption adds little; (3) combination stronger; (4) neither reliable — all valid.

---

## 6. Artifacts

Root: `artifacts/rp611_crypto_volume_absorption/`

```
registration.json
journals/
  RP611C_candidates.jsonl
  RP611C_confirmations.jsonl
  RP611C_l1_telemetry.jsonl
  RP611C_simulated_trades.jsonl
  RP611C_arms.jsonl
  RP611C_daily_log.jsonl
schemas/
  README.md
```

---

## 7. Governance boundaries

| System | Disposition |
|--------|-------------|
| RP-608 Policy B/C + trade plan | Frozen / unchanged |
| Existing OF latch/exit | Unchanged |
| RP-609 / RP-610 | Unchanged |
| RP-611N | Sibling programme; separate ledger |
| Gate5 | No behavioural logic |
| Live order authority | 0% |
