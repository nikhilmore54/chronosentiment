# RP-608-REGIME-BREAKOUT-001 Protocol

**Protocol Reference:** `RP-608-REGIME-BREAKOUT-001`  
**Scope:** Binance BTC/USDT (15m) and NSE Cash Equities (15m Intraday, 119 Symbols)  
**Execution Authority:** Strictly Read-Only Shadow Mode (**Live Order Authority: 0.0%**)  
**Holdout Partition:** Sessions 41–68 **PERMANENTLY SEALED & UNTOUCHED**  
**Development Partition:** Sessions 1–40 only  
**Bootstrap Seed:** `20261028`  
**Date:** 2026-10-28  

**Canonical promotion of:** `artifacts/rp608_successor_charter/rp608_regime_breakout_architecture_and_preregistration.md`  
**Implementation contract:** Phase 0–1 first; Phases 2–5 blocked until Arm A invariance passes.

Aligned with `.cursor/rules/chronosentiment-core.mdc`: deterministic event-driven decisions, multi-scenario robustness, sealed research ops.

---

## 1. Governing objective

Establish, through prospective experimentation, whether regime-aware, event-driven breakout selection produces a repeatable, statistically defensible, positive **net** trading edge after costs—without compromising research integrity or enabling live trading prematurely.

**Research question:** Does restricting the existing strategy to better-defined regimes—and then requiring a confirmed breakout—improve forward outcomes and net expectancy enough to justify lost coverage?

- If Arm B fails → regime filtering alone has not demonstrated value.
- If Arm B passes but Arm C fails → breakout confirmation has not demonstrated incremental value.
- If neither improves on Arm A → accept that result; do not tune around it.

Prior underperformance evidence (crypto forward-path / NSE Gate 3A–3B) is **motivation only**, not evidence that Arm C succeeds.

---

## 2. Stage 1 — Market regime classifier

Features (completed bars only; never fabricate):

1. **Kaufman ER_16:** \(ER = |C_t - C_{t-16}| / \sum |ΔC|\)
2. **VolRatio_48:** \(ATR_{14}(t) / Median(ATR_{14}, 48)\)
3. **ADX_14** (Wilder)

**Deterministic precedence:**

1. `UNAVAILABLE` — insufficient history or corrupt data  
2. `SIDEWAYS` — \(ER < 0.25\) OR \(ADX < 20\) OR \(VolRatio < 0.80\)  
3. `DIRECTIONAL` — \(ER \ge 0.40\) AND \(ADX \ge 25\) AND \(0.80 \le VolRatio \le 1.80\)  
4. `TRANSITIONING` — all remaining valid combinations  

Implementation: `scripts/market_regime_classifier.py`.

### NSE two-level context (Phase 2+)

- Market breadth, equal-weight return, dispersion.  
- `MARKET_CHOP` when \(40\% \le Breadth \le 60\%\) and \(|\bar R_{market}| < 10\) bps → stock ER hurdle rises to **0.50**.

---

## 3. Stage 2 — Breakout lifecycle (Phase 3+; semantics frozen now)

- Donchian-20 from completed bars `t-20..t-1` only (forming candle excluded).  
- Long: \(P_{ask} > Upper + \delta\); Short: \(P_{bid} < Lower - \delta\)  
- \(\delta = \max(2.0\ \mathrm{bps\ of\ mid},\ 0.5 \times \mathrm{full\_spread})\)  
- **Half-spread** recorded; caps are **half-spread**: NSE ≤ 1.80 bps; Crypto ≤ 2.50 bps.  
- Fills at ask/bid; statutory friction must not double-count spread.

### Per-quote event order (crossing before WATCH)

1. Quote age > 500 ms → `stale`; if lifecycle active → `INVALIDATED`; else no-op. Stop.  
2. Half-spread > cap → `spread`; if active → `INVALIDATED`; else no-op. Stop.  
3. Executable cross of boundary+buffer → `BREAKOUT_DETECTED` → Net R:R ≥ 1.0 → `TRADE_QUALIFIED` else `INVALIDATED` (`insufficient_rr`). Stop.  
4. Else within 0.25% of boundary and regime ∈ {TRANSITIONING, DIRECTIONAL} → `WATCH`. Stop.  
5. Else apply retreat/expiry; otherwise leave unchanged.

### Active-state handling

- **Retreat:** `{WATCH, BREAKOUT_DETECTED}` and price left approach band without cross → `INVALIDATED` (`retreat`).  
- **Expiry:** session/bar window ends without qualify → `EXPIRED` (terminal).  
- Terminal: `TRADE_QUALIFIED` | `INVALIDATED` | `EXPIRED` — no further transitions for that dedupe key.  
- Dedupe: `SHA256(session_date|symbol|boundary_price|direction)`.

---

## 4. Five critical safeguards

### 4.1 Arm A immutable

Regime/breakout fields are **observational only** until Arms B/C are explicitly enabled. Enabling telemetry must not alter baseline decision, entry, exit, holding period, or costs. Regression compares telemetry on vs off on identical fixtures.

### 4.2 Shared opportunity universe and returns

- **Universe:** every Arm A tradeable entry recommendation. Key `(market, session_date, symbol, bar_idx, direction)`.  
- Arm A non-entries → evaluation census only (not B/C candidates).  
- B/C rejects remain in denominator: `taken` | `regime_reject` | `breakout_reject`.

| Family | Definition |
|--------|------------|
| Taken-trade | Realized net return only where the arm entered |
| Opportunity-level | Taken return if entered; else **Arm A counterfactual** path |

Label `opp_net_bps` with `outcome_source: taken | arm_a_counterfactual` vs `taken_net_bps`. Rejected B/C opportunity-level return is **not** that arm’s realized return.

**Paired incremental returns:**  
\(\Delta B_i = outcome_{B,i} - outcome_{A,i}\),  
\(\Delta C_i = outcome_{C,i} - outcome_{A,i}\),  
\(\Delta C{\text{-}}B_i = outcome_{C,i} - outcome_{B,i}\).

### 4.3 Breakout semantics

As in Section 3 (frozen; crossing before WATCH).

### 4.4 No statistical leakage

Decision-time data only; warm-up → `UNAVAILABLE`; bootstrap 10,000 paired/clustered resamples, seed `20261028`. Labels: `development_is` | `prospective_shadow` | `diagnostic_replay`.

### 4.5 Executable promotion gate

Hypotheses (per market, never pooled): H_B, H_C_inc, H_C_vs_A on mean(Δ) > 0.

**Eligibility floors (not power guarantees):**

- NSE: ≥ 40 completed sessions; ≥ 100 Arm A opportunities  
- Crypto: ≥ 20 session-equivalent time-blocks; ≥ 50 Arm A opportunities  
- ≥ 30 taken trades per arm for taken-trade stats else `INCONCLUSIVE`

If floors unmet → **`INCONCLUSIVE`**; **do not relax criteria**. Phase 0 must census actual availability before interpreting IS gates.

**Conjoint pass (floors met):** opp expectancy > 0; PF ≥ 1.30 (zero-loss → `+inf` only if win/coverage floors met; zero wins & losses → fail); taken WR ≥ 45% and payoff ≥ 1.20× breakeven; coverage ≥ 20%; Holm–Bonferroni p < 0.05 on {H_B, H_C_inc, H_C_vs_A} (m=3).

---

## 5. Three-arm design (Phase 4+)

| Arm | Logic |
|-----|--------|
| A | Frozen baseline (Policy B crypto / NSE momentum) |
| B | A ∩ `DIRECTIONAL` |
| C | `DIRECTIONAL` ∩ `TRADE_QUALIFIED` |

---

## 6. Phase sequence and gates

| Phase | Exit criterion |
|-------|----------------|
| 0 | Protocol promoted; IS census recorded; Arm A fixtures frozen |
| 1 | Completed-bar regime telemetry; Arm A invariance passes |
| 2–5 | **Blocked until Arm A invariance passes** |
| Prospective (corrected Arm C) | See `rp608_regime_breakout_001_prospective_observation_protocol.md` — **not authorized until explicit versioned gate**; 20-session window with integrity at 5 |

Live order authority remains **0.0%**. Holdout Sessions 41–68 untouched. Gate 4 development floors (§4.5) are **not** automatic proof that a 20-session prospective window is sufficient.

---

## 7. Artifact index

- Classifier: `scripts/market_regime_classifier.py`  
- Census: `artifacts/rp608_regime_breakout/phase0_is_census.json`  
- Fixtures: `fixtures/rp608_regime_breakout/arm_a_*`  
- Phase 0 gate: `artifacts/rp608_regime_breakout/phase0_gate_result.json`  
- Phase 1 gate: `artifacts/rp608_regime_breakout/phase1_arm_a_invariance_gate.json`  
- Gate 4 reconciliation: `artifacts/rp608_regime_breakout/gate4_reconciliation_note.md`  
- Arm C measurement disposition: `artifacts/rp608_regime_breakout/arm_c_measurement_disposition.md`  
- Breakout lifecycle fixtures: `tests/test_rp608_breakout_lifecycle_fixtures.py`  
- Prospective observation (corrected Arm C): `docs/research/rp608_regime_breakout_001_prospective_observation_protocol.md`  
- Prospective authorization status: `artifacts/rp608_regime_breakout/prospective_v1_authorization.json`  
- Prospective preflight: `scripts/run_rp608_prospective_v1_preflight.py` → `artifacts/rp608_regime_breakout/prospective_v1_preflight.json`
