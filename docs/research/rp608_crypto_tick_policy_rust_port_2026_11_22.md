# RP-608 Track C — Crypto tick-policy Rust port (started 2026-11-22)

**Authority:** 0% · **Rules:** `.cursor/rules` · Gate 5 frozen · books never pooled  
**Reference:** `scripts/rp608_crypto_tick_policy.py` · protocol `RP-608-SHADOW-TICK-001`

## Status

| Piece | State |
| --- | --- |
| Rust module `product::crypto_tick_policy` | **Landed** — validate, buffer, snapshot, evidence, policy eval, emission gate |
| Differential / unit tests | `cargo test … crypto_tick_policy` |
| CLI `--tick-eval-loop` | **Landed** — JSONL quote+buffer+genomes → B/C + `features_tick` |
| Client | `scripts/rp608_crypto_tick_policy_client.py` — persistent loop session |
| Daemon wire | **Landed 2026-11-24** — `RP608_CRYPTO_TICK_POLICY=python\|rust\|compare` (default `python`) |
| Pi soak / default-on | **Done 2026-11-25** — compare soak 0 diverge (BTC/ETH); systemd `RP608_CRYPTO_TICK_POLICY=rust` |

## Note on 0.30

`IMBALANCE_THRESHOLD = 0.30` in tick Policy B/C is **microstructure evidence**, not baseline OF paper L1 ±0.30 (`OF_SUP`). Do not conflate; OF construct lock unchanged.

## Next

1. Keep soak audit for regressions (`rp608_crypto_tick_policy_soak_audit.py`).  
2. Python evaluate path = fallback only when bin/loop down.  
3. Track C exit met for crypto tick core; full WS/daemon port remains out of scope.
