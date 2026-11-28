# RP-608-SCI-ADAPTER-REC-001 — P2 sealed twin (scoped)

**Protocol:** `RP-608-SCI-ADAPTER-REC-001`  
**Schema:** `1`  
**Date:** 2026-11-27 (Asia/Calcutta)  
**Authority:** 0.0% · observational twin only  
**Parent:** P1 BTC `ADVANCE_CANDIDATE` · charter S1–S10 freeze  
**Coralys:** no-touch  

## Frozen apply path

| Piece | Location |
| --- | --- |
| Frozen models | `docs/research/rp608_sci_adapter_rec_p1/btc_p1_side_models.json` |
| Cuts | `p_min=0.30`, `p_lb=0.18` (validation-locked; **do not ratchet on soak**) |
| Ladder | A (GLM + isotonic) |
| Rust apply | `rp608_position_decide --sci-adapter-rec-loop [--sci-pack PATH]` |
| Python client | `scripts/rp608_sci_adapter_rec.py` (fail-closed) |
| Twin arm | `scripts/run_rp608_shadow_daemon.py` when `RP608_SCI_ADAPTER_REC_TWIN=1` |
| Scorecard | `scripts/research/rp608_sci_adapter_rec_status.py` |

## Twin book (BTC only)

| Book | Dir | Book id |
| --- | --- | --- |
| BTC | `trade_plan_overlay_sci_adapter_rec_v1` | `CRYPTO_PLAN_SCI_ADAPTER_REC` |
| ETH / NSE | — | **Not armed** (P2) |

## Behaviour (S1=b)

- Generator may ENTER even when Coralys/Estimand emits `NO_TRADE`.
- Side = argmax(\(\hat p_L,\hat p_S\)) subject to joint cuts.
- Uses completed 15m **closes + volumes** on the decision card.
- Independent of Gate 2E B and `adapter_select` (baselines stay running).

## Env

| Var | Default | Role |
| --- | --- | --- |
| `RP608_SCI_ADAPTER_REC_TWIN` | `0` | Arm BTC observational twin (**Anuj-gated**) |
| `RP608_SCI_ADAPTER_REC_PACK` | P1 models path | Frozen pack |
| `RP608_SCI_ADAPTER_REC_BIN` / `RP608_POSITION_DECIDE_BIN` | resolve | Binary with `--sci-adapter-rec-loop` |

## Out of scope

- Flip Gate 2E B / `adapter_select` knobs  
- `coralys-*` edits · Gate 5 floor/OF/stop retunes · book pooling · ETH arm  

## P3 status

**Armed** 2026-11-27 IST Asia/Calcutta on BTC observer only — see `docs/research/rp608_sci_adapter_rec_p3_pi_arm.md`.

## Advance (soak)

Per Gate 2 v2 / S10: chronological soak W/L direction matches offline ADVANCE_CANDIDATE, coverage floor, payoff/net disclosed. No Gate 5 from this memo alone.
