# RP-608 — Confirmation-Only Holding State

**Effective:** 2026-11-07  
**Amended:** 2026-11-15 (Gate 2 v2)  
**Status:** **ACTIVE — H2 confirmation track** · research workstreams **partially reopened** under Gate 2E/2X  
**Authority:** 0.0% · live baseline / OF ±0.30 **FROZEN** until Gate 5  

## Formal disposition

H2 confirmation infrastructure remains in force for **session-regime evidence**.  
Gate 2 **amendment v2** (`rp608_gate2_amendment_v2.md`) lifts the blanket ban on all research workstreams so entry (**2E**) and exit/cost (**2X**) packages can proceed **offline** without waiting for H2 floors.

| Area | Current conclusion |
| --- | --- |
| H2 | Descriptive session-regime dominance (`SUPPORTED_SESSION_REGIME_DOMINANT`) |
| Confirmation | Continue Pi DQ until floors; last sealed monitor may be stale — refresh before assessment |
| Gate 2 | **v2 ACTIVE** — split 2E / 2X; v1 DO NOT ADVANCE records retained as history |
| Baseline OF | L1 resting liquidity only (`ACCEPT_CONSTRUCT_DISTINCTION`) |
| H1 / H3 | May open only via dated note + availability checklist (ledger) |
| H4 | `UNTESTABLE_UNDER_CURRENT_ENGINE_DESIGN` until engine redesign |
| Live execution | **Unchanged** — no daemon stop/target/horizon/`traj15` edits without Gate 5 |

### Operating decision (2026-11-15 — Anuj)

**Amend Gate 2** (split 2E/2X). Continue H2 confirmation accumulation. Permit offline 2X packages and 2E prep/capture. Do **not** ship production execution changes.

### Prior operating decision (2026-11-07 — Nikhil)

Held confirmation-only with no new workstreams. **Superseded for workstream ban only** by 2026-11-15 amendment; H2 floors/labels unchanged.

## Permitted operations

1. Accumulate H2 confirmation sessions (IST ≥ 2026-11-06); run Pi DQ monitor.  
2. When floors met, run H2 confirmation **assessment** under frozen labels.  
3. Preserve sealed Pi P4 fingerprints.  
4. **Gate 2X:** offline exit/cost experiments on frozen entries (multi-session scope).  
5. **Gate 2E prep:** decision-time feature capture, leakage audits, sealed window design; H1 open note when ready.  
6. Construct distinction lock unchanged — baseline OF ≠ aggressor OFX.  

## Still forbidden

- Live / shadow **production** edits to stops, targets, horizons, Estimand A threshold, OF ±0.30 without Gate 5  
- Gate 3E/3X without matching Gate 2 ADVANCE memo  
- Remining contaminated / consumed discovery windows as a “new” pass  
- Pooling books; WR-only success; hour/symbol ban lists as sole rule  
- Declaring H2 confirmed from underpowered samples  
- Reopening H4 under current crypto engines  

## Trigger map

| Decision | Trigger |
| --- | --- |
| H2 confirmation label | Frozen DQ floors + assessment protocol |
| Gate 2E ADVANCE | New sealed entry package per amendment §2.1 |
| Gate 2X ADVANCE | Multi-session frozen-entry exit package per amendment §2.2 |
| Production change | Gate 5 after 3E and/or 3X |

## Canonical refs

- Gate 2 amendment v2: `docs/research/rp608_gate2_amendment_v2.md`  
- Accumulation: `docs/research/rp608_h2_prospective_confirmation_accumulation.md`  
- Assessment protocol: `docs/research/rp608_h2_prospective_confirmation_protocol.md`  
- Decision ledger: `docs/research/rp608_research_decision_ledger.md`  
- WL plan: `docs/research/rp608_wl_improvement_plan.md`  
