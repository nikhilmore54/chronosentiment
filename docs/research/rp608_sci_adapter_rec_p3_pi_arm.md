# RP-608-SCI-ADAPTER-REC-001 — P3 Pi arm (BTC)

**Protocol:** `RP-608-SCI-ADAPTER-REC-001`  
**Date:** 2026-11-27 ~13:39–13:42 IST Asia/Calcutta  
**Host:** `ultraspeed@192.168.0.9` (`LCUtest`)  
**Authority:** 0.0% · observational twin only  
**Anuj:** continue → P3 arm  
**Rules:** chronosentiment-core §3b/§3c · S1=b · S6 baselines · Coralys/Gate 5 untouched · `.cursor/rules` OFX boundary

## Deployed

| Item | Action |
| --- | --- |
| `rp608_position_decide.aarch64-linux` | Zigbuild aarch64; `.new`→`mv`; bak `*.bak.20261127T133948_pre_sci_adapter_rec` |
| `scripts/rp608_sci_adapter_rec.py` | Installed (fail-closed client) |
| `scripts/run_rp608_shadow_daemon.py` | Installed (SCI twin hook; BTC-only) |
| `scripts/rp608_trade_plan_overlay.py` | Installed + **Gate2E B import fix** (see below) |
| `docs/research/rp608_sci_adapter_rec_p1/btc_p1_side_models.json` | Frozen pack |
| `scripts/research/rp608_sci_adapter_rec_status.py` | Scorecard |
| BTC unit drop-in | `/etc/systemd/system/rp608-shadow-observer.service.d/sci-adapter-rec-twin.conf` → `RP608_SCI_ADAPTER_REC_TWIN=1` |
| ETH unit | **No SCI env** (still `ADAPTER_SELECT_15M_TWIN=1` only) |
| NSE | **Not touched** (remains `failed`) |

## Restarts

1. `rp608-shadow-observer` (BTC) — SCI arm  
2. BTC + ETH — pick up Gate2E B import fix (pre-existing `NameError` since adapter_select arm ~11:16 IST)

## Verified arms (journal)

- BTC: `adapter_select_15m_twin` **and** `sci_adapter_rec_twin` **and** `gate2e_trend_agree_twin` armed  
- ETH: `adapter_select_15m_twin` **and** `gate2e_trend_agree_twin` armed; **no** SCI twin dir/env  
- Twin dir: `live_capture/shadow_validation/crypto/trade_plan_overlay_sci_adapter_rec_v1` (`CRYPTO_PLAN_SCI_ADAPTER_REC`)  
- Post-fix: no `trend_agreement` NameError on either book  

## Collateral fix (Gate2E B)

`rp608_trade_plan_overlay.py` called `trend_agreement` / `TREND_AGREE_PROTOCOL_ID` without importing them (present since adapter_select Pi arm). Restored:

```python
from scripts.rp608_gate2e_15m_trend_agree import (
    PROTOCOL_ID as TREND_AGREE_PROTOCOL_ID,
    agreement as trend_agreement,
)
```

This restores Gate 2E B soak filtering; PRIMARY / OF ±0.30 / Coralys unchanged.

## Scorecard

```bash
ssh ultraspeed@192.168.0.9 'cd ~/ChronoSentiment_MEGA_FINAL && python3 scripts/research/rp608_sci_adapter_rec_status.py'
```

At arm: SCI / ADAPTER_SELECT / GATE2E_B `armed=True`; closed W/L still empty (soak just started).

## Out of scope / still locked

- No Gate 5 · no Coralys edits · no book pooling · no ETH SCI arm  
- Cuts stay `p_min=0.30` / `p_lb=0.18` (no soak-time ratchet)  
- Next: chronological soak vs P1 ADVANCE_CANDIDATE (P4 promote criteria only after Anuj)
