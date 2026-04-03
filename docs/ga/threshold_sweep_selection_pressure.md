# Threshold sweep, gate diagnostics, and selection pressure

This note records design decisions and findings from the March 2026 work on **deterministic** threshold sweeps, **analysis-vs-live** gating, and **selection pressure** (Top-K, rank gamma, GA scenario cap). It aligns with **ChronoSentiment Core** (`.cursor/rules/chronosentiment-core.mdc`): determinism, bounded env parameters, event-driven state, and clean separation of analysis tooling from production execution.

---

## 1. Problem: flat threshold sweeps

**Symptom:** `run_threshold_sweep` produced identical rows across many `(confidence_floor, score_floor)` combinations.

**Root causes (in order of discovery):**

1. **Edge override in the gate** — When `execution_fitness >= EDGE_OVERRIDE_THRESHOLD`, `evaluate_gate` skips confidence and score floors (`override_by_edge`). Strong GA fitness made floors irrelevant for many scenarios.

2. **Intrinsic signal strength** — After sweep-only override disable, diagnostics showed **effective confidence** and **composite score** well above typical grid floors (e.g. eff_conf ≈ 1.0, composite score mean ≈ 0.88 on folder fixtures). Floors in the 0.30–0.70 band often **do not bind**.

3. **Portfolio Top-K** — With many gated candidates and `SIGNAL_TOP_K = 5`, the same top signals can be selected regardless of small floor changes; **trades** can stay at 5 while sweep metrics look flat.

4. **GA scenario Top-K** — `aggregate_strategy_reports` keeps only the top **N** scenarios by rank score before fitness aggregation (`GA_SCENARIO_TOP_K` / default cap). That can **freeze** which scenarios influence GA fitness even when ranking is sharpened.

None of these invalidate the pipeline; they define **which knob** is actually moving the needle at each stage.

---

## 2. Implemented behavior (summary)

### 2.1 Sweep disables edge override (analysis only)

In `generate_latest_signals_with_thresholds_internal`, when `sweep_disable_edge_override` is true (used by `run_threshold_sweep` only), `edge_override_threshold` is set to **`f64::MAX`**, so `override_by_edge` is never true for finite fitness. **Live** paths (`generate_latest_signals_with_thresholds`, saved-strategy loads, etc.) still use `EDGE_OVERRIDE_THRESHOLD` from the environment.

**Files:** `core/src/pipeline.rs` (`evaluate_gate`, `generate_latest_signals_with_thresholds_internal`, `run_threshold_sweep`).

### 2.2 Intrinsic gate diagnostics (`SWEEP_GATE_DEBUG`)

When `SWEEP_GATE_DEBUG=1`, the **first** grid cell of `run_threshold_sweep` collects per-scenario **intrinsic** `(effective_confidence, composite_score)` for scenarios above `MIN_TRADABLE_EDGE` (same pre-floor formulas as the gate). One summary line is printed after the sweep grid completes.

**Purpose:** See whether your floor grid overlaps the **binding** region of the distribution.

### 2.3 Example: `RUN_MODE=sweep` and grids

`core/examples/run_pipeline.rs` supports **`RUN_MODE=sweep`** to run only `run_threshold_sweep` (skip `evaluate_on_real_data`), print the **full** table, and document env-driven grids:

- **`SWEEP_CONF_FLOORS` / `SWEEP_SCORE_FLOORS`** — comma-separated floats.
- **`SWEEP_PRESET=high`** (or **`calibration`**) — defaults both axes to **0.60, 0.70, 0.80, 0.90** when explicit lists are not set.

### 2.4 Rank sharpness: `RANK_SCORE_EDGE_GAMMA`

**`selection_cap::rank_score_edge_confidence`** uses:

`confidence * max(0, edge)^gamma`, with `gamma = resolved_rank_score_edge_gamma()` (default **1.0**, clamped **[0.5, 3.0]**).

Used for pipeline `TradeSignal.rank_score`, portfolio Top-K ordering, and GA scenario ordering. Helps spread ordering when confidence saturates.

**File:** `core/src/selection_cap.rs`.

### 2.5 GA Top-K env alias: `GA_TOPK`

Logs print `GA_TOPK: scenarios_in=…, scenarios_used=…, cap=…`. The env var for GA aggregation is **`GA_SCENARIO_TOP_K`**. **`GA_TOPK`** is accepted as an **alias** when `GA_SCENARIO_TOP_K` is unset (same semantics: `0` = no cap, `N` = cap).

**File:** `core/src/selection_cap.rs` (`resolved_ga_scenario_top_k`).

### 2.6 Removed redundant logging

Duplicate **`GA_SELECTION_DEBUG`** output under `apply_ga_top_k_selection` was removed; **`GA_TOPK:`** remains the single line when scenarios are truncated.

**File:** `core/src/ga.rs`.

### 2.7 Strategy genome → scenario outcomes (`evaluate_strategy`)

Previously, the deterministic **roll** for entry aggressiveness hashed only `queue_threshold` (plus market/scenario keys), while **`base_edge` was unused** in the simulation path — so many genomes produced identical entry/exit behavior and flat GA plateaus.

**Now (deterministic, bounded):**

- **Hasher** includes `base_edge`, `take_profit`, and `stop_loss` so `roll` differs across the full genome.
- **`agg_threshold`** gets a small clamped bias from `base_edge` so similar queue/vol signals can still split across the roll.
- **`min_hold`** (bars before TP/SL scan) is `3 + (base_edge + take_profit + stop_loss) % 15`, so when TP/SL never trigger, exit-at-last-bar paths still differ by genome.

**File:** `core/src/ga.rs` (`evaluate_strategy`).

---

## 3. Environment reference (quick)

| Variable | Role |
|----------|------|
| `EDGE_OVERRIDE_THRESHOLD` | Live gate: edge can bypass confidence/score floors |
| `SIGNAL_TOP_K` | Pipeline portfolio cap after gating (`0` = unlimited) |
| `GA_SCENARIO_TOP_K` / `GA_TOPK` | GA aggregation: max scenarios before fitness combine |
| `GA_WEIGHTED_SCENARIO_PNL` | `1` = rank-weighted scenario `avg_pnl` / variance in GA fitness (default off) |
| `RANK_SCORE_EDGE_GAMMA` | Edge exponent in rank score |
| `MIN_TRADABLE_EDGE` | Gate: minimum execution fitness to trade |
| `SWEEP_GATE_DEBUG` | Print intrinsic eff_conf / composite score summary (first sweep cell) |
| `SWEEP_CONF_FLOORS`, `SWEEP_SCORE_FLOORS` | Sweep grids in `run_pipeline` (comma-separated) |
| `SWEEP_PRESET=high` \| `calibration` | High-band default grids in `run_pipeline` |

---

## 4. Interpreting typical results

- **`SWEEP_GATE_DEBUG`** with eff_conf mean ≈ 1.0 and composite score mean ≈ 0.88: many grids **below** the binding region; **score_floor** near **0.90** may be the first place aggregate metrics move (e.g. `global_avg_pnl` across all scenarios).
- **`trades=5`** with **`SIGNAL_TOP_K=5`**: portfolio cap often explains constant trade count.
- **`GA_TOPK: … cap=3`** vs **`cap=5`**: Raising GA scenario cap **expands who enters aggregation**; if **SCENARIO_DIST** leaders stay identical, the data may have a **stable dominant set** of edges (repeatable optimum), not necessarily a bug.

---

## 5. Staged mental model (from this thread)

1. **Threshold / calibration** — Sweep in the right band; use sweep-only override and `SWEEP_GATE_DEBUG` to see the distribution.
2. **Rank sharpness** — `RANK_SCORE_EDGE_GAMMA` to separate candidates when confidence is saturated.
3. **Selection expansion** — `SIGNAL_TOP_K`, `GA_SCENARIO_TOP_K` / `GA_TOPK`, `MIN_TRADABLE_EDGE` to widen or tighten pools.
4. **Aggregation weighting** — Opt-in via `GA_WEIGHTED_SCENARIO_PNL=1`: rank-score-weighted mean/variance of scenario `avg_pnl` inside `aggregate_strategy_reports_inner`. Default remains unweighted mean for backward-compatible participation/sparsity behavior.
5. **Strategy sensitivity** — Full genome mixed into deterministic roll; `min_hold` and `agg_threshold` bias tie outcomes to parameters (see §2.7).
6. **Diversity / entropy (optional later)** — Only if product needs explicit diversity pressure beyond deterministic sensitivity.

---

## 6. Weighted scenario `avg_pnl` in GA (implemented, opt-in)

**Behavior:** When `GA_WEIGHTED_SCENARIO_PNL=1` (or `true` / `yes` / `on`), `aggregate_strategy_reports_inner` computes **rank-score-weighted** mean and variance of per-scenario `avg_pnl`. Weights match **`ga_scenario_rank_score`** (same signal as scenario Top-K: edge × win-rate confidence, with `RANK_SCORE_EDGE_GAMMA` on edge). **Default is off** (unweighted mean across all scenarios) so participation and sparsity penalties stay aligned with existing tests.

**Caveat:** If many scenarios are inactive (zero trades), rank weights can emphasize active scenarios and **raise** aggregate `avg_pnl` versus a flat mean — use weighted mode when the evaluated pool is mostly active, or interpret alongside participation metrics.

**Code:** `core/src/ga.rs` (`aggregate_strategy_reports_inner`), `core/src/selection_cap.rs` (`resolved_ga_weighted_scenario_pnl`).

**Constraints:** Deterministic; same inputs → same outputs; no randomness.

---

## 7. How to run a sweep (example)

```bash
cd /path/to/ChronoSentiment_MEGA_FINAL
SWEEP_PRESET=high SWEEP_GATE_DEBUG=1 RUN_MODE=sweep DATA_SOURCE=folder \
  cargo run -p chronosentiment_core --example run_pipeline
```

Use a **single line** or `&&` between env and `cargo`; avoid a stray `\` before `cargo` (shell may treat `cargo` as a command name).

---

## 8. Related code and docs

- `core/src/pipeline.rs` — `evaluate_gate`, `run_threshold_sweep`, intrinsic metrics, `generate_latest_signals_with_thresholds_internal`
- `core/src/selection_cap.rs` — Top-K, gamma, `GA_TOPK` alias
- `core/src/ga.rs` — `aggregate_strategy_reports`, `apply_ga_top_k_selection`
- `core/examples/run_pipeline.rs` — `RUN_MODE=sweep`, presets
- Formal specs: `docs/SDS_v2_0.md`, `docs/PRD_v3_3.md` (update those if behavior becomes productized)

---

*Last updated: 2026-03-28. For Anuj; ChronoSentiment Core principles apply.*
