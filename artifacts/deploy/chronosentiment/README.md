# ChronoSentiment deploy artifacts (CLEAN-003-E)

| File | Target | Use |
| --- | --- | --- |
| `rp608_position_decide.aarch64-linux` | Pi / Linux aarch64 | `RP608_POSITION_DECIDE_BIN` / `RP608_GATE2X_MANAGE_BIN` |
| `rp608_position_decide.darwin-arm64` | macOS arm64 | Local smoke |

**Last staged / Pi live:** 2026-11-25 — Track C compare soak (BTC/ETH) + Track B `--depth-snapshot-loop` / `RP608_NSE_DEPTH_WRITER=compare` side-path. Bin: floor-lock + tick-eval + depth-snapshot loops.

## Rebuild / stage (zigbuild)

```bash
cd adapters/chronosentiment
cargo zigbuild -p chronosentiment_adapter --bin rp608_position_decide --release \
  --target aarch64-unknown-linux-gnu
cp target/aarch64-unknown-linux-gnu/release/rp608_position_decide \
  ../../artifacts/deploy/chronosentiment/rp608_position_decide.aarch64-linux

cargo build -p chronosentiment_adapter --bin rp608_position_decide --release
cp target/release/rp608_position_decide \
  ../../artifacts/deploy/chronosentiment/rp608_position_decide.darwin-arm64
```

Do **not** rename protocol CLI flags without updating Pi service units under `etc/systemd/system/`.
