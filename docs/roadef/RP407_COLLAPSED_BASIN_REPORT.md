# RP-407 — Collapsed Basin Investigation

**Status:** Open — evidence gathering in progress  
**Depends on:** RP-410 telemetry (per-generation and per-move instrumentation)  
**Exit gate:** ≥ 5 of 6 collapsed-basin instances reproducibly enter the correct routing family  
**Target deliverable:** Root-cause report with mechanistic hypothesis confirmed or falsified

---

## 1. Background

RP-406C identified two distinct regimes across the 20 Set A instances:

- **Collapsed Basin (6 instances):** setA-06, setA-08, setA-10, setA-13, setA-16, setA-19.
  These instances converge to a common routing family that is lexicographically dominated by
  the sprint-reference solution at nearly every rank.
- **Normal Shape Competition (14 instances):** The remaining instances, where Coralys produces
  the characteristic Peak ✓ / Shoulder ✗ / Transition ✓ / Tail ✓ four-zone signature.

The collapsed-basin instances share a prefix-sum similarity that suggests they are all converging
to the same (or a closely related) routing family, rather than independently failing.

---

## 2. Research Questions

1. **Where does collapse begin?** At which generation does the routing entropy collapse and the
   load vector diverge from the reference trajectory?
2. **Why does it occur?** Is the cause:
   - Construction bias (the initial population lacks the correct routing family)?
   - Operator bias (accepted moves reinforce a suboptimal routing family)?
   - Fitness landscape topology (the correct routing family is in a separate basin with no
     gradient path from the initial population)?
3. **Is it deterministic?** Does collapse occur on every seed, or only on some?
4. **Is it instance-specific?** Do the 6 collapsed-basin instances share a structural property
   (topology, demand matrix, link capacity) that the 14 normal instances do not?

---

## 3. Instrumentation Requirements

The following telemetry must be collected per generation for all 6 collapsed-basin instances
and at least 2 successful instances (setA-12, setA-15 recommended as the regression baseline):

| Field                | Description                                                    |
| -------------------- | -------------------------------------------------------------- |
| `generation`         | Generation index                                               |
| `mlu`                | Maximum Link Utilisation (scalar)                              |
| `load_vector_full`   | Complete sorted load vector (all ranks)                        |
| `top20_prefix`       | First 20 entries of the load vector                            |
| `sdi`                | Shoulder Dominance Index (Ranks 2–20, weighted)                |
| `diversity`          | Population diversity metric (e.g. pairwise Hamming distance)   |
| `routing_entropy`    | Shannon entropy over routing family distribution               |
| `parent_lineage`     | Parent genome IDs for each accepted individual                 |
| `accepted_operators` | Operator types that produced accepted moves this generation    |
| `rejected_operators` | Operator types that produced rejected moves this generation    |

This is a superset of the RP-410 per-move schema. RP-410 instrumentation must be implemented
before RP-407 analysis can begin.

---

## 4. Comparison Protocol

For each of the 6 collapsed-basin instances:

1. Run with RP-410 telemetry enabled, multiple seeds (minimum 5).
2. Identify the generation at which routing entropy drops below a threshold (to be calibrated
   against the successful instances).
3. Compare the top-20 prefix trajectory against setA-12 and setA-15 at matched generation counts.
4. Record the operator distribution at the collapse generation vs. 10 generations before.

---

## 5. Hypotheses

The following hypotheses are to be tested in order of prior probability:

| Hypothesis | Description | Test |
| ---------- | ----------- | ---- |
| H1: Construction bias | The initial population for collapsed-basin instances lacks the correct routing family | Compare initial population routing entropy across instance types |
| H2: Operator reinforcement | Accepted operators in early generations reinforce the wrong routing family | Compare accepted-operator distribution in generations 1–50 |
| H3: Fitness landscape | The correct routing family is in a separate basin with no gradient path | Inject a known-good solution and measure convergence |
| H4: Instance structure | The 6 instances share a structural property that makes the correct family unreachable | Compare topology/demand statistics across instance types |

---

## 6. Findings

*This section will be populated as evidence is gathered.*

### 6.1 Collapse Generation

| Instance  | Median collapse generation | Std dev | Deterministic? |
| --------- | -------------------------- | ------- | -------------- |
| setA-06   | TBD                        | TBD     | TBD            |
| setA-08   | TBD                        | TBD     | TBD            |
| setA-10   | TBD                        | TBD     | TBD            |
| setA-13   | TBD                        | TBD     | TBD            |
| setA-16   | TBD                        | TBD     | TBD            |
| setA-19   | TBD                        | TBD     | TBD            |

### 6.2 Hypothesis Verdicts

| Hypothesis | Verdict | Evidence |
| ---------- | ------- | -------- |
| H1         | TBD     | TBD      |
| H2         | TBD     | TBD      |
| H3         | TBD     | TBD      |
| H4         | TBD     | TBD      |

---

## 7. Proposed Fix

*To be completed after hypothesis testing.*

The proposed fix must:
- Not regress the four durable wins (setA-12, setA-15, setA-17, setA-18).
- Be validated across at least 5 seeds per instance.
- Cause ≥ 5 of the 6 collapsed-basin instances to enter the correct routing family.

---

## 8. Data Files

*To be populated as telemetry runs complete.*

| File | Description |
| ---- | ----------- |
| `rp407_telemetry_setA-06.jsonl` | Per-generation telemetry for setA-06 |
| `rp407_telemetry_setA-08.jsonl` | Per-generation telemetry for setA-08 |
| `rp407_telemetry_setA-10.jsonl` | Per-generation telemetry for setA-10 |
| `rp407_telemetry_setA-13.jsonl` | Per-generation telemetry for setA-13 |
| `rp407_telemetry_setA-16.jsonl` | Per-generation telemetry for setA-16 |
| `rp407_telemetry_setA-19.jsonl` | Per-generation telemetry for setA-19 |
| `rp407_telemetry_setA-12.jsonl` | Per-generation telemetry for setA-12 (control) |
| `rp407_telemetry_setA-15.jsonl` | Per-generation telemetry for setA-15 (control) |

---

## 9. Document History

| Date       | Event                                                    |
| ---------- | -------------------------------------------------------- |
| 2026-08-04 | Stub created. RP-410 instrumentation prerequisite noted. |