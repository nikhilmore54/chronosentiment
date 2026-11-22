# RP-608 position decide CLI (Pi transitional)

**Binary:** `rp608_position_decide`  
**Protocol id:** `RP-608-POSITION-DECIDE-001`  
**Authority:** ChronoSentiment adapter `gate2x_decide` → frozen `RecommendationEngineV1`  
**Aligned with:** `.cursor/rules/chronosentiment-core.mdc` §3c · PVP-003 · Gate 2X v1.3  
**Source:** [`adapters/chronosentiment/src/bin/rp608_position_decide.rs`](../../adapters/chronosentiment/src/bin/rp608_position_decide.rs)

## Role

Rust **`PlanLifecycleManager`** (`--plan-lifecycle-loop`) owns Adaptive TP + hard SL/TP + Gate 2X Protect/Exit. Python crypto/NSE shadows remain WS/entry/OF I/O + journals. Frozen OF ±0.30 construct unchanged (Gate 5 / OFX boundary). Books / REC001h stores **never pooled**. See `rp608_rust_plan_lifecycle_cutover.md`.

**Preferred (full lifecycle cutover):** plan+quote JSONL → lifecycle result:

```bash
export RP608_PLAN_LIFECYCLE_BIN=/path/to/rp608_position_decide
# Python: scripts/rp608_plan_lifecycle_client.py (RP608_PLAN_LIFECYCLE=1)
"$RP608_PLAN_LIFECYCLE_BIN" --plan-lifecycle-loop --book crypto \
  --evidence-dir datasets/recommendation/historical_crypto
```

**Legacy Gate2X-only manage loop** (snapshot → Hold|Protect|Exit):

```bash
export RP608_GATE2X_MANAGE_BIN=/path/to/rp608_position_decide
# Python client starts this with --manage-loop automatically via scripts/rp608_gate2x_adapter_client.py
"$RP608_GATE2X_MANAGE_BIN" --manage-loop --book crypto \
  --evidence-dir datasets/recommendation/historical_crypto
```

**One-shot (fallback):** single snapshot on stdin → one decision line:

```bash
export RP608_POSITION_DECIDE_BIN=/path/to/rp608_position_decide
echo "$SNAPSHOT_JSON" | "$RP608_POSITION_DECIDE_BIN" --book crypto \
  --evidence-dir datasets/recommendation/historical_crypto
```

| Book | `--evidence-dir` | `--book` |
| --- | --- | --- |
| Crypto | `datasets/recommendation/historical_crypto` | `crypto` |
| NSE | `datasets/recommendation/historical` | `nse` |

| Flag / env | Meaning |
| --- | --- |
| `--plan-lifecycle-loop` | plan+quote JSONL → Adaptive TP + SL/TP + Gate 2X (preferred cutover) |
| `--manage-loop` | Gate2X snapshot JSONL → Hold\|Protect\|Exit only |
| `--evidence-dir DIR` | REC001h JSONL directory |
| `--book crypto\|nse` | force book (else inferred from evidence-dir) |
| `--version` | prints `RP-608-POSITION-DECIDE-001` |
| `RP608_PLAN_LIFECYCLE` | `1` (default) enable Rust lifecycle from Python shadows |
| `RP608_PLAN_LIFECYCLE_REQUIRE_RUST` | `1` hard fail-closed: no Python Adaptive/Gate2X manage if Rust unavailable |
| `RP608_GATE2X_MANAGE_BIN` / `RP608_POSITION_DECIDE_BIN` | Python client binary path |
| `RP608_GATE2X_ONESHOT=1` | force per-tick one-shot (debug only) |

Exit codes: `0` ok · `1` load/parse/serialize error · `2` bad args.

Python **never** invents Protect/Exit if the binary is missing — fail-closed `Hold` (`GATE2X_NO_RUST_BIN`).

## JSON contract

### stdin — `Gate2xFeatureSnapshot`

```json
{
  "decision_id": "PLAN-253465-SHORT-…",
  "instrument": "BTCUSDT",
  "direction": "SHORT",
  "reference_price": 58634.81,
  "peak_mfe": 10.0,
  "current_mfe": 1.0,
  "peak_tp_frac": 0.55,
  "hold_s": 200.0,
  "arm_threshold": 1.7,
  "already_armed": true,
  "vol_surge": "HIGH",
  "soft_breach_since_ms": null,
  "now_ms": 1795229910000,
  "params": {
    "enabled": true,
    "arm_min_tp_frac": 0.40,
    "min_hold_s_before_exit": 120.0,
    "meaningful_floor_bps": 5.0,
    "protect_floor": "COST_CLEAR",
    "reversal_volume_confirm": true,
    "immediate_vol_surges": ["HIGH"],
    "med_vol_dwell_ms": 40000,
    "low_vol_dwell_ms": 90000,
    "ratchet_peak_frac": 0.30,
    "deep_giveback_frac": 0.50,
    "deep_giveback_dwell_scale": 0.40,
    "protocol": "RP-608-GATE2X-CRYPTO-MFE-GIVEBACK-001"
  },
  "train_vol_median": 0.001
}
```

`vol_surge`: `HIGH` | `MED` | `LOW` | `UNK`. NSE uses `params.protocol` = `RP-608-GATE2X-NSE-MFE-GIVEBACK-001` and instrument like `RELIANCE.NS` (normalized to `*_NS` store keys).

### stdout — `Gate2xDecision`

```json
{
  "action": "Protect",
  "reason": "MFE_GIVEBACK_TO_COST_CLEAR|VOL_HIGH|…",
  "coralys_action": "Protect",
  "recommendation_action": "Watch",
  "decision_id": "PLAN-253465-SHORT-…",
  "protocol": "RP-608-GATE2X-CRYPTO-MFE-GIVEBACK-001",
  "soft_breach": {
    "soft_breach_since_ms": null,
    "soft_breach_dwell_ms": null,
    "protect_floor": 3.0,
    "ratchet_floor": 3.0,
    "armed": true
  },
  "engine_insufficient": false,
  "authority": "coralys_recommendation_engine_v1"
}
```

`action` ∈ `Hold` | `Protect` | `Exit`. Executor persists `soft_breach` for the next tick; frozen SL/TP still apply independently.

## Build

```bash
# host (dev / macOS)
cargo build -p chronosentiment_adapter --bin rp608_position_decide --release

# Pi aarch64-linux (local cross via zig — preferred; no rustup on Pi)
cd adapters/chronosentiment
cargo zigbuild -p chronosentiment_adapter --bin rp608_position_decide --release \
  --target aarch64-unknown-linux-gnu
# staged copy:
#   adapters/chronosentiment/bin/rp608_position_decide.aarch64-linux
```

On Pi:

```bash
mkdir -p ~/bin
cp adapters/chronosentiment/bin/rp608_position_decide.aarch64-linux ~/bin/rp608_position_decide
chmod +x ~/bin/rp608_position_decide
export RP608_POSITION_DECIDE_BIN=$HOME/bin/rp608_position_decide
# smoke:
echo '{"decision_id":"SMOKE","instrument":"BTCUSDT","direction":"SHORT","reference_price":1,"peak_mfe":10,"current_mfe":5,"peak_tp_frac":0.55,"hold_s":200,"arm_threshold":1.7,"already_armed":true,"vol_surge":"HIGH","soft_breach_since_ms":null,"now_ms":1,"params":{"enabled":true,"arm_min_tp_frac":0.4,"min_hold_s_before_exit":120,"meaningful_floor_bps":5,"protect_floor":"COST_CLEAR","reversal_volume_confirm":true,"immediate_vol_surges":["HIGH"],"med_vol_dwell_ms":40000,"low_vol_dwell_ms":90000,"ratchet_peak_frac":0.3,"deep_giveback_frac":0.5,"deep_giveback_dwell_scale":0.4,"protocol":"RP-608-GATE2X-CRYPTO-MFE-GIVEBACK-001"},"train_vol_median":0.001}' \
  | "$RP608_POSITION_DECIDE_BIN" --evidence-dir datasets/recommendation/historical_crypto
```

Until the binary is present, the Python client (`scripts/rp608_gate2x_adapter_client.py`) uses a stamped geometry local mirror (`*_LOCAL_MIRROR_NO_BIN`).

## Golden tests (Gate 2X v1.3)

```bash
cargo test -p chronosentiment_adapter --lib product::gate2x_decide
```

| Test | Asserts |
| --- | --- |
| `v13_golden_disabled_holds` | `GATE2X_DISABLED` |
| `v13_golden_not_armed_holds` | `GATE2X_NOT_ARMED` |
| `v13_golden_min_hold_holds` | `GATE2X_MIN_HOLD` |
| `v13_golden_above_floor_holds` | `GATE2X_ABOVE_FLOOR` + ratchet stamp |
| `v13_golden_ratchet_floor_stamps` | protect/ratchet floor = 0.30×peak |
| `v13_golden_between_cost_clear_and_ratchet_still_above` | Hold between cost-clear and ratchet |
| `v13_golden_high_vol_immediate` | HIGH → no dwell |
| `v13_golden_med_dwell_40s` | MED 40s dwell then fire |
| `v13_golden_low_dwell_90s` | LOW 90s dwell then fire |
| `v13_golden_deep_giveback_shortens_dwell` | dwell ×0.40 + `DEEP_GB` |
| `v13_golden_vol_confirm_off_fires_without_dwell` | confirm off → immediate |
| `v13_golden_entry_floor_mode` | `MFE_GIVEBACK_TO_ENTRY` |
| `v13_golden_json_roundtrip_snapshot` | snapshot serde |
| `v13_golden_cli_json_stdin_stdout_contract` | CLI stdin/stdout key contract |
| `v13_golden_nse_instrument_normalize` | `.NS` → NSE protocol / store |
| `manage_tick_crypto_in_process_dwells_then_latches` | `Gate2xPositionManager` latch across ticks |
| `manage_tick_nse_book_protocol` | NSE book protocol stamp |

```bash
cargo test -p chronosentiment_adapter --lib product::gate2x_position_manage
```

Adverse Coralys outcomes may be `Protect|Exit|Hold(CORALYS_HOLD)`; reason stamps are golden.

## Related

- Crypto Gate 2X memo: `docs/research/rp608_gate2x_crypto_mfe_giveback_2026_11_21.md`
- NSE Gate 2X memo: `docs/research/rp608_gate2x_nse_mfe_giveback_2026_11_21.md`
- Crypto REC001h: `datasets/recommendation/historical_crypto/`
