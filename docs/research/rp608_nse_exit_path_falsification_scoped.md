# RP-608-NSE-EXIT-PATH-001 — Estimand A Exit-Path & Falsification Report (Scoped)

**Protocol ID:** `RP-608-NSE-EXIT-PATH-001`  
**Disposition:** **SCOPED · QUEUED** — documentation lock; report not yet executed  
**Date locked:** 2026-11-02  
**Challenger freeze:** **LOCKED** (Amendment A1, 2026-11-02)  
**Live order authority:** **0.0%**  
**Arm A / Estimand A exit rules:** **FROZEN** until this report is produced and reviewed  
**Parent:** `RP-608-NSE-SHADOW-001` (Estimand A tape)  
**Related (do not conflate):**  
- `RP-605-EXIT-001` — historical counterfactual trailing (lookback-bias lessons bind here)  
- `RP-608-EXP-002` — offline exit-variant harness (may be reused as engine, not as substitute for this report’s questions)  
- `RP-608-OF-REC-PERF-001` / `RP-608-NSE-OF-PAPER-TRADE-001` — **out of scope** (separate cohorts)  
- `RP-608-EXIT-FALSIFICATION-001` — **registered execution tranche** (Phase 1: 60m counterfactual on square-off/max-hold; Phase 2: policies A–E). See [`rp608_exit_falsification_001_scoped.md`](rp608_exit_falsification_001_scoped.md). This EXIT-PATH scope remains the broader report charter; do not double-count Gate R.

---

## 0. Governance state (preserve)

| Item | State |
| --- | --- |
| RP-608-NSE-EXIT-PATH-001 scope | **LOCKED** |
| Arm A entries and live exit stack | **FROZEN** |
| OF cohorts | **EXCLUDED** |
| P1–P3 challenger freeze | **LOCKED** (Amendment A1) |
| Path reconstruction and report | **NOT EXECUTED** |
| Gate R | **NOT REACHED** |
| Live order authority | **0.0%** |

**Core distinction:** the report tests exit policies against the **same frozen Estimand A entries**. It does **not** infer policy from the outcomes of trades that happened to exit for a particular reason.

No daemon changes, no Arm A exit changes, and no authority change are authorized by this scope or Amendment A1.

---

## 1. Why this report exists

A partition of closed-trade **net P&L by `exit_reason`** is **not** an actionable decision instrument:

| Flaw | Consequence |
| --- | --- |
| Exit reason is endogenous | TP bucket = paths that worked; SL bucket = paths that failed |
| No counterfactual | Never answers “same entries, different exit → Δ expectancy” |
| Unequal mass / absolute ₹ | 551 vs 15 cannot score policies without normalization |
| Confounds calendar with geometry | Square-off ≠ stop/target design |
| Suggests false levers | “Horizon green / stops red” ≠ “remove stops” |

**This protocol forbids leading with exit-reason P&L tables as policy conclusions.** Such tables may appear only as descriptive appendix, never as primary evidence.

---

## 2. Decision standard (what “actionable” means)

The report must answer, with **paired or path-conditioned** evidence after Indian friction:

1. **Target reachability** — Among frozen entries, how often does the path *ever* make TP geometrically attainable (MFE ≥ target_bps), and how often does production exit before that?
2. **Stop necessity vs mistiming** — Among SL exits: was MAE already ≥ stop early, or did the path show recoverable MFE first (giveback / bad timing)?
3. **60m truncation** — Among `TARGET_HORIZON_60M` exits: distance-to-TP at exit; would TP have been tagged by bar 8 / bar 12 / pre-square-off under a **forward-only** rule?
4. **Session-bound damage** — Square-off / max-hold: contribution that is pure calendar vs already-dead paths (MAE already deep).
5. **Policy falsification** — Frozen challengers P1–P3 vs baseline, **same entries**, forward-path replay, paired Δ net ₹ / Δ net bps; session-clustered inference only when the four-session threshold is met (see §4).

No Arm A parameter change, sizing change, or authority raise may be inferred until Gate R (review) passes.

---

## 3. Governance lock (detail)

| Item | Status |
| --- | --- |
| Estimand A entry rule | Frozen |
| Production exit stack (dynamic structure TP/SL + 60m profit-conditioned horizon + 180m max + square-off) | Frozen in live daemon |
| OF ±0.30 / NSE OF paper | Frozen / separate — **excluded** from this report’s primary tables |
| Lookback option / hindsight MFE capture | **Forbidden** (RP-605-EXIT-001 lesson) |
| Challengers P1–P3 | **Frozen** — Amendment A1 |
| P4 trailing | **Excluded** from this run |
| Live authority | 0.0% |
| Report execution | Pending (after preflight) |

---

## 4. Universe, sample power & inputs

| Input | Role |
| --- | --- |
| `rp608_nse_trade_journal.jsonl` (production) | Closed Estimand A outcomes + stamped stop/target/exit_policy |
| `rp608_nse_pending_evaluations.json` / bar history | Path reconstruction for open & closed trades |
| 15m (and, if needed for stop wick audit, 1m) OHLCV | Forward path only, timestamps **> entry** |
| Friction | `calc_indian_equity_friction` — same statutory model as daemon |

**Cohort:** Estimand A LONG shadow trades only.  
**Exclude:** NSE OF paper ledger, EOD watchlist, crypto OF, Estimand joins to OF stamps.

**Sample power:**

- **≥ 2 full sessions** of reconstructible closed trades may support an **initial descriptive** report (taxonomy + horizon/stop deep dives).
- **Session-clustered confidence intervals** for paired Δ claims remain **conditional on ≥ 4 sessions**. Below that threshold, report point estimates and mark CI sections `UNDERPOWERED` — do not invent clustered significance.

---

## 4A. Data-integrity preflight (mandatory before path reconstruction)

Observed operator snapshots already disagreed on production watermarks. These are **not** automatically errors — they may reflect different timestamps, journal appends, or valuation snapshots. The report **must not silently pick one version**.

| Observed discrepancy (examples) | Likely causes to check |
| --- | --- |
| Closed today 53 vs 61 | Journal growth between queries; IST cutoff on entry vs exit timestamp |
| Horizon exits 48 vs 55 | Same watermark drift; filter on `exit_reason` vs session |
| Stop exits 5 vs 6 | Same |
| Open uGross ≈ −₹15.7k vs −₹7.3k | Different mid snapshot time; different `reference_price` age; open set unchanged vs repriced |

**Preflight requirements:**

1. Declare an **authoritative cutoff**: `{journal_path, byte_size or sha256, n_closed_rows, max(exit_timestamp_utc_ms), checkpoint_session_date, generated_utc}`.  
2. Recompute today’s closed / horizon / stop counts **at that cutoff** and publish them as the report’s binding census.  
3. For open-book uGross: record `latest_evaluations` timestamp + bar_idx used for mids; label as **mark snapshot**, not realized P&L.  
4. If two cutoffs are compared, show a short reconciliation table (Δ n, Δ net ₹) — never overwrite without explanation.  
5. Non-reconstructible paths: count and list; never silent drop.

Path reconstruction **starts only after** preflight §4A is written into the report methods appendix.

---

## 5. Required analyses (primary)

### 5.1 Path taxonomy (losers and winners separately)

For each closed trade, using **forward** bar path after entry:

| Tag | Definition |
| --- | --- |
| `ZERO_MFE` | MFE_bps ≤ 0 before exit |
| `MFE_BELOW_FRICTION` | 0 < MFE_bps < effective RT friction bps |
| `MFE_REACHED_NET_GREEN` | MFE_bps ≥ friction but exit still net loss |
| `MFE_REACHED_TP` | MFE_bps ≥ **production** target_bps (path tagged TP attainable under live geometry) |
| `MAE_HIT_STOP` | MAE_bps ≥ stop_bps before or at SL exit |

Report counts and **rates**, not only rupee totals.

### 5.2 Horizon-exit deep dive (`TARGET_HORIZON_60M`)

For each horizon exit:

- `target_bps`, `stop_bps`, stamped `exit_policy`
- MFE_bps, MAE_bps over [entry, exit]
- `distance_to_tp_bps` at exit (= target_bps − favorable_bps_at_exit; signed)
- `tp_path_hit` boolean (MFE ≥ production target before exit)
- `would_tp_by_bar_k` for k ∈ {6, 8, 12} under forward-only first-touch on **completed bar close** (primary) and optional high/low (sensitivity, labeled)

### 5.3 Stop-exit deep dive (`STOP_LOSS`)

- Time-to-stop (bars)
- Whether MFE > 0 before stop (giveback class)
- MAE at stop vs stop distance (overshoot)

### 5.4 Square-off / max-hold

- Share with MFE_REACHED_TP already true (missed TP before calendar exit)
- Share already past stop distance on path (structurally dead)
- Net bps conditional on those classes

### 5.5 Paired exit falsification (mandatory section)

**Control (Policy 0):** production exit as journaled (no rewrite of history for the control column).

#### Amendment A1 — Challenger Freeze (2026-11-02) · LOCKED

> **RP-608-NSE-EXIT-PATH-001 — Challenger Freeze:** P1 (unconditional bar+4 exit), P2 (production TP/SL without the profit-conditioned 60m horizon), and P3 (TP at **75% of the production structure-target distance**) are frozen as the primary **PATH_ONLY** challengers. Production entries, stops, sizing, friction model, max-hold and square-off constraints remain unchanged except where the individual challenger explicitly specifies otherwise. No additional variants or parameter searches are permitted in this run. **P4 is excluded.** This amendment authorizes **offline analysis only** and does **not** authorize live policy changes.

| ID | Frozen rule | Intent |
| --- | --- | --- |
| **P1** | Unconditional flat at **bar+4** (60m) — no early TP/SL | Isolate pure time exit |
| **P2** | Production TP/SL only; **remove** profit-conditioned 60m horizon; max-hold + square-off unchanged | Isolate horizon gate |
| **P3** | TP at **75% of the production structure-target distance** from entry; production stop, max-hold, and square-off **unchanged**; profit-conditioned 60m horizon **unchanged** unless it fires first under production logic | Closer target without a parameter sweep |
| **P4** | — | **Excluded** this run |

**P3 distance definition (normative):**  
Let \(D = \text{production_target_price} - \text{entry_price}\) (LONG).  
P3 target price = \(\text{entry_price} + 0.75 \times D\).  
Equivalent in bps: \(0.75 \times \text{production_target_bps}\) when target was stamped as a bps distance from entry. Use stamped journal fields; if only prices exist, derive from prices.

**Execution constraints:**

1. Identical entry set (trade_id / symbol / entry_bar / entry_price).  
2. Forward path only; exit fills at executable side + friction model.  
3. Report **paired Δ net_pnl_inr** and **Δ net_return_bps** per trade; aggregate mean Δ; **session-clustered CI only if ≥ 4 sessions** (else mark `UNDERPOWERED`).  
4. Primary mode: **PATH_ONLY** (single-name path replay). Optional secondary **PORTFOLIO** via EXP-002 must be labeled separately — never mixed into the PATH_ONLY headline.  
5. **No** filling at historical MFE; **no** `max(original, f(MFE))` payouts.

---

## 6. Metrics (per analysis block)

| Metric | Notes |
| --- | --- |
| n, win rate | Realized net>0 |
| Mean/median net ₹, net bps | After friction |
| Gross ₹ / friction ₹ | Separate |
| MFE / MAE distributions | Primary path evidence; edge ratio with 0.50 bps MAE floor for display only |
| `tp_path_hit` rate | Among all / among horizon exits (production geometry) |
| Paired Δ net | Challenger − control |
| Unmatched / non-reconstructible paths | Count explicitly — never silent drop |
| Preflight census | Cutoff fingerprint + reconciled n |

Open-trade uPNL: appendix only; never mixed into closed expectancy.

---

## 7. Explicit non-goals

- Changing live Arm A stops/targets/horizon in the daemon from this scope alone  
- Using OF paper or OF inspection cohorts as exit evidence  
- Declaring “remove stops” or “always hold to TP” from descriptive buckets  
- Curve-fitting trailing grids or any P4 / extra variants in this run  
- Raising live order authority  
- Silently choosing among inconsistent journal snapshots  

---

## 8. Deliverables

1. **This scope doc** (locked) + Amendment A1 — done.  
2. **Preflight appendix** (cutoff fingerprint + discrepancy reconciliation).  
3. **Report** `docs/research/rp608_nse_exit_path_falsification_report.md` (or dated artifact under `artifacts/`) containing §4A + §5.1–5.5 + methods.  
4. **Reproducible script path** cited in the report (prefer `scripts/run_rp608_nse_exit_path_report.py` or documented reuse of EXP-002).  
5. **Gate R disposition:** `NO_CHANGE` | `DESIGN_NEXT_CHALLENGER` | `OPEN_EXIT_REDESIGN_WORKSTREAM` — live freeze holds until redesign is separately accepted.

---

## 9. Execution order

| Step | Work |
| --- | --- |
| 1 | ~~Freeze P1–P3~~ — **done (Amendment A1)** |
| 2 | **Data-integrity preflight** (§4A) — authoritative cutoff + reconcile prior snapshot deltas |
| 3 | Reconstruct paths for production journal trades at that cutoff |
| 4 | Taxonomy + horizon/stop/square-off deep dives |
| 5 | Paired falsification PATH_ONLY for P1, P2, P3 |
| 6 | Optional PORTFOLIO secondary (labeled) |
| 7 | Write report + Gate R disposition |

---

## 10. Status

| Artifact | Status |
| --- | --- |
| Scope (this document) | **LOCKED** |
| Amendment A1 (P1–P3 freeze; P3 = 75% target distance; P4 excluded) | **LOCKED** |
| Data-integrity preflight | **PENDING** |
| Path reconstruction / report | **NOT EXECUTED** |
| Gate R | **NOT REACHED** |
| Arm A live exit change | **None** |
| Authority | **0.0%** |
