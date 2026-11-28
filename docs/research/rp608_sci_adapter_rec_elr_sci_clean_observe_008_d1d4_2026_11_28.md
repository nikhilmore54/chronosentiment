# ELR-008 D1–D4 — residual reconcile + path/capture audit

**As of:** 2026-11-28 ~16:39 Asia/Calcutta  
**Parent:** `ELR-SCI-CLEAN-OBSERVE-008` · twins frozen · no exit/Gate5 mods  
**Rules:** `.cursor/rules/chronosentiment-core.mdc` §3b/§3c  

## D1 — Net = Gross − Fees − Slippage − Other (highest priority)

### Identity

| Term | Source in journal | Notes |
| --- | --- | --- |
| Gross | `gross_pnl_usdt` | mid/exec × qty; matches px×qty within rounding |
| Fees | `friction_usdt` (stamped) | Python stamp: `(N_entry+N_exit)×6bps` ≈ **~$1.20** RT |
| Slippage | none stamped | `slippage_usdt` not in outcomes; model = 0 |
| Other | **unlabeled residual** | `net_stamped − (gross − friction_stamped)` |

### Aggregate residual

| Slice | n | Σ residual | mean residual |
| --- | ---: | ---: | ---: |
| All 31 | 31 | **-16.8135** | -0.5424 |
| Non-GIVEBACK | 14 | **-16.8112** | **-1.2008** |
| GIVEBACK only | 17 | ≈ −0.0023 | ≈ 0 |

Anuj target **~$16.8135** on 14 non-GIVEBACK @ ~$1.20: **matched exactly** (Σ non-GB residual **−16.8112**; all-31 **−16.8135**).

### Mechanism (measurement failure, not market edge)

1. **GIVEBACK** closes on the **Python** path (`rp608_trade_plan_overlay.py` ~4712–4718): `net = gross − (Ne+Nx)×6/10000` and the same formula stamps `friction_usdt` → residual ≈ 0.
2. **SL / TRAIL / MAX_HORIZON** close via **Rust** lifecycle authority (`rp608_plan_lifecycle.rs`): `friction = (Ne+Nx) * round_trip_friction_bps/10000` with `round_trip_friction_bps = 12` → effective **~(Ne+Nx)×12bps ≈ $2.40**, then `net = gross − that`.
3. Python still **overwrites** `friction_usdt` with the **6 bps/leg** formula (~$1.20) while **keeping Rust `net_pnl`** (~1433–1456).
4. Therefore for the 14 non-GIVEBACK rows: `net_stamped ≈ gross − 2 × friction_stamped` (verified; `|net − (g−2f)| < 0.01`).

**Other ≈ +1× stamped round-trip fee** on Rust-closed rows — double application of the 12 bps construct vs the journaled 6 bps/leg stamp. Slippage remains 0; no separate other cost object exists.

### Per-exit residual

| Exit | n | Σ residual | mean |
| --- | ---: | ---: | ---: |
| GATE2X_MFE_GIVEBACK_PROTECT | 17 | −0.0023 | ≈0 |
| STOP_LOSS_HIT | 7 | −8.4104 | −1.2015 |
| TRAILING_STOP_HIT | 5 | −6.0007 | −1.2001 |
| MAX_HORIZON_60M_EXPIRED | 2 | −2.4001 | −1.2001 |

### Accounting-corrected nets (single friction @ stamped rate)

| Metric | Stamped Σnet | Corrected Σ(g−f) | Δ (residual) |
| --- | ---: | ---: | ---: |
| 31-event | **-58.6900** | **-41.8765** | **-16.8135** |

Corrected expectancy ≈ **−1.35**/closed vs stamped ≈ **−1.89**. Residual does **not** create a win book — economics remain negative after single friction.

## D2 — Classification audit

| Classifier | EXECUTION_FRICTION | DIRECTIONAL_FAILURE | WIN |
| --- | ---: | ---: | ---: |
| v1 on stamped net | 24 | 7 | 0 |
| on accounting-corrected net | 21 | 7 | 3 |

- **Directional 7/7 stable** under both nets (all are STOP_LOSS_HIT with adverse px).
- **3 TRAIL rows misclass as EXECUTION_FRICTION** on stamped net but are **WIN under single-friction** (gross +1.2 − fric ≈ 0+):
  - `PLAN-317980-SHORT-1795849230897` TRAIL g=1.2 stamped_net=-1.2 corrected≈0.0007
  - `PLAN-318194-SHORT-1795852416781` TRAIL g=1.2 stamped_net=-1.2 corrected≈0.0007
  - `PLAN-318227-SHORT-1795853216798` TRAIL g=1.2 stamped_net=-1.2 corrected≈0.0007
- Remaining friction class (21 corrected) still dominates; double-count inflates severity, does not invent the friction problem for GIVEBACK (17/17 already residual-clean and net-negative).

## D3 — Path reconstruction (as far as data allows)

Journal fields available: `path_h_adverse_seen`, `path_h_revisit_seen`, `gate2x_mfe_armed`, `trailing_sl_armed`, `mfe_exec_price`, `mae_exec_price`, `holding_duration_seconds`. **No first-adverse timestamp** → time-to-adverse cannot be reconstructed beyond hold duration + boolean flags.

| Exit | n | adverse | revisit | mfe_armed | trail_armed |
| --- | ---: | ---: | ---: | ---: | ---: |
| GIVEBACK | 17 | 3 | 3 | 17 | 0 |
| STOP_LOSS | 7 | 7 | 3 | 0 | 0 |
| TRAIL | 5 | 0 | 0 | 5 | 5 |
| MAX_HORIZON | 2 | 2 | 2 | 0 | 0 |

- SL: all saw adverse; 3/7 also revisited (scratch path did not fire — hold > scratch window or config).
- TRAIL: no adverse flag; MFE armed then trail fill — path was favorable then gave back to trail.
- GIVEBACK: MFE armed 17/17; mostly no adverse (protect after cost-clear floor).
- MAX_HORIZON: both adverse+revisit, never armed MFE — path never cleared giveback arm threshold.

## D4 — Capture analysis

Capture := `gross / peak_mfe` (negative when gross adverse).

| Exit | n | mean peak MFE | mean gross | mean capture | mean net stamped | mean net corrected |
| --- | ---: | ---: | ---: | ---: | ---: | ---: |
| GIVEBACK | 17 | 2.33 | +0.30 | 0.16 | −0.90 | −0.90 |
| TRAIL | 5 | 2.16 | +1.20 | 0.65 | −1.20 | ≈0.00 |
| SL | 7 | 0.68 | −2.13 | ≪0 | −4.53 | −3.33 |
| MAX_HORIZON | 2 | 0.74 | −0.42 | −0.58 | −2.82 | −1.62 |

- **GIVEBACK capture ~16%** of peak MFE — exit geometry leaves most favorable excursion on the table *and* still loses after single friction.
- **TRAIL capture ~65%** of peak MFE on gross, but stamped net forced to −1.20 by double friction; corrected net ≈ scratch.
- **SL**: MFE never meaningful vs cost; directional path failure primary.

## Decision / next

| Item | Status |
| --- | --- |
| Twins | **frozen** |
| D1 residual | **EXPLAINED** — Rust 12bps×(Ne+Nx) vs Python stamp 6bps×(Ne+Nx); Other = 1× stamped fee on 14 Rust closes |
| Corrective | **not authorized** mid-soak; measurement bug is observational finding only |
| Milestone | still first **DI/regime differential**; plus treat stamped nets as **contaminated on Rust closes** when reading W/L |

Machine ledger: `rp608_sci_adapter_rec_elr008_d1d4_ledger.json`

*End — ELR-008 D1–D4 · 16:39 Asia/Calcutta 2026-11-28.*
