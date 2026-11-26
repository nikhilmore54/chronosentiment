# RP-608 — Pi Research Execution Host

**Locked:** 2026-11-06  
**Host:** `LCUtest` / `ultraspeed@192.168.0.9`  
**Repo:** `/home/ultraspeed/ChronoSentiment_MEGA_FINAL`  
**Authority:** 0.0% · baseline frozen  

## Rule

**WL-IMPROVE / Kite / session diagnostics that need live Kite credentials or Pi journals run on the Pi.**  
Mac is for editing, review, and pulling **artifacts only** — not for Kite historical when the Mac token is stale.

| Concern | Where |
| --- | --- |
| Kite `~/.kite_credentials.json` | Pi (hot-reload / dashboard re-auth) |
| NSE / crypto journals | Pi `live_capture/...` |
| Research scripts | Prefer execute on Pi after rsync |
| Artifacts (`artifacts/rp608_*`) | Produce on Pi → rsync to Mac for review |
| Credentials | **Never** rsync Pi → Mac unless explicitly requested |

## Standard loop

```bash
# From Mac repo root
PI=ultraspeed@192.168.0.9
REPO=/home/ultraspeed/ChronoSentiment_MEGA_FINAL

# 1) Push script / inputs
rsync -az scripts/research/rp608_wl_kite_nse_decision_time_bars.py "$PI:$REPO/scripts/research/"

# 2) Run on Pi
ssh "$PI" "cd $REPO && python3 scripts/research/rp608_wl_kite_nse_decision_time_bars.py \
  --export-dir artifacts/rp608_econ_edge_diag \
  --out-dir artifacts/rp608_econ_edge_diag"

# 3) Pull artifacts only
rsync -az "$PI:$REPO/artifacts/rp608_econ_edge_diag/wl_kite_nse_"* \
  "$PI:$REPO/artifacts/rp608_econ_edge_diag/wl_nse_a_decision_time_features_kite_join.jsonl" \
  artifacts/rp608_econ_edge_diag/
```

Helper: `scripts/research/rp608_pi_research_run.sh`

## Current Pi results (refreshed 2026-11-07)

- Kite NSE A decision-time join (discovery+holdout days): **OK** · 1625/1625 feature-ready  
- NSE A Kite discovery v2: WR replicates on mom/location · **Gate 2 DO NOT ADVANCE** (payoff/net bar)  
- Artifacts: `wl_nse_a_kite_discovery_v2.*`, `wl_nse_a_decision_time_features_kite_join.jsonl`  
- **P4 H2 crypto plan session regime (2026-11-06):** disposition **frozen** `SUPPORTED_SESSION_REGIME_DOMINANT` (descriptive, not edge) · **Gate 2 DO NOT ADVANCE**  
  - Report: `artifacts/rp608_econ_edge_diag/rp608_h2_crypto_plan_session_regime_p4.md`  
  - Memo: `docs/research/rp608_h2_crypto_plan_session_regime_p4_gate2.md`  
- **Programme state:** CONFIRMATION-ONLY HOLDING — `docs/research/rp608_confirmation_only_holding_state.md`  
- **Ops:** run `rp608_h2_confirmation_dq_monitor.py` on Pi as new sessions arrive · no discovery / no new workstreams  
- Crypto H4: `UNTESTABLE_UNDER_CURRENT_ENGINE_DESIGN` · H1/H3 unopened  

## Governance

Does not change OF ±0.30, stops/targets, Gate 2 criteria, or book pooling.
