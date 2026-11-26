# RP-608-WL-IMPROVE-001 — Gate 1 Review Decision

**Date:** 2026-11-06  
**Input:** Phase 1 diagnostic `artifacts/rp608_econ_edge_diag/wl_trade_selection_diag_2026-11-03_to_05.md`  
**Reviewer disposition:** Accept Phase 1 as **descriptive leads only**  
**Authority:** 0.0% · baseline frozen (`.cursor/rules/chronosentiment-core.mdc` §3b–3c)  

## Verdict

**PASS — Gate 1 complete.**

Phase 1 establishes descriptive W/L leads for 2026-11-03 … 2026-11-05.  
It does **not** establish which entry characteristics *reduce loss probability*.  
It does **not** authorize filters, hour bans, H0 changes, OF ±0.30 changes, stops/targets, or Gate 3.

## Confirmed definitions

| Item | Definition locked for this gate |
| --- | --- |
| Cohort | Closed trades by exit-date IST per book export |
| Primary metric | W/L and WR **per book** (never pooled for causal claims) |
| Win | `net_pnl > 0` on book accounting field |
| Entry condition | Best available at-entry tags in export (gaps labeled) |
| Persistence | Same dim key weak/strong across ≥2 of 3 sessions (descriptive) |
| `small_n` | n < 10 (Phase 1) — not filter-eligible |

## Sample integrity

| Book | n (3 sessions) | WR | Integrity note |
| --- | ---: | ---: | --- |
| NSE A | 811 | 46.24% | Complete closed export Nov3–5 |
| NSE OF | 413 | 28.81% | Complete; path MFE often absent |
| Crypto plan | 227 | 27.75% | Complete; H0 × OF tags present |
| Crypto OF | 67 | 34.33% | Complete but **small-n / noisy** |

No book re-run required for Gate 1. Another diagnostic pass is **not** the next step.

## Review decision table

| Book | Phase 1 finding | Gate 1 interpretation |
| --- | --- | --- |
| NSE A | Weak hours 10, 13, 14; mid entry bars 3–8 | Candidate dimensions for **later** entry-quality investigation |
| NSE OF | Weak hours 10–12, 14; weakness across imbalance buckets | OF imbalance alone does **not** explain observed weakness |
| Crypto plan | 27.8% WR; H0 conditions weak with OF on/off | **Highest-priority Phase 2** investigation |
| Crypto OF | 34.3% WR, n=67 | Small sample; findings remain noisy |

## Outcome

Accept the above as **Phase 2 research leads**, not entry filters.

**Authorized next:** Crypto plan Phase 2 only — decision-time entry-quality associations.  
**Deferred:** NSE A / NSE OF / Crypto OF Phase 2 until Crypto plan yields a reproducible decision-time result.  
**Premature artifacts** (prior out-of-sequence Phase 2 NSE A/OF runs): retained for reference only; **not** sequence-authorized and do not advance Gate 2.

## Explicit non-authorizations

- Suppressing hours / symbols  
- Changing H0 plan conditions  
- Modifying OF thresholds (±0.30)  
- Stop / target / hold changes  
- Universal trend filter  
- Gate 3 offline experiment  
- Step 3+ / Gate R / OFX Stage 2 classifier (`.cursor/rules/rp608-ofx.mdc`)

## Limitations (retained)

1. Descriptive association ≠ causal entry-quality.  
2. Multi-session “persistence” here is within the same 3-day window — not holdout replication.  
3. Trend / momentum / price-location often **UNAVAILABLE** without look-ahead-safe bar join.  
4. Success criterion for the next stage: defensible, repeatable explanation of **entry win probability** — not higher WR via trade-count reduction or favorable historical slices.

## Required sequence (locked)

1. ~~Gate 1 review~~ → **DONE (this document)**  
2. Phase 2 Crypto plan (decision-time only)  
3. Cross-session replication beyond 3–5 Nov  
4. One falsifiable hypothesis + evaluation criteria  
5. Gate 2 review (offline experiment proposal only if supported)  

Gate 3 remains unauthorized.
