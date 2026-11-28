# RP-608-SCI-ADAPTER-REC — Aspirational product target

**Date:** 2026-11-27 Asia/Calcutta  
**Owner:** Anuj  
**Rules:** `.cursor/rules/chronosentiment-core.mdc` §3b/§3c · Gate 2 v2 · Coralys no-touch  
**Status:** ASPIRATIONAL TARGET registered · **not** a live cut · **not** Gate 5 · **not** implementation authorization  
**Store mirror:** Agent store `docs/sci-adapter-rec-aspirational-target-85.md`

---

## 1. Target (Anuj)

| Metric | Aspiration |
| --- | --- |
| Hit rate / W/L on **accepted** ACT subset | **85%** |
| Profit margins | **Good** — payoff asymmetry + **after-friction net** positive and durable (not WR-only) |

**Contract interpretation (scientific):**

1. **Hit rate = closed-trade W/L on the ACT book**, not unconditional directional accuracy on every Coralys flicker, and not open MTM.  
2. **Good margins** = avg|win|/avg|loss| and **net** after fees/spread/slippage clear a predeclared hurdle — Gate 2 v2 forbids WR-only success.  
3. Coverage must be **disclosed**. An 85% W/L at vanishing coverage is not product success unless net expectancy and trade count also clear predeclared floors.  
4. Per-book only (BTC first); never pooled.

---

## 2. Literature gap (must stay visible)

| Benchmark | Level |
| --- | --- |
| Peer-reviewed **unconditional** 15m BTC directional accuracy (Jaquart et al. *JFDS* 2021) | **~53–54%** (ensemble 53.786%) |
| Selective daily pooled crypto accuracy (Jaquart et al. *JFDS* 2022, top 10% confidence) | **~57.5–59.5%** |
| Aggressive 70–100% published claims | Different horizon / selection / pooling — quarantined |

**Implication:** An **unconditional** 85% 15m directional classifier is **far outside** the honest peer-reviewed 15m BTC band. The only scientifically plausible path toward ~85% on an **accepted subset** is **extreme selectivity** (low coverage) plus economic filtering — i.e. ML-1D dual-dimension ACT/PASS — **not** replacing Coralys with a deeper always-on side model.

Jaquart 2021 also shows pre-cost “good” returns can **die after TC**. Arain–Snudden 2026: mean hit rate ≠ profits. So the **margin** half of the target is not optional.

---

## 3. How ML-1D maps to the target

| Layer | Role toward 85% + margins |
| --- | --- |
| Coralys (sealed) | Primary side — do not chase 85% by replacing it |
| Reliability \(Q_t\) + uncertainty | Raise **conditional** W/L on the ACT subset via evidence sufficiency |
| Economic \(U_t\) | Enforce **margins** — PASS when net utility ≤ 0 even if “reliable” |
| Deterministic ACT/PASS | Coverage–accuracy trade-off owned by adapter |
| E0–E3 | Prove combination adds after-cost value, not WR cosmetics |

**Anti-patterns that will fake 85%:**

- Remine cuts on soak until WR prints high  
- Pool books / horizons  
- Count open marks  
- Ignore coverage and net  
- Import 82–100% paper numbers as cuts  

---

## 4. Success definition (when we eventually evaluate)

Predeclare before any sealed package (after SELECT):

| Gate | Requirement |
| --- | --- |
| W/L | Closed ACT-subset W/L vs aspiration **85%** (report Wilson LB; do not claim 85% from tiny n) |
| Margins | Payoff asymmetry + friction **net** > predeclared hurdle |
| Coverage | Predeclared band; too-thin coverage = fail even if WR high |
| Negatives | Beat Gate2E B, adapter_select, SCI-v1 on same eligible population (never pooled) |
| HLZ | Modest early lifts ≠ promote; burned holdout once |

Until sample size supports a meaningful Wilson lower bound near the aspiration, status stays **hypothesis / HOLD_SOAK**, not “85% achieved.”

---

## 5. Immediate stance

- Target **registered** as product aspiration.  
- Literature says: path = **selective use of Coralys** + economic filter, not unconditional 85% side generation.  
- Still need **G0** + explicit **SELECT ML-1D** before design/code.  
- No implementation, remine, or Gate 5 from this note.

---

*End — aspirational 85% + margins target · 2026-11-27 Asia/Calcutta.*
