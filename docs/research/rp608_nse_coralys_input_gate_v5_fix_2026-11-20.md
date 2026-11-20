# RP-608 NSE — fix plan after 2026-11-20 losses (Coralys V5)

**Book:** NSE Estimand A (primary) · others noted separately  
**Authority:** 0% · adapter / session guards only  
**Frozen until Gate 5:** Estimand A `traj15≥0.05`, stops/targets/holds, OF ±0.30  

Aligned with `.cursor/rules/chronosentiment-core.mdc`: W/L-first, books never pooled, MFE = supporting only.

## Diagnosis → fix mapping

| Failure | Fix | Status |
|---------|-----|--------|
| Preopen MISSING + V4 `PRIOR_STRONG_TRAJ` bypass (181/194) | **V5:** always `STANDASIDE_PREOPEN_MISSING` | **Shipped** |
| 13 symbols ×~16 re-entries after stops | Max **1 entry/symbol/day**; block after `STOP_LOSS` | **Shipped** (daemon) |
| LATE weak edges | `LATE_TRAJ_FLOOR` 0.12 → **0.15** | **Shipped** |
| Stop-after-green (93% losses MFE≥12bps) | Gate **2X** offline (frozen entries) — not live stop edit | Research |
| NSE OF reverse-OF chop | Gate **2E** OF entry/regime (not ±0.30 tweak) | Research |
| Crypto plan hour-07 SHORT cluster / payoff&lt;1 | Gate **2E/2X** crypto plan (Gate 5 for live) | Research |

## Explicit non-goals

- No live stop/target/hold change  
- No OF ±0.30 change  
- No book pooling / symbol ban lists as sole rule beyond session re-entry cap  
