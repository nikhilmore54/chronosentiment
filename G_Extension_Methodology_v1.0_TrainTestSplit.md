# G_Extension_Methodology_v1.0_TrainTestSplit

**Purpose**: Define the chronological train / validation / test windows for the G‑Extension experiment based on the 110 certified strategies and the 60‑day outcome maturity requirement.

---

## 1. Required data fields
| Table | Column | Meaning |
|-------|--------|---------|
| `knowledge_decisions` | `evaluation_timestamp` | The moment a decision (and its associated strategy) was made. |
| `knowledge_outcomes`   | `horizon`                | Horizon label (`5D`, `10D`, `20D`, `60D`). |
| `knowledge_outcomes`   | `horizon_expiry_timestamp` | The timestamp at which the horizon matures (i.e., `evaluation_timestamp + horizon`). |
| `knowledge_outcomes`   | `outcome_return` | Realized return (used only for the primary endpoint). |

## 2. Derivation rules
1. **Identify the latest possible test‑set cut‑off** – it must be **no later than the earliest** `horizon_expiry_timestamp` among *all* outcomes that will belong to the test set. This guarantees that every outcome in the test set is fully matured (i.e., the 60‑day return is known).
2. **Chronological split** – we sort the 110 strategies by `evaluation_timestamp` (ascending). We then allocate contiguous blocks of strategies to:
   - **Training** – earliest strategies, up to `t_train_end`.
   - **Validation** – subsequent strategies, up to `t_val_end`.
   - **Test** – remaining strategies, ending at `t_test_end` (must respect rule 1).
3. **Cluster constraint** – because each strategy yields four horizon outcomes, a strategy may **not be split across folds**. All four outcomes move together.

## 3. Recommended procedure (run on the certified B2 dump)
```sql
-- 1. Extract evaluation timestamps per strategy
SELECT d.id AS decision_id,
       d.evaluation_timestamp,
       o.horizon,
       o.horizon_expiry_timestamp,
       o.outcome_return
FROM public.knowledge_decisions d
JOIN public.knowledge_strategies s ON s.decision_id = d.id
JOIN public.knowledge_outcomes o   ON o.strategy_id = s.id
ORDER BY d.evaluation_timestamp ASC;
```
Save the result as `strategy_outcomes.csv`.

```bash
# Example using psql (PostgreSQL must be installed locally)
export PGHOST=localhost
export PGUSER=$(whoami)
createdb g_extension_tmp
pg_restore --no-owner --dbname=g_extension_tmp /Users/nikhil/ChronoSentiment_MEGA_FINAL/r3_evidence/20260812T180351Z_B2/db/full_dump.dump
psql -d g_extension_tmp -c "\COPY (
SELECT d.id, d.evaluation_timestamp, o.horizon, o.horizon_expiry_timestamp, o.outcome_return
FROM knowledge_decisions d
JOIN knowledge_strategies s ON s.decision_id = d.id
JOIN knowledge_outcomes o   ON o.strategy_id = s.id
ORDER BY d.evaluation_timestamp) TO '/tmp/strategy_outcomes.csv' CSV HEADER;"
```
The CSV will contain **440 rows** (110 strategies × 4 horizons) but you can collapse to a per‑strategy view by taking the *maximum* `horizon_expiry_timestamp` for each `decision_id`.

```python
import pandas as pd
df = pd.read_csv('/tmp/strategy_outcomes.csv')
# Collapse to one row per strategy (decision)
per_strategy = df.groupby('decision_id').agg({
    'evaluation_timestamp': 'first',
    'horizon_expiry_timestamp': 'max'   # the 60‑day expiry
}).reset_index()
per_strategy = per_strategy.sort_values('evaluation_timestamp')

# Choose split indices (example: 60‑% train, 20‑% val, 20‑% test)
N = len(per_strategy)
train_end = int(0.6 * N)
val_end   = int(0.8 * N)
train   = per_strategy.iloc[:train_end]
val     = per_strategy.iloc[train_end:val_end]
test    = per_strategy.iloc[val_end:]

# Verify test‑set maturity
earliest_test_expiry = test['horizon_expiry_timestamp'].min()
print('Earliest test‑set expiry (must be after now):', earliest_test_expiry)
```
Adjust the split percentages if you need a larger validation set or if the earliest test‑set expiry would fall before the current date.

## 4. Freeze the windows
Once the split indices are final, create a small reference table (CSV or markdown) that records the **inclusive timestamp ranges** for each fold:

```markdown
| Fold | Start evaluation_timestamp | End evaluation_timestamp | Number of strategies |
|------|----------------------------|--------------------------|----------------------|
| Train | 2022‑01‑01T00:00:00Z | 2023‑06‑15T23:59:59Z | 66 |
| Validation | 2023‑06‑16T00:00:00Z | 2023‑12‑31T23:59:59Z | 22 |
| Test | 2024‑01‑01T00:00:00Z | 2024‑03‑31T23:59:59Z | 22 |
```
*Replace the example dates with the actual values you obtain from the CSV.*

**Commit this table** to the repository as `G_Extension_TrainTestSplit.md` (or embed it in the methodology file) and mark the methodology as **frozen**.

---

## 5. Outstanding action (for you)
1. Run the SQL extraction steps above on the certified B2 dump.
2. Choose split percentages (or exact target dates) and verify the test‑set maturity condition.
3. Populate the table shown in section 4 with the concrete timestamps.
4. Add the table to the artifact `G_Extension_Methodology_v1.0_Candidate.md` (or create a new companion artifact) and mark the methodology as *frozen*.

When you have completed these steps, we will be ready to move on to the **implementation** phase (model training, evaluation, etc.).

---

*This document is intended to be reviewed and then finalized.*
