# RP-608-SCI-ADAPTER-REC-001 — P1 offline review (BTC 15m)

**Protocol:** `RP-608-SCI-ADAPTER-REC-001`  
**Date:** 2026-11-27 (Asia/Calcutta)  
**Book:** `BTCUSDT_15M` only (never pooled)  
**Authority:** 0.0%  
**Coralys:** no-touch  
**Aligned with:** `.cursor/rules/chronosentiment-core.mdc` §3b/§3c · Gate 2 v2 · charter S1–S10 freeze  

**Artifact:** `docs/research/rp608_sci_adapter_rec_p1/btc_p1_package_report.json`  
**Models:** `docs/research/rp608_sci_adapter_rec_p1/btc_p1_side_models.json`  
**Runner:** `cargo run --release -p chronosentiment_adapter --bin rp608_sci_adapter_rec_offline`

---

## Verdict

### BTCUSDT_15M — `ADVANCE_CANDIDATE`

Sealed holdout (2024-11-01…2024-12-31) shows **W/L improvement** vs ungated labeled baseline with **coverage floor met**. Payoff asymmetry and net proxy disclosed. ETH/NSE not scored this package.

| Fold | Baseline W/L | ENTER W/L | Coverage | Payoff asym | Net proxy |
| --- | --- | --- | --- | --- | --- |
| Validation | 740/2393 (30.9%) | 333/1002 (33.2%) | 41.9% | 1.91 | −0.0087 |
| **Holdout (sealed)** | 690/2327 (29.7%) | **311/931 (33.4%)** | **40.0%** | **1.98** | **−0.0016** |

- ΔWR holdout ≈ **+3.75 pp** vs baseline  
- Coverage = min(40% floor, n=931 ≥ 80) → **pass**  
- Ladder stage frozen: **A** (GLM + isotonic); Stage B not required  
- Joint cuts frozen on validation only: **p_min=0.30**, **p_lb=0.18**  
- Schema: `price_volume_15m_v1` hash `f0cc1fe4d938d2e7c00d2050fb6e721aec3a325117823df5983ebfb7b2ec6b5f`

### ETH / NSE

**Not run** — ETH 15m+label join not wired; NSE deferred per S4. No pooled metrics.

---

## Method (sealed)

1. **Features (decision-time):** side-aligned returns L3/L8/L20, mag-ranks, path, abs returns, relative volume / ΔlogV / vol-rank (13 cols; price+volume).  
2. **Labels:** REC-001-H `TARGET_BEFORE_RISK`=win, `RISK_BEFORE_TARGET`=loss; `HORIZON` censored.  
3. **Split:** train 2024-01…08 · purge 30 bars · val 2024-09…10 · **holdout 2024-11…12 never remined**.  
4. **Model:** L2 logistic per side (S1=b generator) + isotonic on validation; adaptive A→B ladder (A retained).  
5. **Policy:** ENTER iff chosen side has \(\hat p \ge p_{\min}\) and Wilson LB \(\ge p_{\mathrm{lb}}\).  
6. **Leakage audit:** no MFE/MAE/outcome as features; completed bars only; books not pooled.

---

## Caveats (binding for P2)

1. Cuts sit near the ~30% base rate — lift is real but modest; P2 twin must not ratchet cuts on soak.  
2. Net proxy (target/risk distance) slightly negative; **not** treated as Gate 2E hard fail under v2 because payoff asymmetry ≈ 2.0 (not WR-only destruction). Live friction net still required on twin scorecard.  
3. Labels are REC-001-H geometry outcomes, not Pi crypto-plan journals — P2/P3 should confirm direction on live frozen-exit books.  
4. No Gate 5 / no PRIMARY cutover / no Pi daemon changes from this memo alone.

---

## Next

- **P2** sealed observational twin (new env/protocol; leave Gate 2E B + `adapter_select` baselines running).  
- Wire ETH 15m join before ETH book package.  
- Anuj gate before any Pi arm.
