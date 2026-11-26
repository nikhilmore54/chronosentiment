# Gate 2E — Shadow thin-WR → BASELINE join

**Rule:** `SHADOW_ALLOW_IF_N10_WR_GT_035`  
**Script:** `scripts/research/rp608_gate2e_shadow_baseline_join.py`  
**Live PRIMARY:** `allow_only` **unchanged** · authority 0% · books never pooled

Joins observational shadow stamps (`gate2e_shadow_allows.jsonl`) to BASELINE closed outcomes within ±2 minutes same direction. Reports W/L, payoff, net — required before any Gate 2E knob discussion.

```bash
python3 scripts/research/rp608_gate2e_shadow_baseline_join.py
python3 scripts/research/rp608_gate2e_shadow_baseline_join.py --since-hours 48 --json
```

Empty shadow file → wait for thin-WR PRIMARY suppress. Do not ADVANCE from join alone.

## Backfill (pre-deploy thin-WR → shadow jsonl)

```bash
python3 scripts/research/rp608_gate2e_shadow_thin_wr_backfill.py --dry-run
python3 scripts/research/rp608_gate2e_shadow_thin_wr_backfill.py --apply
```

Idempotent by `plan_id`. Stamps include `entry_price` + `vol_surge` from pattern_id `V*` axis (Gate2E candle volume ≠ OFX). Observational only.

### Pi run 2026-11-22 (resume pick 3)

| Metric | Value |
|--------|-------|
| Backfilled thin-WR | 156 |
| Joined to BASELINE (±2m) | 90 (66 unmatched) |
| W/L | 46/90 · wr≈0.511 |
| Payoff avg\|win\|/avg\|loss\| | ≈0.618 |
| Net USDT sum | −186.12 |
| vol_surge | all **UNK** (`VUNK`) |

**Read:** thin-WR alone is **not** an ADVANCE signal — WR ~coin-flip with adverse payoff asymmetry → negative net. Volume axis was UNK on these pre-deploy stamps; need live post-deploy stamps with real `V*` buckets + price path before any Gate2E knob talk. Live `allow_only` unchanged · OF ±0.30 / Gate5 frozen.
