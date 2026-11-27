# RP-608-SCI-ADAPTER-REC-ML1D-001 — Extension holdout (Ec2 · H9 · R3)

**Date:** 2026-11-27 Asia/Calcutta  
**Parent verdict:** **DO_NOT_ADVANCE** (unchanged · no Pi twin)  
**Extension verdict:** **EXTENSIONS_DO_NOT_RESCUE**  
**Rules:** `.cursor/rules/chronosentiment-core.mdc` §3b/§3c · Coralys sealed · same frozen cuts · no Gate 5  
**Schema:** v1.1 · ladder `R1_EC1_EC2_H9_R3`  
**Artifacts:** `docs/research/rp608_sci_adapter_rec_ml1d/btc_ml1d_package_report.json`

Same sealed BTC chronological firewall as the primary package. Cuts locked on validation E3 (Ec1 dual); extensions scored once on holdout.

---

## Holdout table

| Label | n | WR | cov | Wilson LB | mean net |
| --- | ---: | ---: | ---: | ---: | ---: |
| E0 | 2327 | 0.297 | 1.00 | 0.278 | −0.0507 |
| E1 (R only) | 1121 | 0.328 | 0.48 | 0.301 | −0.0032 |
| E3 Ec1 dual | 536 | 0.327 | 0.23 | 0.288 | −0.0037 |
| E2 Ec2 | 251 | 0.311 | 0.11 | 0.257 | −0.0076 |
| **E3 Ec2 dual** | 242 | 0.322 | 0.10 | 0.267 | **−0.0031** |
| E3 H9 fused (Ec1) | 618 | 0.325 | 0.27 | 0.290 | −0.0010 |
| E3 Ec2+H9 fused | 314 | 0.341 | 0.14 | 0.291 | +0.0211 |
| H11 SCI-v1 | 931 | 0.334 | 0.40 | — | −0.0016 |

---

## Hypothesis reads (extensions)

| ID | Result | Note |
| --- | --- | --- |
| **H8** (Ec2) | Mild net lift vs Ec1 E3 (−0.0031 vs −0.0037) | Coverage thins; not an ADVANCE signal |
| **H9** (dual vs fused) | **Falsified on this holdout** | Ec1 fused net (−0.0010) beats dual (−0.0037) |
| **H6 / R3** | Weak gap | E3 ACT: mean \(Q\) 0.324 vs emp WR 0.327 (`q−wr≈−0.002`) — not strong overconfidence |

### Caution on Ec2+H9 fused (+0.021 net)

Positive net under **improper double-count** + Ec2 on a **single** sealed holdout is **not** promotion evidence. H9 exists to reject that pattern. No twin · no authority · no Gate 5.

---

## Still deferred

- **R2** adaptive/drift conformal (H5)  
- Live-plan outcome label join (practical steer P1b)  
- ETH book (never pool)

---

*End — ML-1D extension holdout · 2026-11-27 Asia/Calcutta.*
