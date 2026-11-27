# SCI-ADAPTER-REC soak check — ~13:55 Fri IST Asia/Calcutta

**Host:** `ultraspeed@192.168.0.9` · P3 armed ~13:40 IST  
**Rules:** `.cursor/rules` chronosentiment-core §3b W/L-first · S6 baselines · Coralys untouched  
**Scorecard fix:** `rp608_sci_adapter_rec_status.py` now reads `trade_plan_*` journals (was looking at missing `outcomes.jsonl` / `events.jsonl`).

## Arms

| Book / twin | Status |
| --- | --- |
| BTC SCI | `RP608_SCI_ADAPTER_REC_TWIN=1` · dir present · daemon active |
| BTC adapter_select | armed |
| BTC Gate2E B | armed (import fix holding — no NameError) |
| ETH SCI | **not** armed (correct) |
| NSE | untouched (`failed`) |

## SCI twin early journal (~15 min)

| Metric | Value |
| --- | --- |
| Closed W/L | **0/0** (no `trade_plan_outcomes.jsonl` yet) |
| SCI STANDASIDE suppress | 0 (generator ENTERING — S1=b firing) |
| Dominant early suppress | `SUPPRESSED_GATE2E_DI_SKEW` (~87+) then some `SUPPRESSED_GATE2E_PATH_INVALID` |
| PLAN_CREATED | 0 so far |
| Direction stamp | SHORT candidates (Coralys NO_TRADE override path) |

Interpretation (hypothesis only): SCI generator is live and proposing SHORT; inherited Gate2E regime/DI_SKEW + path-invalid stack is blocking opens before plan create. This is **stack interaction disclosure**, not a cut change. Compare later to adapter_select / Gate2E B closed books per P4 draft.

## Baselines (snapshot)

Run adapter_select / Gate2E B status separately for closed W/L; do not pool into SCI.

## Verdict

**`HOLD_SOAK`** — twin armed and journaling; closed n=0; continue chronological soak. No Gate 5 / no cut ratchet / no ETH arm.
