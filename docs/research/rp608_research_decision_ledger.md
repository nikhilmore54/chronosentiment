# RP-608 — Research Decision Ledger

**Date:** 2026-11-07 · **Amended:** 2026-11-15 (Gate 2 v2)  
**Purpose:** Prevent repeated reopening of consumed questions.  
**Authority:** 0% · live baseline/OF frozen · Gate 2 **split 2E/2X** (`rp608_gate2_amendment_v2.md`)  

## Master table

| Hypothesis | Current status | Evidence already obtained | Data consumed | Data still eligible | Next permissible action | Unauthorized |
| --- | --- | --- | --- | --- | --- | --- |
| **H1** Continuous entry quality | `NOT_READY` → **2E prep allowed** | NSE continuous magnitudes differ Oct28/30; ranking transfer weak; crypto native mom/trend/loc **absent** | NSE kite join for ranking-001; crypto availability matrix | Future crypto continuous capture or leakage-cleared proxy; new sealed windows | Availability gate + open note → Gate **2E** package | Remining contaminated windows; live entry threshold change without Gate 5 |
| **H2** Session regime | `SUPPORTED_SESSION_REGIME_DOMINANT` (descriptive) | P4 disc+val high WR dispersion; poor val economics; Oct29 concentration | Discovery ≤10-31 (6 sess); validation 11-01…05 (5 sess) | Confirmation ≥11-06 only | Prospective confirmation + assessment | Reclassify history as confirmation; session thr scans as entry filters |
| **H3** OF alignment | `UNOPENED` | Prior OF associations unstable; P4 secondary OF Missing on plan outcomes join | None as formal H3 test | Only if separately opened with decision-time OF integrity | Document evidence requirements; **no test** until open | Mixing post-entry OF with entry alignment; OF ±0.30 change |
| **H4** Adaptive execution velocity | `UNTESTABLE_UNDER_CURRENT_ENGINE_DESIGN` | Inventories + P1: simulated/instant timing; no lifecycle | Crypto plan+OF journals for inventory; overlay code audit | Only after authorized engine redesign + capture | Boundary doc closed for current engines | Latency from `activated_ms`; stub logging as velocity |
| **Gate 2X** Exit/cost geometry | `2X-001` + `2X-002` + **`2X-003 RUN`** | Crypto G/H* → Gate5; NSE F scratch **DO_NOT_ADVANCE**; G_ohlc **DO_NOT_ADVANCE**; CLOSE_GRID **C/F_path ADVANCE_CANDIDATE** (relative only; fidelity gap vs journal) | 3m Kite bar backfill + NSE journal | Gate 3X NSE CLOSE_GRID↔journal fidelity before any Gate 5 | `rp608_gate2x_003_nse_a_close_grid_fpath_review.md` | Live NSE exit edits; treating CLOSE_GRID absolute nets as live PnL |
| **Gate 2 historical** | `RUN 2026-11-15` | Unified 2E+2X on journals: NSE `BAR_ge_12` 2E ADVANCE_CANDIDATE; crypto 2X ADVANCE; crypto 2E no advance | Chrono 60/40 split; consumed windows not remine-exempt | Gate 3X (G); Gate 3E confound audit BAR_ge_12 | `artifacts/rp608_gate2_historical/` · `rp608_gate2_historical_run_review.md` | Hour bans as sole rule; live edits |
| **Gate 3X Crypto G** | `PASS` → **Gate 5 DEPLOYED** 2026-11-15 | Holdout Δnet +249.8 USDT; mid-haircut also +; sess share 1.0 | Ambiguous n=45 (holdout 37) | Monitor Pi outcomes for `ambiguous_resolved_as=TP` | `rp608_gate5_packet_crypto_g_draft.md` · overlay on Pi | H60/H120 without path stress; authority raise |
| **Gate 3X Crypto H\*** | `PASS_BOTH` → **Gate 5 H DEPLOYED** 2026-11-16 | Holdout H60_PATH Δ+255.4 · H120_PATH Δ+381.2 · paired 9/9; PATH≪OPT | 1s Binance path + plan journal; early60=443 | Monitor `EARLY_SL_PATH_SCRATCH` on Pi | `rp608_gate3x_crypto_h_path_review.md` · `rp608_gate5_packet_crypto_h_draft.md` | OPT-only live scratch; OF/ATR change |
| **Gate 3E NSE BAR_ge_12** | `CONFOUND_DOMINANT` | Δsquare_off share +0.44; within-day pairs 79/219 | Gate 2E candidate closed | Do not promote; NSE 2X path-continue after bars | `rp608_gate3_offline_proceed_review.md` | Late-bar entry cutoff in daemon |

---

## H1 — Continuous entry quality

| Field | Record |
| --- | --- |
| Why opened | Categorical bins identical; continuous magnitudes differed (NSE) |
| What tested | Ranking-001 continuous/AUC diagnostics; crypto availability |
| What failed / blocked | Holdout AUC~0.51; crypto native continuous missing |
| Reopen requires | Documented decision-time fields + leakage checks + chronological eval gate (`future H1 availability gate`) |
| Do not | Rank variables ad hoc during H2 wait |

### Future availability gate (prep only — not a test)

Required before any H1 run: decision-time continuous (or approved proxies), valid W/L+econ join, missing-data policy, look-ahead audit, discovery/validation/confirmation windows declared **before** results.

---

## H2 — Session regime

| Field | Record |
| --- | --- |
| Why opened | Session instability suspected; thr-scan path closed |
| What tested | P4 frozen chronological H2 on crypto plan |
| Result | Descriptive support; not transferable edge; Gate 2 not advanced |
| Confirmation | Protocol `rp608_h2_prospective_confirmation_protocol.md` + accumulation charter |
| **2026-11-07 update** | Pi DQ still `NEED_MORE_SESSIONS` (2/5 sess, 88/200 trades). Nov 7 weak vs Nov 6; concentration flagged. **Observation only** — no `PERSISTS` / `WEAKENS`. Operating decision: **no change** to holding / floors / hypothesis. |
| Reopen discovery? | **No** |

---

## H3 — Order-flow alignment

| Field | Record |
| --- | --- |
| Why considered | OF-supported vs not in earlier work |
| Status | Unopened |
| Prep distinction (for future open) | Decision-time OF ≠ entry alignment narrative ≠ subsequent OF changes ≠ exit behavior |
| **Construct lock (2026-11-07)** | Baseline `OF_SUP` / ±0.30 = **L1 resting liquidity imbalance**, **not** executed aggressor flow. Do not open H3 as “aggression alignment” using baseline OF stamps. Aggression + response → **RP-608-OFX** only. |
| Reopen requires | Separate open note + complete decision-time fields + explicit construct (L1 vs aggressor) + no post-entry predictors |

---

## H4 — Adaptive execution velocity

| Field | Record |
| --- | --- |
| Why opened | Continuous state vs urgency hypothesis |
| What tested | Inventories NSE+crypto; P1 engine capability |
| Result | Untestable under current crypto engines |
| Boundary | `rp608_h4_current_engine_boundary.md` |
| Reopen requires | Authorized redesign + capture + integrity + governance |

---

## Cross-cutting consumed windows (do not recycle)

| Dataset / window | Consumed by |
| --- | --- |
| NSE WR discovery 001/002 holdout thr scan | Contaminated for thr confirmation |
| NSE RANKING-001 join windows | Ranking diagnostics |
| Crypto plan ≤2026-10-31 | H2 discovery |
| Crypto plan 2026-11-01…05 | H2 validation |
| Crypto plan ≥2026-11-06 | H2 confirmation only (accumulating) |

## Programme state

### 2026-11-15 — Gate 2 amendment v2 (Anuj)

**Binding:** `docs/research/rp608_gate2_amendment_v2.md`

- Monolithic Gate 2 advance rule **superseded** (v1 DO NOT ADVANCE records retained as history).  
- Split **Gate 2E** (entry) / **Gate 2X** (exit/cost on frozen entries); either may ADVANCE independently.  
- Live baseline / OF ±0.30 / authority **unchanged** until Gate 5.  
- Holding amended: H2 confirmation continues; **2X offline + 2E prep** permitted.  

### 2026-11-07 — prior

**CONFIRMATION-ONLY HOLDING** (workstream ban) — superseded 2026-11-15 for workstream scope only.  
H2 floors/labels unchanged. 85% WR not the H2 pass bar.

**Nov 7 operating decision (Nikhil):** held protocol unchanged. **Nov 15:** Gate 2 amended as above.

### Construct distinction lock (2026-11-07) — interpretation only

| Item | Record |
| --- | --- |
| Decision | `ACCEPT_CONSTRUCT_DISTINCTION` |
| Evidence | `docs/research/rp608_orderflow_side_quantity_semantics_audit.md` §1 / §1b |
| Baseline OF | Resting L1 bid/ask imbalance — **leave unmodified** |
| Intended volume/aggression OF | **Not** baseline OF; research home = **RP-608-OFX** |
| Forbidden | Equating `OF_SUP` with executed buying pressure; using L1 imbalance as absorption proof; retrospective OF relabel; Gate 2 / authority / ±0.30 change from this clarification |
| Holding / Gate 2 / authority | **Unchanged** |  
