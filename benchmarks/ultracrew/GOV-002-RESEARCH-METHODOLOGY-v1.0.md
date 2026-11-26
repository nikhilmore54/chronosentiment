# GOV-002 — Research Methodology
## Version 1.0 — Frozen 2026-07-13

This document is the constitutional research methodology for the Coralys
programme. It changes only through deliberate governance decisions, not because
implementation evolves. Sprint reports reference this document rather than
restating the methodology.

---

## Principle: Evidence Before Implementation

Every optimization change begins with a research question, not an idea.

No platform research work begins without:
1. A product question that motivates it.
2. Evidence that existing benchmarks cannot answer it.
3. A frozen benchmark.
4. A written, measurable hypothesis.

---

## Hypothesis Lifecycle

```
Product question
      ↓
Hypothesis written (before measurement)
      ↓
Benchmark selected or frozen
      ↓
Measurement
      ↓
Analysis
      ↓
Conclusion (one of three allowed outcomes)
      ↓
Freeze
```

---

## Allowed Conclusions

Exactly three conclusions are permitted. No others.

**1. Better solution discovered.**
The hypothesis is confirmed. The change improves the objective.

**2. Bottleneck identified.**
The hypothesis is refuted. The investigation reveals where the real constraint
lies. This is a valid and valuable outcome.

**3. Optimum proven.**
The current solution is shown to be optimal within the benchmark constraints.
No further improvement is possible without changing the problem.

---

## Language Rules

These rules apply to all freeze documents and research reports.

1. Claims must not exceed evidence. If a result is stochastic, say so.
2. "Open question" means the evidence is insufficient to decide. Do not
   substitute "intentional trade-off" or "expected behaviour" without evidence.
3. Ratios and magnitudes observed on one benchmark are specific to that
   benchmark instance. Do not generalize without evidence from multiple instances.
4. Characterization precedes redesign. Do not change objective weights or
   operator design before characterizing current behaviour.
5. Freeze documents are immutable. Corrections require a new version.

---

## Freeze Rule

Once a sprint report, benchmark, or measurement set is frozen and committed:

- The document does not change.
- The benchmark does not change.
- The measurements do not change.

If a correction is required, a new version is created (e.g. v1.1) with an
explicit change log. The original version is preserved.

---

## Hypothesis History (H1–H7)

| ID | Sprint | Hypothesis | Outcome |
|---|---|---|---|
| H1 | S3 | Skill-aware init eliminates HC1 violations at Gen0 | CONFIRMED |
| H2 | S4 | constraint_aware_pick() improves Gen0 average fitness | CONFIRMED |
| H3a | S5 | Population diversity is the convergence bottleneck | REFUTED |
| H3b | S5 | 200-gen run reveals plateau at current operator design | CONFIRMED |
| H3c-1 | S6 | Workload-balanced swap raises the fitness ceiling | REFUTED (ceiling unchanged; avg improved) |
| H4 | S6 | Per-week integer bound proves 9918.4 is the optimum | CONFIRMED |
| H5 | S7 | Normalized fatigue [0,1] yields numerically stable SC2 | CONFIRMED |

Sprint 8 hypotheses H6, H7, H8 are open (characterization, not optimization).

---

## References

- [`GOV-001-PROGRAMME-GOVERNANCE-v1.0.md`](GOV-001-PROGRAMME-GOVERNANCE-v1.0.md) — programme governance and platform research prerequisites
- [`GOV-003-BENCHMARK-LIFECYCLE-v1.0.md`](GOV-003-BENCHMARK-LIFECYCLE-v1.0.md) — benchmark creation and retirement rules