# REC-001-H — Crypto evidence (BTCUSDT)

First-class crypto analogue store for frozen `RecommendationEngineV1` (PVP-003 / CS-P-AUDIT).

- **Source bars:** `datasets/crypto_15m/binance_btcusdt_2024_15m.json`
- **Output:** `BTCUSDT.jsonl` (leakage-free features at bar T; MFE/MAE on T+1…T+10)
- **Loader:** `Rec001hStore::load_from_dir(datasets/recommendation/historical_crypto)`
- **Never pooled** with NSE `datasets/recommendation/historical/`

Used by `gate2x_decide` / `rp608_position_decide` for CRYPTO_PLAN Gate 2X Protect/Exit.
`coralys-*` crates remain frozen — this dataset is adapter/research evidence only.
