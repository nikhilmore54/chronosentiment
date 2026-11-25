# RP-608 Gate 2E — Shadow thin-WR allow stamps (2026-11-22)

**Protocol:** `RP-608-GATE2E-SHADOW-THIN-WR-001`  
**Rule id:** `SHADOW_ALLOW_IF_N10_WR_GT_035`  
**Authority:** 0% · stamp-only · **no trades opened**  
**Aligned with:** `.cursor/rules/chronosentiment-core.mdc` §3b–3c · `rp608_wl_improvement_plan.md` · sealed allow_only package

## Why

Live PRIMARY `RP608_CRYPTO_PATTERN_GATE=allow_only` fail-closes on `ABSTAIN_INSUFFICIENT`. Library adverse/neutral floor remains **n≥30**, so cells with **n∈[10,29]** and **hist_wr>0.35** still suppress (~156 historical suppress events in the thin-high-WR band). Full ABSTAIN counterfactual lost money overall → **keep allow_only**. This line only **observes** the thin-high-WR subset for later join to BASELINE outcomes (books never pooled).

## Behavior

| Live PRIMARY | Shadow stamp |
| --- | --- |
| Suppress `SUPPRESSED_TRADE_PATTERN` | Unchanged |
| Open paper plan | **Never** for shadow |
| OF ±0.30 / traj / stops | **Unchanged** (no Gate 5) |

**Emit when** (PRIMARY suppress path): prediction `ABSTAIN_INSUFFICIENT` (or label `INSUFFICIENT`), `cohort_n ≥ 10`, `hist_wr > 0.35`.

**File:** `{crypto_overlay_dir}/gate2e_shadow_allows.jsonl` (append-only)

**Env:** `RP608_CRYPTO_GATE2E_SHADOW_THIN_WR` — **default ON (`1`)** because stamp-only is safe. Set `0` to disable.

## Schema (core)

`shadow_rule_id`, `pattern_id`, `prediction`, `hist_wr`, `cohort_n`, `plan_id`, `direction`, `suppressed_ms`, `book_id` (+ `match_mode`, `gate_mode`, `authority_pct=0`, `opens_trade=false`).

## Code

- Predicate / schema: `scripts/rp608_crypto_trade_pattern_match.py`
- Emit on suppress: `scripts/rp608_trade_plan_overlay.py` → `_maybe_log_gate2e_shadow_allow`
- Tests: `tests/test_rp608_crypto_trade_pattern.py` · `tests/test_rp608_trade_plan_overlay.py` (`test_18_…`)
- Prospective CF probe (price **+** Gate2E candle vol): `scripts/research/rp608_probe_suppressed_prospective.py`
  - Stamps `vol_burst_20` / `vol_surge` / `exit_vol_*` on each CF row (candle OHLCV/SMA20 ≠ OFX)
  - Batch: one CF per `bar_idx`; W/L on closed-path only (`OPEN_MTM` excluded)
  - Do **not** loosen `allow_only` on price path alone

## Live volume vs BACKOFF `pattern_id` (2026-11-22)

`pattern_id` from BACKOFF can show mined `VUNK` while the decision-time candle axis is `VMED`/`VHIGH`.  
Suppress + shadow stamps now carry:

| Field | Meaning |
| --- | --- |
| `pattern_id` | Matched library cell (may drop/alter V*) |
| `feature_pattern_id` | Exact live axes key (incl. real `V*`) |
| `live_vol_surge` / `live_vol_burst_20` | Gate 2E candle volume at suppress (≠ OFX) |

Join / CF reads must use **live** vol, not the mined `pattern_id` tail. `allow_only` unchanged.

## Live refresh (2026-11-22 ~13:13 IST)

| Book | Finding |
| --- | --- |
| BTC PRIMARY | Thin-WR stamps today **109** (live_vol mix LOW/MED/HIGH). Dominant suppress still BACKOFF → mined `VUNK` parent with `live_vol=LOW` / `feature_pattern_id=…VLOW`. |
| BTC shadow→BASELINE join (24h) | joined **35** · W/L **0/35** · net **−131.11** (all MED in joined set) → **no ADVANCE**; keep `allow_only`. |
| ETH PRIMARY | No `gate2e_shadow_allows.jsonl` yet — suppress cohorts peak hist_wr≈0.29 (&lt;0.35 thin-WR floor). Shadow env default ON; nothing to stamp. |
| Trail on BASELINE opens | Not armed: BTC fav≈28 &lt; arm≈81 (30% risk); ETH fav≈1.81 &lt; arm≈2.15. Manage marks fresh (disk age &lt;1s). |

### Later same day (~13:48 IST / 08:18Z)

| Book | Finding |
| --- | --- |
| BTC PRIMARY | Suppress mix **ABSTAIN 811 / ADVERSE 146** since gate-on — Gate2E adverse cells firing (`STANDASIDE_ADVERSE_PATTERN`). Still **0** PRIMARY opens. |
| BTC BASELINE | Prior 60m hold TIME_EXIT’d; new shorts cycling; since-gate W/L **13/77** net **−120**. |
| ETH BASELINE since gate | **8/8 losses** (trail churn + SL + TIME_EXIT) net ≈**−27** — ungated control still toxic; reinforces ETH `allow_only`. |
| Soak 24h | nonempty books **100% rust** · `PASS_PARTIAL`. |

Books never pooled. OF ±0.30 / Gate 5 unchanged.
