# Clean-slate 15m adapter selection design

**Protocol:** `RP-608-ADAPTER-SELECT-15M-V1`  
**Date:** 2026-11-27 (Asia/Calcutta)  
**Track:** Parallel to Gate 2E A/B soak — **does not** extend, retune, or stack `RP-608-GATE2E-15M-OHLCV-TREND-AGREE-001` / PR #7  
**Authority:** 0.0% observational twin until explicit promote  
**Aligned with:** Project `preferences.md` · `.cursor/rules/chronosentiment-core.mdc` §3b/§3c · Gate 2 v2 (`rp608_gate2_amendment_v2.md`) · Coralys no-touch · Rust-first / Python fail-closed I/O  
**Branch base:** `governance-hardening` → `cursor/clean-slate-15m-adapter-selection-fcb5`  
**Status:** **SLICE 1 LANDED** — D1–D5 frozen (Anuj 2026-11-27). Scoped memo: `docs/research/rp608_adapter_select_15m_v1_scoped.md`. Pi arm/deploy still Anuj-gated (`RP608_ADAPTER_SELECT_15M_TWIN` default off).

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
| CRYPTO BTC 15m | 15m | **Slice 1 armed path** (env-gated) |
| CRYPTO ETH 15m | 15m | **Slice 1 armed path** (env-gated) |
| NSE Estimand A 15m | 15m | **Deferred** (D3 — after crypto proven) |
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

### 3.2 Twin book ids (locked · slice 1)

| Book | Dir (under symbol output) | Book id |
| --- | --- | --- |
| BTC | `trade_plan_overlay_adapter_select_15m_v1` | `CRYPTO_PLAN_ADAPTER_SELECT_15M` |
| ETH | `trade_plan_overlay_adapter_select_15m_v1` | `CRYPTO_PLAN_ADAPTER_SELECT_15M_ETHUSDT` |
| NSE | — | **Not armed** (D3) |

---

## 4. First implementation slice (after design agreement)

### 4.1 In scope (slice 0 → slice 1)

**Slice 0:** design doc + stub branch from `governance-hardening`.

**Slice 1 (landed after D1–D5):**

| Surface | Location | Responsibility |
| --- | --- | --- |
| Pure scorer | `adapters/chronosentiment/src/product/adapter_select_15m.rs` | Features → `RecommendationStamp` |
| CLI mode | `rp608_position_decide --adapter-select-15m-loop` | stdin JSON closes + side → stdout stamp |
| Unit tests | `adapters/chronosentiment` + `tests/test_rp608_adapter_select_15m.py` | Determinism, insufficient-data, side projection |
| Thin Python I/O | `scripts/rp608_adapter_select_15m.py` | Resolve bin, fail-closed |
| Twin path | `scripts/run_rp608_shadow_daemon.py` + overlay `require_adapter_select_15m` | BTC+ETH observational (env default off) |
| Scorecard | `scripts/research/rp608_adapter_select_15m_status.py` | W/L-first per book vs A / BASELINE |
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

## 7. Decisions (frozen 2026-11-27 · Anuj)

| # | Decision | Freeze |
| --- | --- | --- |
| **D1** | Candidate source | **(a)** Score as filter on existing Coralys/Estimand candidate side |
| **D2** | Enter band | **\|S_side\| ≥ 0.35** (no mid-soak retune) |
| **D3** | First books | **BTC + ETH** observational; NSE deferred |
| **D4** | Gate 2E B | Leave soak running; exclude from this promote path |
| **D5** | Design | **Approved** as coding charter |

Slice 2 may revisit D1=(b) stand-alone generator if (a) still under-uses the engine.

---

## 8. Acceptance checklist

- [x] Anuj confirms abandon Gate 2E A/B stacking for this parallel track
- [x] D1–D4 answered
- [x] D5 approve
- [x] Slice 1 scoped memo frozen (`rp608_adapter_select_15m_v1_scoped.md`)
- [x] No Coralys / Gate 5 / Pi restart in slice 1 plan
- [ ] Pi deploy + `RP608_ADAPTER_SELECT_15M_TWIN=1` (Anuj-gated; not done in slice 1 code land)

---

## 9. References

- Project preferences: Coralys no-touch; Rust-first; 15m first; per-horizon books; clean-slate over A/B retune (2026-11-27); D1–D5 freeze
- `.cursor/rules/chronosentiment-core.mdc` §3b/§3c
- `docs/research/rp608_gate2_amendment_v2.md`
- `docs/research/rp608_adapter_select_15m_v1_scoped.md` (slice 1 freeze)
- `docs/research/rp608_gate2e_15m_ohlcv_trend_agree_scoped.md` (abandoned kernel for this track)
- Internal B soak: `internal/gate2e-b-soak-check-2026-11-27-0945.md`
- Hold fork: `internal/gate2e-b-next-pass-hold-2026-11-26.md`
