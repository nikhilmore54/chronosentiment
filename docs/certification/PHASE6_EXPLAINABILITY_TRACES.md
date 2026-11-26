# Phase 6: Explainability Traces

These traces establish the deterministic attribution of execution degradation to environmental geometry and latency perturbation, forming the MVP certification layer.

Session: 2025-01-17 (NIFTY)

Environment
-----------
Volatility Percentile: 50
Trend Strength: 7.42

Replay Response
---------------
Persistence: 1199
Max Occupancy: 1

Perturbation
------------
Latency: +50ms
Missed Fill Probability: 0%

Counterfactual Baseline
-----------------------
Fill Rate: 100.00%
Slippage: 0.50 bps

Observed Outcome
----------------
Fill Rate: 100.00%
Slippage: 77.65 bps

Delta
-----
Fill Rate: 0.00%
Slippage: +77.15 bps

Deterministic Attribution
-------------------------
Rule R1:
    Normal-volatility environment (Percentile 50.4 <= 80)

Rule R2:
    High-latency perturbation (+50ms applied)

Rule R3:
    Historical response surface predicts amplified degradation under R1 + R2

Certification Result
--------------------
Environmental amplification confirmed.

---

Session: 2025-01-06 (NIFTY)

Environment
-----------
Volatility Percentile: 91
Trend Strength: 27.09

Replay Response
---------------
Persistence: 1119
Max Occupancy: 1

Perturbation
------------
Latency: +50ms
Missed Fill Probability: 0%

Counterfactual Baseline
-----------------------
Fill Rate: 100.00%
Slippage: 0.50 bps

Observed Outcome
----------------
Fill Rate: 100.00%
Slippage: 120.16 bps

Delta
-----
Fill Rate: 0.00%
Slippage: +119.66 bps

Deterministic Attribution
-------------------------
Rule R1:
    High-volatility environment (Percentile > 80)

Rule R2:
    High-latency perturbation (+50ms applied)

Rule R3:
    Historical response surface predicts amplified degradation under R1 + R2

Certification Result
--------------------
Environmental amplification confirmed.

---

Session: 2025-04-25 (BANKNIFTY)

Environment
-----------
Volatility Percentile: 92
Trend Strength: 17.81

Replay Response
---------------
Persistence: 1163
Max Occupancy: 1

Perturbation
------------
Latency: +50ms
Missed Fill Probability: 0%

Counterfactual Baseline
-----------------------
Fill Rate: 100.00%
Slippage: 0.50 bps

Observed Outcome
----------------
Fill Rate: 100.00%
Slippage: 124.16 bps

Delta
-----
Fill Rate: 0.00%
Slippage: +123.66 bps

Deterministic Attribution
-------------------------
Rule R1:
    High-volatility environment (Percentile > 80)

Rule R2:
    High-latency perturbation (+50ms applied)

Rule R3:
    Historical response surface predicts amplified degradation under R1 + R2

Certification Result
--------------------
Environmental amplification confirmed.

---

Session: 2025-06-06 (BANKNIFTY)

Environment
-----------
Volatility Percentile: 91
Trend Strength: 29.26

Replay Response
---------------
Persistence: 1213
Max Occupancy: 1

Perturbation
------------
Latency: +50ms
Missed Fill Probability: 0%

Counterfactual Baseline
-----------------------
Fill Rate: 100.00%
Slippage: 0.50 bps

Observed Outcome
----------------
Fill Rate: 100.00%
Slippage: 86.97 bps

Delta
-----
Fill Rate: 0.00%
Slippage: +86.47 bps

Deterministic Attribution
-------------------------
Rule R1:
    High-volatility environment (Percentile > 80)

Rule R2:
    High-latency perturbation (+50ms applied)

Rule R3:
    Historical response surface predicts amplified degradation under R1 + R2

Certification Result
--------------------
Environmental amplification confirmed.

---

Session: 2025-05-29 (NIFTY)

Environment
-----------
Volatility Percentile: 37
Trend Strength: 6.10

Replay Response
---------------
Persistence: 1220
Max Occupancy: 1

Perturbation
------------
Latency: +50ms
Missed Fill Probability: 0%

Counterfactual Baseline
-----------------------
Fill Rate: 100.00%
Slippage: 0.50 bps

Observed Outcome
----------------
Fill Rate: 100.00%
Slippage: 63.56 bps

Delta
-----
Fill Rate: 0.00%
Slippage: +63.06 bps

Deterministic Attribution
-------------------------
Rule R1:
    Normal-volatility environment (Percentile 36.9 <= 80)

Rule R2:
    High-latency perturbation (+50ms applied)

Rule R3:
    Historical response surface predicts amplified degradation under R1 + R2

Certification Result
--------------------
Environmental amplification confirmed.

---

Session: 2025-05-14 (NIFTY)

Environment
-----------
Volatility Percentile: 50
Trend Strength: 7.30

Replay Response
---------------
Persistence: 1135
Max Occupancy: 1

Perturbation
------------
Latency: +50ms
Missed Fill Probability: 0%

Counterfactual Baseline
-----------------------
Fill Rate: 100.00%
Slippage: 0.50 bps

Observed Outcome
----------------
Fill Rate: 100.00%
Slippage: 68.88 bps

Delta
-----
Fill Rate: 0.00%
Slippage: +68.38 bps

Deterministic Attribution
-------------------------
Rule R1:
    Normal-volatility environment (Percentile 49.6 <= 80)

Rule R2:
    High-latency perturbation (+50ms applied)

Rule R3:
    Historical response surface predicts amplified degradation under R1 + R2

Certification Result
--------------------
Environmental amplification confirmed.

---

Session: 2025-05-26 (NIFTY)

Environment
-----------
Volatility Percentile: 30
Trend Strength: 8.16

Replay Response
---------------
Persistence: 1240
Max Occupancy: 1

Perturbation
------------
Latency: +50ms
Missed Fill Probability: 0%

Counterfactual Baseline
-----------------------
Fill Rate: 100.00%
Slippage: 0.50 bps

Observed Outcome
----------------
Fill Rate: 100.00%
Slippage: 68.58 bps

Delta
-----
Fill Rate: 0.00%
Slippage: +68.08 bps

Deterministic Attribution
-------------------------
Rule R1:
    Normal-volatility environment (Percentile 29.9 <= 80)

Rule R2:
    High-latency perturbation (+50ms applied)

Rule R3:
    Historical response surface predicts amplified degradation under R1 + R2

Certification Result
--------------------
Environmental amplification confirmed.

---

Session: 2025-04-02 (NIFTY)

Environment
-----------
Volatility Percentile: 43
Trend Strength: 16.14

Replay Response
---------------
Persistence: 1234
Max Occupancy: 1

Perturbation
------------
Latency: +50ms
Missed Fill Probability: 0%

Counterfactual Baseline
-----------------------
Fill Rate: 100.00%
Slippage: 0.50 bps

Observed Outcome
----------------
Fill Rate: 100.00%
Slippage: 63.61 bps

Delta
-----
Fill Rate: 0.00%
Slippage: +63.11 bps

Deterministic Attribution
-------------------------
Rule R1:
    Normal-volatility environment (Percentile 43.4 <= 80)

Rule R2:
    High-latency perturbation (+50ms applied)

Rule R3:
    Historical response surface predicts amplified degradation under R1 + R2

Certification Result
--------------------
Environmental amplification confirmed.

---

Session: 2025-01-07 (BANKNIFTY)

Environment
-----------
Volatility Percentile: 83
Trend Strength: 4.28

Replay Response
---------------
Persistence: 1246
Max Occupancy: 1

Perturbation
------------
Latency: +50ms
Missed Fill Probability: 0%

Counterfactual Baseline
-----------------------
Fill Rate: 100.00%
Slippage: 0.50 bps

Observed Outcome
----------------
Fill Rate: 100.00%
Slippage: 106.55 bps

Delta
-----
Fill Rate: 0.00%
Slippage: +106.05 bps

Deterministic Attribution
-------------------------
Rule R1:
    High-volatility environment (Percentile > 80)

Rule R2:
    High-latency perturbation (+50ms applied)

Rule R3:
    Historical response surface predicts amplified degradation under R1 + R2

Certification Result
--------------------
Environmental amplification confirmed.

---

Session: 2025-03-13 (BANKNIFTY)

Environment
-----------
Volatility Percentile: 24
Trend Strength: 9.54

Replay Response
---------------
Persistence: 1193
Max Occupancy: 1

Perturbation
------------
Latency: +50ms
Missed Fill Probability: 0%

Counterfactual Baseline
-----------------------
Fill Rate: 100.00%
Slippage: 0.50 bps

Observed Outcome
----------------
Fill Rate: 100.00%
Slippage: 84.00 bps

Delta
-----
Fill Rate: 0.00%
Slippage: +83.50 bps

Deterministic Attribution
-------------------------
Rule R1:
    Normal-volatility environment (Percentile 24.2 <= 80)

Rule R2:
    High-latency perturbation (+50ms applied)

Rule R3:
    Historical response surface predicts amplified degradation under R1 + R2

Certification Result
--------------------
Environmental amplification confirmed.

---

Session: 2025-01-31 (BANKNIFTY)

Environment
-----------
Volatility Percentile: 62
Trend Strength: 13.96

Replay Response
---------------
Persistence: 1208
Max Occupancy: 1

Perturbation
------------
Latency: +50ms
Missed Fill Probability: 0%

Counterfactual Baseline
-----------------------
Fill Rate: 100.00%
Slippage: 0.50 bps

Observed Outcome
----------------
Fill Rate: 99.61%
Slippage: 96.32 bps

Delta
-----
Fill Rate: -0.39%
Slippage: +95.82 bps

Deterministic Attribution
-------------------------
Rule R1:
    Normal-volatility environment (Percentile 62.3 <= 80)

Rule R2:
    High-latency perturbation (+50ms applied)

Rule R3:
    Historical response surface predicts amplified degradation under R1 + R2

Certification Result
--------------------
Environmental amplification confirmed.

---

Session: 2025-06-23 (BANKNIFTY)

Environment
-----------
Volatility Percentile: 17
Trend Strength: 9.58

Replay Response
---------------
Persistence: 1245
Max Occupancy: 1

Perturbation
------------
Latency: +50ms
Missed Fill Probability: 0%

Counterfactual Baseline
-----------------------
Fill Rate: 100.00%
Slippage: 0.50 bps

Observed Outcome
----------------
Fill Rate: 100.00%
Slippage: 71.29 bps

Delta
-----
Fill Rate: 0.00%
Slippage: +70.79 bps

Deterministic Attribution
-------------------------
Rule R1:
    Normal-volatility environment (Percentile 16.8 <= 80)

Rule R2:
    High-latency perturbation (+50ms applied)

Rule R3:
    Historical response surface predicts amplified degradation under R1 + R2

Certification Result
--------------------
Environmental amplification confirmed.

---

Session: 2025-03-26 (BANKNIFTY)

Environment
-----------
Volatility Percentile: 75
Trend Strength: 15.89

Replay Response
---------------
Persistence: 1111
Max Occupancy: 1

Perturbation
------------
Latency: +50ms
Missed Fill Probability: 0%

Counterfactual Baseline
-----------------------
Fill Rate: 100.00%
Slippage: 0.50 bps

Observed Outcome
----------------
Fill Rate: 100.00%
Slippage: 101.75 bps

Delta
-----
Fill Rate: 0.00%
Slippage: +101.25 bps

Deterministic Attribution
-------------------------
Rule R1:
    Normal-volatility environment (Percentile 74.6 <= 80)

Rule R2:
    High-latency perturbation (+50ms applied)

Rule R3:
    Historical response surface predicts amplified degradation under R1 + R2

Certification Result
--------------------
Environmental amplification confirmed.

---

Session: 2025-05-14 (BANKNIFTY)

Environment
-----------
Volatility Percentile: 55
Trend Strength: 5.67

Replay Response
---------------
Persistence: 1141
Max Occupancy: 1

Perturbation
------------
Latency: +50ms
Missed Fill Probability: 0%

Counterfactual Baseline
-----------------------
Fill Rate: 100.00%
Slippage: 0.50 bps

Observed Outcome
----------------
Fill Rate: 100.00%
Slippage: 94.55 bps

Delta
-----
Fill Rate: 0.00%
Slippage: +94.05 bps

Deterministic Attribution
-------------------------
Rule R1:
    Normal-volatility environment (Percentile 55.3 <= 80)

Rule R2:
    High-latency perturbation (+50ms applied)

Rule R3:
    Historical response surface predicts amplified degradation under R1 + R2

Certification Result
--------------------
Environmental amplification confirmed.

---

Session: 2025-02-28 (NIFTY)

Environment
-----------
Volatility Percentile: 47
Trend Strength: 31.04

Replay Response
---------------
Persistence: 1150
Max Occupancy: 1

Perturbation
------------
Latency: +50ms
Missed Fill Probability: 0%

Counterfactual Baseline
-----------------------
Fill Rate: 100.00%
Slippage: 0.50 bps

Observed Outcome
----------------
Fill Rate: 100.00%
Slippage: 80.93 bps

Delta
-----
Fill Rate: 0.00%
Slippage: +80.43 bps

Deterministic Attribution
-------------------------
Rule R1:
    Normal-volatility environment (Percentile 47.1 <= 80)

Rule R2:
    High-latency perturbation (+50ms applied)

Rule R3:
    Historical response surface predicts amplified degradation under R1 + R2

Certification Result
--------------------
Environmental amplification confirmed.

---

Session: 2025-04-22 (NIFTY)

Environment
-----------
Volatility Percentile: 17
Trend Strength: 6.34

Replay Response
---------------
Persistence: 1251
Max Occupancy: 1

Perturbation
------------
Latency: +50ms
Missed Fill Probability: 0%

Counterfactual Baseline
-----------------------
Fill Rate: 100.00%
Slippage: 0.50 bps

Observed Outcome
----------------
Fill Rate: 100.00%
Slippage: 61.12 bps

Delta
-----
Fill Rate: 0.00%
Slippage: +60.62 bps

Deterministic Attribution
-------------------------
Rule R1:
    Normal-volatility environment (Percentile 17.2 <= 80)

Rule R2:
    High-latency perturbation (+50ms applied)

Rule R3:
    Historical response surface predicts amplified degradation under R1 + R2

Certification Result
--------------------
Environmental amplification confirmed.

---

Session: 2025-05-30 (NIFTY)

Environment
-----------
Volatility Percentile: 18
Trend Strength: 10.09

Replay Response
---------------
Persistence: 1197
Max Occupancy: 1

Perturbation
------------
Latency: +50ms
Missed Fill Probability: 0%

Counterfactual Baseline
-----------------------
Fill Rate: 100.00%
Slippage: 0.50 bps

Observed Outcome
----------------
Fill Rate: 99.54%
Slippage: 54.56 bps

Delta
-----
Fill Rate: -0.46%
Slippage: +54.06 bps

Deterministic Attribution
-------------------------
Rule R1:
    Normal-volatility environment (Percentile 17.6 <= 80)

Rule R2:
    High-latency perturbation (+50ms applied)

Rule R3:
    Historical response surface predicts amplified degradation under R1 + R2

Certification Result
--------------------
Environmental amplification confirmed.

---

Session: 2025-01-10 (BANKNIFTY)

Environment
-----------
Volatility Percentile: 93
Trend Strength: 18.70

Replay Response
---------------
Persistence: 1158
Max Occupancy: 1

Perturbation
------------
Latency: +50ms
Missed Fill Probability: 0%

Counterfactual Baseline
-----------------------
Fill Rate: 100.00%
Slippage: 0.50 bps

Observed Outcome
----------------
Fill Rate: 100.00%
Slippage: 133.47 bps

Delta
-----
Fill Rate: 0.00%
Slippage: +132.97 bps

Deterministic Attribution
-------------------------
Rule R1:
    High-volatility environment (Percentile > 80)

Rule R2:
    High-latency perturbation (+50ms applied)

Rule R3:
    Historical response surface predicts amplified degradation under R1 + R2

Certification Result
--------------------
Environmental amplification confirmed.

---

Session: 2025-02-17 (NIFTY)

Environment
-----------
Volatility Percentile: 88
Trend Strength: 11.30

Replay Response
---------------
Persistence: 1183
Max Occupancy: 1

Perturbation
------------
Latency: +50ms
Missed Fill Probability: 0%

Counterfactual Baseline
-----------------------
Fill Rate: 100.00%
Slippage: 0.50 bps

Observed Outcome
----------------
Fill Rate: 100.00%
Slippage: 106.92 bps

Delta
-----
Fill Rate: 0.00%
Slippage: +106.42 bps

Deterministic Attribution
-------------------------
Rule R1:
    High-volatility environment (Percentile > 80)

Rule R2:
    High-latency perturbation (+50ms applied)

Rule R3:
    Historical response surface predicts amplified degradation under R1 + R2

Certification Result
--------------------
Environmental amplification confirmed.

---

Session: 2025-04-17 (NIFTY)

Environment
-----------
Volatility Percentile: 65
Trend Strength: 37.50

Replay Response
---------------
Persistence: 1233
Max Occupancy: 1

Perturbation
------------
Latency: +50ms
Missed Fill Probability: 0%

Counterfactual Baseline
-----------------------
Fill Rate: 100.00%
Slippage: 0.50 bps

Observed Outcome
----------------
Fill Rate: 100.00%
Slippage: 91.91 bps

Delta
-----
Fill Rate: 0.00%
Slippage: +91.41 bps

Deterministic Attribution
-------------------------
Rule R1:
    Normal-volatility environment (Percentile 64.8 <= 80)

Rule R2:
    High-latency perturbation (+50ms applied)

Rule R3:
    Historical response surface predicts amplified degradation under R1 + R2

Certification Result
--------------------
Environmental amplification confirmed.

---

