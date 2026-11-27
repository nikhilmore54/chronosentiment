# Clean-slate 15m adapter selection design

**Protocol (proposed):** `RP-608-ADAPTER-SELECT-15M-V1`  
**Date:** 2026-11-27 (Asia/Calcutta)  
**Track:** Parallel to Gate 2E A/B soak — **does not** extend, retune, or stack `RP-608-GATE2E-15M-OHLCV-TREND-AGREE-001` / PR #7  
**Authority:** 0.0% observational twin until explicit promote  
**Aligned with:** Project `preferences.md` · `.cursor/rules/chronosentiment-core.mdc` §3b/§3c · Gate 2 v2 (`rp608_gate2_amendment_v2.md`) · Coralys no-touch · Rust-first / Python fail-closed I/O  
**Branch base:** `governance-hardening` → `cursor/clean-slate-15m-adapter-selection-fcb5`  
**Status:** **DESIGN ONLY** — no implementation this turn; coding waits on Anuj decisions in §7.

---

## 0. One-line intent

Rebuild **adapter-owned entry selection** from completed **15m OHLCV** multi-lookback features into **horizon-scoped recommendation scores**, scored **W/L-first per book**, on an observational twin that never touches Coralys crates or frozen PRIMARY floors.

---

## 1. Why Gate 2E A/B knobs are abandoned (not patched)

### 1.1 Observed failure mode (B soak)

Latest observational scorecard (~09:45 Fri 2026-11-27 IST Asia/Calcutta, protocol `RP-608-GATE2E-15M-OHLCV-TREND-AGREE-001`):

| Book | B closed W/L | B WR | vs A / BASELINE |
| --- | --- | --- | --- |
| BTC | 0W / 15L | 0% | A ~35.7%, BASELINE ~16.6%; B suppress ≫ closes |
| ETH | 0W / 23L | 0% | A/BASELINE ~15.9% |
| NSE | 10W / 76L | 11.6% | A ~28.8%, BASELINE ~8.7% |

B is **DO_NOT_ADVANCE**. Suppress volume dominates BTC/ETH; when B does open, expectancy is hostile. NSE B underperforms A and is not a promotion path.

### 1.2 Structural diagnosis (not a threshold bug)

The abandoned stack treats selection as a **binary gate on an already-emitted candidate**:

1. Coralys / Estimand A / Policy stack produces a candidate side.
2. Adapter `ohlcv_trend_agreement` (L3/L8/L20 sign votes, ≥2 agree / 0 oppose) **allows or blocks**.
3. Live Gate 2E A knobs (pattern allow_only, B-persist, require-OF, regime align, shadow thin-WR, …) remain a **stacked** PRIMARY filter path.

That architecture fails this track for reasons that **threshold retunes cannot fix**:

| Failure class | Why patching A/B is wrong |
| --- | --- |
| **Post-candidate veto** | Agreement only thins after the engine already chose a side; it cannot re-rank or invent better candidates. Wrong params upstream stay wrong. |
| **Knob stacking** | Each Gate 2E A follow-on (persist / OF / regime / hygiene) is a band-aid on the same decision surface. Further stacking increases suppress without proving expectancy. |
| **Agreement ≠ expectancy** | Pure lookback *agreement* is a directional consistency check, not a W/L-calibrated score. Hostile B closes show consistency ≠ edge. |
| **Same frozen geometry** | Exits/OF/traj15 stay frozen (correct under Gate 5). Entry selection must therefore carry the expectancy load — a veto on a weak generator cannot. |
| **Anuj fork held** | 2026-11-26 hold: “wrong parameters” / strong engine under-used → prefer **clean-slate adapter selection** over retune-B or unstack-A as the *implementation* next step for this parallel track. |

### 1.3 Explicit non-goals for this track

- Do **not** retune L3/L8/L20 agreement thresholds on the B twin.
- Do **not** further stack PRIMARY Gate 2E A knobs.
- Do **not** reuse PR #7 trend-agree loop as the selection kernel (may remain on Pi as observational soak only; this design does not depend on it).
- Do **not** edit `coralys-*` to “fix” recommendation quality.
- Do **not** change OF ±0.30, traj15 floors, stops/targets, or authority without Gate 5 approval.

Gate 2E B soak continues **observational / HOLD implement** per Project preference; this design is the parallel replacement path.

---

## 2. New selection architecture

### 2.1 Principles (locked by preferences + rules)

1. **15m OHLCV first** — completed bars only; no partial/forming bar; candle volume is not a tick trade filter (tick authenticity stays separate per `rp608-tick-volume-trade-filter.mdc`).
2. **Adapter-owned** — selection / hygiene / exit *observation* adapters only; Coralys decision crates no-touch.
3. **Rust owns math** — pure deterministic functions + CLI; Python is thin stdin/stdout / journal fan-out, **fail-closed** if the Rust bin is missing.
4. **Horizon-scoped books** — score BTC, ETH, NSE (and later other intervals) **separately**; never pool W/L or net.
5. **W/L-first promotion** — primary metric wins/losses per book; payoff asymmetry, friction, net are required disclosure (chronosentiment-core §3b); WR-only “wins” that destroy net fail.
6. **Deterministic** — same closes + candidate context → same stamp; no randomness in strategy logic.

### 2.2 Pipeline (conceptual)

```
completed 15m OHLCV closes (decision-time)
        │
        ▼
┌───────────────────────────────┐
│ Rust: feature extract         │  multi-lookback returns, slope ranks,
│       (deterministic)         │  range/structure labels — NOT binary veto
└───────────────┬───────────────┘
                ▼
┌───────────────────────────────┐
│ Rust: horizon scorer          │  side-aware score S ∈ [-1, +1] or ranked
│       → RecommendationStamp   │  bands: ENTER / STANDASIDE / INSUFFICIENT
└───────────────┬───────────────┘
                ▼
┌───────────────────────────────┐
│ Adapter hygiene (existing)    │  MicroConfirm / sticky / trade-like tick
│ (unchanged floors)            │  gates remain; no Coralys edits
└───────────────┬───────────────┘
                ▼
┌───────────────────────────────┐
│ Twin book open?               │  observational only; PRIMARY untouched
│ journal RecommendationStamp   │
└───────────────┬───────────────┘
                ▼
     W/L-first scorecard per book → promote / hold / abandon
```

### 2.3 Multi-lookback 15m features (v1 proposal)

Use **completed** 15m closes available at decision time. Default lookbacks (tunable only after a sealed offline package — not live knob-churn):

| Feature family | Definition (sketch) | Role |
| --- | --- | --- |
| **Signed returns** | \(r_k = (c_{-1}-c_{-k})/|c_{-k}|\) for \(k \in \{3,8,20\}\) | Direction + magnitude (abandoned B used only `sign`) |
| **Magnitude ranks** | Rank \(|r_k|\) vs rolling window of prior completed bars (e.g. 50) | Distinguishes noise flats from real drift |
| **Agreement soft score** | Weighted sum of signed \(r_k\) (not hard ≥2/0 veto) | Continuity with prior research without binary suppress cliff |
| **Path consistency** | Fraction of intermediate closes moving with \(r_{20}\) | Reduces single-bar noise opens |
| **Side alignment** | Score projected onto candidate side from upstream engine **or** stand-alone side from score sign when twin is “generator mode” (decision §7) | Explicit choice before code |

**Insufficient data:** if any required lookback lacks history → `INSUFFICIENT` (fail-closed standaside), never invent bars.

### 2.4 Horizon-scoped recommendations

| Horizon book | Interval | First slice |
| --- | --- | --- |
| CRYPTO BTC 15m | 15m | **In scope** |
| CRYPTO ETH 15m | 15m | **In scope** |
| NSE Estimand A 15m | 15m | **In scope** |
| Other intervals (5m/1h/…) | — | **Out of scope** until 15m promote criteria met |

Each horizon book gets its own:

- Feature / score version stamp (`protocol_id` + `schema_version`)
- Twin journal stream / plan overlay id
- Scorecard (W/L, payoff, friction, net, coverage)
- Promotion decision

### 2.5 W/L-first promotion (Gate 2E v2 compatible)

Predeclare before first Pi soak results are read:

1. Predictors are decision-time only (no post-entry MFE as entry cause).
2. Chronological soak from twin arming; **no remine** of consumed 2026-11-03…05 windows.
3. Primary: twin W/L improves vs **BASELINE** (and vs PRIMARY/A when A has closed \(n \ge 20\)) **per book**.
4. Coverage floor: preferred cohort ≥ **min(40% of matched BASELINE opens, or absolute \(n \ge 80\))** on holdout unless tighter floor justified in review memo.
5. Supporting disclosure: payoff asymmetry, friction, net — required; net-destroying WR-only = fail.
6. Books never pooled; no symbol-ban lists as sole rule; no OF ±0.30 change from this track alone.
7. Explicit review memo before any promote request.

Verdict vocabulary: `SOAK` → `HOLD` / `DO_NOT_ADVANCE` / `ADVANCE_CANDIDATE` (still authority 0% until Gate 5).

---

## 3. Twin-book isolation vs PRIMARY

| Layer | PRIMARY (A / live Gate 2E stack) | Clean-slate twin (this track) |
| --- | --- | --- |
| Role | Unchanged production shadow path | Observational selection experiment |
| Authority | Existing (0% live orders) | **0%** — never places primary-authority orders |
| Selection | Existing A stack (untouched by this PR) | New Rust scorer + stamp |
| Journals | Existing overlay / estimand streams | **New** plan ids / twin journals (names TBD at impl) |
| Gate 2E B trend-agree twin | Continues observational soak if already armed | **Independent** — do not share mutable config |
| Promote | Only via Gate 5 after ADVANCE_CANDIDATE + approval | Same — twin never silently becomes PRIMARY |

### 3.1 Isolation rules

1. Twin **reads** the same market events / completed 15m closes; it does **not** write into PRIMARY decision state.
2. Env flags for this track are **new** (`RP608_ADAPTER_SELECT_15M_*`); they must not flip Gate 2E B trend-agree vars.
3. Scorecard scripts compare twin vs PRIMARY and vs BASELINE **side-by-side**, never pooled.
4. If Rust bin missing → twin **standaside / no-open** (fail-closed); PRIMARY continues.
5. No shared mutable OF policy config; OFX remains separate research line (`rp608-ofx.mdc`).

### 3.2 Suggested twin book ids (impl-time; not armed yet)

| Book | Twin id (proposed) |
| --- | --- |
| BTC | `trade_plan_overlay_adapter_select_15m_v1` |
| ETH | `trade_plan_overlay_eth_adapter_select_15m_v1` |
| NSE | `estimand_a_adapter_select_15m_v1` |

Exact strings locked at first implementation PR after §7 decisions.

---

## 4. First implementation slice (after design agreement)

### 4.1 In scope (slice 0 → slice 1)

**Slice 0 (this turn):** design doc + stub branch from `governance-hardening`. **No runtime code.**

**Slice 1 (after Anuj §7):**

| Surface | Location (proposed) | Responsibility |
| --- | --- | --- |
| Pure scorer | `adapters/chronosentiment/src/product/adapter_select_15m.rs` (new) | Features → `RecommendationStamp` |
| CLI mode | `rp608_position_decide --adapter-select-15m-loop` (new flag) | stdin JSON closes + context → stdout stamp |
| Unit tests | `adapters/chronosentiment` | Determinism, insufficient-data, side projection |
| Thin Python I/O | `scripts/rp608_adapter_select_15m.py` (new) | Resolve bin, fail-closed, journal fan-out only |
| Scorecard | `scripts/research/rp608_adapter_select_15m_status.py` (new) | W/L-first per book vs A / BASELINE |
| Protocol memo | `docs/research/rp608_adapter_select_15m_v1_scoped.md` | Freeze features + floors before soak |

### 4.2 Explicitly out of scope (slice 1)

- Any `coralys-*` crate edits
- Gate 5 floor / OF ±0.30 / traj15 / stop-target / hold retunes
- Changing Gate 2E B soak binaries, env, or journals
- Restarting Pi daemons (deploy/restart is Anuj-gated and separate)
- Multi-timeframe candles beyond 15m
- OFX authority or mechanism labeling
- Book pooling / universal trend filter / symbol bans from single-session loss lists
- Python reimplementation of scoring math
- Promoting twin → PRIMARY

### 4.3 Hygiene / exits (adapter-only, later slices)

- **Hygiene:** reuse existing tick / MicroConfirm / sticky paths; trade-like authenticity remains mandatory for NSE TICK_PATH fills.
- **Exits:** this track is **entry selection**. Exit geometry stays Gate 2X / frozen PRIMARY until a separate package. Twin may *observe* exit stamps but must not retune production exits in slice 1.

---

## 5. Success metrics — first Pi soak of the new design

Predeclare before arming twin (Gate 2 v2 aligned). Evaluate **per book** after a chronological soak window from arming.

### 5.1 Primary (must pass for ADVANCE_CANDIDATE)

| Metric | Rule |
| --- | --- |
| **W/L** | Twin win rate ≥ control (BASELINE; and PRIMARY/A when \(n_A \ge 20\)) **or** clear W/L ratio improvement with disclosed sample |
| **Coverage** | Twin opens ≥ min(40% of matched BASELINE opens, or absolute \(n \ge 80\)) unless tighter floor predeclared |
| **Replication** | Decision-time features only; chronological firewall; no remine of burned windows |
| **Books** | BTC / ETH / NSE judged separately — one book may ADVANCE while another HOLDs |

### 5.2 Supporting (required disclosure; can veto net-hostile WR-only)

| Metric | Expectation |
| --- | --- |
| Payoff asymmetry | avg\|win\| / avg\|loss\| reported |
| Friction | Gross vs net gap reported |
| Net expectancy | Must not collapse solely to buy WR with worse net |
| Suppress / INSUFFICIENT rates | Logged; suppress cliff without W/L lift = DO_NOT_ADVANCE |
| Fail-closed events | Bin-missing / parse errors counted; must not silently open |

### 5.3 Operational health (soak gating, not expectancy)

| Check | Pass bar |
| --- | --- |
| Twin journals append | Continuous stamps when market open |
| PRIMARY undisturbed | No config/binary change to A or Gate 2E B from this track |
| Rust bin present | Fail-closed standaside if absent |
| Daemon restarts | None required for design; deploy only after Anuj approve |

### 5.4 Early abort (optional, predeclare at arming)

If after a minimum closed sample (proposed: **n ≥ 30** closes on a book) twin WR is **≤ half** of BASELINE WR **and** net &lt; 0, mark that book `DO_NOT_ADVANCE` early without waiting for full coverage floor — still no PRIMARY change.

---

## 6. Relationship to existing artifacts

| Artifact | Relationship |
| --- | --- |
| `ohlcv_trend_agreement.rs` / PR #7 | **Abandoned as selection kernel** for this track; leave in tree for B soak continuity |
| Gate 2E A PRIMARY stack | Untouched; comparison control |
| Gate 2E B soak | Observational only; do not modify binaries/config from this track |
| `rp608_tick_rec_hygiene` | Hygiene remains relevant; selection root-cause moves here |
| Coralys recommendation engine | Consumed as black box input (if generator mode) or bypassed for side (if stand-alone mode) — **no crate edits** |

---

## 7. Decisions Anuj must make before coding

| # | Decision | Options | Recommendation |
| --- | --- | --- | --- |
| **D1** | Candidate source | **(a)** Score as filter on existing Coralys/Estimand candidate side · **(b)** Stand-alone side from 15m score sign (engine side ignored for twin opens) · **(c)** Hybrid: stand-alone side only when \|S\| ≥ enter band, else filter mode | **(a)** for slice 1 (lowest blast radius); evaluate (b) as slice 2 if (a) still under-uses the engine |
| **D2** | Enter band | Soft score threshold for ENTER vs STANDASIDE (e.g. \|S\| ≥ 0.35) — freeze one predeclared value | Freeze **one** value in scoped memo before soak; no mid-soak retune |
| **D3** | First book to arm on Pi | BTC only · ETH only · NSE only · all three observational | Start **BTC + ETH** observational; NSE after crypto scorecard path proven (NSE daemon currently fragile) |
| **D4** | Gate 2E B twin fate | Leave B soak running · disarm B twin · ignore B in new scorecards | **Leave running**, exclude from this track’s promote path |
| **D5** | Design agreement | Approve this doc as coding charter · request revisions | Required before slice 1 |

Coding starts only when **D5 = approve** and D1–D4 are explicit.

---

## 8. Acceptance checklist for “design agreed”

- [ ] Anuj confirms abandon Gate 2E A/B stacking for this parallel track (preferences already lean yes)
- [ ] D1–D4 answered
- [ ] D5 approve
- [ ] Slice 1 scoped memo frozen (features + enter band + coverage floor)
- [ ] No Coralys / Gate 5 / Pi restart in slice 1 plan

---

## 9. References

- Project preferences: Coralys no-touch; Rust-first; 15m first; per-horizon books; clean-slate over A/B retune (2026-11-27)
- `.cursor/rules/chronosentiment-core.mdc` §3b/§3c
- `docs/research/rp608_gate2_amendment_v2.md`
- `docs/research/rp608_gate2e_15m_ohlcv_trend_agree_scoped.md` (abandoned kernel for this track)
- Internal B soak: `internal/gate2e-b-soak-check-2026-11-27-0945.md`
- Hold fork: `internal/gate2e-b-next-pass-hold-2026-11-26.md`
