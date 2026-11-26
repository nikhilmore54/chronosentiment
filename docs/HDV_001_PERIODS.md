# HDV-001 — Period Definitions
## Frozen: 2026-08-17

**Status:** FROZEN  
**Frozen at:** 2026-08-17T06:53:00Z (before any HDV-001 implementation begins)  
**Depends on:** HDV_001_METHODOLOGY.md (frozen 2026-08-17)

---

## 1. Historical Data Inventory

The existing certified decision dataset (`stop_research_dataset_v01.json`, schema v0.1)
covers:

| Dimension              | Value                                      |
|------------------------|--------------------------------------------|
| Earliest decision_time | 2026-07-14T18:30:00Z (NSE session open)    |
| Latest decision_time   | 2026-08-13T03:45:00Z (NSE session open)    |
| Total decisions        | 1,144                                      |
| Instruments            | 52 (NSE Nifty 50 universe)                 |
| July 2026 decisions    | 676                                        |
| August 2026 decisions  | 468                                        |
| Data source            | `stop_research_dataset_v01.json` (v0.1)    |
| C3-002 artifact hash   | `5a43b9df97daa76d85edd7f7ef1c12c3a230ef292f7ecfa98ef9587647392121` |

**Observation window:** 10 NSE trading sessions from `decision_time` (inclusive of
session 1, exclusive of session 0). The latest decision (2026-08-13) therefore requires
price data through approximately 2026-08-27.

---

## 2. Period Structure

The existing 30-day dataset is too short for a meaningful three-way historical split.
Splitting it would produce sub-populations of ~380 decisions each — insufficient for
regime analysis and baseline comparison.

**The correct structure is therefore:**

| Period      | Decisions | Date Range                        | Status at freeze |
|-------------|-----------|-----------------------------------|------------------|
| Development | 1,144     | 2026-07-14 → 2026-08-13           | **Open** — methodology development |
| Validation  | TBD       | 2026-08-18 → 2026-10-31           | **Sealed** — not examined until dev complete |
| Holdout     | TBD       | 2026-11-01 → 2026-12-31           | **Sealed** — not examined until validation positive |

**Rationale for forward-looking validation and holdout:**

1. The development period uses all existing certified decisions — no cherry-picking.
2. The validation period is the next ~10 weeks of live Coralys decisions (2026-08-18
   onward), collected as the system runs forward.
3. The holdout is the subsequent 2 months (November–December 2026), sealed now.
4. This structure avoids any look-ahead bias in period selection.
5. The forward structure is stronger than a historical split because it tests
   out-of-sample performance in real time, not on data that was available during
   methodology development.

---

## 3. Development Period — Exact Boundaries

| Boundary               | Value                                      |
|------------------------|--------------------------------------------|
| Start (inclusive)      | 2026-07-14T18:30:00Z                       |
| End (inclusive)        | 2026-08-13T03:45:00Z                       |
| Price data required    | 2026-07-14 → 2026-08-27 (10 sessions after last decision) |
| Decisions              | 1,144 (all records in stop_research_dataset_v01.json) |
| Purpose                | Methodology development and debugging only |
| Result recording       | Not until HDV-001-G freeze gate            |

**The development period must not be used to select the validation or holdout boundaries.**
Those are fixed by calendar (see Section 2) and must not be changed.

---

## 4. Validation Period — Exact Boundaries

| Boundary               | Value                                      |
|------------------------|--------------------------------------------|
| Start (inclusive)      | 2026-08-18T18:30:00Z (first NSE session after freeze date) |
| End (inclusive)        | 2026-10-31T10:00:00Z (last NSE session close) |
| Price data required    | 2026-08-18 → 2026-11-14 (10 sessions after last decision) |
| Decisions              | TBD — collected as Coralys runs forward    |
| Purpose                | Frozen methodology applied to new data     |
| Examined               | Only after HDV-001-G freeze gate           |

**The validation period is sealed.** No validation data may be examined before
HDV-001-G is declared.

---

## 5. Holdout Period — Exact Boundaries

| Boundary               | Value                                      |
|------------------------|--------------------------------------------|
| Start (inclusive)      | 2026-11-01T18:30:00Z                       |
| End (inclusive)        | 2026-12-31T10:00:00Z                       |
| Price data required    | 2026-11-01 → 2027-01-14 (10 sessions after last decision) |
| Decisions              | TBD — collected as Coralys runs forward    |
| Purpose                | Final credibility check                    |
| Examined               | Only if validation primary criteria pass   |

**The holdout period is sealed.** It must not be examined under any circumstances
until the validation period results are recorded and evaluated against the predefined
success criteria in HDV_001_METHODOLOGY.md Section 13.

---

## 6. Decision Universe

| Dimension              | Value                                      |
|------------------------|--------------------------------------------|
| Universe version       | Nifty 50 constituents as of 2026-07-14     |
| Instrument list        | 52 instruments in stop_research_dataset_v01.json |
| Universe changes       | Any instrument added/removed from Nifty 50 after 2026-07-14 is excluded from HDV-001 |
| Minimum liquidity      | Average daily volume ≥ 100,000 shares over 20 sessions preceding decision_time |
| Liquidity check        | Applied at decision_time; not retroactively |

The 52-instrument list is frozen as the instruments present in
`stop_research_dataset_v01.json`. No instruments may be added or removed based on
subsequent performance.

---

## 7. Data Source

| Dimension              | Value                                      |
|------------------------|--------------------------------------------|
| Primary source         | Yahoo Finance (yfinance) — same source as existing dataset |
| Ticker format          | `{SYMBOL}.NS` (NSE suffix)                 |
| OHLCV interval         | Daily (1d)                                 |
| Adjustment             | Back-adjusted for dividends and splits (`auto_adjust=True`) |
| Cache identifier       | `hdv001_price_cache_v1` (to be created at HDV-001-B) |
| Cache hash             | Recorded at HDV-001-B; must not change after HDV-001-G |

---

## 8. Session Calendar

| Dimension              | Value                                      |
|------------------------|--------------------------------------------|
| Exchange               | NSE (National Stock Exchange of India)     |
| Timezone               | Asia/Kolkata (IST, UTC+5:30)               |
| Session open           | 09:15 IST                                  |
| Session close          | 15:30 IST                                  |
| Session count method   | Trading days only (NSE holidays excluded)  |
| Holiday calendar       | NSE official calendar for 2026             |
| Session 1 definition   | First trading day with open > decision_time |

---

## 9. Decision Cutoff

| Dimension              | Value                                      |
|------------------------|--------------------------------------------|
| Information cutoff     | `decision_time` field in each record       |
| Cutoff enforcement     | No bar with `bar_date >= decision_date` may be used to compute MAE/MFE |
| Reference price        | Close of the session immediately preceding `decision_time` |
| Intraday decisions     | If `decision_time` is intraday, reference price is the previous session close |

---

## 10. Observation Horizon

| Dimension              | Value                                      |
|------------------------|--------------------------------------------|
| Primary horizon        | 10 NSE trading sessions from `decision_time` |
| Session 1              | First session with open > decision_time    |
| Session 10             | 10th consecutive trading session           |
| Horizon outcome        | If neither target nor reference risk reached by session 10 close |
| Sensitivity horizons   | 1, 2, 3, 5 sessions (secondary analysis only; not used for primary criteria) |

---

## 11. Corporate-Action Treatment

| Dimension              | Value                                      |
|------------------------|--------------------------------------------|
| Method                 | Back-adjusted (multiplicative)             |
| Applied to             | All OHLCV data before any MAE/MFE calculation |
| Adjustment source      | yfinance `auto_adjust=True`                |
| Adjustment factor      | Recorded per instrument in cache metadata  |
| Verification           | Spot-check 5 instruments with known corporate actions |

---

## 12. Delisted / Suspended Symbols

| Dimension              | Value                                      |
|------------------------|--------------------------------------------|
| Treatment              | Evaluate up to last available bar; close with `HORIZON` outcome |
| Survivorship field     | `survivorship_status`: `ACTIVE`, `DELISTED`, `SUSPENDED`, `MERGED` |
| Exclusion              | None — all 52 instruments included regardless of subsequent status |

---

## 13. Survivorship Treatment

No instrument is excluded from the evaluation based on its subsequent status.
Decisions on instruments that are subsequently delisted, suspended, or merged are
evaluated up to the last available bar and then assigned `OutcomeStatus::Horizon`.

This is a hard rule. It must not be relaxed based on results.

---

## 14. Timezone

All timestamps in this document and in the HDV-001 implementation are in UTC.
IST (Asia/Kolkata, UTC+5:30) is used only for session boundary definitions.
All stored timestamps are UTC.

---

## 15. Data Availability Rule

No price bar may be used in the evaluation if its `bar_date` is on or before the
`decision_date` (the calendar date of `decision_time` in IST).

Specifically:
- `decision_date = decision_time.astimezone('Asia/Kolkata').date()`
- First evaluation bar: `bar_date > decision_date`
- This rule is enforced in the price path extractor (HDV-001-B).

---

## 16. Freeze Gate (HDV-001-G)

The following must be complete and recorded before HDV-001-G is declared:

- [ ] Price cache built and hash recorded (`hdv001_price_cache_v1`)
- [ ] Corporate-action spot-check complete (5 instruments)
- [ ] MAE/MFE calculator verified against known values (≥ 3 manual checks)
- [ ] Outcome classifier verified against known outcomes (≥ 3 manual checks)
- [ ] Baseline strategies implemented and verified
- [ ] Development period run complete (results recorded but not evaluated)
- [ ] No implementation changes pending

After HDV-001-G is declared, no changes to the implementation, data source, or
period definitions are permitted. Any required change creates HDV-002.

---

## 17. Relationship to Existing Dataset

The `stop_research_dataset_v01.json` dataset contains `B_max_adverse_excursion_pct`
and `B_max_favorable_excursion_pct` fields. These are **not** used as the HDV-001
MAE/MFE values because:

1. They are computed under Config B execution mechanics (capital gating, stop-loss
   triggers, position sizing), not at the raw decision level.
2. They reflect the realized path of a trade that was actually entered, not the
   full price path from `decision_time`.
3. HDV-001 requires decision-level MAE/MFE independent of execution mechanics.

HDV-001 recomputes MAE/MFE from raw OHLCV data for every certified decision,
regardless of whether it was realized under Config B or C.

---

*This document is frozen. Modifications require a new version (HDV-002-PERIODS.md).*