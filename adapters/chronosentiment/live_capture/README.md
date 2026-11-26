# live_capture (runtime cwd defaults)

Product binaries (`live005_ledger`, `time009_observe`, …) default ledger paths under
`live_capture/ledger/` relative to process cwd.

**CLEAN-003-B (2026-11-24):** historical Sep-2026 JSONL that lived in this crate was
moved to:

`archive/chronosentiment_live_capture_ledger_202609/`

E4 unit-test fixture:

`tests/fixtures/e4_shadow/shadow_ledger_e4_050_20260914.jsonl`

Do not commit multi-MB shadow ledgers back into this adapter tree. Pi RP-608 journals
remain under workspace `live_capture/shadow_validation/` (outside this crate).
