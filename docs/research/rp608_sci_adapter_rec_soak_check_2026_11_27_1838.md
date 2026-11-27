# SCI-ADAPTER-REC soak check — ~18:38 Fri IST Asia/Calcutta

**Host:** `ultraspeed@192.168.0.9` · P3 armed ~13:40 IST  
**Scorecard UTC:** 2026-11-27T13:08:48Z → **18:38 Asia/Calcutta**  
**Rules:** `.cursor/rules/chronosentiment-core.mdc` §3b W/L-first · S6 baselines · Coralys untouched · books never pooled

## Arms

| Twin | Status |
| --- | --- |
| BTC SCI stacked | armed (`RP608_SCI_ADAPTER_REC_TWIN=1`) |
| BTC SCI-clean P1a | **not armed** (code landed; Anuj gate) |
| ETH SCI | not armed |
| adapter_select / Gate2E B | armed observational |

## Closed-trade scorecard (per book — not pooled)

| Book | W/L | n | WR | net | Notes |
| --- | ---: | ---: | ---: | ---: | --- |
| **SCI_ADAPTER_REC** | **0/8** | 8 | 0.0 | −23.72 | Early hostile; disclose |
| PRIMARY_A | 794/1428 | 2222 | 0.357 | −2031 | long-run baseline |
| BASELINE | 119/616 | 735 | 0.162 | −1217 | |
| ADAPTER_SELECT | 0/5 | 5 | 0.0 | −10.68 | |
| GATE2E_B | 0/21 | 21 | 0.0 | −40.69 | |

## SCI stack interaction (events)

| Event | Count |
| --- | ---: |
| `SUPPRESSED_GATE2E_DI_SKEW` | 87 |
| `SUPPRESSED_GATE2E_PATH_INVALID` | 60 |
| `SUPPRESSED_OPEN_PLAN` | 625 |
| `PLAN_CREATED` | 8 |

Interpretation (hypothesis): calibrator ENTER fires; inherited DI_SKEW + PATH_INVALID + open-plan stack dominate coverage. Supports **P1a SCI-clean** twin (separate book) to isolate expectancy — not a cut ratchet on the stacked SCI book.

## P4 verdict

**`HOLD_SOAK`** — twin journaling; closed n=8 all losses (hostile early signal, not yet conclusive vs offline ADVANCE_CANDIDATE direction under thin n + stack). Continue chronological soak. No Gate 5 · no cut ratchet · no ETH arm from this check.

---

*End — soak check 2026-11-27 ~18:38 Asia/Calcutta.*
