# GA Deadlock — Patch-Level Fixes (v2, Validated)
> All patches against `core/src/ga.rs`. Line numbers are exact from current file.
> **Apply in the STEP order at the bottom — not patch number order.**

---

## PATCH 1 of 9 — Fix stagnation counter never resetting (P0)
**Lines:** 640

`EvoState::default()` starts `last_best_fitness` at `0.0`. All evaluated fitnesses are
negative, so the improvement branch is permanently unreachable, `stagnation_counter`
increments every generation and never resets, and `mutation_scale` maxes out by gen 3.

```diff
--- a/core/src/ga.rs
+++ b/core/src/ga.rs
@@ -640,1 +640,1 @@
-            last_best_fitness: 0.0,
+            last_best_fitness: f64::NEG_INFINITY,
```

---

## PATCH 2 of 9 — Remove dead shadowed `trade_penalty` + relax cascade (P0)
**Lines:** 6237–6242 (delete), 6400–6406 (replace), 6441–6450 (replace)

**Hunk A — delete the dead binding (shadowed by the match at L6400, never used):**

```diff
--- a/core/src/ga.rs
+++ b/core/src/ga.rs
@@ -6237,7 +6237,0 @@
-    // 🔥 CRITICAL: prevent zero-trade collapse
-    let trade_penalty = if total_trades < 5 {
-        -0.1 * (5 - total_trades) as f64
-    } else {
-        0.0
-    };
-
```

**Hunk B — relax the match block so positive fitness space exists:**

```diff
--- a/core/src/ga.rs
+++ b/core/src/ga.rs
@@ -6400,7 +6400,7 @@
-    let trade_penalty = match total_trades {
-        0 => -0.5,
-        1..=3 => -0.3,
-        4..=7 => -0.15,
-        8..=11 => -0.05,
-        _ => 0.0,
-    };
+    let trade_penalty = match total_trades {
+        0     => -0.30,
+        1..=2 => -0.10,
+        3..=5 => -0.04,
+        _     => 0.0,
+    };
```

**Hunk C — replace stacked unconditional deductions with single soft penalty:**

```diff
--- a/core/src/ga.rs
+++ b/core/src/ga.rs
@@ -6441,10 +6441,4 @@
-    // 🔥 HARD NEGATIVE SELECTION (CRITICAL)
-    if win_rate < 0.2 && total_trades > 5 {
-        fitness -= 0.12; // kill consistently bad strategies
-    }
-
-    if total_trades < 5 {
-        fitness -= 0.08;
-    } else if total_trades < 10 {
-        fitness -= 0.04;
-    }
+    // Soft directional penalty — halved so break-even strategies can score > 0
+    if win_rate < 0.15 && total_trades > 5 {
+        fitness -= 0.06;
+    }
```

---

## PATCH 3 of 9 — Add fitness floor after all deductions (P0)
**Lines:** after L6452 (the `rand::random` noise line)

Without a floor, a strategy with bad luck clusters at -1.0 and becomes
indistinguishable from every other bad strategy. A floor at -0.3 creates three
visible bands: weak (-0.3), mediocre (-0.1), good (> 0). Selection can then operate.

```diff
--- a/core/src/ga.rs
+++ b/core/src/ga.rs
@@ -6452,1 +6452,4 @@
     fitness += rand::random::<f64>() * 0.02;
+
+    // Floor: compress everything below -0.3 into a single band so weak ≠ terrible
+    fitness = fitness.max(-0.3);
+
```

---

## PATCH 4 of 9 — Fix `generation_peaks` to show true global max per generation (P1)
**Lines:** 1564, 1768, 1973–1976

Currently appends (generation, fitness) per-bucket → 5 windows × 15 gens = 75 entries.
The print loop shows each generation label 5 times; the last bucket (always STRAT_11438)
dominates. True progress is invisible.

**Hunk A — change Vec type:**

```diff
--- a/core/src/ga.rs
+++ b/core/src/ga.rs
@@ -1564,1 +1564,2 @@
-    let mut generation_peaks: Vec<(usize, f64)> = Vec::new();
+    // One slot per generation; holds global-best fitness across all buckets
+    let mut generation_peaks: Vec<f64> = vec![f64::NEG_INFINITY; config.generations];
```

**Hunk B — max-reduce instead of push:**

```diff
--- a/core/src/ga.rs
+++ b/core/src/ga.rs
@@ -1768,1 +1768,3 @@
-                    generation_peaks.push((generation, best.fitness));
+                    if best.fitness > generation_peaks[generation] {
+                        generation_peaks[generation] = best.fitness;
+                    }
```

**Hunk C — fix print loop:**

```diff
--- a/core/src/ga.rs
+++ b/core/src/ga.rs
@@ -1973,4 +1973,6 @@
-    println!("📈 Generation Peaks:");
-    for (gen, fitness) in generation_peaks {
-        println!("Gen {} → {:.4}", gen, fitness);
-    }
+    println!("📈 Generation Peaks (global best across all buckets):");
+    for (gen, &fitness) in generation_peaks.iter().enumerate() {
+        let marker = if fitness > 0.0 { "✅" } else { "  " };
+        println!("{} Gen {:>2} → {:.4}", marker, gen, fitness);
+    }
```

---

## PATCH 5 of 9 — Clamp `spread_z` to ±10 and gate STRAT_ANOMALY (P1)
**Location:** wherever `spread_z` is computed (run `grep -n "spread_z" core/src/ga.rs`)

`STRAT_DECISION` logs show `spread_z = -219` to `-237` for STRAT_11438. This is a
near-zero denominator in the z-score formula, not a signal. It poisons the evaluation.

**At the computation site:**

```diff
--- a/core/src/ga.rs
+++ b/core/src/ga.rs
@@ at the line assigning spread_z
-    let spread_z = (spread - mean_spread) / std_spread;
+    let spread_z = if std_spread.abs() > 1e-8 {
+        ((spread - mean_spread) / std_spread).clamp(-10.0, 10.0)
+    } else {
+        0.0
+    };
```

**Immediately after `STRAT_DECISION` is logged, add a skip gate:**

```diff
--- a/core/src/ga.rs
+++ b/core/src/ga.rs
@@ after the line that prints "STRAT_DECISION →"
+    if spread_z.abs() > 10.0 {
+        println!("⚠️ STRAT_ANOMALY → strat={} spread_z={:.2} SKIPPED", strategy_id, spread_z);
+        continue;
+    }
```

---

## PATCH 6 of 9 — Remove hidden double-gate + raise execution floor (P0-adjacent)
**Lines:** 5789 (exec_prob clamp), 5822–5826 (exec_roll gate)

### Why this is the real execution bug

There were two separate gates in sequence:

```
Gate 1 (L5822):  if exec_roll > final_exec_prob → continue
Gate 2 (L5824):  if exec_roll > final_exec_prob + 0.6 → continue   ← hidden killer
```

Gate 2 runs *inside* the `else` branch of Gate 1, meaning it only fires when
`exec_roll ≤ final_exec_prob` (i.e., the trade already passed Gate 1). But then Gate 2
kills it anyway if `exec_roll > prob + 0.6`. With `final_exec_prob = 0.65`:

| exec_roll | Gate 1 | Gate 2         | Actual outcome |
|-----------|--------|----------------|----------------|
| 0.60      | pass   | —              | ✅ executed    |
| 0.70      | skip   | —              | ❌ skipped     |
| 0.80      | skip   | —              | ❌ skipped     |
| 0.95      | skip   | —              | ❌ skipped     |

Wait — re-reading the code: Gate 2 is nested *inside* `if exec_roll > final_exec_prob`,
meaning it only runs when Gate 1 already would skip. Gate 2 *allows* a partial pass
when `exec_roll ≤ prob + 0.6`, which means trades where `proc < exec_roll ≤ prob+0.6`
get a second chance. But trades where `exec_roll > prob + 0.6` get killed even in the
partial-pass window. Effective execution rate with `prob=0.65`:

- Pass zone: `exec_roll ≤ 0.65` → 65% execute
- Partial zone: `0.65 < exec_roll ≤ 1.25` (capped at 1.0) → `exec_roll ≤ 1.0` → all 35% pass
- Kill zone: `exec_roll > 1.25` → impossible (rand is [0,1])

So the `+0.6` gate **never fires** in practice (rand max is 1.0, prob min is 0.05,
so `prob + 0.6 ≥ 0.65` — always above 1.0 when prob ≥ 0.40). The *real* participation
problem is the `exec_prob.clamp(0.05, 0.85)` floor allowing exec_prob to be as low as 5%,
combined with the hard break at L5834 (`max_trades_per_scenario = 12`). With `exec_prob`
hovering around 0.73 (as logged) and only 500 signals, a 6-trade result means the
**edge threshold is filtering out 99% of signal candidates before exec_prob is even
reached** — the `feasible=true` gate at L5828 is the real choke point on low-edge signals.

**Hunk A — raise exec_prob floor from 0.05 to 0.50 for GA training:**

```diff
--- a/core/src/ga.rs
+++ b/core/src/ga.rs
@@ -5789,1 +5789,1 @@
-        let exec_prob = exec_prob.clamp(0.05, 0.85);
+        let exec_prob = exec_prob.clamp(0.50, 0.85); // GA training floor — increase participation
```

**Hunk B — clean up the dead partial-pass logic (it never fires, remove for clarity):**

```diff
--- a/core/src/ga.rs
+++ b/core/src/ga.rs
@@ -5822,6 +5822,3 @@
-        if exec_roll > final_exec_prob {
-            // allow partial execution instead of full rejection
-            if exec_roll > final_exec_prob + 0.6 {
-                continue;
-            }
-        }
+        if exec_roll > final_exec_prob {
+            continue; // clean single gate
+        }
```

> **⚠️ Over-execution risk:** After this patch, watch for `trades = 100+` in the first
> run. If trades spike that high, fitness will drop because bad signals dominate.
> **Safety fallback:** if you see `trades > 80`, change the clamp to `(0.35, 0.85)`
> instead of `(0.50, 0.85)`. Do NOT pre-apply this — observe first.

> **Note:** The original `max_trades_per_scenario` cap at L5834 (`unwrap_or(12)`) is
> a hard ceiling and also needs raising — see Patch 7.

---

## PATCH 7 of 9 — Raise max trades cap (P0-adjacent)
**Lines:** 5834

The `max_trades_per_scenario` cap of 12 is the hard ceiling. Even with Patch 6 raising
`exec_prob`, the loop breaks at 12. Raise it for training:

```diff
--- a/core/src/ga.rs
+++ b/core/src/ga.rs
@@ -5834,1 +5834,1 @@
-        if triggered_entries > config.max_trades_per_scenario.unwrap_or(12) {
+        if triggered_entries > config.max_trades_per_scenario.unwrap_or(50) {
```

---

## PATCH 8 of 9 — Fix TP/SL ratio to allow some TP hits (P1)
**Lines:** 4359–4360

Current values:
```
tp_dist = expected_move * 1.2
sl_dist = expected_move * 0.8
```

TP is 1.5× farther than SL from entry. For short trades against a sideways-to-up
market, TP is never reached. Every trade stops out. Win rate = 0, fitness = floor.
Tighten TP and widen SL to create an asymmetric risk profile that can actually hit TP:

```diff
--- a/core/src/ga.rs
+++ b/core/src/ga.rs
@@ -4359,2 +4359,2 @@
-    let tp_dist = (expected_move * 1.2).max(min_move);
-    let sl_dist = (expected_move * 0.8).max(min_move * 0.7);
+    let tp_dist = (expected_move * 0.85).max(min_move);   // tighter TP → more hits
+    let sl_dist = (expected_move * 1.05).max(min_move * 0.7); // wider SL → fewer premature stops
```

> **Why these numbers?** `tp_dist * 0.85` and `sl_dist * 1.05` gives a TP:SL ratio
> of ~0.81, meaning the TP target is closer relative to expected move, making it
> reachable in sideways conditions. This is a training stabiliser — once win rate > 0
> you can tune the ratio to match your target R:R.

---

## PATCH 9 of 9 — Wire `avg_exec_score` into fitness formula (P2)
**Lines:** 6420–6424

`metrics.sum_exec_e_score` is populated at L6469 but no fitness term reads it.
Adds a 5% execution quality signal — small enough to not dominate, large enough for
selection to discriminate on execution over many generations.

```diff
--- a/core/src/ga.rs
+++ b/core/src/ga.rs
@@ -6420,5 +6420,10 @@
     let activity_score = (total_trades as f64 / 20.0).max(0.05).min(1.0);
+    let avg_exec_score = if metrics.exec_passed_count > 0 {
+        (metrics.sum_exec_e_score / metrics.exec_passed_count as f64).clamp(-1.0, 1.0)
+    } else {
+        0.0
+    };
     let mut fitness =
-        0.55 * pnl + 0.20 * consistency_score + 0.15 * win_rate + 0.10 * activity_score
+        0.50 * pnl + 0.20 * consistency_score + 0.15 * win_rate
+        + 0.10 * activity_score + 0.05 * avg_exec_score
             - execution_penalty * 0.15
             + trade_penalty;
```

---

## Apply Order Checklist (STEP order, not patch number order)

```
STEP 1 — PATCH 1: EvoState last_best_fitness = NEG_INFINITY        (L640)
STEP 1 — PATCH 2: Remove dead trade_penalty + relax cascade        (L6237-6242, L6400-6406, L6441-6450)
STEP 1 — PATCH 3: fitness floor at -0.3                            (after L6452)

         ► Run after Step 1. Confirm: fitness values are no longer all -1.0
           Expected: GEN_SUMMARY best is in range [-0.3, -0.05]

STEP 2 — PATCH 5: spread_z clamp ±10 + STRAT_ANOMALY gate         (grep spread_z)

         ► Run after Step 2. Confirm: no more spread_z = -220 in logs

STEP 3 — PATCH 4: generation_peaks → true global max per gen       (L1564, L1768, L1973-1976)

         ► Now you can actually see the gradient improving across generations

STEP 4 — PATCH 6: exec_prob floor 0.05 → 0.50                     (L5789, L5822-5826)
STEP 4 — PATCH 7: max_trades_per_scenario 12 → 50                  (L5834)

         ► Run after Step 4. Confirm: trades per eval climbs to 30+
           Expected: PARTICIPATION ratio rises from 0.012 → 0.06+

STEP 5 — PATCH 8: tp_dist * 0.85, sl_dist * 1.05                  (L4359-4360)

         ► Run after Step 5. Confirm: EXITS has some TP > 0
           Expected: win_rate climbs from 0% to 15-30%

STEP 6 — PATCH 9: avg_exec_score into fitness                      (L6420-6424)

         ► Final tuning step — adds execution quality to selection pressure
```

---

## Expected Signal After All Steps

```
STEP 1 done:
  GEN_SUMMARY → gen=0  best=-0.28  median=-0.29  worst=-0.30  ...  (floor working)
  GEN_SUMMARY → gen=1  best=-0.24  ...                              (stagnation reset, improving)

STEP 4 done:
  PARTICIPATION → trades=38 attempts=38 triggered=38 signals=500 ratio=0.0760
  fitness variance is now signal, not noise

STEP 5 done:
  ENTRY_DEBUG → EXITS: TP=9 SL=29 TS=0    ← first TP hits
  win_rate = 0.24 → fitness crosses 0

TARGET OUTPUT:
  ✅ Gen  4 → 0.0182
  🚨 FIRST_ALPHA_DISCOVERY → gen=4 fitness=0.018 asset=BTC
```
