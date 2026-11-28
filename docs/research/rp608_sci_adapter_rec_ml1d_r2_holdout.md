# RP-608-SCI-ADAPTER-REC-ML1D-001 — R2 drift / ACI holdout (H5)

**Date:** 2026-11-27 Asia/Calcutta  
**Parent verdict:** **DO_NOT_ADVANCE** (unchanged)  
**H5 read:** **PARTIAL_EARLIER_FLAG_NO_WL_LIFT**  
**Rules:** `.cursor/rules/chronosentiment-core.mdc` §3b/§3c · Coralys sealed · no Gate 5  
**Schema:** v1.2 · ladder includes R2  
**Method:** ACI-style nonconformity monitor (γ=0.05, α\*=0.30) vs static rolling Brier (W=200, δ=0.03)

---

## Design (predeclared)

| Piece | Spec |
| --- | --- |
| Nonconformity | win → \(1-\hat Q\); loss → \(\hat Q\) |
| ACI update | \(\ell_{t+1}=\mathrm{clip}(\ell_t+\gamma(e_t-\alpha^\star))\) |
| R2 ACT | E1 evidence_ok ∧ \(\hat Q \ge 1-\ell_t\) (prospective) |
| Static flag | rolling Brier > val baseline + δ |
| ACI flag | \(\ell_t > \alpha^\star+0.05\) |

Warm on validation, score holdout online (update after reveal).

---

## Holdout results

| Monitor | First flag index |
| --- | --- |
| Static rolling Brier | **None** |
| ACI level | **1** (earlier) |

| Policy | n | WR | net proxy |
| --- | ---: | ---: | ---: |
| E1 static | 1121 | 0.328 | −0.0032 |
| E1 + R2 ACI | 55 | 0.291 | −0.135 |

| Stratum | Brier |
| --- | ---: |
| High vol_rel | 0.207 |
| Low vol_rel | 0.210 |

---

## H5 verdict

ACI **flags earlier** than static (static never trips on this holdout), but the R2 ACT gate **collapses coverage** and **worsens W/L+net**.  
→ **PARTIAL** for detection · **not** an ADVANCE lever · no twin.

---

## Ladder status

R1 ✓ · Ec1/Ec2/H9/R3 ✓ · **R2 scored** · primary **DO_NOT_ADVANCE** · extensions do not rescue · R2 does not rescue.

---

*End — ML-1D R2 holdout · 2026-11-27 Asia/Calcutta.*
