# RP-611 Registration Record

**Governance event:** `approve and register`  
**Recorded:** 2026-11-01  
**Authority:** 0.0%  
**chronosentiment-core:** honored (deterministic observation; no live orders)

**Disposition:** **REGISTERED · CLOCK RUNNING · PHASE-1 DEPLOYED**

| Protocol | Status | Clock | Observer | Doc |
|----------|--------|-------|----------|-----|
| `RP-611-NSE-VOLUME-ABSORPTION-001` | REGISTERED · CLOCK RUNNING | **started** T0 `2026-11-01T10:47:24Z` | **Phase-1 deployed** (see [deploy receipt](./rp611_phase1_deploy_receipt.md)) | [rp611_nse_volume_absorption_protocol.md](./rp611_nse_volume_absorption_protocol.md) |
| `RP-611-CRYPTO-VOLUME-ABSORPTION-001` | REGISTERED · CLOCK RUNNING | **started** T0 `2026-11-01T10:47:24Z` | **Phase-1 deployed** (spot kline + bookTicker) | [rp611_crypto_volume_absorption_protocol.md](./rp611_crypto_volume_absorption_protocol.md) |

Machine-readable locks:

- `artifacts/rp611_nse_volume_absorption/registration.json`
- `artifacts/rp611_crypto_volume_absorption/registration.json`

**Clock start:** `start the experiment clock` → T0 `2026-11-01T10:47:24Z`.

**Phase-1 deploy:** authorized and executed 2026-11-01; receipts in [rp611_phase1_deploy_receipt.md](./rp611_phase1_deploy_receipt.md). Pre-deployment capture gap ~12 minutes recorded. Post-T0 NSE bars pending next session; crypto awaits next completed post-T0 15m close.

**Isolation still mandatory:** no RP-608/OF/Gate5/RP-609/RP-610 behavioural changes; authority 0%.

**Next optional event:** schedule recurring Phase-1 observer runs / Pi timer (still read-only).
