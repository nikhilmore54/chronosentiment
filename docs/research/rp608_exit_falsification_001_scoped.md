# RP-608-EXIT-FALSIFICATION-001 — Estimand A Exit-Policy Falsification

**Protocol ID:** `RP-608-EXIT-FALSIFICATION-001`  
**Disposition:** **REGISTERED · PHASE 1 COMPLETE · PHASE 2 PATH_ONLY COMPLETE (2026-11-02) · Gate R NOT REACHED**  
**Date registered:** 2026-11-02  
**Programme:** RP-608 Estimand A (`RP-608-NSE-SHADOW-001`)  
**Live order authority:** **0.0%**  
**Production changes:** **Not authorized**  
**Original production journal:** **Read-only**

**Related (do not conflate):**

| Protocol | Relationship |
| --- | --- |
| `RP-608-NSE-SHADOW-001` | Parent tape — frozen entries |
| `RP-608-NSE-EXIT-PATH-001` | Broader exit-path report (taxonomy / P1–P3). This protocol is the **authorized execution tranche** that starts with the calendar-exit 60m counterfactual, then runs policies A–E. |
| `RP-608-EXP-002` | Historical offline harness — may be reused as engine, not as substitute |
| `RP-608-NSE-OF-PAPER-TRADE-001` / OF-REC-PERF | **Out of scope** |

---

## 0. Governance state

| Item | State |
| --- | --- |
| Protocol registration | **REGISTERED** 2026-11-02 |
| Entry selection / sizing | **FROZEN** |
| Existing protective exits (production TP/SL, profit-conditioned 60m, max-hold, square-off) | **FROZEN** in live daemon |
| Original `rp608_nse_trade_journal.jsonl` | **Read-only** |
| Research outputs | **Separate artifacts only** |
| Phase 1 (60m counterfactual on 64 trades) | **COMPLETE** — `artifacts/rp608_exit_falsification_001/phase1_60m_report_2026-11-02.md` |
| Phase 2 (policies A–E PATH_ONLY) | **COMPLETE** — `artifacts/rp608_exit_falsification_001/phase2_path_only_report_2026-11-02.md` |
| Gate R / live implementation | **NOT REACHED / NOT AUTHORIZED** |
| Live order authority | **0.0%** |
| Production / daemon exit changes | **Not authorized** by this protocol |

**Objective:** Falsify whether alternative exit policies improve **net** outcomes on **identical** Estimand A entries.

No daemon change, no Arm A exit change, and no authority raise are authorized by registration or by Phase 1/2 evidence alone.

---

## 1. Registration specification (binding)

| Field | Decision |
| --- | --- |
| Protocol | `RP-608-EXIT-FALSIFICATION-001` |
| Programme | RP-608 Estimand A |
| Objective | Falsify whether alternative exit policies improve net outcomes |
| Phase 1 | 60-minute counterfactual reconstruction of the 64 square-off + max-hold trades |
| Phase 2 | Controlled comparison of exit variants **A–E** |
| Entry selection and sizing | Frozen |
| Existing protective exits | Frozen |
| Original production journal | Read-only |
| Research outputs | Separate artifacts |
| Production changes | Not authorized |
| Live authority | 0% |

---

## 2. Why this protocol (narrower than EXIT-PATH)

Descriptive exit-reason P&L showed that on 2026-11-02:

- `TARGET_HORIZON_60M` was the only additive exit class (net +₹16.0k).
- `MANDATORY_INTRADAY_SQUARE_OFF` (41) + `MAX_HOLD_HORIZON_180M` (23) = **64 trades**, ≈ **−₹33.6k** net (~60% of day net loss).

That diagnosis **motivates** a controlled counterfactual; it does **not** authorize policy change. Phase 1 answers: *at the production 60m decision point, what was the mark / path quality of those 64 trades?* Phase 2 then falsifies frozen exit variants on identical entries.

---

## 3. Phase 1 — 60-minute counterfactual (authorized now)

### 3.1 Cohort

| Filter | Value |
| --- | --- |
| Session | Declared cutoff (fingerprint in artifact) |
| Protocol | `RP-608-NSE-SHADOW-001` / Estimand A LONG |
| `exit_reason` ∈ | `MANDATORY_INTRADAY_SQUARE_OFF`, `MAX_HOLD_HORIZON_180M` (alias `TARGET_HORIZON_180M` if present) |
| Expected n (2026-11-02 census) | **64** (41 + 23) |

### 3.2 Required fields (per trade)

Reconstruct **forward-only** from entry to the production 60m gate (`bars_held >= 4`, i.e. `exit_bar_idx_cf = entry_bar_idx + 4`):

| Field | Definition |
| --- | --- |
| `mtm_price_60m` | Primary: bar **close** (or live `p_ref` if bar-aligned quote replay available) at `entry_bar_idx + 4` |
| `gross_mtm_60m_inr` | `(mtm_price_60m − entry_price) × quantity` |
| `friction_60m` | Same statutory model as daemon (`calc_indian_equity_friction`) at entry→60m exit prices |
| `net_mtm_60m_inr` | gross − friction |
| `mfe_60m_bps` / `mae_60m_bps` | Favorable / adverse excursion over **(entry, +60m]** only — not full-path journal MFE/MAE |
| `would_horizon_green` | `mtm_price_60m > entry_price` (production `TARGET_HORIZON_60M` predicate) |
| `delta_vs_realized_net` | `net_mtm_60m_inr − realized_net_pnl_inr` (counterfactual flat-at-60m vs production exit) |

**Forbidden:** filling at full-path MFE; rewriting the production journal; mixing OF paper cohorts.

### 3.3 Preflight

Authoritative cutoff fingerprint before reconstruction:

`{journal_path, sha256 or byte_size, n_cohort, max(exit_timestamp_utc_ms), session_date, generated_utc}`

Non-reconstructible paths: count and list — never silent drop.

### 3.4 Phase 1 deliverables

1. Read-only input snapshot under `artifacts/rp608_exit_falsification_001/` (journal subset + bar slice used).  
2. `phase1_60m_counterfactual.jsonl` + summary markdown.  
3. No Phase 2 / no policy ranking until Phase 1 is written.

---

## 4. Phase 2 — Exit policies A–E (queued; frozen definitions)

**Control and challengers are frozen below.** Phase 2 may begin only after Phase 1 artifacts exist. PATH_ONLY paired Δ net ₹ / Δ net bps; session-clustered CI only if ≥ 4 sessions (else `UNDERPOWERED`).

| ID | Rule | Intent |
| --- | --- | --- |
| **A** | **Control** — production exit as journaled | Baseline |
| **B** | Unconditional flat at `bars_held >= 4` (60m) — TP/SL still honored if they fire first | Pure time exit (maps EXIT-PATH P1) |
| **C** | Production TP/SL + max-hold + square-off; **remove** profit-conditioned 60m horizon | Isolate horizon gate (maps EXIT-PATH P2) |
| **D** | TP at **75%** of production structure-target distance; stop / max-hold / square-off / profit-conditioned 60m unchanged unless production logic fires first | Closer target (maps EXIT-PATH P3) |
| **E** | At `bars_held >= 4`: if `p_ref ≤ entry_price`, flat; else continue production stack | Cut underwater paths at the 60m gate; keep green paths on production rules |

**P3 / D distance:** \(D = \text{production_target_price} - \text{entry_price}\) (LONG); target = \(\text{entry} + 0.75D\).

**Excluded this run:** trailing / giveback grids; OF-conditioned exits; entry or sizing changes.

---

## 5. Explicit non-goals

- Changing live Estimand A exits from Phase 1 or Phase 2 alone  
- Using OF paper / OF inspection as exit evidence  
- Declaring “always exit at 60m” from descriptive buckets without paired Δ  
- Mutating `rp608_nse_trade_journal.jsonl`  
- Raising live order authority  

---

## 6. Execution order

| Step | Work | Status |
| --- | --- | --- |
| 0 | Register this protocol + INDEX entry | **DONE** |
| 1 | Phase 1 preflight cutoff + 60m reconstruction (64 trades) | **DONE** (2026-11-02) |
| 2 | Write Phase 1 artifact | **DONE** |
| 3 | Phase 2 PATH_ONLY A–E (after Phase 1) | **DONE** (2026-11-02; CLOSE_GRID; first-exit dedupe) |
| 4 | Gate R disposition | **not reached** — no live implementation |

**Gate R labels (future):** `NO_CHANGE` | `DESIGN_NEXT_CHALLENGER` | `OPEN_EXIT_REDESIGN_WORKSTREAM` — live freeze holds until a redesign is separately accepted.

---

## 7. Artifact paths

| Path | Role |
| --- | --- |
| `docs/research/rp608_exit_falsification_001_scoped.md` | This scope (canonical) |
| `artifacts/rp608_exit_falsification_001/` | Research outputs only |
| `scripts/research/rp608_exit_falsification_001_phase1_60m.py` | Phase 1 reconstruction |
| `live_capture/.../rp608_nse_trade_journal.jsonl` | **Read-only** production SoT (Pi) |
