# Gate5 Operator IA (1C / 2B)

Presentation-only operator surface in `services/ui` over Pi `/api/state`. Aligns with ChronoSentiment deterministic observation: Gate5 does **not** re-implement Policy B/C, OF thresholds, NSE ranking/quarantine, or trade-plan exits.

## Architecture

| Layer | Role |
|-------|------|
| Pi `scripts/rp608_live_dashboard.py` `/api/state` | Source of truth telemetry |
| Vite proxy `/pi-dashboard` → `192.168.0.9:8765` | Transport |
| `services/ui/src/operator/*` | Typed subset DTOs + workspaces |
| Pi Command Center HTML | Reference / rollback until parity pass |

## Contracts (`services/ui/src/operator/`)

- `types.ts` — `CryptoStatus`, `NseStatus`, `SystemHealth`, `OfRecommendation`, `TradePlanView`, `NextSessionWatch`, `OperatorState`
- `piClient.ts` — `fetchOperatorState()` → `GET /pi-dashboard/api/state` with `fetched_at_ms` / HTTP errors
- `mapState.ts` — presentation mapping only (fail closed on missing sections)
- `freshness.ts` — age / stale / `UNAVAILABLE` (never invent healthy numbers)

### Pi fields consumed (subset)

**Crypto:** `latest_card`, `trade_plan`, `order_flow_recommendation`, `order_flow_last_exit`, `of_candle_markers`, `recent_cards`, `telemetry_gates`, `service_status`, `checkpoint_integrity`, `accounting`

**NSE (`nse_operations`):** `status`, `order_authority`, `coverage`, `rp607_summary` (119/102/17), `next_session_recommendations`, `evaluations_summary`, `coralys_input_gate` (phase / prior / pre-open / standaside), `production_session.session_phase`, `services`, `daemon_pid`, `scrips`

## First viewport (1C)

Operator tab default → Overview:

1. Crypto strip — B/C, plan state, OF active/exited, mid, quote age  
2. NSE strip — session + Coralys phase, input-gate prior/preopen/standaside, next-session date/n/OF, 102/119/17, board-empty reason  
3. System strip — service / gates / checkpoint / IST  
4. Authority chip — **0%**  
5. CTAs → Crypto / NSE / Multi-Horizon workspaces  

No 119-row table, hardcoded RP-607 compartment cards, or Sharpe essay in the first viewport.

## App shell (kept)

Operator shell only (Overview · Crypto · NSE).

**Discarded from shell:** Multi-Horizon Books, RP-611 strip, Research & Forensics, Decision Surface, Paper Trading (`Gate5PaperDashboard`), Observatory, Replay Timeline, Trade Inspector, Research Console, RP609 dashboard.

## Workspaces

- **Crypto** — Policy B/C · Trade plan (60m) · OF latch/exit (independent) · candle marks · recent cards with ENTRY/EXIT  
- **NSE** — universe strip · Coralys input gate panel · next-session + OF badges · session board (incl. STANDASIDE count)

**Pi Command Center slim (2026-11-16):** single command surface — removed Multi-Horizon tab, Sharpe/Calmar essay, Gate C panel, RP-611 panel, 119-row symbol explorer, static G1–G4 essay grid. `/api/state` payloads unchanged.

## Parity checklist (cutover gate)

Fetch Pi `/api/state` and assert Gate5 mapped DTO equality:

| Check | Pi field | Gate5 DTO |
|-------|----------|-----------|
| Authority | `order_authority` / implied 0 | `live_order_authority === 0` |
| Policy B/C | `latest_card.policy_b_action` / `policy_c_action` | `crypto.policy_b` / `policy_c` |
| Plan | `trade_plan.plan_id`, `state`, `is_active` | `crypto.trade_plan.*` |
| OF active | `order_flow_recommendation.active` | `crypto.order_flow.active` |
| OF exit | `order_flow_last_exit` | `crypto.order_flow_last_exit` |
| OF candles | `of_candle_markers.entry/exit` | `crypto.of_candle_markers.*` |
| Next-session n | `nse_operations.next_session_recommendations.n_recommendations` | `nse.n_recommendations` |
| OF count | `n_order_flow_supported` | `nse.n_order_flow_supported` |
| Top watches + OF flags | `recommendations[]` | `nse.recommendations[]` |
| Census | `rp607_summary` 119/102/17 | `census_total` / `classified_count` / `quarantined_count` |
| Coralys gate | `coralys_input_gate.session_phase` / `prior_status` / `n_standaside` | `nse.coralys_input_gate.*` / `nse.session_phase` |
| Fail closed | HTTP/error/empty | `fetch_status !== 'ok'` → strips show `UNAVAILABLE` |

**Rollback:** keep Pi Command Center (`http://192.168.0.9:8765/`) live; Gate5 Overview links to it as reference. Cutover only after checklist pass.

**Formal acceptance:** see [`docs/ui/parity/ACCEPTANCE_GATE.md`](parity/ACCEPTANCE_GATE.md). Feature development frozen until live parity validation passes; production cutover remains gated.

## Out of scope

Daemon / OF math / NSE rec builder changes; Gate5 paper-trading & forensics tabs unchanged (Operator is additive).
