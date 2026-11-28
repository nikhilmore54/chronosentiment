# Sealed comparative OBSERVE — SCI_CLEAN vs stacked SCI (BTC)

**As of:** 2026-11-28 ~08:46 Asia/Calcutta  
**RQ (Anuj lock):** Does removing inherited Gate2E selection contamination materially change SCI recommendation quality, coverage ownership, and after-cost economics on the same BTC book?  
**Rules:** `.cursor/rules/chronosentiment-core.mdc` §3b/§3c · twins **not** modified · no pooling · no ML-1D · no Coralys/Gate 5 · no mid-soak cut/feature changes  
**Tool:** `scripts/research/rp608_sci_adapter_rec_clean_vs_stacked_observe.py`  
**DNA:** stacked = inherited Gate2E selection stack · CLEAN = separate frozen diagnostic (`skip_gate2e_regime_align`; PATH_INVALID retained)

---

## One-line status

**INCONCLUSIVE_SAMPLE** — post-arm closed n=1 identical on both twins; DI/PATH idle in window; lifetime stacked still shows Gate2E contamination (PATH 435 · DI 87) that CLEAN has not yet differentiated on closed economics.

---

## Board (independent books — never pooled)

| Book | Armed | Closed W/L | Net Σ | Open | DNA |
| --- | --- | --- | --- | --- | --- |
| Stacked SCI | Yes | **1/38** (n=39) | **−116.58** | 1 | HOLD_SOAK · do not retune |
| SCI_CLEAN | Yes | **0/1** (n=1) | **−4.27** | 1 | Diagnostic twin · do not retune |
| Gate2E B | Yes | 0/35 | −95.44 | 0 | Comparator only |

---

## Eligible recs & coverage ownership

Attempt = `PLAN_CREATED` + SCI suppress + DI_SKEW + PATH_INVALID (excludes `OPEN_PLAN`).

| Scope | Attempt | Created | Created/attempt | DI_SKEW | PATH_INVALID | SCI suppress |
| --- | --- | --- | --- | --- | --- | --- |
| Stacked **lifetime** | 653 | 40 | 0.061 | **87** | **435** | 91 |
| Stacked **post-arm** | 2 | 2 | 1.0 | **0** | **0** | 0 |
| CLEAN **lifetime** | 2 | 2 | 1.0 | **0** | **0** | 0 |

**Coverage read:** Lifetime stacked is heavily PATH-dominated (≈67% of attempts) with DI ≈13%. Post-arm window has **not** yet produced DI/PATH pressure, so CLEAN’s DI/regime bypass has **no differential coverage event** to own yet. PATH_INVALID remains in CLEAN config DNA (retained by design) — 0 stamps so far = no path blocks in window, not “PATH removed.”

---

## Unique event overlap (post-arm)

| Metric | Value |
| --- | --- |
| CLEAN `PLAN_CREATED` plan_ids | 2 (`…3208693`, `…4063692`) |
| Stacked post-arm `PLAN_CREATED` | same 2 |
| Overlap | **2/2 (100%)** |
| CLEAN-only plans | **0** |
| Stacked-window-only plans | **0** |

Event tallies post-arm are synchronized: each book `OPEN_PLAN=117`, `PLAN_CREATED=2`, `CLOSED_SL=1` on the shared first plan.

---

## After-cost economics (failure-distribution preferred)

### Stacked lifetime (fail observation — unchanged DNA)

| | |
| --- | --- |
| W/L | 1/38 |
| Avg net win | +0.20 |
| Avg \|net loss\| | 3.07 |
| Net expectancy (avg net) | **−2.99** |
| Σ gross / fric / net | −37.35 / 46.81 / −116.58 |
| Failure primary | EXECUTION_FRICTION **24** · DIRECTIONAL **14** · WIN **1** |

### Post-arm aligned closed (shared plan_id)

| | CLEAN | Stacked aligned |
| --- | --- | --- |
| n | 1 | 1 |
| W/L | 0/1 | 0/1 |
| Exit | STOP_LOSS_HIT | STOP_LOSS_HIT |
| Gross / fric / net | −1.87 / 1.20 / **−4.27** | identical |
| Failure primary | DIRECTIONAL_FAILURE | DIRECTIONAL_FAILURE |
| Peak MFE | 0.86 (&lt; fric+$1) | same |

**Hit-rate contrast withheld as primary** — n=1; failure class = directional SL, not friction-primary on this single event.

---

## PATH_INVALID attribution

| Book / scope | PATH_INVALID count | Role |
| --- | --- | --- |
| Stacked lifetime | **435** | Inherited selection contamination (dominant attempt share) |
| Stacked post-arm | **0** | Idle in current window |
| CLEAN lifetime | **0** | PATH gate still enabled; no firings yet |

DI_SKEW: stacked lifetime **87** · post-arm **0** · CLEAN **0** (bypass holding).

---

## RQ answer (sealed OBSERVE)

| Question slice | Answer now |
| --- | --- |
| Recommendation quality | **Not yet separable** — shared plans, identical closed outcome |
| Coverage ownership | **Not yet demonstrated differentially** — no CLEAN-only plans; DI/PATH idle post-arm. Lifetime stacked still proves contamination exists upstream of CLEAN’s short sample |
| After-cost economics | **No material delta on aligned closed set** (n=1 identical −4.27). Lifetime stacked remains hostile (friction-dominant failure mix) |

**Verdict:** `INCONCLUSIVE_SAMPLE` — keep both DNA frozen; continue OBSERVE until DI/PATH pressure creates CLEAN-only eligibility or aligned n supports failure-distribution contrast. One CLEAN loss ≠ discard (Anuj). Do not cut-ratchet. Next FIX still governed by ELR failure teaching rule (execution/exit-geometry on stacked lifetime), not by this incomplete coverage test alone.

---

## Non-actions (locked)

- Do not modify stacked SCI or SCI_CLEAN mid-soak  
- Do not pool metrics  
- Do not remine ML-1D / change Coralys / Gate 5 floors  
- Gate2E B remains comparator only  

---

*End — CLEAN vs stacked sealed OBSERVE · 2026-11-28 Asia/Calcutta.*
