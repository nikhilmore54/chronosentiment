# Temporal Leakage Root Cause

**Status:** Closed diagnostic (read-only).  
**G-GATE v1.1:** INCONCLUSIVE / LEAKAGE FAIL — not rerun.  
**B3 / v1.0 / v1.1 / G-GATE witness:** not modified.

Decision: **REPAIR** (Case A). Do not regenerate B4 in this document. Do not rerun G-GATE.

---

## 1. Observed defect

G-GATE v1.1 leakage check 1 failed:

> Feature timestamps `<=` decision `evaluation_timestamp`

On the B3 restore, **every** strategy-linked assessment has `knowledge_assessments.evaluation_timestamp` **after** the linked `knowledge_decisions.evaluation_timestamp`.

Observed lag: **589–1746 days**.

---

## 2. Exact affected population

Read-only query on working restore `chrono_g_gate_b3_readonly` (B3 dump, unmodified):

| Population | N | `assess_eval > decision_eval` | `assess_eval <= decision_eval` |
|------------|---|-------------------------------|--------------------------------|
| Decisions with `assessment_id` | 195 | 195 | 0 |
| Strategy-linked subset (G-GATE) | 110 | 110 | 0 |

G-GATE evaluated the 110 strategies. The same inversion holds for all 195 assessment–decision pairs.

---

## 3. Timestamp fields involved

G-GATE join (v1.1 §2) is:

```sql
knowledge_strategies s
JOIN knowledge_decisions d ON s.decision_id = d.id
JOIN knowledge_assessments a ON d.assessment_id = a.id
```

Compared fields:

| Artifact | Column | Observed values |
|----------|--------|-----------------|
| Assessment | `evaluation_timestamp` | 2026-08-13 09:03:28–31 IST only |
| Assessment | `recorded_at` | 2026-08-13 09:03:28–31 IST (persist time) |
| Decision | `evaluation_timestamp` | 2021-10-31 … 2024-12-31 15:30 UTC (replay grid) |
| Decision | `recorded_at` | 2026-08-13 (persist time) |
| Outcome | `evaluation_timestamp` | same as decision replay `dt` |

The audit did **not** join the wrong tables or the wrong IDs. It compared the two `evaluation_timestamp` columns that v1.1 names.

Schema already distinguishes `evaluation_timestamp` from `recorded_at`. Assessment `evaluation_timestamp` is not a separate “convention” field; it is the as-of column, filled with persist-time.

Example (rank-1 TRAIN strategy, 5D):

| Field | Value |
|-------|--------|
| `strategy_id` | `04921177-db49-4144-a9ab-d940146e8002` |
| assessment `evaluation_timestamp` | 2026-08-13 09:03:28.934797+05:30 |
| assessment `recorded_at` | 2026-08-13 09:03:28.935807+05:30 |
| decision `evaluation_timestamp` | 2021-10-31 21:00:00+05:30 |
| outcome `evaluation_timestamp` | 2021-10-31 21:00:00+05:30 |

---

## 4. Source-code generation path

B3 was produced by `m4_populate_knowledge_lake`.

Replay as-of time `dt` is a month-end 15:30 UTC grid, 2021–2024:

```180:189:adapters/chronosentiment/src/bin/m4_populate_knowledge_lake.rs
    for dt in &timestamps {
        for (inst_id, _inst) in &instrument_map {
            let req = ReplayRequest {
                research_session_id: "val_gate".to_string(),
                universe: "Nifty50".to_string(),
                evaluation_timestamp: *dt,
```

Observations admitted into the context are cut off at `dt`. The populator panics if any observation has `effective_from > dt`.

Assessment persistence:

```213:218:adapters/chronosentiment/src/bin/m4_populate_knowledge_lake.rs
            let profile = AssessmentEngine.assess(&metric_report, &[Concept::Trend, Concept::Momentum, Concept::Volatility]);
            let _evidence = EvidenceEngine.evaluate(&profile);
            
            knowledge_repo.store(&profile).await?;

            let decision = decision_engine.evaluate(&profile, *dt, *inst_id);
```

`AssessmentEngine.assess` does **not** receive `dt`. It stamps mock metadata:

```109:111:adapters/chronosentiment/src/reasoning/assessment.rs
    pub fn assess(&self, metrics: &MetricReport, active_concepts: &[Concept]) -> AssessmentProfile {
        self.assess_with_metadata(metrics, active_concepts, ArtifactMetadata::mock(), None)
    }
```

```45:51:adapters/chronosentiment/src/repository/knowledge.rs
    pub fn mock() -> Self {
        Self {
            artifact_id: Uuid::new_v4(),
            // ...
            created_at: Utc::now(),
            evaluation_timestamp: Utc::now(),
```

`PostgresKnowledgeRepository` stores `meta.evaluation_timestamp` into `knowledge_assessments.evaluation_timestamp`.

Decision path **overwrites** the mock clock with the replay date:

```96:104:adapters/chronosentiment/src/reasoning/decision.rs
        let mut metadata = crate::repository::knowledge::ArtifactMetadata::mock();
        metadata.artifact_type = crate::repository::knowledge::ArtifactType::Decision;
        metadata.evaluation_timestamp = eval_dt;
        // ...
            evaluation_timestamp: eval_dt,
```

Strategy copies `decision.evaluation_timestamp`. Outcome `measure_outcome` is called with `*dt`.

That is why decisions/outcomes sit on 2021–2024 and assessments sit on 2026-08-13 (B3 populate wall clock). The 589–1746 day gap is `populate_now − replay_dt`.

---

## 5. Expected temporal relationship

v1.1 and the schema intend:

```
assessment.evaluation_timestamp  <=  decision.evaluation_timestamp  =  replay dt
```

Feature construction is required to use only observations with `effective_from <= dt`.

`recorded_at` may be populate wall-clock. `evaluation_timestamp` must not.

---

## 6. Actual relationship

```
assessment.evaluation_timestamp  ≈  assessment.recorded_at  ≈  2026-08-13 populate time
decision.evaluation_timestamp    =  replay dt ∈ [2021-10-31, 2024-12-31]
outcome.evaluation_timestamp     =  replay dt
```

So:

```
assessment.evaluation_timestamp  >>  decision.evaluation_timestamp
```

for 195/195 linked pairs.

---

## 7. Root cause

**Population-generator metadata stamp, not a G-GATE join error.**

1. `AssessmentEngine.assess()` uses `ArtifactMetadata::mock()` → `evaluation_timestamp = Utc::now()`.
2. `m4_populate_knowledge_lake` never calls `assess_with_metadata(..., dt, ...)`.
3. `DecisionEngine.evaluate` and `StrategyEngine.generate` reset `evaluation_timestamp` to replay `dt`; assessments do not.
4. G-GATE compared the columns v1.1 specifies. Those columns are the right fields with the wrong assessment values.

**Feature construction vs metadata:** replay still filters observations to `effective_from <= dt`. The populator panics on a leak of raw bars past `dt`. `signature_hash` is computed from that as-of metric report. The **content** of the assessment was produced from historically admissible observations. The **stored as-of timestamp** on the assessment row is persist-time, so it cannot certify that fact.

E-GATE v2 did not compare `knowledge_assessments.evaluation_timestamp` to `knowledge_decisions.evaluation_timestamp`. Lineage counts and strategy→outcome matching can PASS while this metadata invariant fails.

---

## 8. Impact on B3 / E-GATE / G-GATE

| Item | Impact |
|------|--------|
| G-GATE v1.1 | Execution complete. Leakage FAIL is correct under the frozen rule. Classification **INCONCLUSIVE**. No predictive-value claim. |
| G-GATE audit implementation | Not a wrong join. Do not “fix the audit” to ignore assessment timestamps. That would change v1.1 after the result. |
| B3 | Frozen and unmodified. **Not usable for leakage-sensitive research that treats `knowledge_assessments.evaluation_timestamp` as as-of time.** |
| E-GATE v2 | Remains PASS for the checks it actually ran. It did not certify this temporal metadata invariant. That is a coverage gap, not a reason to rewrite E-GATE here. |
| v1.0 / v1.1 | Unchanged. |

---

## 9. Corrective action

**Not in this step.** Required later, as a dataset repair, not a G-GATE rerun:

1. Stamp assessments with the replay `dt` (e.g. `assess_with_metadata` / set `metadata.evaluation_timestamp = dt` before store).
2. Keep `recorded_at` as persist time.
3. Generate a new snapshot (**B4**), do not mutate B3.
4. Extend E-GATE to require `assessment.evaluation_timestamp <= decision.evaluation_timestamp` for every linked pair.
5. Only if that gate PASSes: new G-GATE against the same v1.1 protocol.

Do **not**: change v1.1, `Y_h`, 55/27/28, the candidate, the bootstrap, or reinterpret G-GATE as `NOT_DETECTED`.

---

## 10. Decision

**REPAIR**

Case A: the stored assessment timestamps genuinely occur after the decisions. The audit used the specified fields. B3 cannot support leakage-sensitive G-Extension research until the populator stamps assessment as-of time correctly and a new snapshot is certified.

Case B is rejected: the join is `d.assessment_id = a.id`; the compared columns are both `evaluation_timestamp`.

Case C is rejected: mock `Utc::now()` vs replay `dt` is sufficient explanation.

**STOP further G-GATE experimentation on B3.**
