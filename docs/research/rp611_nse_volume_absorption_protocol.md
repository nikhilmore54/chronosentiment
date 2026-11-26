# RP-611N — NSE Volume Absorption Shadow Experiment

**Protocol Reference:** `RP-611-NSE-VOLUME-ABSORPTION-001`  
**Short ID:** `RP-611N`  
**Status:** **REGISTERED** (2026-11-01)  
**Live Order Authority:** **0.0%**  
**Market:** NSE Cash Equities  
**Universe:** Frozen RP-608 119-symbol census  
**Candle interval:** 15 minutes  
**Directions:** LONG (accumulation) and SHORT (distribution)  
**Execution:** Shadow simulation only  

Aligned with `.cursor/rules/chronosentiment-core.mdc`: deterministic observation, multi-scenario evaluation later, no live authority.

**Not RP-609.** RP-609 remains Indian Retail Multi-Horizon. This programme is independent.

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
| Initial stop | Beyond absorption candle extreme (LOW for LONG, HIGH for SHORT) |
| Max holding | 4 completed bars / 60 minutes |
| Ambiguous same-bar TP+SL | Stop-first (conservative) |
| Controls | A, B, C, D (identical friction/risk assumptions) |
| Initial evaluation window | 30 NSE sessions (minimum; not auto-promotion) |
| Threshold adaptation | **Forbidden** during the registered window |

**Experiment clock:** **STARTED** (see `artifacts/rp611_nse_volume_absorption/registration.json`). Phase `1_data_integrity`. Full signal observer **not deployed** until separately authorized.

---

## 2. Hypothesis

Unusually high trading volume with limited price progress against the candle direction may indicate absorption. Subsequent price confirmation may help distinguish accumulation from distribution. We test that hypothesis; we do not assume institutional agency.

Evaluate LONG and SHORT **separately** before any combined summary.

---

## 3. Universe policy

| Category | n | Treatment |
|----------|---|-----------|
| RP-608 NSE census | 119 | Observe all |
| RP-607 classified | 102 | Metadata only |
| Quarantined | 17 | Observe independently; no automatic exclusion |

No changes to RP-607 quarantine or RP-608 watchlist membership.

---

## 4. Signal definition (frozen)

For every **completed** 15m candle (zero-range → invalid):

- `Range = High - Low`
- `Body = |Close - Open|`
- `BodyEfficiency = Body / Range`
- `RVOL = CurrentVolume / Mean(Previous 20 completed candles)` — **exclude current candle**
- `UpperWick = (High - max(Open,Close)) / Range`
- `LowerWick = (min(Open,Close) - Low) / Range`

**LONG accumulation candidate:** red candle ∧ RVOL ≥ 1.50 ∧ BodyEfficiency ≤ 0.35 ∧ LowerWick ≥ 0.40  

**SHORT distribution candidate:** green candle ∧ RVOL ≥ 1.50 ∧ BodyEfficiency ≤ 0.35 ∧ UpperWick ≥ 0.40  

**Confirmation:** next completed candle closes beyond absorption midpoint (above for LONG, below for SHORT). Entry sim at following executable bar open with modeled friction. No intrabar look-ahead.

Rejected candidates remain in the ledger with reason codes.

---

## 5. Control cohorts

| Cohort | Logic |
|--------|--------|
| A | Colour reversal without volume filter |
| B | RVOL ≥ 1.5 without wick/body absorption |
| C | Absorption without confirmation |
| D | Absorption + next-candle confirmation (full strategy) |

Principal comparisons: **D vs A**, **D vs B**, after costs.

---

## 6. Phases (30 sessions)

1. Sessions 1–5: Data integrity  
2. Sessions 6–10: Signal diagnostics (no threshold changes)  
3. Sessions 11–20: Descriptive performance  
4. Sessions 21–30: Evidence assessment / extend-or-stop  

Insufficient confirmed trades → **inconclusive**, not failure of science.

---

## 7. Artifacts

Root: `artifacts/rp611_nse_volume_absorption/`

```
registration.json          # status, lock hash, clock flags
journals/
  RP611N_candidates.jsonl
  RP611N_confirmations.jsonl
  RP611N_simulated_trades.jsonl
  RP611N_controls.jsonl
  RP611N_session_log.jsonl
schemas/
  README.md
```

---

## 8. Governance boundaries

| System | Disposition |
|--------|-------------|
| RP-608 | Frozen — no mutation |
| RP-607 quarantine / classification | Unchanged |
| RP-609 / RP-610 | Unchanged |
| Crypto OF lifecycle | Unchanged |
| Gate5 | Presentation only — no behavioural logic for RP-611 |
| Live order authority | 0% |
| Automatic parameter tuning | Forbidden |

---

## 9. Success criterion

Scientific success = determine whether absorption adds predictive/economic information beyond controls A/B after friction. Profitability is not required for a valid research outcome.
