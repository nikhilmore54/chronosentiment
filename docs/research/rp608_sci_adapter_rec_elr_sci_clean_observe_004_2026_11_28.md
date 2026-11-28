# ELR-SCI-CLEAN-OBSERVE-004 — broad adverse interval (2026-11-28 ~11:37 Asia/Calcutta)

**Interval:** ~11:00–11:30 Asia/Calcutta (entry clocks ~05:28–05:57 UTC)  
**Rules:** `.cursor/rules/chronosentiment-core.mdc` §3b/§3c · twins frozen · no mods · no pooling / remine / Coralys / Gate 5  
**Anuj:** SCI twins each Δ**−$8.83** / **+5 closes**; broad losses also on Gate2E B + adapter_select · next = cross-control attribution (not classifier work)

---

## (1) Five new SCI plan IDs — identical across twins?

| plan_id | stacked | CLEAN | identical exit/gross/fric/net |
| --- | --- | --- | --- |
| `PLAN-317730-LONG-1795843698291` | Yes | Yes | **Yes** |
| `PLAN-317744-SHORT-1795844007764` | Yes | Yes | **Yes** |
| `PLAN-317779-SHORT-1795845117304` | Yes | Yes | **Yes** |
| `PLAN-317789-SHORT-1795845326949` | Yes | Yes | **Yes** |
| `PLAN-317802-SHORT-1795845450154` | Yes | Yes | **Yes** |

Between-book Δnet on all five = **0**. `clean_only` = ∅.

---

## (2) Overlap with Gate2E B / adapter_select?

| Check | Result |
| --- | --- |
| Shared `plan_id` with B or AS | **None** |
| Shared `source_decision_event_idx` with B/AS near-window | **None** |
| Same-window B/AS behavior | Parallel **SHORT** + **GIVEBACK_PROTECT** losses on **different** idxs (317772, 317783, 317793, 317806, 317814) |

**Read:** broad adverse tape / exit-geometry stress across controls — **not** the same decision events as SCI. Books remain unpooled; attribution is cross-control *regime*, not shared plan identity.

---

## (3) Failure class + gross / fric / net

| plan_id | side | exit | gross | fric | net | primary |
| --- | --- | --- | --- | --- | --- | --- |
| …3698291 | LONG | STOP_LOSS_HIT | −2.02 | 1.20 | **−4.42** | **DIRECTIONAL_FAILURE** |
| …4007764 | SHORT | GIVEBACK_PROTECT | +0.34 | 1.20 | **−0.86** | **EXECUTION_FRICTION** |
| …5117304 | SHORT | GIVEBACK_PROTECT | −0.26 | 1.20 | **−1.46** | **EXECUTION_FRICTION** |
| …5326949 | SHORT | GIVEBACK_PROTECT | +0.20 | 1.20 | **−1.00** | **EXECUTION_FRICTION** |
| …5450154 | SHORT | GIVEBACK_PROTECT | +0.11 | 1.20 | **−1.09** | **EXECUTION_FRICTION** |

**Pair sum net = −8.83** (4× friction-primary giveback + 1× directional SL).

---

## (4) DI / regime differential

Post-arm stacked DI/PATH/regime = **0** · CLEAN DI/PATH = **0** · treatment differentiation **still absent**.

---

## ELR table

| Field | Value |
| --- | --- |
| Experiment ID | `ELR-SCI-CLEAN-OBSERVE-004` |
| Result | **Inconclusive** (twins identical; adverse interval shared with other controls via parallel—not identical—events) |
| Treatment differentiation | **Not observed** |
| Learning | −$8.83 is same-book pair sum on shared SCI plans; friction giveback dominates (4/5); B/AS also hurt nearby without event-id overlap → cross-control stress, not CLEAN bypass value |
| Next | Continue wait for first DI/regime differential; cross-control attribution only — **no** new classifier / sealed package |
| Decision | **hold** both DNA |

---

*End — ELR-SCI-CLEAN-OBSERVE-004 · 11:37 Asia/Calcutta 2026-11-28.*
