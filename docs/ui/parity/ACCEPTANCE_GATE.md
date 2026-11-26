# Gate5 Operator — Parity Acceptance Gate

**Status classification (locked)**

| Milestone | Status |
|-----------|--------|
| Gate5 Operator implementation | **Shipped** |
| Presentation-only architecture | **Established** |
| Live parity validation | **Two snapshots PASS** (cutover still gated on freshness + UI eye-check) |
| Production cutover | **Gated** |

**Policy:** freeze feature development until parity passes. Gate5 remains presentation-only over Pi `/api/state`. Pi Command Center stays operational fallback. 0% authority non-negotiable (chronosentiment-core).

## Required evidence

| Validation area | Required evidence |
| --------------- | ----------------- |
| Crypto state | Policy B/C, plan and OF states match Pi |
| OF lifecycle | Reverse-flow exit represented correctly |
| NSE universe | 119 census, 102 classified, 17 quarantined |
| NSE watches | Same candidates, ranking and OF marks |
| Freshness | Stale / unavailable match source conditions |
| Authority | Always explicitly 0% |
| State mapping | No semantic changes or silent defaults |
| Rollback | Pi Command Center remains accessible |

## Cutover conditions (all required)

1. Compare Gate5 and Pi on the **same timestamped** `/api/state` snapshots.
2. Verify behavioural edge cases (e.g. `NO_TRADE` with open/display plan; `OF EXITED` with plan still present).
3. Record discrepancies → resolve → **rerun** checklist to clean PASS.

## Snapshot run — 2026-11-01T08:37:25Z

- Artifact: `pi_state_20261101T083725Z.json` (~553 KB)
- Report: `parity_report_20261101T083725Z.json`
- Checklist helper: `services/ui/src/operator/parityChecklist.ts` → **PASS (0 fails)**
- Pi `/api/state` wall time on host: **~44 s** (latency risk for Gate5 5 s poll / freshness UX)

| Area | Evidence (this snapshot) |
|------|--------------------------|
| Crypto | B/C `NO_TRADE` / `NO_TRADE` |
| Plan / OF edge | Plan `OPEN` + `is_active` with OF **not** active and `order_flow_last_exit` present |
| NSE universe | 119 / 102 / 17 |
| NSE watches | n=20, OF=9; **all 20** symbol/side/OF flags match mapped DTO (0 mismatches) |
| Authority | mapped `0`, NSE `order_authority` `0` |
| Rollback | Pi `http://192.168.0.9:8765/` still the reference link from Overview |

### Still required before cutover = PASS

- [x] Full 20-watch symbol/side/OF flag equality on snapshot `20261101T083725Z` (0 mismatches)
- [x] Edge case present on same snapshot: `NO_TRADE` + plan `OPEN` + OF exited (not active)
- [x] Second independent timestamped snapshot PASS (`pi_state_2026-11-01T08-33-41-241Z.json`, proxy path, B/C LONG + plan OPEN + OF exited)
- [ ] Freshness: confirm Gate5 shows `UNAVAILABLE` / age when Pi `/api/state` slow or down (not silent healthy)
- [ ] UI eye-check of edge case copy (OF EXITED vs plan OPEN independence)
- [ ] Ops note: Pi `/api/state` ~44s build time — Gate5 poll freshness UX until Pi speeds up or caches (no Gate5 behavioural change)

**Cutover rule:** only after the above are checked; then Gate5 primary, Pi fallback.
