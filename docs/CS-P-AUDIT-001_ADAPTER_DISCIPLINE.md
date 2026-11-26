# CS-P-AUDIT-001 — ChronoSentiment Adapter Architecture & Discipline Audit

**Document type:** Read-only architecture inventory  
**Status:** Accepted — PR-1 and PR-2 executed; CS-P-006-A (Policy Discovery Contract) in progress; optimizer not started  
**Date:** 2026-08-14  
**Scope:** `adapters/chronosentiment` (modules, binaries, tests, repositories, migrations)  
**Does not:** modify code, delete assets, create a candidate policy, run a backtest, reopen G-GATE, freeze v1.0  

`.cursor/rules/chronosentiment-core.mdc`: deterministic state machine; no invented methodology.

**Baseline commit (pre-audit):** `8405d6da5`

**Question this document answers:** what in the adapter violates, or could violate, the ChronoSentiment discipline — so a later cleanup pass can be authorized by category, not by instinct.

---

## 1. Discipline under audit

1. Event/as-of driven — no `Utc::now()` in decision-producing paths; persist time ≠ evaluation time  
2. Deterministic — same inputs + engine/policy version → identical decision  
3. Temporal firewall — inputs ≤ T; outcomes never construct decisions  
4. One-way flow — Information → Assessment → Policy → TradingDecision → Ledger → Outcome → Performance  
5. No invented information — missing ≠ zero; unavailable ≠ confidence  
6. Coralys learns; ChronoSentiment evaluates  
7. Research ≠ product; G-GATE v1.1 remains closed  

Classification: **KEEP / REMOVE / REFACTOR / QUARANTINE / NEEDS-EVIDENCE**

Lane: **PRODUCT / RESEARCH / LEGACY**

---

## 2. Map of what exists

```text
adapters/chronosentiment
├── PRODUCT (intended)
│   ├── decision_support::{replay, policy, backtest, forward, outcome, performance}
│   ├── reasoning::assessment (enrichment)
│   ├── metrics::instrument
│   └── TradingDecision contract
├── RESEARCH (must not be the product path)
│   ├── src/research/          (G-GATE / Uuid::new_v4 runs)
│   ├── m5_assessment_predictive_value
│   ├── m6_predictive_value_experiment
│   └── m6_phase_g_experiment (untracked)
└── LEGACY / SCAFFOLDING
    ├── reasoning::{decision, strategy, evidence, hypothesis, historical_reasoning}
    ├── m1–m4 demo/populate/gate binaries
    ├── week1/week2 tests (week2 does not compile)
    ├── evidence_engine / hypothesis_engine (commented out of reasoning/mod.rs)
    └── scripts/phase_c_gate.sh (prints fabricated PASS)
```

**Two decision objects exist.** Lake persist uses `reasoning::decision::Decision`. Product replay/forward uses `decision_support::TradingDecision`. They do not share identity, confidence, or policy.

---

## 3. Headline answers

### Is `TrendMappingPolicy` production or scaffolding?

**Scaffolding that became the live default.** It is the only `DecisionPolicy` impl. `decide_from_inputs` always calls it. Replay, forward tick, laboratory, and factor ecology all consume it. It is **not** a Coralys-discovered policy and **not** Decision Engine v1.0.

Recommendation: **QUARANTINE as named baseline fixture** (`baseline.trend_mapping.v0`), not silent “the ChronoSentiment strategy.” Do not delete until a Coralys candidate exists — deleting it would strand CS-P-002/003/004 evidence. Do not let it remain an unnamed default.

### Does ChronoSentiment contain its own optimizer?

**Not in the product adapter path.** No `rand` / MOGA search inside `decision_support`. **Yes in the research lane:** `src/research/` is a G-GATE laboratory compiled into the library (`pub mod research` in `lib.rs`). That is the coupling to remove or quarantine before Coralys policy discovery.

---

## 4. Inventory

| ID | Lane | File / symbol | Finding | Class |
|----|------|---------------|---------|-------|
| AUD-001 | PRODUCT | `decision_support/policy.rs` `TrendMappingPolicy`; `replay.rs` `decide_from_inputs` | Implicit trading rule is the adapter default. Bypasses Coralys discovery. Always-on LONG/SHORT from Trend. | **QUARANTINE** (keep as explicit baseline; stop treating as product strategy) |
| AUD-002 | PRODUCT | `replay.rs` `UNFROZEN_ENGINE_VERSION = "unfrozen-dev"` | Placeholder version is hardcoded into replay, forward, lab, CS-P-002/003/004 binaries. | **KEEP** until a frozen policy version exists; **REFACTOR** later to require an explicit version argument with no default |
| AUD-003 | PRODUCT | `replay.rs` `UNFROZEN_HORIZON_DAYS = 5` | Hidden horizon constant on every TradingDecision. Not a documented policy parameter. | **REFACTOR** (make policy/version-owned; do not silently keep 5) |
| AUD-004 | PRODUCT | `decision_support/mod.rs` vs `reasoning/decision.rs` | Two decision types. Lake `Decision` uses `Uuid::new_v4`, fabricated 0.5 confidence decomposition, comments “Baseline Decision Policy v1.0”. Product `TradingDecision` is the CS-P-002 contract. Populate still writes the lake type. | **REFACTOR** lake persist to consume `DecisionPolicy`/`TradingDecision` or **QUARANTINE** `DecisionEngine` as lake-only heritage |
| AUD-005 | LEGACY | `reasoning/decision.rs` `DecisionEngine::evaluate` | Hard-coded Trend→Opportunity; `Uuid::new_v4` identity; all confidence fields 0.5; labeled v1.0 in comments. Bypasses `DecisionPolicy`. | **QUARANTINE** |
| AUD-006 | LEGACY | `reasoning/strategy.rs` `StrategyEngine::generate` | Hidden ATR thresholds (+2/−1 ATR, 0.1 entry band); `confidence: 0.5`; only Positive opportunity emits a strategy (SHORT never gets lake strategies → 85 unevaluated SHORTs). | **QUARANTINE** (explains B4 SHORT outcome gap; do not use as product risk model) |
| AUD-007 | LEGACY | `m4_populate_knowledge_lake.rs` `atr_14.unwrap_or(current_close * 0.02)` | Invented ATR when missing. Same in `m4_validation_gate.rs`. | **REFACTOR** if populate remains; else **QUARANTINE** with gate binaries |
| AUD-008 | PRODUCT | `reasoning/assessment.rs` Trend confidence `0.82` / Momentum `0.73` | Fabricated assessment confidence. Product path no longer copies it onto `TradingDecision.confidence` (KEEP that firewall). Assessment scores still exist and appear on `evidence.factors.assessment_confidence`. | **REFACTOR** (UNAVAILABLE at assessment layer too, or clearly mark as non-decision) |
| AUD-009 | PRODUCT | `repository/knowledge.rs` `ArtifactMetadata::mock()` | Sets `created_at` and `evaluation_timestamp` to `Utc::now()`, `artifact_id` to `Uuid::new_v4`. `assess_at` overwrites eval time but still uses mock for persist clock and id. | **REFACTOR** (split persist metadata from replay T; stop calling it mock on the populate path) |
| AUD-010 | PRODUCT | `m4_populate_knowledge_lake.rs` `Instrument.created_at` / `recorded_at: Utc::now()` | Wall-clock on observation persist. Acceptable as `recorded_at` if never used as T. Instrument ids are `Uuid::new_v4` → dump SHA not a content identity. | **KEEP** recorded_at; **REFACTOR** instrument identity to stable ticker-derived ids if snapshots must repeat byte-for-byte |
| AUD-011 | RESEARCH | `historical_reasoning.rs` `evaluate` | Fallback calls `Utc::now()` as evaluation timestamp. `Uuid::new_v4` mock cases. Hard-coded 0.88/0.81/0.17 scores. | **QUARANTINE** |
| AUD-012 | LEGACY | `reasoning/evidence_engine.rs` | Wall-clock timestamp; `volatility > 0.3` hidden High/Low rule; `Uuid::new_v4`. File is **commented out** of `reasoning/mod.rs` but still on disk. Same for `hypothesis_engine.rs`, `scenario_engine.rs`, `journal.rs`. | **QUARANTINE** (do not re-export) |
| AUD-013 | LEGACY | `validation/mod.rs` `ValidationEngine::enrich_observation` | `observed_at` / `effective_from` default to `Utc::now()`. Comment admits this is a fallback. | **REFACTOR** or **QUARANTINE** if unused by product path |
| AUD-014 | PRODUCT | `csp003_forward_session.rs` `None => Utc::now()` | Forward clock may use wall-clock when `--now` omitted. Correct for live ticks if session `as_of` is still the latest bar ≤ now. **NEEDS-EVIDENCE** that `decide_latest_session` never uses now as a bar timestamp. | **KEEP** with constraint: now is a cutoff, never a fake bar |
| AUD-015 | PRODUCT | `forward_tick.rs` `profile.metadata.created_at = t` | Conflates persist time with evaluation T on the forward path. | **REFACTOR** |
| AUD-016 | RESEARCH | `src/research/*` `ResearchLaboratory::execute_experiment` | `run_id: Uuid::new_v4()`, `execution_time: Utc::now()`. Compiled into lib. G-GATE methodology, not CS-P-002. | **QUARANTINE** (remove from `lib.rs` or feature-gate) |
| AUD-017 | RESEARCH | `research/predictive_value.rs` JOIN `knowledge_outcomes` | Legitimate for G-GATE measurement; **must not** be reachable from `decide_at`. Currently separate module — KEEP separation; QUARANTINE the module from product builds. | **QUARANTINE** |
| AUD-018 | PRODUCT | `decision_support/outcome.rs` SELECT `knowledge_outcomes` | Correct layer. Replay adapter source-inspected to not SELECT it (CS-P-TEST-001 DEC-006). | **KEEP** |
| AUD-019 | PRODUCT | `csp005_factor_ecology.rs` SELECT `knowledge_outcomes` | Research measurement, not `decide_at`. Outcomes attached after row construction. | **KEEP** (research binary; do not merge into ReplayAdapter) |
| AUD-020 | LEGACY | `m1_reliance.rs`, `m2_week1_demo.rs`, `m2_week2_demo.rs`, `m3_reasoning_demo.rs`, `m4_time_machine_demo.rs` | Demo binaries; `mock()` metadata; look like runnable product. Cargo compiles every `src/bin`. | **QUARANTINE** (move to `examples/` or `bins/legacy`) |
| AUD-021 | RESEARCH | `m4_validation_gate.rs`, `m5_assessment_predictive_value.rs`, `m6_predictive_value_experiment.rs`, untracked `m6_phase_g_experiment.rs` | G-GATE / lake gate. Duplicate populate+assess loops. `m4_validation_gate` still uses `DecisionEngine` + invented ATR. | **QUARANTINE** |
| AUD-022 | LEGACY | `scripts/phase_c_gate.sh` | **Prints PASS without running tests.** Contradicts reproducibility claims. | **REMOVE** (or rewrite to actually invoke tests; current file is false evidence) |
| AUD-023 | LEGACY | `tests/week2_tests.rs` | References `AssessmentValue` which does not exist. Broken; can be mistaken for current assessment contract. | **REMOVE** or quarantine from default test set |
| AUD-024 | LEGACY | `tests/week1_tests.rs`, `replay_tests.rs` | Older replay/metric contracts. **NEEDS-EVIDENCE** whether still green. | **NEEDS-EVIDENCE** |
| AUD-025 | PRODUCT | `ingestion/yahoo.rs` `as_f64().unwrap_or(0.0)` | Null Yahoo fields become 0.0 at ingest. ATR path now refuses high/low ≤ 0, but SMA/ROC can still see a zero close. | **REFACTOR** (drop null bars rather than zero-fill) |
| AUD-026 | PRODUCT | `metrics/instrument.rs` ROC `roc > 0` else Negative | Zero momentum is labeled Negative, not Neutral. Semantic default. | **NEEDS-EVIDENCE** (specify in next policy; do not silently change) |
| AUD-027 | PRODUCT | `m4_populate_knowledge_lake.rs` `HashMap` instrument loop | Persist order is hash-order. Content hashes are per-row; dump row order is not a logical identity. | **KEEP** if identity is signature not dump bytes; **REFACTOR** if byte-identical dumps are required |
| AUD-028 | PRODUCT | `replay.rs` consumes `knowledge_decisions` into `input_set_hash` | Lake decisions ≤ T are in the audit set. After CS-P-TEST-001, they are not in `decision_id`. Still a hidden coupling to lake DecisionEngine artifacts. | **REFACTOR** (product replay should not need lake decisions to decide) |
| AUD-029 | PRODUCT | `crate::policy::PolicySnapshot` vs `decision_support::policy::DecisionPolicy` | Two “policy” vocabularies. User risk-policy snapshot is unused on the product path. | **KEEP** snapshot type; **REFACTOR** naming to avoid collision |
| AUD-030 | PRODUCT | Duplicate replay stacks | `validation::replay::ReplayEngine` (observations as-of T) vs `decision_support::replay::ReplayAdapter` (assessments as-of T). Both valid if layered; easy to call the wrong one. | **KEEP** both; **REFACTOR** names (`ObservationReplay` vs `DecisionReplay`) |
| AUD-031 | PRODUCT | Duplicate outcome stacks | `validation::outcome::OutcomeEngine` (strategy target/stop paths, `Uuid::new_v4` metadata) vs `decision_support::outcome` (ledger + lake rows). | **QUARANTINE** lake strategy outcome engine from product docs; KEEP product OutcomeEngine |
| AUD-032 | RESEARCH | `research/laboratory.rs` vs `decision_support/laboratory.rs` | Same word, different jobs. G-GATE lab vs CS-P-004 product lab. | **REFACTOR** names |
| AUD-033 | LEGACY | Personal product modules `evidence`, `hypothesis`, `timeline`, `workspace`, `learning` | PRD co-pilot primitives; not wired to `TradingDecision`. Risk of a second decision loop. | **KEEP** as unused product surface; **NEEDS-EVIDENCE** before any wiring to decide_at |
| AUD-034 | PRODUCT | `decision_support/{backtest,forward,outcome,performance}` | One-way flow holds on the product path. CS-P-TEST-001 covers this. | **KEEP** |
| AUD-035 | PRODUCT | `factor_ecology.rs` uses `TrendMappingPolicy` to label current actions | Descriptive, not optimization. Safe if reports stay measurement-only. | **KEEP** |
| AUD-036 | LEGACY | `tests/fixtures/phase_c_replay/` + `phase_c_gate.sh` | Phase C heritage. Script is dishonest; fixtures may still be useful. | fixtures **NEEDS-EVIDENCE**; script **REMOVE** |
| AUD-037 | RESEARCH | Untracked `m6_phase_g_experiment.rs`, `tests/phase_g_predictive_value_tests.rs` | G-GATE residue (CLN-014). Not in commit `8405d6da5`. | **QUARANTINE** (do not compile into product) |
| AUD-038 | PRODUCT | Forward/replay both call `decide_from_inputs` | Correct reuse. | **KEEP** |
| AUD-039 | PRODUCT | CS-P-TEST-001 identity excludes unused factors / outcomes / `created_at` | Aligns with discipline. | **KEEP** |
| AUD-040 | LEGACY | `ConfidenceDecomposition` all 0.5 on lake Decision | Fabricated confidence on every lake row. | **QUARANTINE** with DecisionEngine |

---

## 5. Feedback arrows (the ones that matter)

| From | To | Present? | Notes |
|------|----|----------|-------|
| Outcome → TradingDecision | No on product path | KEEP | ReplayAdapter does not SELECT outcomes |
| Performance → decide_at | No | KEEP | `measure_performance(&Ledger, &OutcomeReport)` |
| Laboratory → TradingDecision | No | KEEP | CS-P-004 forbids it |
| Factor ecology outcomes → state_key | No | KEEP | Tested |
| StrategyEngine ATR/target → lake outcomes → later G-GATE | Yes, research | QUARANTINE | Historical; do not reuse for Coralys fitness without a new spec |
| `src/research` compiled into lib | Yes | QUARANTINE | Coupling risk if someone calls it from product code |

**No product-path arrow from performance back into decisions was found.** The danger is **social**: `TrendMappingPolicy` remaining the default looks like a learned strategy.

---

## 6. Recommended lanes (no deletion in this step)

```text
PRODUCT — retain / harden
  TradingDecision, DecisionPolicy trait, ReplayAdapter, ForwardAdapter,
  DecisionLedger, OutcomeEngine (decision_support), PerformanceEngine,
  assess_at, factor_status, CS-P-TEST-001

RESEARCH — Coralys boundary (quarantine inside ChronoSentiment)
  src/research/, m5, m6, m4_validation_gate, G-GATE tests
  Future CS-P-006 should live in Coralys, with ChronoSentiment as evaluator only

LEGACY — remove/quarantine after review
  m1–m3 demos, m4_time_machine_demo, phase_c_gate.sh (false PASS),
  week2_tests.rs, commented engines, DecisionEngine/StrategyEngine as product
```

---

## 7. What not to do next

- Do not delete `TrendMappingPolicy` before a replacement policy artifact exists  
- Do not freeze it as v1.0  
- Do not start Coralys discovery until this inventory is accepted and a cleanup pass is scoped  
- Do not run another performance backtest as a substitute for cleanup  
- Do not treat `phase_c_gate.sh` as evidence of anything  

---

## 8. Suggested cleanup authorization (later)

A later pass, if approved, should be **three PRs**, not one:

1. **Quarantine** research/G-GATE/demo binaries from the default product surface (`lib.rs` feature gate or `[[bin]]` move)  
2. **Rename** `TrendMappingPolicy` to an explicit baseline and require policy name on `decide_from_inputs`  
3. **Only then** CS-P-006 Coralys Policy Discovery, with ChronoSentiment as evaluator  

This audit does not authorize those PRs.
