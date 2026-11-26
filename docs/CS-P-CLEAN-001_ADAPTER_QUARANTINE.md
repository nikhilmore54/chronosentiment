# CS-P-CLEAN-001 — Adapter quarantine (PR-1)

**Document type:** Architecture hardening  
**Status:** Complete — PR-2 is CS-P-CLEAN-002  
**Date:** 2026-08-14  
**Parent:** CS-P-AUDIT-001  
**Does not:** create a candidate policy, run a backtest, regenerate B3/B4, reopen G-GATE, freeze v1.0  

`.cursor/rules/chronosentiment-core.mdc`: deterministic evaluation; no invented methodology.

---

## What this is

PR-1 of the authorized three-lane cleanup. ChronoSentiment remains the domain decision/evaluation system. Coralys remains the learning/policy-discovery system. This change is **quarantine**, not an experiment.

Evidence chain unchanged:

```text
B3 ── immutable
B4 ── immutable
G-GATE v1.1 ── closed / INCONCLUSIVE
CS-P-002 … CS-P-005, CS-P-TEST-001 ── unchanged
```

---

## Boundary

```text
Default product build (`chronosentiment_adapter`, no extra features)
    TradingDecision, DecisionPolicy, Replay, Forward, Ledger,
    Outcome, Performance, assess_at

--features legacy-lake
    DecisionEngine + StrategyEngine + m4_populate_knowledge_lake
    (B3/B4 provenance; do not repair SHORT omission)

--features research
    G-GATE laboratory (implies legacy-lake)
```

Lake type `reasoning::decision::Decision` remains on the default compile graph so B3/B4 dumps can be read. The **generators** are feature-gated. The two decision objects are not merged.

---

## Done in this PR

* `src/research/` moved to `research/` and compiled only with `--features research`
* Demo / G-GATE / lake-populate binaries moved out of `src/bin`; product `src/bin` is CS-P-* only
* `DecisionEngine` / `StrategyEngine` preserved, `legacy-lake` only
* `week2_tests.rs` quarantined (not compiled)
* `phase_c_gate.sh` **removed** (printed PASS without running tests). Fixtures kept.
* Invariant tests in `tests/adapter_discipline_invariants.rs`

## Not done (later PRs)

* CS-P-006-B protocol freeze and CS-P-006-C Coralys search (CS-P-006-A is the consumption contract)

PR-2 (explicit `DecisionPolicy`) is `docs/CS-P-CLEAN-002_EXPLICIT_POLICY_CONTRACT.md`.
