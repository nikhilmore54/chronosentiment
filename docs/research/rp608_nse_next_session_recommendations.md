# RP-608-NSE-NEXT-SESSION-001 — Post-Close Next-Session Watchlist

**Authority:** 0.0%  
**Trigger:** Daemon `POST_CLOSE` (after 15:30 IST calendar close), once per session date  
**Artifact:** `live_capture/shadow_validation/nse/rp608_nse_next_session_recommendations.json`

## Purpose

After the continuous session (and 15:00–15:15 close window) ends, emit ranked **next-session intraday watches** from today’s completed 15m fetch history — so the Command Center can show tomorrow’s watchlist overnight without inventing overnight/CAS trades.

## Ranking

For each NSE-119 symbol with ≥5 completed bars today:

- `traj15_eod` — same tanh family as live intraday entry (last ≤5 closes)
- `session_return_bps` — first→last session close
- `score = traj15 + 0.25 * (session_return_bps / 10000)`
- Keep if `traj15 ≥ 0.05` **or** session return ≥ +25 bps
- Top 20 by score (ties: symbol)

## Governance

- Status: `WATCH_NEXT_SESSION` — **not** executable overnight or in CAS
- Becomes eligible for live MIS shadow only on `next_session_date` after bar ≥5, subject to live traj15 / L1 / entry cutoff (bar ≤19) and 15:00 square-off
- Deterministic: same bar history → same list

## Ops

- Auto-emitted by `run_rp608_nse_shadow_daemon` on POST_CLOSE
- Manual: `python3 scripts/run_rp608_nse_shadow_daemon.py --emit-next-session-recs`
