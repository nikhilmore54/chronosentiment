# RP-605 / RP-615: Pre-Open Control Gate Verification Report

**Session Date:** `2026-10-05` (Monday Pre-Open 09:15 IST)  
**Execution UTC:** `2026-10-22T04:03:10.935111+00:00`  
**Control Verdict:** **PRE-OPEN CERTIFIED (PASS)**  

---

## 1. Governance Boundary & Execution Authority Summary

* **RP-605 Baseline Control:** **100% Live NSE Authority** (Sole execution engine).
* **RP-615-NSE Track:** **0% Live Authority** (Stage 5 Forward Shadow on 99 symbols).
* **RP-615-Crypto Sandbox:** **0% Live Authority** (Simulated Paper Shadow Validation).

---

## 2. Pre-Open Invariant Assertions

| Control Gate | Checked Property | Expected Value | Actual Value | Verdict |
| :--- | :--- | :--- | :--- | :---: |
| **RP-605 Checksum** | Canonical NSE Ledger SHA-256 | `46119e8dafc90338f132ca84ef606783c86d8d0ce0dfcc5e19e9b637c0b7a23c` | `46119e8dafc90338f132ca84ef606783c86d8d0ce0dfcc5e19e9b637c0b7a23c` | **PASS** |
| **Credential Isolation** | Order API Tokens in Env | `0 Tokens` | `0 Tokens` | **PASS** |
| **WAL Disambiguation** | Event Key Identity | `EVENT-{sym}-{type}-{arm}-{ts}` | Verified | **PASS** |
| **Model Weights Freeze**| Calibrator Signature | Verified SHA-256 | `4147ff575f35041726c60b3ce3eea7b1332eef5cf34169daccf1aac317b3059f` | **PASS** |
| **Manifest Anchor** | Root Evidence Anchor | Immutable Manifest | `9f036be9d17567a8056f860548172fabfd003dbedc171334872ea7df2d7a68ef` | **PASS** |

---

## 3. Operational Invariants for Monday 09:15 IST Session

1. **Zero Byte Mutation:** Any mutation to `rp615_tier4_frozen_ledger.csv` immediately terminates the shadow pipeline.
2. **Read-Only Shadow Logging:** Shadow worker threads ingest market data in read-only mode and emit decision cards and WAL entries asynchronously.
3. **Continuous Cryptographic Verification:** Pre-flight and post-session checksum assertions are executed automatically.
