# ChronoSentiment — Provider Chronology Observatory

> **ChronoSentiment is a deterministic chronology interpreter with constrained operational governance.**

The platform provides a rigorous framework to observe market provider synchronization, validate chronology, and enforce fail-closed execution constraints. It does not attempt to predict market behavior, extract alpha, or define market cognition.

---

## 🌍 The Core Philosophy: Deterministic Chronology

ChronoSentiment has moved beyond the search for static, globally invariant trading laws or abstract market interpretation. The core operating principle is: **execution integrity requires absolute timeline determinism.**

We categorize the system into three distinct operational layers:

1. **Rust Kernel:** Deterministic chronology interpretation.
2. **Governor:** Operational constraint enforcement based on synchronization limits.
3. **Observatory:** Infrastructure measurement and delayed reconciliation profiling.

---

## 🔬 Provider Chronology Observatory

The system's findings are explored via a high-performance **observability dashboard** that visualizes timeline infrastructure reality:

| Diagnostic Domain | Key Metric |
| :--- | :--- |
| **Provider Synchronization** | Percentage of cohort successfully fetched at target timestamp |
| **Temporal Fragmentation** | Synchronization dispersion and cross-symbol non-uniformity |
| **Chronology Constraints** | Live governor permission (Nominal, Throttled, Halted) |
| **Reconciliation Duration** | Age-based completion delays for end-of-day closure ticks |
| **Trade Replay** | Step-by-step causal auditor ensuring exact replay equivalence |

---

## ⚙️ Architectural Disentanglement

ChronoSentiment separates the physical forces of execution through a frozen back-end engine:

*   **The Execution Simulation Engine (ESE):** Implements exact integer-scaled tick-by-tick order reconstruction. Decouples timing latency and execution friction.
*   **The Governor:** Dynamically constrains execution based on strict synchronization limits. If provider fragmentation exceeds safe thresholds, the governor halts execution.
*   **Fail-Closed Invariants:** The system assumes data is fragmented unless mathematically proven otherwise. No missing data is interpolated.

---

## 📊 Summary of Out-of-Sample (OOS) Baseline Checks

*Frozen Architecture, 30-Day Windows*

| Component | Validation Step | Required Outcome |
| :--- | :--- | :--- |
| **Kernel Replay** | `scripts/certify_replay_chain.py` | 100% Deterministic Match |
| **Cohort Baseline** | `scripts/verify_cohort_baseline.py` | Validated Dedupe & Signature Intact |
| **System Check** | `scripts/validate_system.sh` | Zero Archival Corruptions |

---

## 🚀 Scientific Roadmap

Our current development cycle targets **Track B: Soak Era Observation**:

1.  **Continuous Live Soak:** Letting the system run unedited for 2-6 weeks to observe natural boundary breaks.
2.  **Constraint Hardening:** Enforcing strict fail-closed bounds when temporal fragmentation occurs.
3.  **Observability Maturation:** Measuring provider delays directly rather than hypothesizing about market structures.

---

## 📂 Quick Start for Operational Proving

Run the live operational soak directly. Do not use legacy research scripts.

```bash
# 1. Establish Clean Baseline
./scripts/validate_system.sh

# 2. Run Live Provider Observation
python3 scripts/run_live_session.py --batch-id 003 --cycles 4 --live-only

# 3. Serve the Observatory
cd observatory
python3 -m http.server 8888
```
