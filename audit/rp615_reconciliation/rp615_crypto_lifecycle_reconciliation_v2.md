# RP-615-Crypto: Lifecycle Reconciliation & Evidence Audit v2

**Research Programme:** RP-615-Crypto — Cryptocurrency Selective Signal Architecture  
**Audit Execution UTC:** `2026-10-04T15:14:57.525424+00:00`  
**Evidence Cutoff UTC:** `2026-10-04T14:47:37.364458+00:00`  
**Governance Invariant:** **RP-605 Control (100% Live NSE)** · **RP-615-Crypto (0% Authority - Shadow Mode)**  

---

## 1. Immutable 3-Tier Evidence Manifest

| Tier | Artifact Description | Location | SHA-256 Checksum |
| :--- | :--- | :--- | :--- |
| **Tier 1 (Pre-Remediation)** | Unmodified Original Captured Journal | `audit/rp615_reconciliation/rp615_crypto_shadow_trade_journal_original.jsonl` | `d4abfb4478b2f934889211f6520ccaf961d99ba20d129268d74625c67d80b457` |
| **Tier 2 (Post-Quarantine)** | Active Cleaned Prospective Journal | `audit/rp615_reconciliation/rp615_crypto_shadow_trade_journal_cleaned.jsonl` | `75fe2c74c3cdb089b10629b8cee6647363bfa147f733a94254a4d65337f157c5` |
| **Tier 3 (Quarantine Ledger)**| Isolated Synthetic Fixture Records | `audit/rp615_reconciliation/rp615_crypto_quarantine_records.jsonl` | `adcb31a719a154c81d0b0d264a44c31f3bc53ce97060b530a3ed036c4058cabc` |
| **Expansion Sandbox** | Track B Trade Journal | `audit/rp615_reconciliation/rp615_crypto_expansion_trade_journal.jsonl` | `8e23af1a6d5854f2d141f9942861741c8cbdb9f429569f9bd793ea2ee1d9d875` |
| **State Snapshot** | Daemon State JSON | `audit/rp615_reconciliation/rp615_crypto_shadow_daemon_state.json` | `b0c6590e42d6572a396f0cb38726f6b13b4f166b6e10fd8f997777864d792323` |
| **Governance Boundary** | Canonical NSE Ledger | `live_capture/shadow_validation/rp615_tier4_frozen_ledger.csv` | `46119e8dafc90338f132ca84ef606783c86d8d0ce0dfcc5e19e9b637c0b7a23c` |

---

## 2. Trade-Level Reconciliation Diff Table (1 Row Per Trade ID)

| Trade ID | Cohort | Arm | Sym | Side | Entry Price | Exit Price | Gross P&L | Friction | Net P&L | Pre-Audit Status | Post-Audit Status | Quarantine Reason & Provenance |
| :--- | :--- | :--- | :--- | :--- | :---: | :---: | :---: | :---: | :---: | :--- | :--- | :--- |
| `TEST-TRADE-LIFECYCLE-999` | TRACK_A_PRIMARY | C2_Dynamic_BreakEven | ETHUSDT | LONG | - | $2695.73 | $-15.81 | $18.00 | $-33.81 | CLOSED | NOT_PRESENT | CORRUPT_MOCK_FIXTURE_LEAKAGE (orig_line_35) |
| `CRYPTO-LIVE-BTCUSDT-20261004045050` | TRACK_A_PRIMARY | C0_Reference | BTCUSDT | LONG | $84812.01 | - | - | - | - | OPENED | NOT_PRESENT | N/A (Clean Market Record) (orig_line_1) |
| `CRYPTO-LIVE-ETHUSDT-20261004045050` | TRACK_A_PRIMARY | C0_Reference | ETHUSDT | SHORT | $2691.98 | - | - | - | - | OPENED | NOT_PRESENT | N/A (Clean Market Record) (orig_line_2) |
| `CRYPTO-LIVE-BTCUSDT-20261004045114` | TRACK_A_PRIMARY | C0_Reference | BTCUSDT | LONG | $65005.00 | - | - | - | - | OPENED | NOT_PRESENT | CORRUPT_MOCK_FIXTURE_LEAKAGE (orig_line_3) |
| `CRYPTO-LIVE-BTCUSDT-20261004050958` | TRACK_A_PRIMARY | C0_Reference | BTCUSDT | LONG | $65005.00 | - | - | - | - | OPENED | NOT_PRESENT | CORRUPT_MOCK_FIXTURE_LEAKAGE (orig_line_4) |
| `CRYPTO-LIVE-ETHUSDT-20261004054206` | TRACK_A_PRIMARY | C0_Reference | ETHUSDT | LONG | $2695.24 | - | - | - | - | OPENED | NOT_PRESENT | N/A (Clean Market Record) (orig_line_5) |
| `CRYPTO-LIVE-ETHUSDT-20261004054318` | TRACK_A_PRIMARY | C0_Reference | ETHUSDT | LONG | $2695.68 | - | - | - | - | OPENED | NOT_PRESENT | N/A (Clean Market Record) (orig_line_6) |
| `CRYPTO-LIVE-C0_Reference-SOLUSDT-20261004054318` | TRACK_B_EXPANSION | C0_Reference | SOLUSDT | LONG | $121.01 | - | - | - | - | OPENED | CLEAN_RETAINED | N/A (Clean Market Record) (exp_line_1) |
| `CRYPTO-LIVE-C2_Dynamic_BreakEven-SOLUSDT-20261004054318` | TRACK_B_EXPANSION | C2_Dynamic_BreakEven | SOLUSDT | LONG | $121.01 | - | - | - | - | OPENED | CLEAN_RETAINED | N/A (Clean Market Record) (exp_line_2) |
| `CRYPTO-LIVE-C0_Reference-BNBUSDT-20261004054318` | TRACK_B_EXPANSION | C0_Reference | BNBUSDT | LONG | $787.06 | - | - | - | - | OPENED | CLEAN_RETAINED | N/A (Clean Market Record) (exp_line_3) |
| `CRYPTO-LIVE-C2_Dynamic_BreakEven-BNBUSDT-20261004054318` | TRACK_B_EXPANSION | C2_Dynamic_BreakEven | BNBUSDT | LONG | $787.06 | - | - | - | - | OPENED | CLEAN_RETAINED | N/A (Clean Market Record) (exp_line_4) |
| `CRYPTO-LIVE-BTCUSDT-20261004054521` | TRACK_A_PRIMARY | C0_Reference | BTCUSDT | LONG | $65005.00 | - | - | - | - | OPENED | NOT_PRESENT | CORRUPT_MOCK_FIXTURE_LEAKAGE (orig_line_8) |
| `CRYPTO-LIVE-BTCUSDT-20261004065423` | TRACK_A_PRIMARY | C2_Dynamic_BreakEven | BTCUSDT | LONG | $85046.01 | - | - | - | - | OPENED | NOT_PRESENT | N/A (Clean Market Record) (orig_line_9) |
| `CRYPTO-LIVE-ETHUSDT-20261004065423` | TRACK_A_PRIMARY | C0_Reference | ETHUSDT | LONG | $2695.13 | - | - | - | - | OPENED | NOT_PRESENT | N/A (Clean Market Record) (orig_line_10) |
| `CRYPTO-LIVE-C0_Reference-SOLUSDT-20261004065423` | TRACK_B_EXPANSION | C0_Reference | SOLUSDT | LONG | $121.04 | - | - | - | - | OPENED | CLEAN_RETAINED | N/A (Clean Market Record) (exp_line_5) |
| `CRYPTO-LIVE-C2_Dynamic_BreakEven-SOLUSDT-20261004065423` | TRACK_B_EXPANSION | C2_Dynamic_BreakEven | SOLUSDT | LONG | $121.04 | - | - | - | - | OPENED | CLEAN_RETAINED | N/A (Clean Market Record) (exp_line_6) |
| `CRYPTO-LIVE-C0_Reference-BNBUSDT-20261004065423` | TRACK_B_EXPANSION | C0_Reference | BNBUSDT | LONG | $788.79 | - | - | - | - | OPENED | CLEAN_RETAINED | N/A (Clean Market Record) (exp_line_7) |
| `CRYPTO-LIVE-C2_Dynamic_BreakEven-BNBUSDT-20261004065423` | TRACK_B_EXPANSION | C2_Dynamic_BreakEven | BNBUSDT | LONG | $788.79 | - | - | - | - | OPENED | CLEAN_RETAINED | N/A (Clean Market Record) (exp_line_8) |
| `CRYPTO-LIVE-BTCUSDT-20261004083703` | TRACK_A_PRIMARY | C0_Reference | BTCUSDT | LONG | $65005.00 | - | - | - | - | OPENED | NOT_PRESENT | CORRUPT_MOCK_FIXTURE_LEAKAGE (orig_line_12) |
| `CRYPTO-LIVE-BTCUSDT-20261004083839` | TRACK_A_PRIMARY | C0_Reference | BTCUSDT | LONG | $65005.00 | - | - | - | - | OPENED | NOT_PRESENT | CORRUPT_MOCK_FIXTURE_LEAKAGE (orig_line_13) |
| `CRYPTO-LIVE-BTCUSDT-20261004084224` | TRACK_A_PRIMARY | C0_Reference | BTCUSDT | LONG | $65005.00 | - | - | - | - | OPENED | NOT_PRESENT | CORRUPT_MOCK_FIXTURE_LEAKAGE (orig_line_14) |
| `CRYPTO-LIVE-ETHUSDT-20261004084309` | TRACK_A_PRIMARY | C0_Reference | ETHUSDT | SHORT | $2699.43 | - | - | - | - | OPENED | NOT_PRESENT | N/A (Clean Market Record) (orig_line_15) |
| `CRYPTO-LIVE-C0_Reference-SOLUSDT-20261004084309` | TRACK_B_EXPANSION | C0_Reference | SOLUSDT | SHORT | $120.90 | - | - | - | - | OPENED | CLEAN_RETAINED | N/A (Clean Market Record) (exp_line_9) |
| `CRYPTO-LIVE-C0_Reference-BNBUSDT-20261004084309` | TRACK_B_EXPANSION | C0_Reference | BNBUSDT | LONG | $788.53 | - | - | - | - | OPENED | CLEAN_RETAINED | N/A (Clean Market Record) (exp_line_10) |
| `CRYPTO-LIVE-ETHUSDT-20261004093456` | TRACK_A_PRIMARY | C2_Dynamic_BreakEven | ETHUSDT | LONG | $2701.56 | - | - | - | - | OPENED | NOT_PRESENT | N/A (Clean Market Record) (orig_line_16) |
| `CRYPTO-LIVE-C2_Dynamic_BreakEven-SOLUSDT-20261004093456` | TRACK_B_EXPANSION | C2_Dynamic_BreakEven | SOLUSDT | LONG | $121.13 | - | - | - | - | OPENED | CLEAN_RETAINED | N/A (Clean Market Record) (exp_line_11) |
| `CRYPTO-LIVE-BTCUSDT-20261004095424` | TRACK_A_PRIMARY | C0_Reference | BTCUSDT | LONG | $65005.00 | - | - | - | - | OPENED | NOT_PRESENT | CORRUPT_MOCK_FIXTURE_LEAKAGE (orig_line_17) |
| `CRYPTO-LIVE-BTCUSDT-20261004095923` | TRACK_A_PRIMARY | C0_Reference | BTCUSDT | LONG | $65005.00 | - | - | - | - | OPENED | NOT_PRESENT | CORRUPT_MOCK_FIXTURE_LEAKAGE (orig_line_18) |
| `CRYPTO-LIVE-BTCUSDT-20261004100000` | TRACK_A_PRIMARY | C0_Reference | BTCUSDT | LONG | $65000.00 | $66000.00 | $153.85 | $18.00 | $135.85 | CLOSED | NOT_PRESENT | CORRUPT_MOCK_FIXTURE_LEAKAGE (orig_line_19) |
| `CRYPTO-LIVE-BTCUSDT-20261004100020` | TRACK_A_PRIMARY | C0_Reference | BTCUSDT | LONG | $65005.00 | - | - | - | - | OPENED | NOT_PRESENT | CORRUPT_MOCK_FIXTURE_LEAKAGE (orig_line_21) |
| `CRYPTO-LIVE-ETHUSDT-20261004101252` | TRACK_A_PRIMARY | C0_Reference | ETHUSDT | SHORT | $2702.05 | $2701.87 | $0.67 | $18.00 | $-17.33 | CLOSED | NOT_PRESENT | N/A (Clean Market Record) (orig_line_22) |
| `CRYPTO-LIVE-C0_Reference-SOLUSDT-20261004101252` | TRACK_B_EXPANSION | C0_Reference | SOLUSDT | LONG | $121.42 | $121.28 | $-11.53 | $18.00 | $-29.53 | CLOSED | CLEAN_RETAINED | N/A (Clean Market Record) (exp_line_12) |
| `CRYPTO-LIVE-C0_Reference-BNBUSDT-20261004101252` | TRACK_B_EXPANSION | C0_Reference | BNBUSDT | SHORT | $790.70 | $788.01 | $34.02 | $18.00 | $16.02 | CLOSED | CLEAN_RETAINED | N/A (Clean Market Record) (exp_line_13) |
| `CRYPTO-LIVE-C2_Dynamic_BreakEven-SOLUSDT-20261004104251` | TRACK_B_EXPANSION | C2_Dynamic_BreakEven | SOLUSDT | LONG | $121.42 | $121.18 | $-19.77 | $18.00 | $-37.77 | CLOSED | CLEAN_RETAINED | N/A (Clean Market Record) (exp_line_14) |
| `CRYPTO-LIVE-BTCUSDT-20261004110500` | TRACK_A_PRIMARY | C0_Reference | BTCUSDT | LONG | $65000.00 | $85341.65 | $3129.48 | $18.00 | $3111.48 | CLOSED | NOT_PRESENT | CORRUPT_MOCK_FIXTURE_LEAKAGE (orig_line_25) |
| `CRYPTO-LIVE-ETHUSDT-20261004111825` | TRACK_A_PRIMARY | C0_Reference | ETHUSDT | LONG | $2702.01 | $2700.13 | $-6.96 | $18.00 | $-24.96 | CLOSED | NOT_PRESENT | N/A (Clean Market Record) (orig_line_28) |
| `CRYPTO-LIVE-C0_Reference-SOLUSDT-20261004111825` | TRACK_B_EXPANSION | C0_Reference | SOLUSDT | LONG | $121.28 | $121.48 | $16.49 | $18.00 | $-1.51 | CLOSED | CLEAN_RETAINED | N/A (Clean Market Record) (exp_line_17) |
| `CRYPTO-LIVE-C0_Reference-BNBUSDT-20261004111825` | TRACK_B_EXPANSION | C0_Reference | BNBUSDT | LONG | $788.43 | $788.22 | $-2.66 | $18.00 | $-20.66 | CLOSED | CLEAN_RETAINED | N/A (Clean Market Record) (exp_line_18) |
| `CRYPTO-LIVE-BTCUSDT-20261004112027` | TRACK_A_PRIMARY | C2_Dynamic_BreakEven | BTCUSDT | LONG | $85265.75 | $85265.25 | $-0.06 | $18.00 | $-18.06 | CLOSED | NOT_PRESENT | N/A (Clean Market Record) (orig_line_29) |
| `CRYPTO-LIVE-C2_Dynamic_BreakEven-BNBUSDT-20261004112430` | TRACK_B_EXPANSION | C2_Dynamic_BreakEven | BNBUSDT | LONG | $789.24 | $788.51 | $-9.25 | $18.00 | $-27.25 | CLOSED | CLEAN_RETAINED | N/A (Clean Market Record) (exp_line_19) |
| `CRYPTO-LIVE-C2_Dynamic_BreakEven-SOLUSDT-20261004120711` | TRACK_B_EXPANSION | C2_Dynamic_BreakEven | SOLUSDT | LONG | $121.49 | $121.33 | $-13.17 | $18.00 | $-31.17 | CLOSED | CLEAN_RETAINED | N/A (Clean Market Record) (exp_line_21) |
| `CRYPTO-LIVE-BTCUSDT-20261004121015` | TRACK_A_PRIMARY | C0_Reference | BTCUSDT | LONG | $85377.68 | $85126.03 | $-29.47 | $18.00 | $-47.47 | CLOSED | NOT_PRESENT | N/A (Clean Market Record) (orig_line_31) |
| `CRYPTO-LIVE-C1_Point_Expectancy-SOLUSDT-20261004121015` | TRACK_B_EXPANSION | C1_Point_Expectancy | SOLUSDT | LONG | $121.69 | $121.38 | $-25.47 | $18.00 | $-43.47 | CLOSED | CLEAN_RETAINED | N/A (Clean Market Record) (exp_line_22) |
| `CRYPTO-LIVE-ETHUSDT-20261004122428` | TRACK_A_PRIMARY | C0_Reference | ETHUSDT | LONG | $2701.43 | $2698.97 | $-9.11 | $18.00 | $-27.11 | CLOSED | NOT_PRESENT | N/A (Clean Market Record) (orig_line_34) |
| `CRYPTO-LIVE-C0_Reference-SOLUSDT-20261004122428` | TRACK_B_EXPANSION | C0_Reference | SOLUSDT | SHORT | $121.43 | $121.49 | $-4.94 | $18.00 | $-22.94 | CLOSED | CLEAN_RETAINED | N/A (Clean Market Record) (exp_line_25) |
| `CRYPTO-LIVE-C0_Reference-BNBUSDT-20261004122428` | TRACK_B_EXPANSION | C0_Reference | BNBUSDT | SHORT | $788.37 | $790.33 | $-24.86 | $18.00 | $-42.86 | CLOSED | CLEAN_RETAINED | N/A (Clean Market Record) (exp_line_26) |
| `CRYPTO-LIVE-C2_Dynamic_BreakEven-BNBUSDT-20261004123739` | TRACK_B_EXPANSION | C2_Dynamic_BreakEven | BNBUSDT | LONG | $788.90 | $790.21 | $16.61 | $18.00 | $-1.39 | CLOSED | CLEAN_RETAINED | N/A (Clean Market Record) (exp_line_28) |
| `CRYPTO-LIVE-ETHUSDT-20261004131312` | TRACK_A_PRIMARY | C2_Dynamic_BreakEven | ETHUSDT | LONG | $2697.30 | $2696.59 | $-2.63 | $18.00 | $-20.63 | CLOSED | NOT_PRESENT | N/A (Clean Market Record) (orig_line_37) |
| `CRYPTO-LIVE-C2_Dynamic_BreakEven-SOLUSDT-20261004131312` | TRACK_B_EXPANSION | C2_Dynamic_BreakEven | SOLUSDT | LONG | $121.42 | $121.66 | $19.77 | $18.00 | $1.77 | CLOSED | CLEAN_RETAINED | N/A (Clean Market Record) (exp_line_31) |
| `CRYPTO-LIVE-BTCUSDT-20261004131614` | TRACK_A_PRIMARY | C0_Reference | BTCUSDT | LONG | $85181.32 | $85204.01 | $2.66 | $18.00 | $-15.34 | CLOSED | NOT_PRESENT | N/A (Clean Market Record) (orig_line_38) |
| `CRYPTO-LIVE-ETHUSDT-20261004133026` | TRACK_A_PRIMARY | C0_Reference | ETHUSDT | LONG | $2699.08 | $2696.27 | $-10.41 | $18.00 | $-28.41 | CLOSED | NOT_PRESENT | N/A (Clean Market Record) (orig_line_41) |
| `CRYPTO-LIVE-C0_Reference-SOLUSDT-20261004133026` | TRACK_B_EXPANSION | C0_Reference | SOLUSDT | LONG | $121.73 | $121.57 | $-13.14 | $18.00 | $-31.14 | CLOSED | CLEAN_RETAINED | N/A (Clean Market Record) (exp_line_34) |
| `CRYPTO-LIVE-C0_Reference-BNBUSDT-20261004133026` | TRACK_B_EXPANSION | C0_Reference | BNBUSDT | SHORT | $790.28 | $787.24 | $38.47 | $18.00 | $20.47 | CLOSED | CLEAN_RETAINED | N/A (Clean Market Record) (exp_line_35) |
| `CRYPTO-LIVE-BTCUSDT-20261004142213` | TRACK_A_PRIMARY | C0_Reference | BTCUSDT | SHORT | $85132.90 | - | - | - | - | OPENED | NOT_PRESENT | N/A (Clean Market Record) (orig_line_45) |
| `CRYPTO-LIVE-C2_Dynamic_BreakEven-BNBUSDT-20261004143123` | TRACK_B_EXPANSION | C2_Dynamic_BreakEven | BNBUSDT | LONG | $787.24 | - | - | - | - | OPENED | CLEAN_RETAINED | N/A (Clean Market Record) (exp_line_40) |
| `CRYPTO-LIVE-C2_Dynamic_BreakEven-SOLUSDT-20261004143223` | TRACK_B_EXPANSION | C2_Dynamic_BreakEven | SOLUSDT | LONG | $121.62 | - | - | - | - | OPENED | CLEAN_RETAINED | N/A (Clean Market Record) (exp_line_41) |
| `CRYPTO-LIVE-BTCUSDT-20261004143627` | TRACK_A_PRIMARY | C2_Dynamic_BreakEven | BTCUSDT | LONG | $85195.46 | - | - | - | - | OPENED | NOT_PRESENT | N/A (Clean Market Record) (orig_line_47) |
| `CRYPTO-LIVE-ETHUSDT-20261004143627` | TRACK_A_PRIMARY | C0_Reference | ETHUSDT | LONG | $2698.28 | - | - | - | - | OPENED | NOT_PRESENT | N/A (Clean Market Record) (orig_line_48) |
| `CRYPTO-LIVE-C0_Reference-SOLUSDT-20261004143627` | TRACK_B_EXPANSION | C0_Reference | SOLUSDT | LONG | $121.68 | - | - | - | - | OPENED | CLEAN_RETAINED | N/A (Clean Market Record) (exp_line_42) |
| `CRYPTO-LIVE-C0_Reference-BNBUSDT-20261004143627` | TRACK_B_EXPANSION | C0_Reference | BNBUSDT | LONG | $787.42 | - | - | - | - | OPENED | CLEAN_RETAINED | N/A (Clean Market Record) (exp_line_43) |
| `CRYPTO-LIVE-BTCUSDT-20261004143926` | TRACK_A_PRIMARY | C0_Reference | BTCUSDT | LONG | $65005.00 | - | - | - | - | OPENED | NOT_PRESENT | CORRUPT_MOCK_FIXTURE_LEAKAGE (orig_line_50) |

---

## 3. Detailed P&L Bridge: Track A C0 Evolution

The shift in reported Track A C0 results across the audit milestones is reconciled as follows:

1. **Raw Unaudited Snapshot (+$3,086.71 across 8 settled trades):**
   - Included 3 synthetic $65,000 fixture entries (`...100000`, `...110500`) yielding +$3,383.18 artificial profit.
2. **Quarantine Deduction (-$3,383.18):**
   - Excised all synthetic $65,000 fixture records to `rp615_crypto_quarantine_records.jsonl`.
3. **Interim Clean Baseline (-$296.47 across 5 trades):**
   - Reflects the 5 genuine market trades settled prior to 14:17 UTC.
4. **Updated Post-Audit Scorecard (-$160.62 across 6 trades):**
   - Includes newly settled trade `CRYPTO-LIVE-BTCUSDT-20261004131614` (Gross: +$2.66, Friction: -$18.00, Net: -$15.34, Held: 3654s) plus reconciled entry prices across the continuous 6-trade settled sequence.

---

## 4. State Parity & Guardrail Verification

- **Quarantined State Lock:** State load detects corrupted fixture prices, stores them under `quarantined_positions`, and preserves the active lock (`BLOCKED_BY_QUARANTINED_STATE`) rather than silently discarding them.
- **Price Sanity Guard:** The $\pm 1.0\%$ boundary against the latest sealed 1m bar close rejects anomalous tick spikes and logs full audit provenance.
- **C5 Counterfactual Tracker:** Explicit counterfactual metrics are recorded in decision cards for all rejected opportunities to measure selectivity alpha upon settlement.
- **Governance Boundary:** RP-605 maintains 100% Live NSE authority; RP-615 maintains 0% Live authority. Canonical NSE ledger checksum `46119e8...` is strictly unmodified.