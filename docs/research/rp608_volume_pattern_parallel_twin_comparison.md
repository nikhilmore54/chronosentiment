# RP-608 — Volume-Pattern Book vs Baseline Parallel Twin

**Protocol:** `RP-608-VOLUME-PATTERN-PARALLEL-TWIN-001`  
**Date:** 2026-11-20  
**Authority:** 0.0%  
**Aligned with:** `.cursor/rules/chronosentiment-core.mdc` §3b–3c · `.cursor/rules/rp608-ofx.mdc`

## Purpose

Run **older (pre-volume) pattern logic** in parallel with the **volume-aware** pattern book so W/L, payoff, and net can be compared without a blind cutover.

Empty books beat forced bad trades; this twin makes “would the old book have traded / stamped differently?” observable.

## Hard rules

| Rule | Enforcement |
| --- | --- |
| Books never pooled | Separate ledgers / stamps; no blended W/L |
| OF ±0.30 / Estimand A geometry | Unchanged (Gate 5 freeze) |
| OFX aggression | Inert — candle OHLCV only on volume book |
| Pattern gates | Default **off** on both; enable per-book via env later |
| Live authority | 0% |

## Twin map

### Crypto

| Book ID | Directory | Volume axes | L1 OF paper |
| --- | --- | --- | --- |
| `CRYPTO_PLAN` (primary) | `…/crypto/trade_plan_overlay_v1` | **Yes** (library v2) | Yes |
| `CRYPTO_PLAN_BASELINE_PARALLEL` | `…/crypto/trade_plan_overlay_baseline_parallel_v1` | **No** (baseline library) | **No** (OF inert) |

Same Policy B decision cards feed both overlays. TP/SL/60m geometry identical; only pattern key schema + volume latch differ.

### NSE A

| Layer | Behavior |
| --- | --- |
| Fills | Single Estimand A paper book (frozen) |
| Primary stamp | `trade_pattern_match` / `trade_pattern_exit` with volume axes |
| Twin stamp | nested `baseline_parallel` without volume axes |

NSE dual **fills** are not authorized here; dual **stamps** enable Gate 2E/2X comparison without Gate 5.

## Libraries

```bash
# Volume-aware (primary)
python3 scripts/rp608_crypto_trade_pattern_library.py

# Pre-volume baseline twin
python3 scripts/rp608_crypto_trade_pattern_library.py --baseline-parallel
```

NSE continues to use `rp608_nse_a_trade_pattern_library.json` (v2); baseline match strips volume axes at lookup time.

## Comparison metrics (per book, never pooled)

Primary: **W/L** → payoff asymmetry → gross → friction → net → MFE/path → concentration.

Script: `scripts/research/rp608_volume_pattern_twin_compare.py`

## Restart epoch

Twin armed with near-simultaneous systemd restart of:

- `rp608-shadow-observer.service` (crypto) — **2026-11-20T12:59:49Z / 18:29:49 IST** · PID 42540
- `rp608-nse-shadow-daemon.service` (NSE) — **2026-11-20T12:59:37Z / 18:29:37 IST** · PID 42499

Use crypto `ActiveEnterTimestamp` as the comparative session start for CRYPTO_PLAN vs twin.

## Gate 2E follow-up (2026-11-20)

ABSTAIN study showed losses were **fast SL re-entry / path-invalid opens**, not “HIGH vol = good.”  
Crypto overlay **v1.2.5** adds same-direction **5m cooldown after fast SL/scratch** and **pre-entry forming-path invalid** suppressions (both books). See `docs/research/rp608_gate2e_crypto_abstain_entry_study_2026_11_20.md`.  
**v1.2.6 observational:** `GATE2E_PATH_DUAL_STAMP` journals tick/last vs forming-H/L diverge; live authority remains forming-H/L.

## Gate 2X follow-up (2026-11-21)

Post–Gate 2E session audit (CRYPTO_PLAN **16/33**): durable wins are quality holds to TP; **do not** promote instant TP cascade or HIGH-vol ABSTAIN (cascade not suppressed — observational only). Actionable losses are **give-backs** into SL/horizon. Overlay **v1.1**: arm `GATE2X_MFE_GIVEBACK_PROTECT` only when peak **tp_frac ≥ 0.40** and MFE clears cost+floor; after hold ≥120s exit at **cost-clear** floor. First live fire: PLAN-252313 scratch −1.21. See `docs/research/rp608_gate2x_crypto_mfe_giveback_2026_11_21.md`.

**NSE Estimand A:** same v1.1 protect on LONG tick/bar manage — `docs/research/rp608_gate2x_nse_mfe_giveback_2026_11_21.md` (books never pooled).

**v1.2 volume confirm (crypto + NSE):** HIGH candle surge → protect exit now; LOW/MED/UNK soft-breach dwells before exit; recovery clears breach (fake-reversal guard).

**v1.3:** ratchet floor `max(cost_clear, 0.30×peak_mfe)`; MED dwell 40s / LOW 90s; deep giveback (≥50% peak) shortens dwell ×0.40.

## Non-goals

- No Gate 5 change to stops/targets/OF ±0.30
- No enabling pattern gates by default
- No pooling twin PnL into baseline session reports
- No OFX volume-surge substitution for candle `vol_surge`
- No promoting ABSTAIN→ALLOW from tonight’s HIGH-vol coincidence
