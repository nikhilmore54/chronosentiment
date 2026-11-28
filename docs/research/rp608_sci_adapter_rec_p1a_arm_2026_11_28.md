# P1a SCI-clean — TRY arm (2026-11-28 Asia/Calcutta)

**Method:** try → fail → observe → fix → repeat (Anuj)  
**Rules:** `.cursor/rules/chronosentiment-core.mdc` §3b/§3c · Coralys sealed · books never pooled · no Gate 5  
**Host:** `ultraspeed@192.168.0.9` · `rp608-shadow-observer`  

---

## What we tried

Arm separate observational book **without** inherited DI_SKEW/regime block, same frozen BTC P1 pack/cuts as stacked SCI.

| Piece | Value |
| --- | --- |
| Env | `RP608_SCI_ADAPTER_REC_CLEAN_TWIN=1` |
| Drop-in | `/etc/systemd/system/rp608-shadow-observer.service.d/sci-adapter-rec-clean-twin.conf` |
| Book | `CRYPTO_PLAN_SCI_ADAPTER_REC_CLEAN` |
| Dir | `trade_plan_overlay_sci_adapter_rec_clean_v1` |
| Pack/cuts | Unchanged `btc_p1_side_models.json` |
| Stacked SCI | **Still armed** (comparator — not flipped) |

## Immediate verify (~08:03 IST)

| Book | armed | closed | note |
| --- | --- | --- | --- |
| SCI stacked | True | 1/37 | hostile soak continues |
| **SCI_CLEAN** | **True** | 0/0 | PLAN_CREATED=1 · open=1 |

## Observe next

Score CLEAN vs stacked separately (never pool). Watch: DI_SKEW suppress on CLEAN should stay ~0; PATH_INVALID may still fire; closed W/L+payoff+net.

## Fix later (only after observe)

If CLEAN still fails after-cost → next sealed package from that failure mode (not remine). If CLEAN owns coverage and improves expectancy → evidence for selection-ownership thesis.

---

*End — P1a TRY arm · 2026-11-28 Asia/Calcutta.*
