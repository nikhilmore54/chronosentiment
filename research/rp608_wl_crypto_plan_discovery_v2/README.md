# Moved into the ChronoSentiment research adapter

Authoritative implementation:

- `adapters/chronosentiment/research/src/wl_crypto_plan_discovery_v2/`
- `adapters/chronosentiment/research/bin/rp608_wl_crypto_plan_discovery_v2.rs`

```bash
cargo run -p chronosentiment_adapter --features research --release \
  --bin rp608_wl_crypto_plan_discovery_v2 -- \
  --journal live_capture/shadow_validation/crypto/trade_plan_overlay_v1/trade_plan_outcomes.jsonl \
  --out-dir artifacts/rp608_econ_edge_diag
```

Scope: `docs/research/rp608_wl_crypto_plan_discovery_v2_scoped.md`
