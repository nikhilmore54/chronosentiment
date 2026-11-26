# ChronoSentiment — Stabilization Protocol

**Phase:** Core v0.1 Stabilized Kernel  
**Duration:** 2–4 weeks minimum  
**Rule:** No new abstractions enter the execution path.

---

## What This Phase Is

The boundary cleanup is complete. The execution path is defined. The governor is wired. The validation chain is canonical.

This phase proves the simplified kernel is trustworthy under stress — not theoretically, empirically.

---

## The Milestone: Core v0.1 Stabilized Kernel

Requirements (all must be demonstrated, not assumed):

| Requirement | How to Prove |
|---|---|
| Deterministic replay certified | Same archive → same timeline fingerprint across runs |
| Chronology enforcement proven | Timestamp violations rejected by repair state machine |
| Governor transitions proven | NOMINAL → THROTTLE → HALT → NOMINAL under controlled telemetry |
| Recovery lineage proven | Gap detection → repair → re-certification succeeds |
| Archive integrity proven | `cs-ingest verify` passes after multi-session ingest |
| Restart equivalence proven | Resume from checkpoint produces identical archive to fresh run |
| Cold-start behavior proven | Governor starts at NOMINAL with no archive, transitions correctly |
| Execution path frozen | No new scripts enter `scripts/` without a defined live-path consumer |

---

## What Is Allowed During Stabilization

- Bug fixes
- Replay integrity fixes
- Chronology correctness improvements
- Operational ergonomics (CLI flags, error messages, logging)
- Observability reduction (removing unused fields, simplifying records)
- Performance cleanup
- Documentation
- Deterministic validation improvements

## What Is Not Allowed During Stabilization

- New terminology or interpretation semantics
- New telemetry fields
- ML or adaptive intelligence integration
- New provider synchronization dimensions
- Dynamic or self-modifying governors
- New propagation theory
- New abstraction layers

The filter for any proposed change: **what operational uncertainty does this reduce?**  
If the answer is vague, it belongs in `scripts/research/` or nowhere.

---

## The Stabilization Test Loop

Run repeatedly during this phase:

```
1. live ingest (cs-ingest cohort)
2. replay certification (certify_replay_chain.py)
3. deterministic equivalence (compare_replay_equivalence.py)
4. governor state transitions (cs-ingest governor --once)
5. recovery scenario (cs-ingest repair detect + process)
6. archive integrity (cs-ingest verify)
7. cold-start recovery (delete governor_state.json, restart governor)
8. multi-session continuity (resume=True run after fresh run)
9. timeline fingerprint verification (cs-ingest timeline)
```

The only question being asked: **does the simplified kernel remain truthful under stress?**

---

## What Comes After Stabilization

Only after the Core v0.1 milestone is proven:

- **Telemetry compression:** separate the 5 diagnostics-only fields (`previous_state`, `next_state`, `corridor_id`, `transition_confidence`, `local_density`) into a lower-frequency debug record
- **Semantic audit:** every abstraction (instability subtypes, chronological variants, synchronization transitions) must answer "what execution decision changes because this exists?"
- **Replay-era scaffolding audit:** compatibility bridges, transitional archive logic, migration-era helpers
- **Observatory operational leverage:** connect synchronization phase labels to at least one governor decision, then replay with/without to falsify whether it improves outcomes

None of these start until the stabilization loop runs cleanly for 2+ weeks.