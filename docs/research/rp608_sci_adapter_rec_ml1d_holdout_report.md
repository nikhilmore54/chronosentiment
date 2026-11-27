# RP-608-SCI-ADAPTER-REC-ML1D-001 — Sealed BTC holdout report

**Date:** 2026-11-27 Asia/Calcutta  
**Book:** BTCUSDT 15m only (never pooled)  
**Rules:** `.cursor/rules/chronosentiment-core.mdc` §3b/§3c · Coralys sealed · ACT/PASS only · no Gate 5  
**Protocol:** `RP-608-SCI-ADAPTER-REC-ML1D-001`  
**Artifacts:** `docs/research/rp608_sci_adapter_rec_ml1d/`  
**Hypotheses:** `docs/research/rp608_sci_adapter_rec_ml1d_hypotheses.md`

---

## Verdict

**DO_NOT_ADVANCE** — no Pi twin.

| Gate | Result |
| --- | --- |
| E3 coverage floor (`enter_n≥80`) | Pass (n=536) |
| E3 W/L vs E0 | Pass (32.65% vs 29.65%) |
| E3 net vs E0 | Pass (less negative) |
| E3 net ≥ max(E1,E2) (H2) | **Fail** |
| E3 vs H11 SCI-v1 (net+WR) | **Fail** (SCI-v1 competitive) |
| Absolute \(U_t>0\) coverage | Near-empty (disclosed; rank `u_min=-0.05` used) |

---

## Join / firewall

| Split | n |
| --- | --- |
| Train (≤2024-08-31) | 9365 |
| Validation (2024-09–10, purged +30 bars) | 2393 |
| Holdout (2024-11–12 sealed) | 2327 |
| Eligible samples (non-HORIZON) | 14097 |

Coralys side = frozen-plan `direction`. Labels = `TARGET_BEFORE_RISK` / `RISK_BEFORE_TARGET`. Features = decision-time price+volume only (reuses `price_volume_15m_v1`).

---

## R1 reliability (\(Q_t\))

Validation Brier: raw **0.501** → isotonic **0.214** (H3 supportive on val).  
Frozen reliability cuts: `p_min=0.30`, `p_lb=0.18`.

---

## Ec1 economic (\(U_t\))

\[
\hat U_t = \hat Q_t\cdot\text{target} - (1-\hat Q_t)\cdot\text{risk} - \text{friction\_proxy}
\]

Friction proxy: 2 bps floor + `0.15·vol_rel·risk` (predeclared, not venue table).  
Absolute \(U>0\) almost empty at ~30% calibrated base + ~2:1 geometry → validation locked **rank cut** `u_min=-0.05` (disclosed).

---

## Holdout scorecard (primary)

| Exp | enter_n | WR | cov | Wilson LB | mean net proxy |
| --- | ---: | ---: | ---: | ---: | ---: |
| **E0** (always ACT) | 2327 | 0.2965 | 1.000 | 0.278 | −0.0507 |
| **E1** (R only) | 1121 | 0.3283 | 0.482 | 0.301 | −0.0032 |
| **E2** (Ec only) | 571 | 0.3240 | 0.245 | 0.287 | −0.0038 |
| **E3** (dual) | 536 | 0.3265 | 0.230 | 0.288 | −0.0037 |
| **H11 SCI-v1** | 931 | 0.3340 | 0.400 | — | −0.0016 |

Baseline holdout WR (all Coralys sides followed) ≈ **29.7%** — far from ~85% aspiration without extreme further selection (H10 still OPEN; Wilson LB ≪ 0.85).

---

## Hypothesis read (sealed holdout)

| ID | Read |
| --- | --- |
| H0 | E0 after-cost hostile — consistent with needing a filter |
| H1 | Partial: E1/E3 lift WR+net vs E0; not enough for ADVANCE |
| H2 | **Falsified here** — E3 net ≰ E1 net |
| H3 | Supportive on validation Brier |
| H4 | Mild WR lift; not near aspiration |
| H5–H6 | Deferred (R2/R3) |
| H7 | Weak — Ec filter does not improve net vs E1 |
| H8–H9 | Deferred |
| H10 | Not approached (Wilson LB ~0.29) |
| H11 | SCI-v1 still competitive vs E3 on this holdout |

---

## Non-advance implications

- No twin · no authority · no Gate 5.  
- Next research (if any): Ec2 liquidity conditioning, R2 drift, selection-aware R3 — **not** Coralys edits or soak remine.  
- SCI-v1 remains the frozen H11 comparator (`S1=b`).

---

*End — ML-1D sealed holdout · 2026-11-27 Asia/Calcutta.*
