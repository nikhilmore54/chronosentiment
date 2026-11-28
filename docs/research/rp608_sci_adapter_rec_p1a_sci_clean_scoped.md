# RP-608-SCI-ADAPTER-REC — P1a SCI-clean twin (scoped)

**Date:** 2026-11-27 Asia/Calcutta  
**Status:** Code landed · **default off** · Anuj-gated arm only  
**Parent:** practical steer P1a · soak shows DI_SKEW-heavy suppress on stacked SCI  
**Rules:** `.cursor/rules/chronosentiment-core.mdc` §3b/§3c · Coralys sealed · books never pooled · no Gate 5

---

## Claim

The stacked SCI twin (`CRYPTO_PLAN_SCI_ADAPTER_REC`) inherits Gate2E **DI_SKEW / regime-align** (and PATH_INVALID). Early soak: 87 DI_SKEW + 60 PATH suppress vs 8 plans. P1a isolates **calibrator expectancy** under frozen exits by bypassing DI/regime only — selective classifier owns abstention.

## Book (separate — never pooled)

| Piece | Value |
| --- | --- |
| Book id | `CRYPTO_PLAN_SCI_ADAPTER_REC_CLEAN` |
| Dir | `trade_plan_overlay_sci_adapter_rec_clean_v1` |
| Env | `RP608_SCI_ADAPTER_REC_CLEAN_TWIN` (default **0**) |
| Pack | Same frozen BTC P1 `btc_p1_side_models.json` |
| Cuts | Unchanged `p_min`/`p_lb` (no soak ratchet) |
| Symbol | BTCUSDT only (ETH deferred) |

## What changes vs stacked SCI

| Guard | Stacked SCI | P1a SCI-clean |
| --- | --- | --- |
| SCI ENTER cuts | on | on |
| DI_SKEW / regime-align **block** | on | **off** (stamp still written; `p1a_sci_clean_bypass`) |
| PATH_INVALID | on | on (fidelity) |
| OF / stops / traj15 | frozen | frozen |

## Code anchors

- `scripts/rp608_sci_adapter_rec.py` — `clean_twin_enabled()`
- `scripts/run_rp608_shadow_daemon.py` — clean overlay arm
- `scripts/rp608_trade_plan_overlay.py` — `skip_gate2e_regime_align`
- `scripts/rp608_gate2e_regime_align.py` — book tokens `SCI_ADAPTER_REC_CLEAN` / `SCI_CLEAN`
- `scripts/research/rp608_sci_adapter_rec_status.py` — `SCI_CLEAN` row

## Arm procedure (Anuj only)

1. Deploy updated daemon scripts to Pi (no cut change).  
2. Drop-in: `RP608_SCI_ADAPTER_REC_CLEAN_TWIN=1` on BTC observer only.  
3. Keep stacked SCI armed as comparator.  
4. Score both books separately via status script.  
5. Never average CLEAN with stacked SCI / PRIMARY / ETH.

## Non-goals

No Gate 5 · no Coralys edits · no stripping DI_SKEW on the **stacked** SCI book · no ETH clean arm · no ML-1D twin (DNA).

---

*End — P1a SCI-clean scoped · 2026-11-27 Asia/Calcutta.*
