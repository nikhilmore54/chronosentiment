# RP-608-NSE-CORALYS-INPUT-GATE-001 — live observation (2026-11-16)

**Authority:** 0% · adapter-only · coralys crates untouched · OF ±0.30 / Estimand A thresholds frozen (Gate 5 not opened).

## Deploy
- Env: `RP608_NSE_CORALYS_INPUT_GATE=1` on `rp608-nse-shadow-daemon.service`
- Pi PID single writer; module `scripts/rp608_nse_coralys_input_context.py`
- Decision code when blocked: `STANDASIDE_CORALYS_INPUT_GATE`

## Historical baseline (closed A-like journal, entry clock IST)
Source: `live_capture/shadow_validation/nse/test_run/rp608_nse_trade_journal.jsonl` (11 sessions).

| Bucket | n | W/L | WR | avg net (INR) |
|--------|---|-----|----|---------------|
| Entry before 10:45 | 4361 | 1829/2532 | 41.9% | (noisy; see journal) |
| Entry ≥10:45 | 3159 | 1351/1808 | **42.8%** | −50.28 |

Post-10:45 WR historically ≈ pre-10:45; use this WR/net as the comparison for gated continuous-session entries today onward.

## Live readiness (deploy ~10:41 IST)
- Pre-open snapshots for 2026-11-16: present under `test_run/`
- Next-session watch file found under `nse/` but **stale** (`next_session_date=2026-11-13`) → prior support fail-open (`prior_support=null`); continuous gate still uses pre-open gap + traj15
- Observe: entry rate, `STANDASIDE_*` reasons, W/L and net for entries with `entry_timestamp` ≥10:45 IST vs historical row above

## Explicit non-goals
- No Gate 5 baseline change
- No OF / OFX authority
- Do not pool books
