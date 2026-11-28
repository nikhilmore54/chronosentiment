# ELR-SCI-CLEAN-OBSERVE-005 — multi-book adverse interval (2026-11-28 ~12:06 Asia/Calcutta)

**Interval:** ~11:30–12:00 Asia/Calcutta  
**Rules:** `.cursor/rules/chronosentiment-core.mdc` §3b/§3c · twins frozen · books never pooled · no mods  
**Anuj:** identical aggregate Δ ≠ automatic identical events — verify event ledger. BTC SCI twins each +2 / **−$5.42**; ETH B & AS each +4 / **−$8.15**.

---

## BTC SCI twins — two new closes

| plan_id | stacked=CLEAN? | side | exit | gross | fric | net | class |
| --- | --- | --- | --- | --- | --- | --- | --- |
| `PLAN-317808-SHORT-1795845588784` | **identical** | SHORT | GIVEBACK_PROTECT | +0.62 | 1.20 | **−0.58** | EXECUTION_FRICTION |
| `PLAN-317816-SHORT-1795845808143` | **identical** | SHORT | STOP_LOSS_HIT | −2.44 | 1.20 | **−4.84** | DIRECTIONAL_FAILURE |

**Pair sum = −5.42** · between-book Δnet = **0** · idxs 317808, 317816 · `clean_only` = ∅.

Event check: aggregate match **and** plan-level identity confirmed (not assumed).

---

## ETH Gate2E B vs adapter_select — four closes

| plan_id | on B | on AS | exit | gross | fric | net | class |
| --- | --- | --- | --- | --- | --- | --- | --- |
| `PLAN-094635-SHORT-…5195068` | Yes | Yes | STOP_LOSS_HIT | −2.50 | 1.20 | **−4.91** | DIRECTIONAL_FAILURE |
| `PLAN-094695-SHORT-…5550310` | Yes | Yes | GIVEBACK_PROTECT | −0.31 | 1.20 | **−1.51** | EXECUTION_FRICTION |
| `PLAN-094716-SHORT-…5673323` | Yes | Yes | TRAILING_STOP_HIT | +1.20 | 1.20 | **−1.20** | EXECUTION_FRICTION |
| `PLAN-094778-SHORT-…6118848` | Yes | Yes | GIVEBACK_PROTECT | +0.67 | 1.20 | **−0.53** | EXECUTION_FRICTION |

**Overlap:** plan_id ∩ = **4/4** · decision_idx ∩ = **{94635, 94695, 94716, 94778}** · pair sum **−8.15** each book.

Event check: identical aggregate Δ **was** identical events here (shared rec→exec pathway on ETH). Still score B vs AS as separate books — do not pool metrics.

---

## DI / regime differential (BTC SCI)

Post-arm stacked DI/PATH = **0** · CLEAN DI/PATH = **0** · treatment differentiation **still absent**.

---

## ELR table

| Field | Value |
| --- | --- |
| Experiment ID | `ELR-SCI-CLEAN-OBSERVE-005` |
| Result | **Inconclusive** (no DI contrast; adverse shared pathway) |
| Treatment differentiation | **Not observed** |
| Learning | Verified event ledger: BTC SCI twin closes identical; ETH B↔AS closes identical this window — aggregate Δ alone is insufficient process, but here events matched |
| Next | First DI/regime differential · keep cross-control event attribution · no classifier |
| Decision | **hold** twins frozen |

---

*End — ELR-SCI-CLEAN-OBSERVE-005 · 12:06 Asia/Calcutta 2026-11-28.*
