# RP-608-GATE2X-001 — Gate 2X Review Memo

**Date:** 2026-11-15  
**Authority:** 0% · no Gate 5 / live edits  
**Artifacts:** `artifacts/rp608_gate2x_001/gate2x_001_multisession_results.*`  
**Scope:** `docs/research/rp608_gate2x_001_scoped.md`  
**Amendment:** `docs/research/rp608_gate2_amendment_v2.md`

## Formal dispositions (per book)

| Book | Policy | Label | Δnet vs A | Paired sessions Δ≥0 |
| --- | --- | ---: | ---: | --- |
| Crypto plan | **G** (ambiguous→TP) | **ADVANCE_CANDIDATE** | **+302.57 USDT** | 21/21 |
| Crypto plan | **H60** (SL&lt;60s→scratch) | **ADVANCE_CANDIDATE** | **+1246.19 USDT** | 21/21 |
| Crypto plan | **H120** (SL&lt;120s→scratch) | **ADVANCE_CANDIDATE** | **+1324.42 USDT** | 21/21 |
| NSE A | **F** (friction-horizon→scratch) | **DO_NOT_ADVANCE** | −39107.50 INR | 2/12 |
| NSE A | F0 (void) | DIAGNOSTIC_ONLY | (bound only) | — |

## Interpretation

### Crypto plan — ADVANCE_CANDIDATE (Gate 3X offline authorized for G / H*)

- **G** is the cleanest: same-tick TP∧SL currently forces SL; flipping to TP is a pure tie-break change with full mark known.  
- **H60 / H120** show large net lift by scratching early noise stops instead of paying full ATR stop. **Caveat:** scratch@entry is an optimistic fill assumption (journal-anchored). Gate 3X must stress-test with tick/L1 path before any Gate 5 proposal. Prefer validating **G** first, then H* with path.

**Gate 3X (crypto) authorized to design offline:** implement G and H* in a research replay harness only — **not** in `rp608_trade_plan_overlay.py` production path until Gate 5.

### NSE A — DO_NOT_ADVANCE on F

Scratching friction-eaten 60m greens **worsens** net: statutory RT friction at entry exceeds the small negative nets those exits already booked. The needed challenger is **continue path** (skip horizon when post-friction net ≤ 0), which requires durable bar history — currently **empty on Pi**.

**Next NSE 2X step (ops + research):** restore `rp608_nse_bar_history.json` on Pi → CLOSE_GRID multi-session policies A/C/E + friction-aware continue (new **F_path**).

## What this does *not* authorize

- Live / shadow daemon edits  
- Estimand A `traj15` / OF ±0.30 changes  
- Treating F0 void as a policy  
- Gate 5 production rollout  

## Ledger update

Gate 2X-001 crypto **G/H*** → Gate 3X offline design permitted. NSE F → closed for scratch CF; path package pending bars.
