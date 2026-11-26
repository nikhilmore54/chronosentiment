# RP-608 — Blocker Resolution Plan

**Date:** 2026-11-06  
**Authority:** 0.0% · Gate 2 CLOSED · baseline/OF frozen  
**Companion:** `rp608_hypothesis_validation_programme.md`

## Principle

Resolve each blocker at its **source**. Do not pause the entire programme when only one hypothesis is blocked. Do not extract velocity evidence from market-state fields.

## Blocker classes

| ID | Class | Resolution class |
| --- | --- | --- |
| **B1** | Missing execution instrumentation (submit→ack→fill) | Prospective capture — historical inference forbidden |
| **B2** | Incomplete decision-time reference data | Quote integrity / availability; touch-only stays descriptive |
| **B3** | Predictive instability (no reliable session transfer) | Chronological validation design — not more thr scans |
| **B4** | Governance restriction (Gate 2 closed, exec frozen, auth 0%) | Keep observational; package evidence — do not bypass |

B4 is a **boundary**, not a technical stop on observational analysis.

## Workstreams

| Path | Allowed |
| --- | --- |
| Observational | Availability, integrity, frozen hypotheses, W/L+economics, chronological validation, docs |
| Separate authorization | New instrumentation **implementation**, filters, urgency, stop/target/horizon, live authority |

## Blocker ledger

| Blocker | Impact | Resolution | Owner | Exit criterion | Status |
| --- | --- | --- | --- | --- | --- |
| B1-NSE | H4 NSE A | NSE instrumentation charter + prospective capture | Research / Pi | Full event chain + integrity floors | **Open** — charter registered |
| B1-CRYPTO-PLAN | H4 crypto plan | Capture spec + capability audit | Research / Pi | Useful lifecycle without behavior change, or permanent unavailability | **Closed** — `UNTESTABLE_UNDER_CURRENT_ENGINE_DESIGN` (`rp608_h4_current_engine_boundary.md`) |
| B1-CRYPTO-OF | H4 crypto OF | Capability audit | Research | Useful lifecycle without behavior change, or permanent unavailability | **Closed** — same boundary; touch descriptive only |
| B2-CRYPTO-PLAN | H1 quote quality; weak refs | Decision-time quote join integrity; do not invent | Research | Proven decision-time bid/ask/mid completeness documented | **Open** |
| B2-CRYPTO-OF | Limited | Touch 108/108 descriptive only — not velocity | Research | N/A for velocity; touch analysis labeled descriptive | **Partial** — touch OK, lifecycle no |
| B3-NSE | H1/H2 NSE | Chronological validation; no 002 holdout reuse | Research | Untouched validation window defined + run | **Open** |
| B3-CRYPTO | H2 crypto plan | Chronological H2 P4 + confirmation accumulation | Research | Multi-session confirmation without one-day reliance | **In progress** — P4 descriptive support locked; confirmation via `rp608_h2_prospective_confirmation_accumulation.md` |
| B4 | All deployable changes | Governance evidence package only | Governance | Explicit Gate 2/3 decision | **Active boundary** |

**Resolved** = required evidence exists and passes integrity. It does **not** mean the hypothesis passed.

### Milestone ladder (do not collapse)

1. Instrumentation complete  
2. Latency / fills measurable  
3. Association with outcomes tested (may fail)  
4. Chronological validation passes (may fail)  
5. Governance authorizes a **separate** experiment (only then)

## Priority table

| Priority | Work item | Dependency | Completion condition |
| --- | --- | --- | --- |
| **P0** | Crypto prospective capture specification | Crypto inventory closed | Events + integrity rules documented | **DONE** |
| **P1** | Crypto paper-engine capability audit | P0 | Useful emit-without-behavior-change? | **DONE** — No; H4 permanently unavailable under current engines |
| **P2** | Crypto H1–H3 availability matrix | Journals + decision cards | Completeness + join validity | **DONE** |
| **P3** | Freeze hypothesis test protocols | P2 | Outcomes, fields, exclusions, populations | **DONE** — `rp608_hypothesis_p3_freeze.md` |
| **P4** | Run supported observational tests | P3 | H2 crypto plan first | **DONE** — session-regime dominant (descriptive); confirmation thin |
| **P5** | Governance review | P4 | Evidence package; no auto-authorization | Optional — no Gate 2 advance from H2 alone |

## Explicit non-actions

- No broad discovery / thr scan  
- No pooling books  
- No substituting continuous-state for H4  
- No relabeling modeled or `slippage_model: none` as observed  
- No execution or live-authority change from this plan alone  
