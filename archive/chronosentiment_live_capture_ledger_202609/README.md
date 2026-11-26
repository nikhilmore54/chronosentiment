# ChronoSentiment adapter live_capture ledger archive (Sep 2026)

**CLEAN-003-B.** Moved out of `adapters/chronosentiment/live_capture/ledger/` so the
adapter crate stays source-only.

| File | Role |
| --- | --- |
| `shadow_ledger_e4_050_*.jsonl` | E4 shadow validator historical captures |
| `position_management_events_*.jsonl` | Position management event dumps |
| `SHA256SUMS` | Content hashes at archive time |

Fixture copy for unit tests: `adapters/chronosentiment/tests/fixtures/e4_shadow/`.
