# RP-608-OFX — Crypto `aggTrade` Durable Shadow Capture

**Protocol:** `RP-608-OFX-CRYPTO-AGGTRADE-CAPTURE-001`  
**Status:** **AUTHORIZED · IMPLEMENTED** (2026-11-07)  
**Authority:** 0.0% · observational capture only  
**Depends on:** `docs/research/rp608_ofx_executed_quantity_directional_flow.md` (library complete — **not rebuilt**)

---

## Clarification (locked)

| Question | Answer |
| --- | --- |
| Is a crypto data **source** available? | **Yes — Binance `aggTrade`** |
| Do we need another venue or aggressor proxy? | **No** |
| What was missing? | Durable capture + journaling on the shadow research path |
| Status now | Capture **implemented**; durable series accumulates when the crypto shadow daemon runs with capture enabled |

### Binance fields persisted (`DIRECT`)

| Binance field | Meaning for OFX |
| --- | --- |
| `q` | Executed quantity |
| `m = false` | Buyer was taker → BUY aggressor |
| `m = true` | Buyer was maker → SELL aggressor |
| `s` | Symbol |
| `T` / `E` | Trade / event timestamps |
| `a` | Aggregate trade ID |

---

## Implementation

| Component | Path |
| --- | --- |
| Journal writer | `scripts/rp608_ofx_crypto_aggtrade_journal.py` |
| Daemon hook (separate WS) | `scripts/run_rp608_shadow_daemon.py` → `ingest_ofx_aggtrade` |
| Journal root | `{crypto_output_dir}/ofx_aggtrade_capture_v1/aggtrades.jsonl` |
| Coverage | `…/capture_coverage.json` |
| DQ + aggregator feed | `scripts/research/rp608_ofx_crypto_aggtrade_dq_report.py` |
| Volume/direction mark | `scripts/rp608_ofx_crypto_volume_flow_mark.py` → `ofx_volume_flow_snapshot.json` (parallel to L1 OF) |
| Unit tests | `tests/test_rp608_ofx_crypto_aggtrade_journal.py` · `tests/test_rp608_ofx_volume_flow_mark.py` |

**Enable/disable:** env `RP608_OFX_CRYPTO_AGGTRADE_CAPTURE` (`1`/`0`; default **enabled**).

**Invariants:** baseline OF inert · no entry/exit hooks · authority 0% · fail-closed on OFX task errors · does not rebuild `executed_flow` library.

### DQ check

```bash
python3 scripts/research/rp608_ofx_crypto_aggtrade_dq_report.py \
  --journal live_capture/shadow_validation/crypto/ofx_aggtrade_capture_v1/aggtrades.jsonl \
  --out artifacts/rp608_ofx/crypto_aggtrade_dq_report.json
```

---

## Explicit non-goals (still forbidden)

- Rebuilding the OFX executed-quantity library  
- Connecting OFX ΔQ to trading behavior  
- Predictive Gate 2 claims from OFX alone  
- Relabeling historical baseline OF  
- Changing H2 / H3 / Gate 2 / confirmation protocol / execution authority  
- Promoting NSE Kite LTQ path to DIRECT  

## NSE reminder

NSE source = **Kite Connect** (available). Aggressor remains **DERIVED_VALIDATED** only; never DIRECT from LTQ alone.
