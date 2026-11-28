# ELR-008 D5–D8 — cost contract, exit families, path observability, sealed hypotheses

**As of:** 2026-11-28 ~16:49 Asia/Calcutta  
**Parent:** `ELR-SCI-CLEAN-OBSERVE-008` (preserved) · D1–D4 stand  
**Rules:** `.cursor/rules/chronosentiment-core.mdc` §3b/§3c · twins frozen · no exit / Coralys / Gate5 mods  
**Scope:** **D5 = design memo only** (no implement unless Anuj says). D6–D8 observational / deferred.

---

## D5 — Cost contract (design memo; not armed)

### Authoritative economics identity

```
Fees_usdt     = (N_entry + N_exit) × (6 / 10_000)     # 6 bps per leg
Slippage_usdt = 0                                      # until a sealed slippage model exists
Other_usdt    = 0                                      # no unlabeled residual allowed
Net_usdt      = Gross_usdt − Fees_usdt − Slippage_usdt − Other_usdt
```

Equivalent RT statement: **12 bps round-trip on average notional** is *(5 fee + 1 slip) × 2 legs* in CONFIG comments, but the **executable contract for Net** is the **6 bps/leg × (Ne+Nx)** form used by the Python GIVEBACK path today — because that is what `friction_usdt` already stamps.

### Single Net formula (both authorities)

| Closer | Today | Contract (design) |
| --- | --- | --- |
| Python GIVEBACK | `net = gross − (Ne+Nx)×6/10000` + same stamp | **keep** (already compliant) |
| Rust lifecycle (SL/TRAIL/TP/HORIZON) | `friction = (Ne+Nx) × round_trip_friction_bps/10000` with `round_trip_friction_bps=12` → **~2× stamped fee** | change to **`(Ne+Nx)×(fee_bps_per_leg/10000)` with `fee_bps_per_leg=6`**, then `net = gross − friction`; stamp identical `friction_usdt` / `fees_usdt` / `slippage_usdt=0` |
| Python merge when Rust returns `net_pnl` | keeps Rust net, **re-stamps** 6bps friction → residual | either recompute net from stamped fees **or** trust Rust only after Rust matches contract; **forbid** mismatched stamp |

### Invariant (must hold on every closed outcome)

```
| net_pnl_usdt − (gross_pnl_usdt − friction_usdt − slippage_usdt) | < 0.02 USDT
```

Fail → `DATA_MEASUREMENT` tag; do not feed W/L / expectancy without flag.

### Explicit non-goals for D5 implement (when authorized)

- No change to OF ±0.30, stops/targets/traj15, Gate 5 Adaptive DNA, Coralys crates, twin arms, or authority %.
- No mid-soak rewrite of historical journals; optional **offline recomputed column** `net_contract_v1` for research reads only until a sealed cutover.
- No conflating `round_trip_friction_bps=12` as “multiply (Ne+Nx) by 12bps” — that misread is the D1 bug. Prefer rename in design: `fee_bps_per_leg = 6`, `slippage_bps_per_leg = 0` (or 1 only if slip model sealed separately).

### Suggested sealed package id (later)

`RP-608-COST-CONTRACT-V1` — implement only on Anuj go; Gate 5 not required if measurement-only and books stay 0% authority, but still needs Anuj arm + soak comparator.

---

## D6 — Exit-family diagnosis (31-event CLEAN / matched stacked)

Evidence from ELR-008 + D1–D4; accounting-corrected nets preferred for severity.

| Family | n | Path sketch | Economics (corrected) | Primary failure class | Design implication |
| --- | ---: | --- | --- | --- | --- |
| **GIVEBACK** | 17 | MFE armed 17/17; adverse rare (3); capture ~16% of peak MFE | mean net ≈ **−0.90**; residual ≈0 | **EXECUTION / exit geometry** — cost-clear floor exits leave most MFE; still lose after single fee | Later sealed hyp: exit geometry / floor, **not** selection cut; **no mid-soak change** |
| **TRAIL** | 5 | MFE+trail armed; adverse 0; capture ~65% gross | corrected net ≈ **0**; stamped −1.20 | Mostly **measurement** on stamped net; economically scratch after contract | Fix cost contract first; trail DNA frozen for now |
| **STOP_LOSS** | 7 | adverse 7/7; revisit 3/7; MFE small | mean corr net ≈ **−3.33** | **DIRECTIONAL** stable under both nets | Selection / side quality hyp later; books never pooled |
| **MAX_HORIZON** | 2 | adverse+revisit; MFE never armed | corr ≈ **−1.62** | Weak path + time — horizon mismatch secondary to never arming giveback | Observability (D7) before horizon retune |

**Family ranking for next FIX (after DI milestone / Anuj):** (1) cost-contract measurement → (2) GIVEBACK capture economics → (3) directional SL selection — never one universal corrective.

---

## D7 — Timestamped path observability (design)

### Gap

Today: `path_h_adverse_seen`, `path_h_revisit_seen` booleans; `mfe_exec_price` / `mae_exec_price`; `holding_duration_seconds`. **No** first-adverse / first-revisit / MFE-arm / trail-arm / giveback-trigger timestamps → D3 could not compute time-to-adverse.

### Minimal stamp set (design; not implementing)

| Field | Meaning |
| --- | --- |
| `path_first_adverse_ms` | first tick MAE clears meaningful floor vs entry |
| `path_first_revisit_ms` | first return toward entry after adverse |
| `gate2x_mfe_armed_ms` | when peak MFE crossed arm threshold |
| `trailing_sl_armed_ms` | when trail armed |
| `giveback_trigger_ms` | when giveback condition first true |
| `peak_mfe_time_ms` | time of `mfe_exec_price` extreme |

Derived: `time_to_adverse_s`, `time_to_mfe_arm_s`, `mfe_to_exit_s`. Fail-closed: nulls allowed; never invent.

### Boundary

Observability-only; does not change exits. Can ship as journal enrichment under separate sealed id after Anuj go. Prefer Rust lifecycle authority as single writer for path stamps (determinism).

---

## D8 — Later sealed hypotheses (not opened)

Hold until: (a) cost-contract reading policy agreed, and (b) first DI/regime differential **or** Anuj explicitly opens a FIX.

| ID (placeholder) | Hypothesis | Depends on | Failure criteria (sketch) |
| --- | --- | --- | --- |
| H-COST-1 | Aligning Rust+Python to D5 removes residual and stops TRAIL false EXEC labels without changing W/L direction of GIVEBACK/SL families | D5 implement | Residual ≠0 on new closes; or GIVEBACK expectancy worsens vs frozen baseline under contract nets |
| H-GB-CAP-1 | GIVEBACK low capture (~16%) is the dominant after-cost bleed once measurement is clean — exit geometry package beats selection cut | D5 + D6 | Corrected GIVEBACK mean net not improved vs holdout; or thins coverage without payoff lift (Gate2 v2) |
| H-SL-DIR-1 | SL cluster is selection/side failure separable from friction — CLEAN bypass value only testable on DI-differential events | DI milestone + D6 | No DI exposure → inconclusive; with DI, CLEAN must beat stacked on event-aligned expectancy |
| H-PATH-1 | Timestamped path (D7) changes failure taxonomy enough to reorder FIX priority | D7 | Taxonomy stable → observability nice-to-have only |

No package sealed here. No remine / ML-1D / Coralys / Gate5 / pooling.

---

## Decision board

| Item | Status |
| --- | --- |
| ELR-008 | **preserved** |
| Twins | **frozen** |
| D5 | **design memo only** — await Anuj implement |
| D6–D8 | recorded · no arm |
| Milestone | first DI/regime differential · treat Rust-closed stamped nets as contaminated until D5 |

---

## Anuj disposition addendum (2026-11-28 ~16:56 Asia/Calcutta)

Source: store `sci-adapter-rec-elr-008-d5-disposition-anuj-2026-11-28.md`. **Not implementation authorization.** Docs only. D1–D4 preserved; no predictive experiment; twins frozen; exits / Coralys / Gate5 untouched.

### Locked disposition

| Item | Decision |
| --- | --- |
| ELR-008 D1–D4 | Preserved (retrospective research sufficient) |
| D5 | Design memo only; clarify fee/slippage semantics **before any code** |
| D6 | Retrospective analysis; `EXECUTION_FRICTION` ≠ proven root cause |
| D7 | Observability specification; event-order + single-writer required |
| D8 | Deferred hypotheses with authorization boundaries |
| BTC twins | Frozen |
| ETH research | Independent |
| ML-1D | DO_NOT_ADVANCE |
| New predictive model | Not selected |
| Exit modifications | Not authorized |
| Coralys / Gate 5 | Untouched |

### D5.1 BLOCKER — cost semantics (Anuj must settle before implement)

Config comments: `6 bps/leg = 5 fee + 1 slippage`. Earlier memo draft: Fees=6, Slippage=0. Same total, different components.

| Interpretation | Fee | Slippage | Total (per leg) |
| --- | ---: | ---: | ---: |
| Actual 6 bps fee | 6 | 0 | 6 |
| Actual 5+1 | 5 | 1 | 6 |
| All-in friction proxy | not attributed | not attributed | 6 |

**Anuj preference (pending confirm):** if config 5+1 is authoritative:

```
Fees     = (Ne + Nx) * 5 / 10000
Slippage = (Ne + Nx) * 1 / 10000
Other    = 0
Net      = Gross - Fees - Slippage - Other
```

Else: **6 fee + 0 slip**, and reconcile old config comments. **Do not** call a composite estimate “actual exchange fees.”

**Invariant (every closed outcome):**

```
| Net − (Gross − Fees − Slippage − Other) | < 0.02 USDT
```

Fail → `DATA_MEASUREMENT`; exclude from unqualified W/L and expectancy.

Also preserve: (1) strict mathematical net sign; (2) near-zero economic outcome flag. **Do not** promote ~+0.0007 USDT TRAIL scratches into meaningful profitability evidence.

### Recommended sequence (no implement until D5.1)

1. **D5.1** — Freeze cost semantics (Anuj confirms **5+1 vs 6+0**). No implementation.
2. **D5.2** — Offline accounting tests; Python/Rust parity across GIVEBACK/SL/TRAIL/MAX_HORIZON; preserve original journals; compute `net_contract_v1` separately.
3. **D5.3** — Accounting validation on historical samples + synthetic boundaries.
4. **D7** — Path observability design: timestamp semantics, ordered events / sequence numbers, common journal-writing interface (**Python GIVEBACK must reach same ordered journal as Rust**). Instrumentation only.
5. **D6** — Exit-family retrospective under reconciled economics; record aggregation method for capture ratios (mean of ratios vs ratio of means); MFE is hindsight, not proof of executable capture.
6. **D8** — Select **one** failure mechanism + one falsifiable corrective; explicit Anuj authorization before any sealed package.

### D6 / D8 notes (disposition)

| Hypothesis | Assessment | Condition |
| --- | --- | --- |
| H-COST-1 | **Priority 1** | Cost parity + historical **restatement** of all candidates/comparators under same contract |
| H-GB-CAP-1 | Deferred | Separate authorized exit-research path; **Gate 5**; hypothesis ≠ permission |
| H-SL-DIR-1 | Conditional | Requires actual DI differential exposure |
| H-PATH-1 | Useful | Observability-only |

- `EXECUTION_FRICTION` label ≠ proven economic root cause until D5.1–D5.3 + D6 under reconciled nets.
- GIVEBACK: positive gross / negative after single-friction remains under investigation (layer 3), not a license to retune exits.

### Four research layers (keep separate)

1. **Measurement reliability** — Rust/Python accounting defect (14 rows).
2. **Directional reliability** — 7 SL failures remain after correction.
3. **Economic capture** — GIVEBACK positive gross, negative after single-friction; cause under investigation.
4. **Selection differentiation** — SCI_CLEAN vs stacked still untested (no DI differential).

*End — ELR-008 D5–D8 + Anuj disposition · 16:56 Asia/Calcutta 2026-11-28.*
