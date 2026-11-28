# Empirical engineering charter — Coralys complementary research

**Date:** 2026-11-28 Asia/Calcutta  
**Author:** Anuj (locked)  
**Rules:** `.cursor/rules/chronosentiment-core.mdc` §3b/§3c  

## Philosophy shift

| From | To |
| --- | --- |
| Research → select architecture → build complete solution → validate | **Observe → Hypothesize → Experiment → Measure → Diagnose → Improve** |

Literature helps generate hypotheses. It must not determine architecture in advance.

Guiding principle:

> Experiment before committing. Measure before interpreting. Diagnose before correcting. Repeat until the evidence supports a durable improvement.

Addendum: **Every failure must teach something that changes the next experiment.** Repeating with different thresholds until something succeeds is not the process.

## Loop stages

| Stage | What we do | Required output |
| --- | --- | --- |
| TRY | One bounded hypothesis | Sealed experiment |
| FAIL | Evaluate vs frozen baseline | Honest result |
| OBSERVE | Individual events + failure patterns | Root-cause evidence |
| FIX | Change the demonstrated weakness | Revised hypothesis |
| REPEAT | New experiment (not soak mutation) | Independent evidence |

### Failure categories (OBSERVE)

| Category | Investigate |
| --- | --- |
| Directional failure | Recommendation did not predict expected direction |
| Timing failure | Direction reasonable; timing poor |
| Outcome-horizon mismatch | Research label ≠ frozen plan outcome |
| Regime sensitivity | Performance shifts under particular conditions |
| Execution failure | Friction eliminates otherwise useful predictions |
| Selection failure | Adapter accepts poor / rejects good recommendations |
| Data/measurement failure | Bad attribution, eligibility, or outcome reconstruction |

No corrective until evidence identifies the category. FIX addresses the cause, not the metric.

### Artefact

**Experimental Learning Register (ELR):** `rp608_sci_adapter_rec_experimental_learning_register.md`

### Operational holds (unchanged)

- BTC SCI soak → `HOLD_SOAK` · do not retune  
- ETH SCI-v1 → Anuj auth before twin arm  
- ML-1D → `DO_NOT_ADVANCE` · preserve findings  
- P1a SCI-clean → frozen diagnostic DNA · Result Inconclusive · treatment differentiation not observed  
- **Milestone (2026-11-28 09:02):** first valid DI/regime-alignment differential event — not N more trades; no new sealed package until event + outcome classified  
- Next NEW experiment emerges from that differential OBSERVE (not a menu of P1b/P3)  

---

*End — empirical engineering charter · 2026-11-28 Asia/Calcutta.*
