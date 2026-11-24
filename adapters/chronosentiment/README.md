# ChronoSentiment adapter (`chronosentiment_adapter`)

Domain evaluation / decision bridge for Coralys. **Coralys learns; ChronoSentiment evaluates.**  
Aligned with `.cursor/rules/chronosentiment-core.mdc` and CS-P-CLEAN-001 / **CS-P-CLEAN-003**.

## Layout

```text
PRODUCT (default cargo build / test)
  src/product/          Gate2X, plan lifecycle, paper, observatory product surface
  src/bin/              Product + LIVE/TIME binaries (see inventory below)
  bin/                  README only — deploy artifacts relocated (CLEAN-003-E)
  tests/                Default: discipline + product Gate2X/NSE/TIME contracts
                        CS-P-006 ecology / heritage / sqlx → --features research (CLEAN-003-D)

LEGACY (--features legacy-lake)
  legacy/bin/           B3/B4 lake demos & CS-P heritage binaries
  legacy/quarantine/    → archived under archive/chronosentiment_legacy_quarantine/ (CLEAN-003-C)
                        Not compiled; provenance only

RESEARCH (--features research; implies legacy-lake)
  research/             G-GATE laboratory + wl discovery helpers
                        Must not be required for product compile

DEPLOY
  ../../artifacts/deploy/chronosentiment/   rp608_position_decide.* (CLEAN-003-E)

RUNTIME / CAPTURE (not source of record)
  live_capture/         Runtime ledger cwd default; Sep-2026 JSONL moved to
                        archive/chronosentiment_live_capture_ledger_202609/ (CLEAN-003-B)

BUILD
  target/               gitignored (~3GB local); cargo clean -p chronosentiment_adapter
```

## Features

| Feature | Purpose |
| --- | --- |
| *(default)* | Product TradingDecision / Gate2X / lifecycle path |
| `legacy-lake` | B3/B4 generators + heritage bins |
| `research` | G-GATE lab + CS-P-006 ecology / heritage / sqlx tests |

```bash
cargo test -p chronosentiment_adapter
cargo test -p chronosentiment_adapter --features research
cargo build -p chronosentiment_adapter --release --bin rp608_position_decide
```

## `src/bin` inventory (CLEAN-003-A)

| Binary | Lane | Notes |
| --- | --- | --- |
| `rp608_position_decide` | **PRODUCT** | Gate2X / plan-lifecycle decide; Pi shadow daemons |
| `live001_snapshot` … `live005_ledger` | PRODUCT / LIVE | Live admission & ledger chain |
| `time002_reconstruct` … `time009_observe` | PRODUCT / TIME | Time-machine observe/replay |
| `intraday001_observe` | PRODUCT | Intraday observe over TIME-009 ledger |
| `rec001h_historical_reconstruction` | PRODUCT / REC | Historical reconstruction |
| `csp006_p_enrich` | PRODUCT / CS-P | Enrichment |
| `deferred_live_decision_loop` | DEFERRED / PRODUCT | As-of prospective loop |
| `deferred_live_yahoo` | DEFERRED | Yahoo feed helper |

## Hygiene

- Plan: `docs/CS-P-CLEAN-003_CHRONOSENTIMENT_ADAPTER_HYGIENE.md`
- Quarantine history: `docs/CS-P-CLEAN-001_ADAPTER_QUARANTINE.md`
- Do not reopen G-GATE v1.1 from `research/`
