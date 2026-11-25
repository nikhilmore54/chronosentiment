# RP-608 — Gate 2 Amendment v2 (Split Gate)

**Protocol:** `RP-608-GATE2-AMENDMENT-V2`  
**Effective:** 2026-11-15  
**Authority:** 0.0% · live baseline **still FROZEN** until Gate 5  
**Supersedes:** monolithic Gate 2 success rule in `rp608_wl_improvement_plan.md` (2026-11-06) for *advance criteria only*  
**Does not erase:** prior Gate 2 **DO NOT ADVANCE** records (they remain valid negatives under v1 criteria)

## 1. Problem statement (why amend)

Monolithic Gate 2 required, simultaneously:

1. decision-time **entry** explanation only,  
2. out-of-sample replication,  
3. preferred cohort clears **W/L + payoff + net**,  
4. success **without** material coverage reduction,  
5. **no** change to stops / targets / horizons / Estimand A thresholds / OF ±0.30.

Execution-geometry audits (2026-11-15) show a large share of expectancy failure is **exit/cost structure** (crypto net RR ≪ gross RR; NSE friction-blind 60m greens; winner truncation vs loser extension). Asking entry selection alone to clear payoff+net under frozen adverse geometry, while treating any thinning as failure, is **self-blocking** relative to the programme goal of durable positive expectancy.

**Formal finding:** under current live exits, **entry-only Gate 2 (v1) is not a sufficient path** to the ultimate goal. This is a governance defect, not a license to ship untested filters.

## 2. Amendment — split Gate 2

| Gate | Name | Question | May propose |
| --- | --- | --- | --- |
| **Gate 2E** | Entry selection | Do decision-time observables **replicate** a W/L (and supporting econ) difference? | Offline **entry-filter / ranking** experiment (Gate 3E) |
| **Gate 2X** | Exit / cost geometry | On **frozen identical entries**, does an alternate exit/cost predicate improve **net expectancy** with predeclared risk bounds? | Offline **exit-policy** experiment (Gate 3X); builds on `RP-608-EXIT-FALSIFICATION-001` |

**Either gate may advance independently.** Neither requires the other to pass first.  
**Live daemon / paper production changes still require Gate 5** (and book-specific rollout). Authority remains 0% until then.

### 2.1 Gate 2E — entry (amended bar)

**Advance when all hold:**

1. Predictors are **decision-time only** (no post-entry MFE/MAE/path as entry causes).  
2. Association **replicates** on a predeclared holdout (chronological firewall).  
3. Primary: **W/L** improvement on the preferred cohort vs control (per book).  
4. Supporting: report payoff, friction, net, coverage — **required disclosure**, not a single veto that collapses the gate when geometry is known-hostile.  
5. **Coverage floor (declared before results):** preferred cohort must retain ≥ **min(40% of book trades, or absolute n ≥ 80)** on holdout unless a tighter floor is predeclared with justification. Thin slices below the floor = **fail** (anti-cherry-pick). Slices above the floor are **not** automatic fails solely for reducing coverage.  
6. Books never pooled; no hour/symbol ban lists as the sole rule; no OF ±0.30 change from 2E alone.  
7. Explicit Gate 2E review memo.

**Fail / DO NOT ADVANCE** when replication fails, coverage floor breached, or only WR improves via contaminated thr re-scan.

**Relation to v1 negatives:** crypto plan / NSE A / WR-discovery Gate 2 **DO NOT ADVANCE** under v1 remain on file. They do **not** auto-fail Gate 2E under v2; a **new** sealed package is required (new windows and/or new features). Remining 3–5 Nov or contaminated holdouts stays forbidden.

### 2.2 Gate 2X — exit / cost geometry (new)

**Advance when all hold:**

1. Entries **frozen** (identical trade_ids / entry stamps); only exit predicate varies.  
2. Policies predeclared (no tuning on the evaluation session).  
3. Primary metric: **net expectancy** (and payoff) vs production exit stack **per book**.  
4. W/L reported; W/L-only “wins” that destroy net = **fail**.  
5. Session-clustered / multi-session evidence with declared power rule (single-session PATH_ONLY = diagnostic, not Gate 2X advance).  
6. Explicit statement of what would change in production (e.g. friction-aware 60m gate; ambiguous TP/SL tie-break; min-hold before ATR stop) — still **not applied** until Gate 5.  
7. Explicit Gate 2X review memo.

**Prior art:** `RP-608-EXIT-FALSIFICATION-001` Phase 1–2 remain valid diagnostics; Gate R was not reached. Under this amendment, a **multi-session** 2X package may be proposed without waiting for Gate 2E.

## 3. What stays frozen until Gate 5

- Live / shadow **production** stops, targets, horizons, Estimand A `traj15` threshold, crypto plan ATR multipliers  
- OF ±0.30 / reverse-OF / baseline OF construct  
- Live order authority 0%  
- Book pooling  

## 4. Interaction with confirmation-only holding

**Amended 2026-11-15:** confirmation-only holding no longer forbids **all** research workstreams.

| Still required | Now permitted |
| --- | --- |
| H2 confirmation accumulation + DQ on Pi | Gate **2X** offline packages (frozen entries) |
| No live execution edits | Gate **2E** prep: decision-time feature capture / leakage audit / sealed window design |
| No OF ±0.30 change | H1 **open note** when availability gate met (ledger) |
| H2 assessment labels unchanged | Parallel 2E/2X memos |

H2 confirmation completion is **not** a prerequisite for starting 2X work. It remains required before treating session-regime as anything beyond descriptive.

## 5. Ultimate-goal alignment

Programme goal: durable positive expectancy per book.  
**Path:** improve selection (**2E**) and/or repair cost/exit geometry (**2X**), then Gate 3 offline, then Gate 5 production — without pretending entry filters must single-handedly overcome a hostile exit stack.

## 6. Immediate next actions (authorized by this amendment)

> **Historical note (2026-11-25):** Items 1–3 below were the *initial* authorized actions as of the 2026-11-15 effective date. They are largely **completed or superseded** by subsequent Gate 2E pattern packages, Gate 2X / Gate 3X offline reviews, Gate 5 **shadow** deploys (authority 0%), and Gate 2X floor-lock observational twins. **Advance criteria in §§2–5 remain binding.** Live OF ±0.30 / Estimand A baseline still require Gate 5 for production cutover; shadow ≠ authority raise.

1. Draft Gate **2X** multi-session scope (crypto plan + NSE A separate) using frozen journals + exit-falsification engine.  
2. Draft Gate **2E** availability / capture checklist for crypto continuous decision-time fields (H1 prep).  
3. Update ledger + holding state to cite this amendment.  
4. **Do not** change frozen live baseline (OF ±0.30, Estimand A entry thresholds/stops/targets/holds) without Gate 5. Observational / shadow / authority-0% wiring after ADVANCE memos is tracked in `rp608_wl_improvement_plan.md` and `rp608_research_decision_ledger.md` — not a license to raise authority.

## 7. References

- Prior v1 Gate 2: `rp608_wl_improve_crypto_plan_hypothesis_and_gate2.md`, `rp608_wl_improve_nse_a_kite_gate2.md`  
- Exit falsification: `rp608_exit_falsification_001_scoped.md`  
- Exec algo flaws: `artifacts/rp608_econ_edge_diag/rp608_execution_algorithm_flaw_analysis.md`  
- Historical W/L audit: `artifacts/rp608_econ_edge_diag/rp608_historical_flaw_audit_crypto_nse_a.md`  
- Plan: `rp608_wl_improvement_plan.md`  
- Ledger: `rp608_research_decision_ledger.md`  
