# Policy D — Pre-open Informed, Gap-Neutral Warmup

**Protocol:** `RP-608-POLICY-D-PREOPEN-GAP-NEUTRAL-DIAG-001`  
**Disposition:** INVESTIGATION ONLY · **0.0% order authority**  
**Live G3 / G1–G5 thresholds:** unchanged · **Deployment:** not authorized

## Research question

Can previous-session history **plus** pre-open context improve early-session decisions beyond Policy C (gap-neutral cross-session warmup), without letting overnight repricing masquerade as continuous-market `traj15`?

## Information separation (mandatory)

| Layer | Source | Role |
| --- | --- | --- |
| Previous-session history | Kite historical 15m/1m OHLCV | Prior close, vol, trajectory context |
| Pre-open observation | Live capture archive (REST and/or WS) | Overnight repricing / opening discovery |
| Continuous-session momentum | Completed continuous 15m bars | Existing `traj15` (threshold **+0.05** frozen) |

**Design rule:** Opening context (A) may inform gap treatment. Intraday momentum (B) alone may produce G4 PASS. Pre-open data must **not** independently mint a G4 momentum PASS.

## Policy matrix

| Policy | Information used |
| --- | --- |
| A | Same-session history (live baseline) |
| B | Previous-session history, naive |
| C | Previous-session history, gap-neutral (**leading candidate**) |
| D | C + pre-open context for gap decomposition / quality filters |

Policy D must beat C on persistence and economics; otherwise pre-open adds complexity without value.

## Kite capability boundary

- Historical API: prior-session candles (usable for backfill).
- WebSocket: live quotes/depth (usable for forward pre-open archive **if** packets are delivered).
- No historical tick-replay API → pre-open tick research requires **our** capture archive.

## Gate governance

- Live G3 remains same-session until multi-session economics under C (and D vs C) justify a separately versioned experimental policy.
- RP-607 remains descriptive only.
