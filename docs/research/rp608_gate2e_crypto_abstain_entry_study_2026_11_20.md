# RP-608 Gate 2E — Crypto ABSTAIN entry study (2026-11-20)

**Protocol:** `RP-608-GATE2E-CRYPTO-FAST-SL-COOLDOWN-001`  
**Authority:** 0.0%  
**Books:** `CRYPTO_PLAN` / `CRYPTO_PLAN_BASELINE_PARALLEL` (never pooled)  
**Aligned with:** `.cursor/rules/chronosentiment-core.mdc` §3b–3c · twin doc `rp608_volume_pattern_parallel_twin_comparison.md`

## Question

When pattern stamp is `ABSTAIN_INSUFFICIENT`, why were some trades profitable, and what should the crypto plan change?

## Session facts (CRYPTO_PLAN since twin restart)

| Cohort | n | W/L | Note |
| --- | --- | --- | --- |
| ABSTAIN closed | 37 | **10/27** | gates off → still traded |
| ALLOW closed | 3 | 1/2 | too few to generalize |

### What profitable ABSTAIN looked like

- All **SHORT**, entry **vol=HIGH**, exit **PROFIT_TARGET_HIT**
- Avg win ≈ **+9.68**; several quality holds ~5–18m
- Late cascade (21:43–21:45) also printed instant TPs (path-through target) — **not** durable entry alpha

### What losing ABSTAIN looked like

- Also mostly **SHORT + HIGH** → volume stamp alone does **not** separate win vs loss
- **22/27** losses hold **&lt;60s**; **14/27** &lt;30s
- Re-entry gaps after losses: median ~42s, **22/27** &lt;2m
- Exit reasons: `STOP_LOSS_HIT` / `EARLY_SL_PATH_SCRATCH`

## Verdict (W/L-first)

| Hypothesis | Result |
| --- | --- |
| Promote ABSTAIN→ALLOW when HIGH vol | **Reject** — losses share HIGH |
| Exit geometry (widen/tighten SL/TP) | **Not tonight’s fix** — wins clear TP; Gate 5 frozen |
| Entry anti-churn after fast SL + refuse path-invalid opens | **Accept (Gate 2E)** |

Empty book beats fast re-entry into forming-path stops.

## Crypto plan update (v1.2.5)

In `scripts/rp608_trade_plan_overlay.py` / shadow daemon:

1. **Post-fast-SL cooldown (5m, same direction)** after `STOP_LOSS_HIT` or `EARLY_SL_PATH_SCRATCH` with hold &lt;120s → `SUPPRESSED_GATE2E_FAST_SL_COOLDOWN`
2. **Pre-entry path invalid** if forming 15m high/low already through proposed stop → `SUPPRESSED_GATE2E_PATH_INVALID`
3. Daemon latches `latest_forming_kline` extremes onto decision quotes

**Not changed:** OF ±0.30, ATR stop/target multipliers, pattern-gate default (`RP608_CRYPTO_PATTERN_GATE=0`), ABSTAIN still observational unless adverse standaside gate is enabled.

## Non-goals

- No Gate 5 stop/target change  
- No symbol bans  
- No pooling twin books  
- No treating instant path-through TP as an entry pattern to learn  

## Follow-up (2026-11-21)

After Gate 2E, residual W/L damage shifted to **MFE give-backs** (horizon/SL after peak cleared costs). Handled as Gate 2X — see `rp608_gate2x_crypto_mfe_giveback_2026_11_21.md`.

## Observational dual-stamp (2026-11-21) — tick vs forming H/L

**Protocol:** `RP-608-GATE2E-PATH-DUAL-STAMP-001` · **authority 0%**  
**Live open/suppress:** still **forming 15m H/L** path-invalid only (unchanged Gate 2E).  
**Observational:** on each pre-entry path check, compare tick/last mid vs forming H/L. When they disagree, journal `GATE2E_PATH_DUAL_STAMP` (`FORMING_BLOCKS_TICK_ALLOWS` | `TICK_BLOCKS_FORMING_ALLOWS`). Suppress events also carry `tick_path_invalid` / `diverge_kind`.

**Non-goals:** no Gate 5 geometry change; no live switch to tick-only path-invalid; books never pooled (`.cursor/rules` RP-608).  
**Purpose:** quantify how often candle-extreme Gate 2E blocks paths that tick/last would still allow — evidence before any Gate 2E reopen.
