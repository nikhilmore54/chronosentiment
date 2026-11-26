# RP-608 H2 — Prospective Confirmation Assessment Protocol

**Protocol:** `RP-608-H2-CONFIRMATION-ASSESSMENT-001`  
**Date:** 2026-11-07 (frozen **before** further confirmation evaluation)  
**Complements:** `rp608_h2_prospective_confirmation_accumulation.md`  
**Does not:** reopen P4 discovery/validation · thr-scan · open H1/H3/H4  

**Authority:** 0% · Gate 2 CLOSED · baseline/OF frozen  

## Purpose

Specify how confirmation evidence will be interpreted **before** more post–Nov-5 sessions arrive. Stopping criteria and labels are locked here — not after seeing results.

## Frozen context

| Item | Value |
| --- | --- |
| H2 P4 disposition | `SUPPORTED_SESSION_REGIME_DOMINANT` (descriptive) |
| Discovery / validation | Consumed (6 + 5 sessions); **not** confirmation |
| Confirmation window | IST sessions ≥ `2026-11-06` only |
| Nov 6 | Initial observation only |

## 1. Eligible session

A confirmation session is eligible if **all** hold:

1. Book = crypto recommendation plan outcomes (`trade_plan_outcomes.jsonl`).  
2. IST calendar day of `entry_time_ms` ≥ `2026-11-06`.  
3. At least one closed trade with non-null `entry_time_ms` and `net_pnl_usdt`.  
4. Not reclassified from discovery/validation windows.

### Session size classes (reporting)

| Class | Rule | Use in dispersion |
| --- | --- | --- |
| Powered | n ≥ 5 | Included in WR range / stdev |
| Thin | 1 ≤ n < 5 | Reported; **excluded** from dispersion range |
| Empty / missing day | No eligible trades | Logged as `NO_ELIGIBLE_TRADES` if market day expected; not a WR observation |

Incomplete outcome rows (missing net or entry time) are excluded from the session denominator and counted in an integrity `excluded` bucket — never silently dropped from the narrative.

## 2. Minimum data completeness (evaluation readiness)

Before issuing a confirmation **assessment** (beyond `NEED_MORE_SESSIONS`):

| Gate | Floor (locked) |
| --- | ---: |
| Distinct powered confirmation sessions (n≥5) | ≥ 5 |
| Total confirmation eligible trades | ≥ 200 |
| Single-session share of confirmation trades | Flag if ≥ 60% (does not auto-fail; blocks “persistence without one-day dependence”) |

Floors may be **tightened** by a dated amendment; they must **not** be loosened after peeking at results.

Until gates met → status remains `NEED_MORE_SESSIONS`.

## 3. Session WR dispersion measurement (locked)

Same as P4:

- Per session: WR% = 100 × wins / (wins+losses), win = `net_pnl_usdt > 0`.  
- Dispersion over **powered** sessions only (n≥5): `wr_min`, `wr_max`, `wr_range_pp`, `wr_stdev` (population stdev).  
- **High dispersion** criterion (aligned with P4): `wr_range_pp ≥ 15`.  

Also report payoff and net per session and in aggregate — same population as W/L (no separate filtered econ set).

## 4. How confirmation differs from discovery / validation

| Aspect | Discovery / validation (P4) | Confirmation |
| --- | --- | --- |
| Role | Formed H2 descriptive claim | Tests whether claim **persists** |
| Sessions | ≤10-31 / 11-01…05 | ≥11-06 only |
| May redefine H2? | No (frozen) | No |
| May thr-scan? | No | No |
| Success = high WR day? | No | No |

## 5. Interpretation labels (locked before evaluation)

Issue **one** primary label when readiness gates are met:

| Label | Criteria |
| --- | --- |
| `PERSISTS_DESCRIPTIVE` | ≥5 powered sessions; `wr_range_pp ≥ 15`; top-session trade share &lt; 60% **or** high dispersion remains after noting concentration; economics reported (may be poor) |
| `WEAKENS` | Readiness met; `wr_range_pp < 15` on powered sessions — session-regime dominance no longer clear |
| `ONE_DAY_DEPENDENT` | Readiness met on n, but ≥60% of confirmation trades in a single session **and** removing that session drops below readiness or kills high dispersion |
| `INCONCLUSIVE` | Readiness met but mixed signals (e.g. high range with unstable economics narrative requiring governance note) without clean PERSISTS/WEAKENS |
| `NEED_MORE_SESSIONS` | Readiness gates unmet |

**Persistence ≠ predictive edge.** `PERSISTS_DESCRIPTIVE` still does **not** authorize a session predictor, filter, or Gate 2 advance.

## 6. Stopping / evaluation cadence

1. **Do not** evaluate for PERSISTS/WEAKENS until readiness gates are met.  
2. Optional interim snapshots may refresh tables while status stays `NEED_MORE_SESSIONS`.  
3. First formal confirmation assessment: when gates met, produce one artifact under this protocol — no iterative threshold tuning.  
4. Any change to floors/labels requires a new protocol id + governance note **before** re-evaluation.

## 7. Forbidden

- Using Oct 26–Nov 5 as confirmation  
- Inventing pass thresholds after seeing confirmation WR  
- Opening H1/H3/H4 under this protocol  
- Inferring latency from timestamps  
- Declaring Gate 2 advance from confirmation alone  

## Artifacts (when evaluated)

- Assessment report: `artifacts/rp608_econ_edge_diag/rp608_h2_confirmation_assessment_*.{md,json}`  
- Accumulation charter remains the operational waitlist doc  
