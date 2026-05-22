# QFTH Accelerator 2026 — Application Answers
## ChronoSentiment

> **Core identity:** A deterministic chronology observability and execution-governance system for AI-driven financial infrastructure.

---

## 1. Describe your product or service

ChronoSentiment is an execution integrity platform for AI-driven financial systems.

It measures synchronisation ratio and temporal dispersion across provider data cohorts in real time, and gates execution through a deterministic governor with three states: NOMINAL, THROTTLED, and HALTED.

The core problem it solves: AI trading models execute on data that appears valid but is chronologically unsound. When provider feeds arrive with gaps, out of sequence, or with temporal drift, the model's view of the market is partial — but the system has no mechanism to detect or act on this. ChronoSentiment makes that failure mode visible and governable.

The platform produces a replay-safe timestamp at every ingest cycle: the last point at which deterministic reconstruction of the market view is guaranteed. Any execution beyond that boundary is suppressed.

---

## 2. What problem are you solving, and why does it matter?

AI-driven execution systems have a structural blind spot: they do not know whether the data they are acting on is chronologically trustworthy.

This is not a data quality problem in the conventional sense. Individual data points may be valid. The issue is temporal coherence across a cohort — whether the full set of symbols the model depends on was synchronised at the same point in time.

Our empirical observation on NSE demonstrates this concretely:

- **batch_003** (NSE broad market, 500 symbols): 54.2% synchronisation at market open, dispersion 2.47. The system was operating on a partial, temporally fragmented view of the market — every session, repeatably.
- **batch_910** (NSE banking cohort, 13 symbols): 100% atomic synchronisation, dispersion ~0.0. Full chronology integrity.

Most AI trading systems have no instrumentation to distinguish these two states. They execute regardless. ChronoSentiment makes the distinction explicit, measurable, and actionable.

This matters because:
1. Silent execution on corrupted chronology is a systemic risk that compounds as AI models operate at sub-second timescales.
2. Regulatory frameworks (SEBI algorithmic trading rules, DFSA AI governance guidelines) are trending toward requiring explainability and audit trails for automated execution. The infrastructure to support this does not yet exist as a standalone product.
3. Post-hoc reconstruction of what a model "saw" at execution time is impossible without a certified replay-safe timestamp. Auditability is broken by design in current systems.

---

## 3. What is your solution and how does it work?

ChronoSentiment operates as an execution integrity layer between data ingestion and trading execution.

**Ingest layer (cs-ingest, Rust):** Canonical ingest engine certified at 18,618/18,618 exact matches against the legacy Python implementation on the full NSE batch_003 dataset. Deterministic output is guaranteed.

**Measurement layer:** At every ingest cycle, the system computes:
- Synchronisation ratio: percentage of cohort symbols successfully fetched at the target timestamp
- Synchronisation dispersion: temporal spread across the cohort (standard deviation of fetch timestamps)

**Governor (Python, deterministic):** Translates measurements into execution state:
- NOMINAL (sync ≥85%, dispersion <1.5): full execution, multiplier 1.00
- THROTTLED (sync 70–85%, dispersion 1.5–2.0): reduced participation, multiplier 0.40
- HALTED (sync <70% or dispersion >2.0): execution fully suppressed, multiplier 0.00

**Chronology certification:** The replay-safe timestamp — the last point at which deterministic reconstruction is guaranteed — is written to an immutable ledger at every cycle.

**Observatory:** A 4-surface monitoring interface (Observatory, Replay Timeline, Trade Inspector, Research Console) surfaces real-time and historical synchronisation state, governor decisions, and causal audit trails.

Every suppression event is fully traceable: feed received → sync measured → dispersion computed → threshold breach → HALT issued → execution suppressed.

---

## 4. What is your traction or validation to date?

**Technical validation:**
- cs-ingest Rust crate certified at 18,618/18,618 exact parity against legacy Python on NSE batch_003 (500 symbols, full dataset)
- Two-regime empirical finding documented across NSE open, midday, pre-close, close, and post-close sessions — the batch_003 / batch_910 split is repeatable and session-independent
- Governor state machine implemented and operational; suppression screen demonstrates all three states with live data

**Empirical foundation:**
- The synchronisation failure pattern in batch_003 is not a hypothesis. It is a documented, repeatable observation across real NSE market sessions. The system has been observing and logging this pattern systematically.
- The control cohort (batch_910) confirms the measurement infrastructure is sound — when a cohort is genuinely synchronised, the system correctly reports 100% sync and NOMINAL state.

**Infrastructure maturity:**
- Production-grade ingest pipeline with zero-divergence certification
- Immutable research ledger with session-by-session empirical observations
- Institutional-grade observatory and governance UI

---

## 5. Who is your target customer, and what is the market opportunity?

**Primary:** AI-driven hedge funds and proprietary trading desks operating on NSE, BSE, and Gulf exchanges (ADX, DFM, Tadawul) that require execution integrity guarantees and regulatory audit trails.

**Secondary:** Systematic trading desks building compliance infrastructure for SEBI algorithmic trading framework requirements and DFSA AI governance guidelines.

**Emerging:** Fintech infrastructure providers building AI execution layers for institutional clients who need to demonstrate chronology integrity to their own clients and regulators.

**Why the timing is right:**
- AI execution is accelerating across MENA and South Asian markets, tightening latency and data integrity requirements
- No incumbent product addresses chronology observability as a standalone infrastructure layer — risk systems exist, but execution integrity monitoring does not
- Regulatory pressure is building: SEBI and DFSA are both moving toward requiring explainability and audit trails for automated execution
- The empirical finding is documented and reproducible — this is not a speculative market thesis

---

## 6. Why QFTH, and what do you need from the accelerator?

QFTH provides the specific access that ChronoSentiment cannot acquire independently at this stage:

**Gulf exchange data feeds:** Validating the two-regime finding on ADX, DFM, and Tadawul data is the highest-priority technical milestone post-submission. QFTH's exchange relationships make this possible.

**Institutional trading desk introductions:** The product requires a design partner — a trading desk willing to instrument their execution pipeline with ChronoSentiment and validate the governor in a live environment. QFTH's network is the fastest path to that partnership.

**Regulatory dialogue:** Direct engagement with DFSA on AI execution audit requirements would allow ChronoSentiment to position ahead of the regulatory curve rather than reacting to it. QFTH's regulatory relationships are unique in this regard.

**Co-development partnerships:** AI trading infrastructure firms building execution layers for institutional clients are natural integration partners. QFTH's cohort and alumni network provides access to exactly this segment.

What we bring to QFTH: a production-grade technical foundation, a documented empirical finding, a clear regulatory positioning, and a product identity that is coherent, differentiated, and institutionally legible.

---

## 7. Describe your team

*(To be completed with founder details)*

---

## 8. What is your business model?

ChronoSentiment is positioned as infrastructure, not a trading product. The business model is B2B SaaS:

- **API access tier:** Per-cohort, per-session synchronisation measurement and governor state — priced per exchange and cohort size
- **Audit trail tier:** Immutable ledger access, replay-safe timestamp certification, and causal chain export for regulatory compliance
- **Observatory tier:** Full 4-surface monitoring interface with historical replay and trade inspection

The regulatory compliance use case (audit trail, replay certification) is the highest-value tier because it is non-discretionary spend for regulated entities operating AI execution systems.

---

## One-sentence description

ChronoSentiment is a deterministic chronology observability and execution-governance system for AI-driven financial infrastructure — it measures whether your data is trustworthy before your model acts on it.