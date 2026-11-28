# RP-608-SCI-ADAPTER-REC-001 — P4 promote criteria (DRAFT)

**Status:** Draft only — **not** a promote / Gate 5 packet.  
**Date:** 2026-11-27 Asia/Calcutta  
**Parent:** P1 BTC `ADVANCE_CANDIDATE` · P3 Pi arm · Gate 2 v2  
**Rules:** chronosentiment-core §3b/§3c · S10 · Coralys no-touch · books never pooled

## Purpose

Predeclare how soak evidence will be judged before anyone asks for Gate 5. No live knobs change from this doc.

## Required evidence (BTC book only until ETH join)

| Gate | Requirement | Source |
| --- | --- | --- |
| **W/L direction** | Closed-trade WR on `CRYPTO_PLAN_SCI_ADAPTER_REC` moves in the **same direction** as P1 holdout vs its book baseline (P1: SCI 33.4% vs base 29.7%) | `trade_plan_outcomes.jsonl` via `rp608_sci_adapter_rec_status.py` |
| **Coverage floor** | Disclose enter rate / suppress mix; do not promote on WR alone if coverage collapses vs P1 ~40% holdout enter | events + state `sci_adapter_rec_suppressions` + DI_SKEW / PATH_INVALID counts |
| **Payoff + net** | Report payoff asymmetry + cumulative net alongside W/L (Gate 2 v2 — not WR-only) | outcomes |
| **Baselines intact** | Gate 2E B + `adapter_select` still armed observational; PRIMARY OF ±0.30 / traj15 / stops untouched | systemd env + journals |
| **Construct freeze** | Cuts stay `p_min=0.30` / `p_lb=0.18`; ladder A; pack hash unchanged during soak | frozen pack + unit env |
| **No pooling** | BTC SCI never averaged with ETH/NSE/PRIMARY | scorecard |

## Explicit non-goals

- No Gate 5 from this draft alone  
- No Coralys edits · no ETH SCI arm until join package  
- No soak-time cut ratchet  
- DI_SKEW / PATH_INVALID on the SCI twin are **disclosed** as stack interaction, not silently ignored; any proposal to strip them is a separate Anuj-gated design change

## Decision vocabulary

| Verdict | Meaning |
| --- | --- |
| `HOLD_SOAK` | Twin healthy; closed n too small or W/L inconclusive |
| `ADVANCE_CANDIDATE_SOAK` | W/L direction + coverage + payoff/net support offline ADVANCE_CANDIDATE → draft Gate 5 packet only after Anuj |
| `DO_NOT_ADVANCE` | W/L direction fails or coverage collapses with weak expectancy |
| `ABANDON` | Twin broken / construct drift / baseline contamination |

## Ops check

```bash
ssh ultraspeed@192.168.0.9 \
  'cd ~/ChronoSentiment_MEGA_FINAL && python3 scripts/research/rp608_sci_adapter_rec_status.py'
```
