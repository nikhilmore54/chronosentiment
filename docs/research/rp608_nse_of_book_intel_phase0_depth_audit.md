# RP-608-NSE-OF-BOOK-INTELLIGENCE-001 — Phase 0 Depth-Capture Audit

**Protocol:** `RP-608-NSE-OF-BOOK-INTELLIGENCE-001`  
**Phase:** `PHASE_0_DEPTH_CAPTURE_AUDIT`  
**Disposition:** **EXECUTED · VERDICT LOCKED**  
**Date:** 2026-11-02  
**Authority:** 0.0%  
**Artifact (Pi):** `live_capture/shadow_validation/nse/of_book_intelligence_v1/phase0_depth_capture_audit.json`  
**Harness:** `scripts/research/rp608_nse_of_book_intel_phase0_depth_audit.py`

Parent scope: [`rp608_nse_of_book_intelligence_scoped.md`](rp608_nse_of_book_intelligence_scoped.md)

---

## Governance (unchanged)

| Item | State |
| --- | --- |
| Book-intelligence scope | Locked |
| Intrabar aggressor absorption | Independent, feed-gated |
| OF ±0.30 | Frozen |
| Reverse-OF paper exit | Frozen |
| Estimand A / EXIT-PATH | Separate |
| Live order authority | 0.0% |
| Replenishment / withdrawal metrics | **Not authorized** until durable L1–L5 history exists |

---

## Pipeline under audit

```
Kite full packet (L1–L5)
        ↓
Live ticker decode  — reads L1–L5 locals, returns L1 scalars only
        ↓
In-memory cache     — latest-only L1
        ↓
Durable journals    — L1 touch qty / OF imbalance (no L2–L5 arrays)
```

**Key finding:** data arrives on the wire; the durable observation layer is incomplete. L2–L5 on the wire does **not** establish historical depth availability.

---

## Four checks

| Check | Result | Evidence |
| --- | --- | --- |
| **Wire coverage** | **PASS (structural)** | Kite `mode=full` 184-byte packets encode 5 bid + 5 ask levels. Live feed health (2026-11-02): connected, 119/119 tokens cached, packets flowing. Prior `live_probe_evidence.json`: 10/10 sample ticks carry `depth_buy`/`depth_sell` length 5. |
| **Cache coverage** | **FAIL for L2–L5** | `scripts/rp608_kite_ticker_cache.py` `decode_full_mode_packet` loops `range(5)` then **drops** arrays; returned dict is L1-only (`best_bid/ask`, `bid_qty/ask_qty`, LTP, volume). Classification: **CACHE_SCHEMA_DROP_L2_L5**. Probe decoder retains `depth_buy`/`depth_sell` — proves decode is possible; live cache chooses not to. |
| **Durable coverage** | **FAIL for L2–L5** | Pi `rp608_nse_latest_evaluations.json` → `evaluations` (**119/119** L1 touch qty; **119/119** OF imbalance; **0/119** L2–L5 arrays). OF paper journals store L1 `depth_quality.*` only. **No** depth-history jsonl / book-snapshot tape. Classification: **PERSISTENCE_L1_ONLY**. |
| **Temporal integrity** | **INSUFFICIENT for book evolution** | Cache is **latest-only overwrite** (not an ordered snapshot tape). Exchange + receipt timestamps exist on L1 ticks, but without durable ordered L1–L5 history we cannot measure replenishment inter-arrival, duplicates, or persistence gaps for deeper levels. |

### Wire note (nonzero qty ≠ structural presence)

In the archived probe sample, L1 qty was nonzero on 5/10 ticks; L2–L5 qty were **0** in that short window. Structural five-level slots are consistently present; economically meaningful size at L2–L5 is **session/symbol dependent** and must be quantified on a durable tape once journaling exists — not assumed from packet layout alone.

---

## Symbol coverage matrix (wire → journal)

Pattern is **uniform across the 119-symbol universe** (not a per-symbol decoder flake):

| Layer | L1 | L2–L5 |
| --- | --- | --- |
| Wire (`mode=full`) | Present | Present (structural) |
| Live cache decode return | Retained | **Dropped** |
| In-memory cache | Latest L1 | **Absent** |
| Durable eval / OF journals | Touch qty + OF | **Absent** |

Full symbol-by-symbol rows are emitted in `phase0_depth_capture_audit.json` → `symbol_matrix_journaled`.

---

## Root cause

| Question | Answer |
| --- | --- |
| Wire gap? | **No** — full packets carry L1–L5 |
| Decoder inability? | **No** — probe path already keeps five-level arrays |
| Cache-schema issue? | **Yes** — live ticker cache discards L2–L5 after parse |
| Persistence issue? | **Yes** — nothing journals five-level snapshots; cache is latest-only |

**Primary classification:** `CACHE_SCHEMA_THEN_PERSISTENCE`

---

## Verdict

| Item | Status |
| --- | --- |
| Durable L1–L5 history exists? | **No** |
| Build replenishment / withdrawal / \(I_5\) features? | **Blocked** |
| Change OF ±0.30 / paper / Estimand A? | **Forbidden** |
| Next milestone | **Evidence of durable L1–L5 snapshot history** (timestamped, ordered, stale/missing flagged) — not a new OF signal or threshold |

Optional follow-up (still Phase 0 / pre-Phase 1): re-run harness with `KITE_API_KEY` + `KITE_ACCESS_TOKEN` in the process environment and `--live-seconds N` for a contemporaneous wire nonzero-qty matrix by level. Not required to lock this verdict.

---

## Disposition

Phase 0 audit **complete**. Phase 1 (durable five-level snapshot journal) is the next authorized engineering step for book-intelligence — still observational, still 0% authority, still no OF rule change.
