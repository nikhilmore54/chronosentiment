# RP-608 — ETHUSDT Observational Twin

**Status:** wired (2026-11-22) · **Authority:** 0% · **SOL:** deferred until ETH proven  
**Rules:** `.cursor/rules/chronosentiment-core.mdc` · books never pooled · Gate 5 for BTC baseline · OF ±0.30 L1 frozen on BTC

## Purpose

Portability check of the crypto shadow stack (tick Policy B/C → trade-plan overlay → OF paper → OFX aggTrade volume paper) on **ETHUSDT** without contaminating the live **BTCUSDT** PRIMARY/BASELINE books or Gate 2E `allow_only` cohorts.

## Isolation contract

| Dimension | BTC (live) | ETH (twin) |
|---|---|---|
| Process | `rp608-shadow-observer.service` | `rp608-shadow-observer-eth.service` |
| CLI | default / `--symbol BTCUSDT` | `--symbol ETHUSDT` |
| Output root | `live_capture/shadow_validation/crypto` | `…/crypto_eth` |
| Singleton lock | per output-dir | per output-dir (parallel OK) |
| Book IDs | `CRYPTO_PLAN` / `…_BASELINE_PARALLEL` | `CRYPTO_PLAN_ETHUSDT` / `…_ETHUSDT` |
| Gate 2E | PRIMARY `allow_only` (BTC library) | **forced `0`** (no BTC cohort reuse) |
| W/L reporting | BTC books only | ETH books only — **never pool** |

## Launch (Pi)

```bash
# After deploying code + unit file:
sudo cp etc/systemd/system/rp608-shadow-observer-eth.service /etc/systemd/system/
sudo systemctl daemon-reload
sudo systemctl enable --now rp608-shadow-observer-eth.service
# Verify streams (symbol=ETHUSDT in journal):
journalctl -u rp608-shadow-observer-eth.service -n 40 --no-pager
```

Dry-run locally:

```bash
python3 -u scripts/run_rp608_shadow_daemon.py --symbol ETHUSDT --dry-run
```

## SOL (held)

`--symbol SOLUSDT` is allow-listed and maps to `…/crypto_sol`, but **do not enable** a SOL systemd unit until ETH twin shows clean cards → overlay → closed-trade W/L path under Pi memory budget.

## Non-goals

- No Gate 5 change to BTC OF ±0.30 / traj15 / stops / targets
- No shared Gate 2E pattern library across symbols
- No dashboard PRIMARY swap to ETH (BTC remains the operator-facing book)
