# ChronoSentiment Roadmap V2: MVP Alignment

**Date:** 2026-06-03  
**Status:** DRAFT (Requires Approval)

## The Strategic Pivot
The ecology research program (Phases 1–3) successfully characterized a stable, asymmetric, and externally-forced market geometry. However, **ecology is not the MVP.**

The core promise of ChronoSentiment—as defined in the PRD, MVP, and SRS—is to provide a **deterministic execution validation environment**. The system's value lies in understanding execution realism, queue dynamics, latency, and observability, rather than predicting market direction.

Therefore, the ecology program will pivot from being a standalone product to serving as a **Validation Workload** for the MVP platform. Ecology discovery is the evidence that the replay engine and observability layer can accurately reconstruct non-obvious market structure.

---

## MVP Track (Phases 4–6)

### Phase 4: Replay Reproducibility
*Can ecology results be reproduced perfectly from replay?*
- **Focus:** Replay determinism, archive certification, identical execution paths.
- **Test:** Re-run the raw data through the replay engine. Ensure that the identical ecology geometry is extracted from the replay as was extracted from the raw historical catalogs.
- **Deliverable:** `Replay → Ecology (100% reproducible)`

### Phase 5: Execution Sensitivity
*What happens to the ecology when execution conditions change?*
- **Focus:** Execution realism modeling (latency, queue position, fill probability, slippage).
- **Test:** Inject varying degrees of latency or queue delays into the replay environment for a benchmark strategy.
- **Question:** Does the strategy's portfolio outcome alter its resulting "ecological consequence"? Do simulated execution failures cluster inside Ecology A or Ecology B?

### Phase 6: Explainability & Validation
*Can the system explain why a strategy behaved differently?*
- **Focus:** The core MVP success criteria (Traceability and Observability).
- **Test:** Demonstrate the full causal chain:
  `Trade → Queue Delay → Missed Fill → Portfolio Outcome → Ecological Consequence`
- **Deliverable:** A transparent explainability surface that uses the ecological geometry to summarize execution degradation.

---

## Research Track (Phases 7–9)

### Phase 7: Alpha Discovery
*Now that execution is validated, can we find an edge?*
- Moving beyond descriptive geometry to actionable trading signals, incorporating both internal geometry and the execution dynamics validated in Phase 5.

### Phase 8: Ecology Applications
*How can the geometry improve active strategies?*
- E.g., dynamically adjusting execution urgency or order sizing based on whether the market is in the baseline attractor (Ecology A) or the excursion state (Ecology B).

### Phase 9: Live Observatory
*Real-time validation.*
- Deploying the ChronoSentiment platform in a live-feed or paper-trading environment to monitor ecological transitions and execution fidelity in real-time.

---

## Immediate Next Steps
If this roadmap is approved, the immediate next action is to begin **Phase 4: Replay Reproducibility**. This will involve auditing the current replay engine and running the Q1/Q2 catalogs through it to verify deterministic reconstruction of the established two-ecology partition.
