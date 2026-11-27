# REC-001-H — Crypto evidence (BTCUSDT · ETHUSDT)

First-class crypto analogue store for frozen `RecommendationEngineV1` (PVP-003 / CS-P-AUDIT).

| Book | Source bars | Output |
| --- | --- | --- |
| BTCUSDT | `datasets/crypto_15m/binance_btcusdt_2024_15m.json` | `BTCUSDT.jsonl` / `.jsonl.gz` |
| ETHUSDT | `datasets/crypto_15m/binance_ethusdt_2024_15m.json` | `ETHUSDT.jsonl` / `.jsonl.gz` |

- Leakage-free features at bar T; MFE/MAE on T+1…T+10  
- **Builders:** `scripts/rp608_rec001h_crypto_btcusdt_build.py` · `scripts/rp608_rec001h_crypto_ethusdt_build.py`  
- **Loader:** `Rec001hStore::load_from_dir(datasets/recommendation/historical_crypto)`  
- **Never pooled** across BTC/ETH or with NSE `datasets/recommendation/historical/`

Used by `gate2x_decide` / `rp608_position_decide` for CRYPTO_PLAN Gate 2X Protect/Exit.
`coralys-*` crates remain frozen — this dataset is adapter/research evidence only.
