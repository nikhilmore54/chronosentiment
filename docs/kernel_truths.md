# ChronoSentiment — Kernel Truths

**Immutable invariants. Change only if empirically disproven.**

---

## Chronology Invariants

1. T_provider MUST equal T_barrier exactly. A bar with a mismatched timestamp is not a late bar — it is a different bar. It must be rejected or repaired, never silently accepted.
2. The timeline fingerprint (SHA-256 of sorted timestamps, first 16 hex chars) is the canonical identity of a frozen substrate. Two runs with different fingerprints are not comparable.
3. Chronology gaps are recoverable. The repair state machine (PENDING → FETCHED → VERIFIED_TS_MATCH → RECOVERED) is the only authorized recovery path.

## Replay Invariants

4. Deterministic replay means: same frozen substrate + same PCA weights + same observatory binary → same archive, same timeline fingerprint, same telemetry records. Any deviation is a bug.
5. `fresh_wipe_archive` before a replay run is not optional when verifying parity. Incremental state from a prior run contaminates the comparison.
6. Replay certification (`certify_replay_chain.py`) must pass before any archive is treated as authoritative.

## Archive Invariants

7. Every write to a shared state file uses `os.replace(tmp, final)`. Partial writes do not exist. Readers never observe incomplete state.
8. The dedupe index is the authority on whether a (symbol, ts) pair has been persisted. It is rebuilt from archive on resume, not trusted from memory.
9. `latest.json` per symbol is a best-effort cache of the most recent record. It is not authoritative for replay — the `.jsonl.gz` archive is.

## Governor Invariants

10. Absence of telemetry is not evidence of instability. Cold-start behavior is NOMINAL (`multiplier=1.0`, `gate_open=true`). The governor never halts on missing data.
11. The governor is a deterministic execution throttle, not an adaptive intelligence layer. Same telemetry window → same multiplier. No randomness, no learning, no memory beyond the window.
12. The governor state file is the only interface between the governor process and the live session. No other IPC mechanism is authorized.

## Execution Authority Rules

13. A script in `scripts/research/` has no execution authority. It cannot be imported by, called by, or depended on by any script in the live session sequence.
14. A new telemetry field does not enter the execution kernel unless it changes a real execution decision, survives replay falsification, and remains deterministic.
15. A new script does not enter `scripts/` unless it has a defined consumer in the live session sequence.

## Anti-Drift Rules

16. The correct filter for any proposed addition: "what operational uncertainty does this reduce?" If the answer is vague, it belongs in `scripts/research/` or nowhere.
17. Observability is a cost center. Every persistent field has storage cost, replay cost, cognitive cost, and schema cost. It must justify itself against those costs.
18. Naming a phenomenon is not the same as having operational leverage over it. An abstraction that only describes without changing decisions is annotation, not architecture.