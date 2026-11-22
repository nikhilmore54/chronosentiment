# RP-608 Track C — Crypto tick-policy Rust port (started 2026-11-22)

**Authority:** 0% · **Rules:** `.cursor/rules` · Gate 5 frozen · books never pooled  
**Reference:** `scripts/rp608_crypto_tick_policy.py` · protocol `RP-608-SHADOW-TICK-001`

## Status

| Piece | State |
| --- | --- |
| Rust module `product::crypto_tick_policy` | **Landed** — validate, buffer, snapshot, evidence, policy eval, emission gate |
| Differential / unit tests | `cargo test … crypto_tick_policy` |
| Daemon cutover | **Not yet** — live still Python; env `RP608_CRYPTO_TICK_POLICY` reserved (default `python`) |
| Client stub | `scripts/rp608_crypto_tick_policy_client.py` |

## Note on 0.30

`IMBALANCE_THRESHOLD = 0.30` in tick Policy B/C is **microstructure evidence**, not baseline OF paper L1 ±0.30 (`OF_SUP`). Do not conflate; OF construct lock unchanged.

## Next

1. Add `--tick-eval-loop` on `rp608_position_decide` (JSONL quotes → B/C actions).  
2. Wire `run_rp608_shadow_daemon.py` behind env; side-by-side soak BTC+ETH.  
3. Default-on after zero divergence.
