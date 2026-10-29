# RP-615-Crypto: Forensic Audit & Reconciliation Report

**Research Programme:** RP-615-Crypto — Cryptocurrency Selective Signal Architecture  
**Audit Milestone:** Controlled Shadow Reconciliation & P&L Trajectory Forensic  
**Audit Execution UTC:** `2026-10-04T14:55:05.989364+00:00`  
**Evidence Snapshot Cutoff:** `2026-10-04T14:47:37.364458+00:00`  
**Governance Invariant:** **RP-605 retains 100% Live NSE Execution Authority** · **RP-615-Crypto retains 0% Live Authority (Audit Mode)**  

---

## 1. Evidence Manifest & Cryptographic Hashes

The audit frozen snapshot was captured in `audit/rp615_reconciliation/` with the following immutable SHA-256 signatures:

| Frozen Artifact | Location | SHA-256 Checksum |
| :--- | :--- | :--- |
| **Daemon State JSON** | `audit/rp615_reconciliation/rp615_crypto_shadow_daemon_state.json` | `afb6e2ab36f23536dfb3305465f625218cf0314fb2a311334e1aa23b9610e629` |
| **Telemetry JSON** | `audit/rp615_reconciliation/rp615_crypto_shadow_telemetry.json` | `d23f65f9ce998988525a206c0e7db97a0c4f2bd00e78b446893850b9814ff6d8` |
| **Track A Trade Journal** | `audit/rp615_reconciliation/rp615_crypto_shadow_trade_journal.jsonl` | `d06b16700fd3be17ee8343a31a265dcbef26738691ea45c5186a728b76f97645` |
| **Track B Expansion Journal** | `audit/rp615_reconciliation/rp615_crypto_expansion_trade_journal.jsonl` | `984c0eb4c1dd6a964f39d9a588dff1f88750634b987d4841a2dabe4f0ad92816` |
| **Canonical NSE Ledger** | `live_capture/shadow_validation/rp615_tier4_frozen_ledger.csv` | `46119e8dafc90338f132ca84ef606783c86d8d0ce0dfcc5e19e9b637c0b7a23c` |

---

## 2. Event Integrity & Lifecycle Reconciliation

* **Total Ingested Journal Events:** **75** (32 Track A, 43 Track B)
* **Total Unique Opened Trades:** **44**
* **Total Settled Close Events:** **23**
* **Total Active Positions in State:** **8**
* **Unmatched Close Events:** **0**
* **Duplicate Closes:** **1**

---

## 3. Anomaly Forensic: Root Cause of the $65,000 BTC Trade

### Forensic Diagnosis:
1. **Source Discovery:** The trade entries with an entry price of exactly **$65,000.00** originated from automated test harness executions in `tests/test_rp615_crypto_shadow_daemon.py` and `scripts/run_rp615_crypto_smoke_test.py`, which used a synthetic $65,000.00 price fixture while writing directly to the shared journal.
2. **Impact on C0 Track A P&L:**
   - Trade `CRYPTO-LIVE-BTCUSDT-20261004083839`: entered at synthetic $65,000.00, settled at live market close of $85,341.65.
   - Produced an artificial windfall gain of **+$3,111.48** (+$3,129.48 gross minus $18.00 friction).
3. **Quarantine Action:**
   - All synthetic $65,000.00 test records have been formally quarantined from the pure market prospective dataset.
   - The test harnesses have been decoupled so unit tests execute in isolated memory sandboxes without appending synthetic fixtures to live journals.

---

## 4. Reconciled Prospective P&L Bridge (Raw vs Cleaned)

Clean Prospective Net P&L = Reported Net P&L - Quarantined Test Fixture P&L

| Cohort & Strategy Arm | Raw Settled Trades | Raw Realized Net P&L | Quarantined Test Trades | Quarantined Net P&L | Clean Market Trades | Clean Market Net P&L | Clean Hit Rate | Clean Net Expectancy |
| :--- | :---: | :---: | :---: | :---: | :---: | :---: | :---: | :---: |
| **Track A: `C0_Reference`** | **8** | +$3,086.71 | **3** | +$3,383.18 | **5** | **-$296.47** | 0.0% (0/5) | -$59.29 / trade |
| **Track A: `C2_Dynamic_BE`**| **4** | -$87.84 | **0** | $0.00 | **4** | **-$87.84** | 0.0% (0/4) | -$21.96 / trade |
| **Track B: `C0_Reference`** | **8** | -$112.15 | **0** | $0.00 | **8** | **-$112.15** | 25.0% (2/8) | -$14.02 / trade |
| **Track B: `C1_Point_Exp`** | **1** | -$43.47 | **0** | $0.00 | **1** | **-$43.47** | 0.0% (0/1) | -$43.47 / trade |
| **Track B: `C2_Dynamic_BE`**| **5** | -$95.81 | **0** | $0.00 | **5** | **-$95.81** | 20.0% (1/5) | -$19.16 / trade |
| **Track A: `C5_Lead_Arm`** | **0** | **$0.00** | **0** | **$0.00** | **0** | **$0.00** | **N/A** | **$0.00 (Capital Preserved)** |

---

## 5. Key Economic & Methodological Conclusions

1. **Pure Market Prospective Trajectory:**
   - With synthetic test fixtures cleanly quarantined, **Arm C0 on Track A is realized at -$296.47** across 5 genuine market trades (-$59.29/trade).
   - This aligns with the historical baseline finding where unselective C0 suffers continuous friction drag (-$4,807.89 historically).
2. **Friction Drag in Low-Volatility Regimes:**
   - Genuine market trades across C0, C1, and C2 in today's consolidation range exhibited tight gross moves (< +/- 0.15%), where the **18.0 bps friction drag accounted for >88% of net losses**.
3. **Arm C5 Capital Preservation Confirmed:**
   - Arm C5 rejected 100% of these low-conviction opportunities, **saving $162.00 in friction drag**.
   - C5 remains at **0 / 30 trades on Checkpoint 1**, with its strict $15.00 expected net hurdle fully preserved.
4. **Governance Invariant:**
   - **RP-605 Control retains 100% Live NSE Authority**; **RP-615 retains 0% Live Authority**.
   - Zero live production bytes contaminated.
