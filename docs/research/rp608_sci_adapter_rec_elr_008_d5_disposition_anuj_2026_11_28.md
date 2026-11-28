# ELR-008 D5–D8 Anuj disposition (2026-11-28 ~16:56 IST)

Source: Anuj chat review of D5–D8 memo. Not an implementation authorization.

## Locked disposition

| Item | Decision |
|------|----------|
| ELR-008 D1–D4 | Preserved (retrospective research sufficient) |
| D5 | Design memo only; clarify fee/slippage semantics before any code |
| D6 | Retrospective analysis; EXECUTION_FRICTION ≠ proven root cause |
| D7 | Observability specification; event-order + single-writer required |
| D8 | Deferred hypotheses with authorization boundaries |
| BTC twins | Frozen |
| ETH research | Independent |
| ML-1D | DO_NOT_ADVANCE |
| New predictive model | Not selected |
| Exit modifications | Not authorized |
| Coralys / Gate 5 | Untouched |

## D5.1 blocker — cost semantics (Anuj must settle)

Config comments: `6 bps/leg = 5 fee + 1 slippage`. Proposed memo: Fees=6, Slippage=0. Same total, different components.

| Interpretation | Fee | Slippage | Total (per leg) |
|----------------|-----|----------|-----------------|
| Actual 6 bps fee | 6 | 0 | 6 |
| Actual 5+1 | 5 | 1 | 6 |
| All-in friction proxy | not attributed | not attributed | 6 |

**Anuj preference (pending confirm):** if config 5+1 is authoritative:

```
Fees = (Ne+Nx) * 5/10000
Slippage = (Ne+Nx) * 1/10000
Net = Gross - Fees - Slippage - Other
```

Else: 6 fee + 0 slip, and reconcile old config comments. Do not call a composite estimate “actual exchange fees.”

**Invariant (every closed outcome):** `|Net - (Gross - Fees - Slippage - Other)| < 0.02`. Fail → `DATA_MEASUREMENT`; exclude from unqualified W/L and expectancy.

Also preserve: (1) strict mathematical net sign; (2) near-zero economic outcome flag. Do not promote ~+0.0007 USDT TRAIL scratches into meaningful profitability evidence.

## Recommended execution sequence (no implement until D5.1)

1. **D5.1** — Freeze cost semantics (Anuj confirms 5+1 vs 6+0). No implementation.
2. **D5.2** — Offline accounting tests; Python/Rust parity across GIVEBACK/SL/TRAIL/MAX_HORIZON; preserve original journals; compute `net_contract_v1` separately.
3. **D5.3** — Accounting validation on historical samples + synthetic boundaries.
4. **D7** — Path observability design: timestamp semantics, ordered events / sequence numbers, common journal-writing interface (Python GIVEBACK must reach same ordered journal as Rust). Instrumentation only.
5. **D6** — Exit-family retrospective under reconciled economics; record aggregation method for capture ratios (mean of ratios vs ratio of means); MFE is hindsight, not proof of executable capture.
6. **D8** — Select one failure mechanism + one falsifiable corrective; explicit Anuj authorization before any sealed package.

## D8 hypothesis boundaries

| Hypothesis | Assessment | Condition |
|------------|------------|-----------|
| H-COST-1 | Priority 1 | Cost parity + historical restatement of all candidates/comparators under same contract |
| H-GB-CAP-1 | Deferred | Separate authorized exit-research path; Gate 5; hypothesis ≠ permission |
| H-SL-DIR-1 | Conditional | Requires actual DI differential exposure |
| H-PATH-1 | Useful | Observability-only |

## Four research layers (keep separate)

1. Measurement reliability — Rust/Python accounting defect (14 rows).
2. Directional reliability — 7 SL failures remain after correction.
3. Economic capture — GIVEBACK positive gross, negative after single-friction; cause under investigation.
4. Selection differentiation — SCI_CLEAN vs stacked still untested (no DI differential).

## Worker instructions

Append this disposition into PR #9 research docs (ELR-008 D5–D8 addendum + ELR index). Design/docs only. No code implementation. No twin/exit/Coralys/Gate5 changes. Twins remain frozen. Commit + push; wait for CI. After commit, sync store copies under `/cursor/stores/bc-5dbda5ed-2596-4da2-815e-4b14076cfa09/docs/`.
