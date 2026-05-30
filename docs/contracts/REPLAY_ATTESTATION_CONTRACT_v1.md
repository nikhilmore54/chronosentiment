# Replay Attestation Contract v1

**Version:** 1.0  
**Status:** Frozen (pre-implementation)  
**Last updated:** 2026-05-29  
**Authority:** Replay and evaluation attestation semantics  
**Companions:** `docs/contracts/SCENARIO_DOMAIN_CONTRACT_v1.md`, `docs/contracts/UI_API_CONTRACT_v1.md`

---

## 1. Purpose

ChronoSentiment currently uses the word **certified** in more than one sense:

| Surface | What "certified" often implies | What is actually compared |
|---------|-------------------------------|---------------------------|
| Inspect UI label | Event stream integrity | Metadata heuristics + `replay_signature` |
| `compute_replay_signature` | Replay hash | Session metadata only |
| `certify.rs` | Deterministic re-run | Event sequence equality + digest |
| `ScenarioResult.replay_status` | Domain replay validity | Evaluation completion heuristics |

This document collapses those notions into **one explicit, testable attestation model**.

Per `chronosentiment-core.mdc` and `AUTHORITY_MAP.md`:

> Observational surfaces verify replay law; they do not redefine replay law.

Attestation defines **what digest represents which artifact** and **what comparison proves equivalence**. UI labels, ranking positions, and narrative text are downstream observations — not certification roots.

---

## 2. Attestation Inventory (As-Is Baseline)

Verified against `infrastructure/observatory/api` as of Phase C completion.

| Artifact | Attested today? | Mechanism | Event-grounded? |
|----------|-----------------|-----------|-----------------|
| Single `SimEvent` | Partial | `compute_event_signature` (BLAKE3 per event) | **Yes** |
| `SimEvent[]` stream | Partial | `certify.rs` → `hash_simulation_events` + re-run compare | **Yes** (weak digest — see §6.1 migration note) |
| Inspect `replay_signature` | No | `session_id \|\| strategy_id \|\| sequence_id \|\| cert_state \|\| event_count` | **No** |
| `ScenarioResult` | No | `replay_status` heuristic only | **No** |
| Aggregate / ranking DTO | No | Derived projection | **No** (must remain so) |
| Substrate (`ingest_hash`) | No | Contract field only | **No** |

### 2.1 Critical finding: strongest path today

The strongest replay artifact in the repository is **`certify.rs`**, not the inspect handler.

Inspect may display `CERTIFIED`, but attestation strength depends entirely on **what was hashed and compared**. Until Phase D implementation, treat inspect certification labels as **observational**, not event-attested.

### 2.2 Falsifiable statement (current gap)

```text
FAIL if: inspect replay_signature unchanged while SimEvent[] content changes
```

This test **should fail today** — proving `replay_signature` is metadata-bound, not event-grounded.

---

## 3. Two Attestation Planes (Must Stay Separate)

Event attestation and result attestation are related but **not interchangeable**.

### 3.1 Event Attestation

```text
(strategy, seed, domain, substrate_reference)
        ↓
   Simulation Harness
        ↓
      SimEvent[]
        ↓
    event_hash
```

**Proves:** the execution trace is reproducible.

**Does not prove:** metrics are correct or stable under a different projection layer.

### 3.2 Result Attestation

```text
(strategy, seed, domain, substrate_reference)
        ↓
   Evaluation projection
        ↓
   ScenarioResult
        ↓
    result_hash
```

**Proves:** the scored outcome fields are reproducible for a given evaluation run.

**Does not prove:** the underlying events are identical (unless combined with event attestation).

### 3.3 Failure modes the contract must distinguish

| Failure | Symptom | Detection |
|---------|---------|-----------|
| Event drift | `event_hash` differs on re-run | Event attestation FAIL |
| Metric drift | `result_hash` differs, `event_hash` stable | Result attestation FAIL |
| Projection bug | Same events, different metrics | Both hashes diverge independently |
| Substrate swap | Same strategy/seed, different `substrate_reference` | `substrate_reference` mismatch in record |

---

## 4. Attestation Levels

Internal levels only. **Not required in UI v1.** Avoid binary `CERTIFIED | NOT CERTIFIED` as the sole semantic.

| Level | Name | Requirement |
|-------|------|-------------|
| **0** | `UNATTESTED` | Default. No stable digest recorded. |
| **1** | `EVENT_ATTESTED` | `event_hash` stable across two consecutive runs with identical inputs. |
| **2** | `RESULT_ATTESTED` | Level 1 **and** `result_hash` stable across two consecutive runs. |
| **3** | `DOMAIN_ATTESTED` | Level 2 holds for **one** registered `ScenarioDomain`. |
| **4** | `AGGREGATION_ATTESTED` | All eligible evaluation domains reach Level 3 **and** aggregation output is deterministic from attested `ScenarioResult[]`. |

### 4.1 Level promotion rules

- Levels are **monotonic within a record version**: a record cannot claim Level 2 without Level 1.
- Level 4 requires explicit aggregation reducer identity (e.g. `robust_min_execution_fitness`).
- **`ResultAttested` means reproducibility** — stable event hash and result hash on consecutive runs. It does **not** mean good strategy, robust strategy, profitable strategy, or production-ready strategy.
- **Ranking position is never a level.** Ranking is derivable from Level 4 inputs; it is not attested.

### 4.2 Mapping from current surfaces (transitional)

| Current label / field | Maps to | Notes |
|-----------------------|---------|-------|
| Inspect `certification_state = CERTIFIED` | ≤ Level 0–1 | Heuristic; not equivalent to Level 2+ until wired |
| `certify.rs` PASS | Level 1 candidate | Must adopt canonical `event_hash` (§6.1) |
| `ScenarioResult.replay_status = VALID` | ≤ Level 0 | Completion heuristic until `AttestationRecord` populated |
| `ScenarioAggregator::robust_min` output | Level 4 candidate | Only after all domain records ≥ Level 3 |

---

## 5. `AttestationRecord` (Per Domain Run)

Each materialized `ScenarioResult` for a `(strategy_id, seed, scenario_id)` run SHOULD carry or reference:

| Field | Type | Required | Purpose |
|-------|------|----------|---------|
| `scenario_id` | string | **Yes** | Domain identifier |
| `substrate_reference` | string | **Yes** | From `SubstrateSource.reference` — divergence diagnosis |
| `event_count` | integer | **Yes** | First check when attestation fails |
| `expected_event_hash` | string | **Yes** (Phase D) | BLAKE3 hex digest of canonical event stream |
| `result_hash` | string | **Yes** (Phase D) | BLAKE3 hex digest of canonical result payload |
| `attestation_level` | integer (0–4) | **Yes** | See §4 |
| `attestation_timestamp` | string (RFC 3339 UTC) | **Yes** | When record was generated — not used in verification |
| `engine_mode` | `"IDEAL"` \| `"REAL"` | **Yes** | From domain `ReplayGuarantee` |
| `reducer_id` | string | No | Required when `attestation_level >= 4` |

### 5.1 Diagnostic fields (non-verifying)

`attestation_timestamp`, `event_count`, and `substrate_reference` are **not inputs to hash verification**. They exist so failures answer:

```text
Did event count change?
Did substrate change?
When was this generated?
```

without archaeology.

### 5.2 Relationship to `ScenarioResult`

Phase D extends the observational substrate:

```rust
ScenarioResult {
    // ... existing Phase C fields ...
    attestation: AttestationRecord,
}
```

`replay_status` remains a coarse summary. **`attestation_level` is authoritative** for attestation semantics.

---

## 6. Hash Definitions (Canonical)

All attestation digests use **BLAKE3**, lowercase hex, 64 characters.

Implementation authority: `infrastructure/observatory/api/src/signatures.rs` (extended in Phase D). No other module may define competing attestation hashes.

### 6.1 `event_hash` — canonical event stream

**Input:** ordered `SimEvent[]` from a completed domain evaluation run.

**Canonicalization:**

1. Serialize each event to a deterministic JSON representation (stable field order via `serde` canonical struct order).
2. Concatenate serialized events in `sequence_id` ascending order, separated by `\n`.
3. Prepend header line: `event_stream_v1:{event_count}:{substrate_reference}:{engine_mode}\n`
4. `event_hash = BLAKE3(header + body)`

**Verification procedure:**

1. Re-run `(strategy, seed, domain)` with identical substrate reference.
2. Recompute `event_hash`.
3. PASS if equal to stored `expected_event_hash`.

**Migration note:** `certify.rs` currently uses a non-BLAKE3 demonstration digest. Phase D implementation MUST replace it with this definition. Until then, `certify.rs` PASS is **Level 1 candidate**, not Level 1 certified under this contract.

### 6.2 `result_hash` — canonical scenario result

**Input:** scored fields from one domain evaluation (excluding attestation metadata).

**Canonical JSON payload (keys sorted lexicographically at serialization):**

```json
{
  "avg_pnl": "<f64>",
  "domain_class": "<DomainClass>",
  "engine_mode": "<IDEAL|REAL>",
  "execution_fitness": "<f64>",
  "fitness": "<f64>",
  "max_drawdown": "<f64>",
  "scenario_id": "<string>",
  "std_dev": "<f64>",
  "trade_count": "<integer>"
}
```

**Digest:**

```text
result_hash = BLAKE3("result_v1:" + canonical_json)
```

**Verification:** re-run evaluation; recompute; compare.

### 6.3 `aggregate_hash` — deterministic aggregation (Level 4)

**Input:** ordered list of `(scenario_id, result_hash)` pairs for all eligible domains, sorted by `scenario_id` ascending, plus reducer id.

```text
aggregate_input = "aggregate_v1:{reducer_id}:" + join(scenario_id + ":" + result_hash, ",")
aggregate_hash  = BLAKE3(aggregate_input)
```

**Forbidden:** hashing ranking position, strategy comparison prose, or UI DTO envelopes.

### 6.4 `kernel_signature` vs `event_hash`

| Digest | Scope | Use |
|--------|-------|-----|
| `kernel_signature` | Single canonical event | Inspect event integrity display |
| `event_hash` | Full stream for one domain run | Replay attestation root |

Do not substitute per-event signatures for stream attestation.

### 6.5 `replay_signature` (legacy metadata digest)

**Current definition** (unchanged for UI compatibility):

```text
session_id : strategy_id : requested_sequence_id : certification_state : event_count
→ BLAKE3
```

**Contract classification:** `METADATA_SIGNATURE` — **not** an event attestation digest.

Phase D MUST NOT relabel this field as `event_hash`. If inspect needs event-grounded certification, populate `AttestationRecord.expected_event_hash` separately.

---

## 7. Domain Certification Semantics

**"Replay-valid across domains"** means:

For each eligible `ScenarioDomain` in `ScenarioRegistry`:

1. `attestation_level >= 3` for `(strategy_id, seed, scenario_id)`.
2. `substrate_reference` matches the registered domain declaration.
3. `event_count > 0` unless domain explicitly allows empty chronology (then `replay_status = INVALID`).

It does **not** mean:

- reproducible ranking order across candidate sets
- identical fitness to a prior API version
- narrative equivalence

### 7.1 Per-domain vs cross-domain

| Scope | Attestation target |
|-------|-------------------|
| Single domain | `AttestationRecord` on one `ScenarioResult` |
| Cross-domain standing | Level 4 on aggregated materialization |
| Ranking endpoint | **Unattested projection** unless trace links to Level 4 record set |

---

## 8. What Is Signed

Phase D v1: **nothing**.

Hash equality under declared canonicalization is sufficient for determinism proof within the trusted engine boundary.

External signing (release artifacts, third-party verification) is out of scope for v1.

---

## 9. Ranking Boundary (Hard Rule)

```text
Ranking is an observational projection — not a certification root.
```

| Allowed | Forbidden |
|---------|-----------|
| Derive ranking from attested `ScenarioResult[]` | Store `ranking_position_hash` |
| Expose `aggregate_hash` in debug/internal surfaces | Certify "strategy A beats strategy B" |
| Trace ranking row → `strategy_id` → `last_scenario_results` | Treat compare prose as attestation |

---

## 10. Falsifiable Tests (Phase D Acceptance)

| ID | Test | Expected |
|----|------|----------|
| T-01 | Re-run same `(strategy, seed, domain)` twice | `event_hash` identical |
| T-02 | Mutate one `SimEvent` in stored stream | `event_hash` differs |
| T-03 | Same events, corrupt metric field in evaluator | `result_hash` differs, `event_hash` stable |
| T-04 | Change `substrate_reference` only | New record; prior attestation not valid for new substrate |
| T-05 | Remove one domain from registry | Aggregate changes; trace shows removed domain |
| T-06 | `replay_signature` stable after event mutation | **FAIL** (proves metadata-only digest) until inspect wired to `event_hash` |
| T-07 | Ranking changes without any `result_hash` change | **FAIL** — ranking must be pure projection |
| T-08 | Claim Level 2 with divergent `event_hash` | **FAIL** — level monotonicity |

---

## 11. Implementation Checklist (Phase D)

| Step | Deliverable | Status |
|------|-------------|--------|
| 1 | This document (`REPLAY_ATTESTATION_CONTRACT_v1.md`) | Done |
| 2 | `AttestationRecord` type + extend `ScenarioResult` | Done |
| 3 | Canonical `event_hash` in `signatures.rs`; migrate `certify.rs` | Partial (`event_hash` done; `certify.rs` migration pending) |
| 4 | Canonical `result_hash`; compute on domain evaluation | Done |
| 5 | Wire attestation into `evaluate_strategy_across_domains` | Done |
| 6 | Level 4 `aggregate_hash` in `ScenarioAggregator` | Done |
| 7 | Inspect: expose `expected_event_hash` without breaking UI contract | Pending |
| 8 | Runtime tests: T-01 through T-08 (Phase D acceptance) | Pending |
| 9 | Fixture: `fixtures/contracts/attestation_record.example.json` | Done |
| 10 | Contract tests: structural invariants (`tests/test_replay_attestation_contract.py`) | Done |

### Phase D success criterion

Phase D succeeds when:

1. Every eligible domain evaluation produces an `AttestationRecord` with stable `event_hash` and `result_hash` on re-run.
2. No consumer confuses `replay_signature` with `event_hash`.
3. Rankings remain derivable from attested artifacts without reverse inference.

Phase D does **not** succeed when:

- UI displays more "CERTIFIED" badges
- Rankings look more realistic

---

## 12. Relationship to Other Contracts

```text
SCENARIO_DOMAIN_CONTRACT_v1  ← declares domains + ScenarioResult substrate
REPLAY_ATTESTATION_CONTRACT_v1 ← declares digests + levels + verification
UI_API_CONTRACT_v1           ← unchanged envelopes; attestation is internal/debug
```

```text
Substrate → Domain Run → SimEvent[] → event_hash     (Event Attestation)
                      → ScenarioResult → result_hash (Result Attestation)
                      → Aggregator       → aggregate_hash (Level 4 only)
                      → Ranking DTO      → projection (unattested)
```

---

## 13. Change Control

Changes to hash canonicalization, level definitions, or required attestation fields require:

1. New contract version (`REPLAY_ATTESTATION_CONTRACT_v2.md`)
2. Migration note for existing `AttestationRecord` values
3. `AUTHORITY_MAP.md` update if attestation authority surfaces change

Adding diagnostic fields to `AttestationRecord` is backward-compatible within v1 if they are non-verifying.

---

## 14. Repository Status (Post Phase C, Pre Phase D)

```text
Contract-driven
Replay-capable
Scenario-aware
Traceable
Partially attested   ← ScenarioResult binding live for execution domains
```

The missing piece is not architecture. It is **what artifact is authoritative, what digest represents it, and what comparison proves equivalence** — defined above, byte-for-byte, testable, and falsifiable.
