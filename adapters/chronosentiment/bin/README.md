# Deploy binaries relocated (CLEAN-003-E)

Checked-in `rp608_position_decide.*` artifacts now live at:

```text
artifacts/deploy/chronosentiment/rp608_position_decide.aarch64-linux
artifacts/deploy/chronosentiment/rp608_position_decide.darwin-arm64
```

Pi systemd units and `scripts/rp608_gate2x_adapter_client.py` resolve the new path.
Zigbuild stage recipe: `docs/research/rp608_position_decide_cli.md`.
