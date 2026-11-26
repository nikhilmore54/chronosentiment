# RP-608 Economic Edge Diagnosis Plan

**Protocol family:** `RP-608-ECON-EDGE-DIAG-001`  
**Status:** APPROVED RESEARCH PROGRAM · 2026-11-03  
**Live order authority:** **0.0%**  
**Production rules:** FROZEN (OF ±0.30, Estimand A entry thresholds, stop formula) unless separately authorized  

## Disposition

Stop treating daily losses as isolated incidents. Systematically identify:

1. Where the system loses economic edge.
2. Why it loses it.
3. Which changes demonstrably improve **net** performance after realistic costs.

Engineering milestones are **not** a substitute for fixing trading evidence.

## Forbidden substitutions

- Changing OF ±0.30 because a ledger is negative.
- Increasing recommendation frequency to compensate for losses.
- Widening stops to improve apparent win rate.
- Removing the 180-minute limit from one cohort.
- Forcing universal 60-minute exits.
- Pooling NSE OF, Estimand A, and crypto plan results.
- Treating software test pass as proof of trading edge.
- Increasing live authority.

## Execution sequence

| Step | Work | Notes |
| --- | --- | --- |
| 1 | Reconcile authoritative multi-book baseline (start: 2026-11-03) | Books kept separate; open ≠ realized |
| 1b | Primary accounting attribution | Mutually exclusive gross/net partitions; not causal |
| 2 | Row-level mechanism reconstruction + Phase-2 reuse map | Assign-only-when-supported; Nov-2 PATH_ONLY not transferable; Nov-3 CF not computed |
| 2-gate | **Evidence-sufficiency review of Step 2 artifacts** | Independent validation of MD/JSONL; **blocks** Step 3+ until passed; assignment counts are not the gate |
| 3 | Recommendation quality vs exits | Direction, tradability, friction, exit effects separated |
| 4 | OF incremental value | Matched recommendation cohorts only |
| 5 | Multi-session consistency | Entry-time / regime / concentration robustness |
| 6 | Evidence-based change proposals | Each proposal: own authorization + validation boundary |

**Hold:** Do not advance to Step 3–6 experiments or rule proposals from Step 2 assignment counts alone. See `docs/research/rp608_econ_edge_2026-11-03_step2_evidence_sufficiency_gate.md`.

**Same-session diagnostics (not Step 3):** After W/L-first attribution, grade losing-trade peak MFE vs recorded friction (`actionable_mfe_grades_*`) to separate entry-like failure from exit/hold-like opportunity **without** treating every favorable excursion as a missed profitable exit. Does **not** authorize execution-policy changes.

**Successor research line (not Step 3+):** `RP-608-WL-IMPROVE-001` — trade selection / win probability; baseline frozen. Scope: `docs/research/rp608_wl_improvement_plan.md`.

**Parallel (not a Step 3 substitute):** `RP-608-OFX` investigates aggression/response order-flow mechanisms independently. It does **not** unlock Step 3+, change OF±0.30, or replace baseline books. Scope: `docs/research/rp608_ofx_order_flow_response_scoped.md`.

## Step 1 — Book contract (never pool)

| Book | Required |
| --- | --- |
| NSE Estimand A | Realized gross, friction, net; open exposure + open MTM (distinct) |
| NSE OF paper | Entry quality, OF persistence, reverse-OF timing; gross + statutory net (legs-API SoT) |
| Crypto recommendation plan | Entry quality, SL concentration, target capture, net |
| Crypto OF paper | OF confirmation, hold duration, reverse-OF exit, friction |

Per book metrics (**W/L always first**): win rate / loss rate / consecutive losses; then payoff asymmetry; expectancy; profit factor; realized net; gross vs friction; MAE/MFE grades; hold-time & entry-time distributions; open exposure/MTM (separate); multi-session W/L consistency when available.

## Success bar for any proposed change

- Improved net expectancy after realistic costs.
- No material drawdown deterioration.
- Consistency across multiple sessions.
- Not driven by a few exceptional trades.
- Reproducible on identical entry events.
- Clear causal explanation.

W/L is the **primary diagnostic** for every session attribution. A higher win rate alone, a green day alone, or gross that vanishes after friction is still **not** sufficient to authorize a rule change — payoff and multi-session net expectancy remain required for proposals.

## Alignment

- `.cursor/rules/chronosentiment-core.mdc` — determinism, event-driven evaluation, no authority inflation.
- Exit study: `docs/research/rp608_exit_falsification_001_scoped.md`
- NSE OF accounting: `docs/research/rp608_nse_of_paper_trade.md` / legs-API V1 path.

## Artifacts (2026-11-03)

* Export: `artifacts/rp608_econ_edge_diag/session_export_2026-11-03.md`
* Accounting attribution: `artifacts/rp608_econ_edge_diag/accounting_attribution_2026-11-03.md`
* Mechanism reconstruction: `artifacts/rp608_econ_edge_diag/mechanism_reconstruction_2026-11-03.md`
* Step 2 evidence-sufficiency gate: `docs/research/rp608_econ_edge_2026-11-03_step2_evidence_sufficiency_gate.md`
* Method: `docs/research/rp608_econ_edge_session_postmortem_method.md`
* Full-session disposition: `docs/research/rp608_econ_edge_2026-11-03_full_session_disposition.md`
