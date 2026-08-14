# G_Extension_Methodology_v1.1_TrainTestSplit

**Status:** FROZEN for G-GATE execution  
**Supersedes:** `G_Extension_Methodology_v1.0_TrainTestSplit.md` (historical; not executable)  
**Parent:** `G_Extension_Methodology_v1.1.md`  
**Dataset:** B3 only. **Do not use B2.**

v1.0 is preserved unchanged. Its example 66/22/22 table, B2 restore commands, and “choose split percentages” actions are **not** part of this freeze.

---

## 1. Required data fields

| Table | Column | Role |
|-------|--------|------|
| `knowledge_strategies` | `id` | Strategy identity; tie-break key |
| `knowledge_strategies` | `decision_id` | Link to decision |
| `knowledge_decisions` | `evaluation_timestamp` | Chronological rank key |
| `knowledge_decisions` | `assessment_id` | Link to decision-time assessment |
| `knowledge_outcomes` | `strategy_id` | Cluster constraint |
| `knowledge_outcomes` | `horizon` | `5D` / `10D` / `20D` / `60D` |
| `knowledge_outcomes` | `horizon_expiry_timestamp` | Maturity check only |
| `knowledge_outcomes` | `outcome_return` | Primary endpoint field (see parent §3) |

---

## 2. Dataset

Read-only B3:

```
r3_evidence/20260813T035739Z_B3/db/full_dump.dump
SHA-256: af11d318b03fb171207f96348fcf210e1b9149b1ab6e699c06c363faec518788
```

Canonical database name used to populate B3: `chrono_b3_test`.

A G-GATE run may restore the dump into a **separate** read-only working database. It must not write to the dump, to B3 provenance, or to `chrono_b3_test` schema/data.

---

## 3. Chronological ranking (frozen)

Sort the 110 strategies by:

1. `knowledge_decisions.evaluation_timestamp` ascending
2. `knowledge_strategies.id` ascending (UUID text / native UUID order as stored by PostgreSQL)

**Explicit v1.1 decision:** timestamp ties are resolved by `strategy_id`. Timestamp ranges alone are not a unique fold key (TRAIN and VALIDATION share `2023-07-31T15:30:00Z`).

```sql
WITH strat AS (
  SELECT s.id AS strategy_id,
         d.id AS decision_id,
         d.evaluation_timestamp
  FROM knowledge_strategies s
  JOIN knowledge_decisions d ON s.decision_id = d.id
),
ordered AS (
  SELECT strategy_id, decision_id, evaluation_timestamp,
         ROW_NUMBER() OVER (
           ORDER BY evaluation_timestamp ASC, strategy_id ASC
         ) AS rn
  FROM strat
)
SELECT
  strategy_id,
  decision_id,
  evaluation_timestamp,
  CASE
    WHEN rn <= 55 THEN 'TRAIN'
    WHEN rn <= 82 THEN 'VALIDATION'
    ELSE 'TEST'
  END AS fold
FROM ordered
ORDER BY rn;
```

Cluster constraint: all four outcomes of a `strategy_id` inherit that fold.

---

## 4. Frozen fold sizes

**Explicit v1.1 decision**, taken from the G-GATE execution brief (not from the v1.0 60/20/20 example):

| Fold | Ranks (inclusive) | Strategies | Outcomes |
|------|-------------------|------------|----------|
| TRAIN | 1–55 | 55 | 220 |
| VALIDATION | 56–82 | 27 | 108 |
| TEST | 83–110 | 28 | 112 |
| Total | 1–110 | 110 | 440 |

`55 + 27 + 28 = 110`.

---

## 5. Frozen inclusive evaluation-timestamp bounds (UTC)

Observed on B3 via the ranking in §3. Bounds are **descriptive**. Membership is defined by rank, not by these timestamps.

| Fold | Start evaluation_timestamp | End evaluation_timestamp | N strategies | Earliest 60D expiry | Latest 60D expiry |
|------|----------------------------|--------------------------|--------------|---------------------|-------------------|
| TRAIN | 2021-10-31T15:30:00Z | 2023-07-31T15:30:00Z | 55 | 2021-12-30T15:30:00Z | 2023-09-29T15:30:00Z |
| VALIDATION | 2023-07-31T15:30:00Z | 2024-03-31T15:30:00Z | 27 | 2023-09-29T15:30:00Z | 2024-05-30T15:30:00Z |
| TEST | 2024-04-30T15:30:00Z | 2024-12-31T15:30:00Z | 28 | 2024-06-29T15:30:00Z | 2025-03-01T15:30:00Z |

### Boundary strategy identities (must match)

| Rank | Fold | evaluation_timestamp | strategy_id | decision_id |
|------|------|----------------------|-------------|-------------|
| 1 | TRAIN | 2021-10-31T15:30:00Z | `04921177-db49-4144-a9ab-d940146e8002` | `db0f370c-5a9b-433d-a093-2fe6763501bc` |
| 55 | TRAIN | 2023-07-31T15:30:00Z | `59660824-e669-4f47-9ef3-7cc48d46d162` | `93609122-4ad2-4da1-9c61-5748f823fcdc` |
| 56 | VALIDATION | 2023-07-31T15:30:00Z | `8a413318-a5fe-4aac-ac0e-a90f560ec206` | `3b3a3d5b-9797-46d0-b47c-e455677ccbe8` |
| 82 | VALIDATION | 2024-03-31T15:30:00Z | `e82c0102-bf6a-4628-b22f-335da95c5532` | `be35a9dc-e09b-4273-be2e-d4ed85cd14d8` |
| 83 | TEST | 2024-04-30T15:30:00Z | `4a2a7b11-eb67-4394-af79-888ccf99f2e6` | `95d67b0c-9ec2-4ea2-bbe1-beaf29882019` |
| 110 | TEST | 2024-12-31T15:30:00Z | `a56feca3-5ae0-4c47-b4e0-2d7b298b3874` | `dc0478aa-6307-4820-815d-ffada7496e56` |

If a G-GATE run cannot reproduce these six boundary identities on B3, the frozen split cannot be reproduced → **STOP / INCONCLUSIVE**. Do not re-choose ranks.

---

## 6. Maturity condition

v1.0 required every test outcome to be fully matured (60-day expiry known).

On B3, the earliest TEST `horizon_expiry_timestamp` is `2024-06-29T15:30:00Z` and the latest is `2025-03-01T15:30:00Z`. Relative to B3 certification date `2026-08-13`, all TEST outcomes are matured.

No test strategy is dropped for immaturity. No rank is shifted.

---

## 7. Documented calendar overlap (not a re-split)

These overlaps are frozen facts for the leakage audit. They do **not** authorize a new split.

- TRAIN and VALIDATION share evaluation timestamp `2023-07-31T15:30:00Z` (ranks 55 vs 56 distinguished only by `strategy_id`).
- Some TRAIN 60D expiries (`…` through `2023-09-29T15:30:00Z`) fall after the first VALIDATION evaluation timestamps.
- Some VALIDATION 60D expiries (`…` through `2024-05-30T15:30:00Z`) fall after the first TEST evaluation timestamps (`2024-04-30T15:30:00Z`).
- TRAIN latest 60D expiry `2023-09-29T15:30:00Z` is before TEST start `2024-04-30T15:30:00Z` (no TRAIN-label / TEST-feature calendar collision).

Labels in TRAIN/VALIDATION may use prices that occur during a later fold’s calendar. Features for a decision remain cutoff at that decision’s `evaluation_timestamp` (parent §4).

---

## 8. What v1.1 does not inherit from v1.0

- Do not restore or query `r3_evidence/20260812T180351Z_B2/`.
- Do not use the example 60%/20%/20% (`66/22/22`) allocation.
- Do not use the example calendar table (`2022-01-01` … `2024-03-31`, 66/22/22).
- Do not “adjust split percentages” after inspecting outcomes.
- Do not consult `G_Extension_Methodology_v1.0_Candidate.md` or split-audit drafts as protocol.

---

## 9. Status

**FROZEN — v1.1**

Fold membership is the rank rule in §3–§4, verified by the boundary identities in §5.
