# CS-P-002-R1 known limitations

This note does **not** change the hashed historical performance report. It does **not** modify B4.

Certified B4 dump SHA-256: `f74e576e8e98b24058cc913b14a567d9ff4b3eabc75662ab9a96901b102f8cd6`.

## Incomplete lake-outcome coverage

CS-P-002-R1 attached B4 `knowledge_outcomes` by `(instrument_id, as_of)`. Those lake rows exist for **110** timestamps (all LONG under `unfrozen-dev`). The **85 SHORT** decisions have `available: false` on every horizon. **NO_TRADE** did not occur (0 of 195).

Therefore the historical report is **not** a complete evaluation of the decision stream. SHORT is unevaluated, not zero. Do not invent SHORT lake rows. Do not retune the policy to chase the LONG baseline.

Forward/Paper measurement (CS-P-003) uses raw prices after T so every decision, including SHORT, can be observed.

Parent: `docs/CS-P-003_FORWARD_PAPER_VALIDATION.md`.
